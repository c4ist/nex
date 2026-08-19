# Compiler Architecture

> **Status: foundations in place.** This page describes the workspace layout
> and the design decisions that are already locked in. The parser and all later
> passes are planned but not yet written.

## Workspace layout

Nex is a Cargo workspace with three crates:

| Crate            | Responsibility                         | Dependencies              |
| ---------------- | -------------------------------------- | ------------------------- |
| `nex-lexer`      | source text → tokens (Phase 1, done)   | none                      |
| `nex-syntax`     | AST types and the future parser        | `nex-lexer` (for `Span`)  |
| `nex-driver`     | the `nex` CLI                          | `nex-lexer`, `clap`       |

Dependencies are pinned exactly (e.g. `clap =4.5.23`, `insta =1.41.1`) because
Cargo only became MSRV-aware in 1.84 and the workspace MSRV is 1.83.

## The compilation pipeline

```
source text
   │  nex-lexer          (Phase 1 — done)
   ▼
tokens
   │  nex-syntax parser  (Phases 3–4 — planned)
   ▼
AST (immutable)
   │  name resolution + type checker  (Phase 6 — planned)
   ▼
checked AST
   ├── tree-walking interpreter   (Phase 5 — planned)
   ├── LLVM native codegen        (Phase 8 — planned)
   └── WebAssembly codegen        (Phase 9 — planned)
```

Every backend must agree: the regression corpus is run under every available
backend and the outputs must be identical.

## AST design (locked in)

The AST node plumbing lives in `nex-syntax` (`node.rs`). Two rules hold across
the whole tree:

1. **Every node has a `Span`**, so later passes (type checker, LSP) can always
   point at the source text that caused something.
2. **Every node has a unique `NodeId`**, so later passes can hang info off a
   node in *side tables* instead of mutating the tree. The AST is immutable
   once parsed.

Key types:

- `NodeId(u32)` — dense, sequential ids handed out by `NodeIdGen` while
  parsing one module. Because they are dense they double as indices into side
  tables (`Vec<T>` keyed by `id.index()`).
- `NodeId::DUMMY` — marks nodes synthesised during error recovery. It has no
  side-table slot and deliberately **panics** on `.index()`, so a dummy can
  never silently corrupt a side table.
- `NodeInfo { id, span }` — the identity + location every AST node embeds.
- `Spanned<T>` — attaches a span to a value that doesn't need its own identity
  (an ident, a field name, an operator).
- `Ident` = `Spanned<String>` — an identifier as written in the source.
- `spanning(items, fallback)` — derives a parent node's span from its children.

Consequence for future passes: type information, resolved names, and lowering
results all live in side tables keyed by `NodeId`, never in the tree itself.

## The CLI (`nex-driver`)

`nex` is a clap-based CLI. The full command surface exists (`build`, `run`,
`check`, `fmt`, `test`, `lex`) but only `lex` is implemented; the others exit
with a "not implemented yet; it arrives in Phase X" message, where X names the
roadmap phase that delivers them.

Diagnostics are currently rendered by a small dependency-free renderer in
`nex-driver/src/diag.rs` that prints the offending line and a caret underline.
It is a deliberate stopgap: it will be replaced by `ariadne` at step 3.11 once
the parser starts producing richer diagnostics.

## Testing strategy

- **Unit tests** live next to the code (lexer, spans, node plumbing).
- **Integration tests** in `crates/nex-lexer/tests/` cover literals, operators,
  trivia and keywords, recovery, plus an insta snapshot suite (`golden.rs`)
  that lexes the `examples/` programs and freezes the output.
- **Robustness tests**: a deterministic mutation fuzzer (xorshift, no external
  deps) runs 2 000 mutated copies of `examples/tour.nex` and asserts the lexer
  always terminates, never panics, and always ends with `Eof`.
- **CI** (GitHub Actions) runs `rustfmt --check` and clippy with
  `-D warnings` on ubuntu, and the full test suite on ubuntu + windows.
  The `justfile` (`just check`) and `scripts/check.ps1` run the same three
  steps locally.

## Deferred / known gaps

- `ariadne` diagnostics — step 3.11
- CI does not yet install LLVM — needed from step 8.2
- No benchmarks yet — parser throughput baseline is due at step 4.12
- Block comments and character literals are not in the language (Phase 7)

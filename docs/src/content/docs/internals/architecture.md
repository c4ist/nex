---
title: Compiler Architecture
description: Workspace layout and the design decisions already locked in.
---

:::note[Status: foundations in place]
This page describes the workspace layout and the design decisions that
are already locked in. Parsing has started (scaffolding only, step 3.1);
name resolution, type checking, and codegen are planned but not yet written.
:::

## Workspace layout

Nex is a Cargo workspace with three crates:

| Crate            | Responsibility                         | Dependencies              |
| ----------------- | --------------------------------------- | -------------------------- |
| `nex-lexer`      | source text → tokens (Phase 1, done)   | none                      |
| `nex-syntax`     | AST types and the future parser        | `nex-lexer` (for `Span`)  |
| `nex-driver`     | the `nex` CLI                          | `nex-lexer`, `clap`       |

Dependencies are pinned exactly (e.g. `clap =4.5.23`, `insta =1.41.1`) because
Cargo only became MSRV-aware in 1.84 and the workspace MSRV is 1.83.

## The compilation pipeline

```text
source text
   │  nex-lexer          (Phase 1 — done)
   ▼
tokens
   │  nex-syntax parser  (Phases 3–4 — scaffolding done, 3.1)
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

The AST now covers expressions, statements, items (fn/struct/enum/use/mod/impl
stub), types (named/generic/array/reference/fn), and patterns
(wildcard/binding/literal/enum-variant/struct/tuple) — the full node set
phase 2 set out to define. `Module` is the root: one parsed file's `Vec<Item>`
plus its own `NodeInfo`.

The [AST coverage review](/internals/ast-coverage/) tracks real gaps found
against the spec — notably, expressions can't yet hold a `::`-qualified path
(`Option::Some`), so the spec's own sample match statement can't be built as
an `Expr` today.

## The parser (`nex-syntax`, step 3.1)

`Parser` wraps a `&[Token]` cursor with `peek`/`advance`/`expect` and
collects recoverable `ParseError`s instead of aborting on the first mistake.
`advance()` is a no-op once the cursor reaches `Eof`, so callers can't walk
off the end of the token stream. `parse_module()` is the entry point; item
parsing itself starts in Phase 4, so for now it just reports one "item
parsing arrives in Phase 4" error per leftover token and consumes it — an
empty file parses to an empty `Module` cleanly, anything else terminates
without a real result yet.

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
  that lexes the `examples/` programs and freezes the output. `nex-syntax`
  has its own `Debug`-round-trip snapshot suites per node family
  (`expr_debug.rs`, `stmt_debug.rs`, `item_debug.rs`, `type_debug.rs`).
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

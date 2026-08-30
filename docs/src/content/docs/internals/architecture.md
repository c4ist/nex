---
title: Compiler Architecture
description: Workspace layout and the design decisions already locked in.
---

The lexer and parser work: `nex parse` turns a source file into a syntax tree.
Name resolution, type checking and codegen are still to come.

## Workspace layout

| Crate | Responsibility | Depends on |
| ----- | -------------- | ---------- |
| `nex-lexer` | source text to tokens | nothing |
| `nex-syntax` | AST types and the parser | `nex-lexer` |
| `nex-interp` | the tree-walking interpreter | `nex-lexer`, `nex-syntax` |
| `nex-driver` | the `nex` CLI | `nex-lexer`, `nex-syntax`, `clap`, `ariadne` |

Dependency versions are pinned exactly (`clap =4.5.23`, `ariadne =0.5.1`,
`insta =1.41.1`). Cargo only became MSRV-aware in 1.84 and the workspace MSRV
is 1.83, so anything looser can resolve to a version that won't build.

## The compilation pipeline

```text
source text
   │  nex-lexer                        done
   ▼
tokens
   │  nex-syntax parser                done
   ▼
AST (immutable)
   │  name resolution + type checker   planned
   ▼
checked AST
   ├── tree-walking interpreter        planned
   ├── LLVM native codegen             planned
   └── WebAssembly codegen             planned
```

Every backend has to agree. The regression corpus runs under all of them and
the outputs must match.

## AST design

The node plumbing lives in `nex-syntax/src/node.rs`. Two rules hold across the
whole tree:

1. Every node has a `Span`, so later passes can always point at the source text
   that caused something.
2. Every node has a unique `NodeId`, so later passes can hang information off a
   node in side tables instead of mutating it. The AST is immutable once
   parsed.

The types that implement this:

- `NodeId(u32)`, handed out in order by `NodeIdGen` while parsing a module.
  They're dense, so they double as indices into side tables (`Vec<T>` keyed by
  `id.index()`).
- `NodeId::DUMMY` marks nodes invented during error recovery. It has no
  side-table slot and panics on `.index()` rather than silently corrupting one.
- `NodeInfo { id, span }`, embedded in every node.
- `Spanned<T>` for values that need a span but not an identity: idents, field
  names, operators.
- `Ident`, an alias for `Spanned<String>`.
- `spanning(items, fallback)` builds a parent's span from its children.

So type information, resolved names and lowering results all belong in side
tables keyed by `NodeId`, never in the tree.

`Module` is the root: one file's `Vec<Item>` plus its own `NodeInfo`. The node
set itself covers expressions, statements, items, types and patterns. See [AST
coverage](/internals/ast-coverage/) for what it still can't represent.

## The parser

`Parser` wraps a `&[Token]` cursor with `peek`, `advance` and `expect`. It
collects `ParseError`s rather than bailing on the first mistake, so one bad
statement doesn't hide the rest of the file: `synchronize()` skips to the next
`;` or `}` and parsing continues.

Expressions use a Pratt loop. `infix_binding_power` holds the whole precedence
table in one place, running from `||` at the loosest end to `*`, `/` and `%` at
the tightest. Prefix operators bind tighter than any of them, postfix calls and
field access tighter still, and ranges looser than all of it.

One thing to know when reading it: `advance()` is a no-op at `Eof`, so any loop
over tokens needs its own `Eof` case or it will spin.

`parse_module()` is the entry point, and `parse_item()` handles `fn`, `struct`,
`enum`, `use` and `mod`, each optionally `pub`. An inline `mod { }` body goes
back through `parse_item`, so modules nest.

## The CLI

`nex` is built on clap. `lex` dumps the token stream and `parse` dumps the
syntax tree as s-expressions; both are development aids. `build`, `run`,
`check`, `fmt` and `test` exist but exit with a message naming the phase that
delivers them.

Diagnostics go through `ariadne` (`nex-driver/src/diag.rs`). One wrinkle: nex
spans are byte offsets while ariadne counts characters, so `diag.rs` converts
between them. Skip that and the reported column drifts on any line with
multi-byte characters in it.

## Testing

Unit tests sit next to the code. Integration tests in `crates/nex-lexer/tests/`
cover literals, operators, trivia, keywords and error recovery, plus an insta
snapshot suite that freezes the token stream for the `examples/` programs.
`nex-syntax` has a `Debug` round-trip suite per node family and separate suites
for the parser.

A deterministic mutation fuzzer (xorshift, no dependencies) runs 2,000 mutated
copies of `examples/tour.nex` and checks the lexer always terminates, never
panics, and always ends with `Eof`.

CI runs `rustfmt --check` and clippy with `-D warnings` on ubuntu, and the test
suite on ubuntu and windows. `just check` and `scripts/check.ps1` run the same
three steps locally.

## Known gaps

- CI doesn't install LLVM yet; needed once codegen starts.
- Parser throughput is ~948k lines/sec in release on a 10k-line file.
- Block comments and character literals aren't in the language yet.

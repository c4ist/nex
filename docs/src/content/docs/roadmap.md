---
title: Roadmap
description: Sixteen phases from lexer to a tagged v0.1.0 release.
---

Sixteen phases from lexer to a tagged `v0.1.0` release with a package manager,
language server, standard library and documentation site. Progress is tracked
one micro-step at a time in [`progress.txt`](https://github.com/c4ist/nex/blob/main/progress.txt)
— one micro-step per commit, no skipping ahead.

## Current position

- **last step:** 3.11
- **current phase:** 4 (Parser: statements & items) — starting
- **next step:** 4.1 — `let` / `let mut` statements

## Phase status

| Phase | Milestone                          | State       |
| ----- | ------------------------------------ | ----------- |
| 0     | Foundations                        | done        |
| 1     | Lexer                              | done        |
| 2     | AST design                         | done        |
| 3     | Parser: expressions                | done        |
| 4     | Parser: statements & items         | planned     |
| 5     | Tree-walking interpreter           | planned     |
| 6     | Name resolution + type checker     | planned     |
| 7     | Language feature wave 2            | planned     |
| 8     | LLVM backend                       | planned     |
| 9     | WASM target                        | planned     |
| 10    | Standard library                   | planned     |
| 11    | Package manager                    | planned     |
| 12    | Editor highlighting                | planned     |
| 13    | Language server                    | planned     |
| 14    | Documentation site                 | planned     |
| 15    | Polish + v0.1.0                    | planned     |

## What each phase delivers

- **0 — Foundations:** workspace, CI, examples, progress tracking.
- **1 — Lexer:** full token stream with error recovery (done; see the
  [lexical structure reference](/reference/lexical-structure/)).
- **2 — AST design:** node identity (`NodeId`), spans, the expression,
  statement, item, type, and pattern node types, an s-expression pretty
  printer, and a coverage review against the spec (done; see
  [AST Coverage](/internals/ast-coverage/) for what's still missing).
- **3–4 — Parser:** expressions first, then statements and items, with error
  recovery and `ariadne` diagnostics (step 3.11).
- **5 — Tree-walking interpreter:** `nex run`.
- **6 — Type checker:** name resolution and static typing; language v0.1
  becomes real.
- **7 — Feature wave 2:** block comments, character literals, and the other
  missing pieces; the language spec freezes as v0.1 at the end of this phase.
- **8–9 — Backends:** LLVM native codegen (`nex build`) and WebAssembly
  (`nex build --target wasm32`); all backends must produce identical output on
  the regression corpus.
- **10–11 — Standard library and package manager:** `nex test` lands with the
  stdlib + test runner.
- **12–14 — Tooling:** editor highlighting, language server, and this
  documentation site.
- **15 — Polish:** formatter (`nex fmt`), benchmarks, and the `v0.1.0` tag.

## Deferred / pending

- CI does not yet install LLVM — needed from step 8.2.
- No benchmarks yet — parser throughput baseline is due at step 4.12.
- Block comments (`/* */`) arrive in Phase 7; they currently lex as operators.
- Character literals (`'a'`) are not in the language.

## Design notes that shape later phases

- `NodeId`s are dense and sequential so they can index side tables directly.
  Later passes store their results in side tables keyed by `NodeId` rather
  than mutating the AST, which stays immutable after parsing.
- `NodeId::DUMMY` marks nodes synthesised during error recovery; it panics on
  `.index()` so a dummy can never silently corrupt a side table.
- Memory management is decided in Phase 8. Current plan: automatic reference
  counting emitted by the code generator for heap values (`str`, arrays, boxed
  enums). No ownership or borrow checking in v0.1.

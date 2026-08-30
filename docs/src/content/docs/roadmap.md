---
title: Roadmap
description: Sixteen phases from lexer to a tagged v0.1.0 release.
---

Sixteen phases from lexer to a tagged `v0.1.0` release with a package manager,
language server, standard library and documentation site. Progress is tracked
one micro-step at a time in [`progress.txt`](https://github.com/c4ist/nex/blob/main/progress.txt),
one step per commit, no skipping ahead.

## Current position

- **last step:** 4.9
- **current phase:** 4 (parser: statements and items)
- **next step:** 4.10, a `nex parse` subcommand

## Phase status

| Phase | Milestone                          | State       |
| ----- | ------------------------------------ | ----------- |
| 0     | Foundations                        | done        |
| 1     | Lexer                              | done        |
| 2     | AST design                         | done        |
| 3     | Parser: expressions                | done        |
| 4     | Parser: statements & items         | in progress |
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

**0, Foundations.** Workspace, CI, examples, progress tracking.

**1, Lexer.** The full token stream, with error recovery. Done; see the
[lexical structure reference](/reference/lexical-structure/).

**2, AST design.** Node identity, spans, and the node types for expressions,
statements, items, types and patterns, plus an s-expression pretty printer for
snapshot tests. Done; [AST coverage](/internals/ast-coverage/) lists what the
tree still can't represent.

**3 and 4, Parser.** Expressions first, then statements and items. Error
recovery and `ariadne` diagnostics land at the end of phase 3.

**5, Tree-walking interpreter.** `nex run` starts working.

**6, Type checker.** Name resolution and static typing. This is where v0.1 of
the language becomes real.

**7, Feature wave 2.** Block comments, character literals and the other missing
pieces. The spec freezes as v0.1 at the end of this phase.

**8 and 9, Backends.** LLVM native codegen for `nex build`, then WebAssembly
via `--target wasm32`. Every backend has to produce identical output on the
regression corpus.

**10 and 11, Standard library and package manager.** `nex test` arrives
alongside the stdlib and its test runner.

**12 to 14, Tooling.** Editor highlighting, the language server, and this
documentation site.

**15, Polish.** The `nex fmt` formatter, benchmarks, and the `v0.1.0` tag.

## Deferred / pending

- CI doesn't install LLVM yet. Needed from step 8.2.
- No benchmarks. A parser throughput baseline is due at step 4.12.
- Block comments (`/* */`) arrive in Phase 7; they currently lex as operators.
- Character literals (`'a'`) are not in the language.

## Design notes that shape later phases

- `NodeId`s are dense and sequential so they can index side tables directly.
  Later passes store their results in side tables keyed by `NodeId` rather
  than mutating the AST, which stays immutable after parsing.
- `NodeId::DUMMY` marks nodes invented during error recovery. It panics on
  `.index()` so a dummy can't silently corrupt a side table.
- Memory management is decided in Phase 8. Current plan: automatic reference
  counting emitted by the code generator for heap values (`str`, arrays, boxed
  enums). No ownership or borrow checking in v0.1.

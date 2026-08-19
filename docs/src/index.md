# Nex

> **Status: pre-alpha.** The lexer works. Nothing runs yet.
> Implementation progress is tracked in [`progress.txt`](../../progress.txt) and
> the [roadmap](roadmap.md) page.

Nex is a lightweight, statically typed, compiled programming language. It aims
to be a small language with a small runtime: C/Go/Rust-flavoured syntax, static
types with local inference, structs, enums and pattern matching, and no build
boilerplate.

```nex
fn main() {
    print("hello, world");
}
```

## Design goals

- **Lightweight.** Small language, small runtime, fast compiles.
- **Statically typed.** Everything is checked before it runs.
- **Familiar.** C/Go/Rust-flavoured syntax; no surprises.
- **No ceremony.** Local type inference, no header files, no build boilerplate.

Non-goals for v0.1: ownership/borrow checking, lifetimes, traits with dynamic
dispatch, async, threads, macros, a central package registry.

See [Language Design](language-design.md) for the full spec.

## How it will run

Programs are interpreted during development and compiled to native code through
LLVM (and to WebAssembly) for release:

1. **Phase 5** — tree-walking interpreter (`nex run`)
2. **Phase 8** — LLVM native code generation (`nex build`)
3. **Phase 9** — WebAssembly target (`nex build --target wasm32`)

All three backends must agree: the regression corpus is run under every
available backend and the outputs must be identical.

## What works today

| Area           | Status                                             |
| -------------- | -------------------------------------------------- |
| Lexer          | done — full token stream with error recovery       |
| AST foundation | in progress — node identity and spans exist        |
| Parser         | not started (Phase 3)                              |
| Interpreter    | not started (Phase 5)                              |
| Type checker   | not started (Phase 6)                              |
| Codegen        | not started (Phases 8–9)                           |
| CLI            | `nex lex` works; `build`/`run`/`check`/`fmt`/`test` are stubs |

The only working subcommand today dumps the token stream:

```sh
cargo run -p nex-driver -- lex examples/hello.nex
```

See [Getting Started](getting-started.md) and the [lexical structure
reference](reference/lexical-structure.md) for details.

## Repository layout

| Path                 | What it is                                    |
| -------------------- | --------------------------------------------- |
| `crates/nex-lexer`   | Source text to tokens                         |
| `crates/nex-syntax`  | AST node plumbing (parser arrives in Phase 3) |
| `crates/nex-driver`  | The `nex` command line tool                   |
| `examples/`          | Sample `.nex` programs used by the test suite |
| `docs/`              | This documentation site                       |
| `progress.txt`       | Which roadmap step is done, and what is next  |

## License

MIT OR Apache-2.0

# nex

A lightweight, statically typed, compiled programming language.

> **Status: pre-alpha.** The lexer works. Nothing runs yet.
> Follow along in [`progress.txt`](progress.txt).

```nex
fn main() {
    print("hello, world");
}
```

## Design in one paragraph

Nex aims to be a small language with a small runtime: C/Go/Rust-flavoured
syntax, static types with local inference, structs, enums and pattern matching,
and no build boilerplate. Programs are interpreted during development and
compiled to native code through LLVM (and to WebAssembly) for release. See
[`docs/src/language-design.md`](docs/src/language-design.md) for the spec.

## Repository layout

| Path                 | What it is                                    |
| -------------------- | --------------------------------------------- |
| `crates/nex-lexer`   | Source text to tokens                         |
| `crates/nex-driver`  | The `nex` command line tool                   |
| `examples/`          | Sample `.nex` programs used by the test suite |
| `docs/`              | Language specification and documentation site |
| `progress.txt`       | Which roadmap step is done, and what is next  |

## Building

Requires a stable Rust toolchain (1.83 or newer).

```sh
cargo build --workspace
cargo test  --workspace
```

Or run the full CI suite locally:

```sh
just check                # with `just` installed
./scripts/check.ps1       # Windows PowerShell
```

## Trying it out

The only working subcommand today dumps the token stream:

```sh
cargo run -p nex-driver -- lex examples/hello.nex
```

## Roadmap

Sixteen phases, from lexer to a tagged `v0.1.0` release with a package manager,
language server, standard library and documentation site. Progress is tracked
one micro-step at a time in `progress.txt`.

| Phase | Milestone                        | State       |
| ----- | -------------------------------- | ----------- |
| 0     | Foundations                      | done        |
| 1     | Lexer                            | done        |
| 2–4   | AST and parser                   | next        |
| 5     | Tree-walking interpreter         | planned     |
| 6–7   | Type checker and language v0.1   | planned     |
| 8–9   | LLVM and WebAssembly backends    | planned     |
| 10–11 | Standard library, package manager| planned     |
| 12–14 | Editor tooling, LSP, docs site   | planned     |
| 15    | Polish and release               | planned     |

## License

MIT OR Apache-2.0

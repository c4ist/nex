#  💽 nex 

A lightweight, statically typed, compiled programming language.

> **Status: pre-alpha.** The lexer and expression parser work. Nothing runs
> yet. Follow along in [`progress.txt`](progress.txt).

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
[`docs/src/content/docs/language-design.md`](docs/src/content/docs/language-design.md)
for the spec.

## Repository layout

| Path                 | What it is                                    |
| -------------------- | --------------------------------------------- |
| `crates/nex-lexer`   | Source text to tokens                         |
| `crates/nex-syntax`  | AST types and the parser                      |
| `crates/nex-driver`  | The `nex` command line tool                   |
| `examples/`          | Sample `.nex` programs used by the test suite |
| `docs/`              | Language specification and documentation site |
| `progress.txt`       | Which roadmap step is done, and what is next  |

## Documentation

The documentation site lives in [`docs/`](docs/src/content/docs/index.mdx) (an
[Astro Starlight](https://starlight.astro.build) project):

```sh
cd docs
npm install
npm run dev
```

It covers the [language spec](docs/src/content/docs/language-design.md), the
[lexical structure reference](docs/src/content/docs/reference/lexical-structure.md),
the [compiler architecture](docs/src/content/docs/internals/architecture.md)
and the [roadmap](docs/src/content/docs/roadmap.md).

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
| 2     | AST design                       | done        |
| 3     | Parser: expressions              | done        |
| 4     | Parser: statements and items     | in progress |
| 5     | Tree-walking interpreter         | planned     |
| 6–7   | Type checker and language v0.1   | planned     |
| 8–9   | LLVM and WebAssembly backends    | planned     |
| 10–11 | Standard library, package manager| planned     |
| 12–14 | Editor tooling, LSP, docs site   | planned     |
| 15    | Polish and release               | planned     |

## Contributing

Nex is built in small numbered steps, one per commit, tracked in
[`progress.txt`](progress.txt). If you want to take one on, open an issue
first. Bug reports, tests and docs are welcome any time. See
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

Dual licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option. Contributions are accepted under the same terms.

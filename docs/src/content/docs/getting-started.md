---
title: Getting Started
description: Build the nex compiler, run the lexer, and find your way around the CLI and docs site.
---

## Requirements

- A stable Rust toolchain, version **1.83 or newer** (the workspace's MSRV).
  `rust-toolchain.toml` pins `stable` with `rustfmt` and `clippy` components.

## Building

```sh
cargo build --workspace
cargo test  --workspace
```

Or run the full CI suite locally:

```sh
just check                # with `just` installed (see justfile)
./scripts/check.ps1       # Windows PowerShell
```

Both run `cargo fmt --check`, `cargo clippy -- -D warnings`, and
`cargo test --workspace --all-targets`, in that order. These are the checks
GitHub Actions CI (rustfmt + clippy on ubuntu, tests on ubuntu and windows).

## Trying it out

The `nex` binary lives in `crates/nex-driver`. Today only one subcommand works:
`lex`, which scans a source file and dumps its token stream. It is a developer
aid and doubles as the output format for the snapshot tests, so the format is
stable.

```sh
cargo run -p nex-driver -- lex examples/hello.nex
```

Output is one `Kind@start..end` line per token, with spans as half-open byte
offsets into the file:

```text
Fn@32..34
Ident("main")@35..39
LParen@39..40
RParen@40..41
LBrace@42..43
Ident("print")@48..53
LParen@53..54
Str("hello, world")@54..68
RParen@68..69
Semi@69..70
RBrace@71..72
Eof@73..73
```

(The file starts with a comment line, so the first token begins at byte 32.)

If the file has lexical errors, the command prints rendered diagnostics to
stderr and exits non-zero:

```text
error: unexpected character `@`
  --> main.nex:1:11
  |
1 | fn main() @
  |           ^
```

The lexer never stops early: one run reports *every* lexical error in the file.
See the [lexical structure reference](/reference/lexical-structure/) for the
full token and error catalogue.

## Example programs

The `examples/` directory contains the sample programs used by the test suite:

- `hello.nex`: minimal hello world
- `tour.nex`: a tour of the planned syntax, covering structs, enums, generics, `const`,
  functions, `if`/`for`/`while`, ranges, `match`, and string escapes

They lex and parse cleanly today. The interpreter can already run functions,
but `nex run` isn't wired to it yet.

## What doesn't work yet

The CLI defines the full command surface, but every subcommand except `lex`
returns "not implemented yet" with the phase it arrives in:

| Command             | Purpose                              | Arrives in |
| -------------------- | ------------------------------------ | ---------- |
| `nex build [--out]` | compile to a native executable       | Phase 8 (LLVM backend) |
| `nex run`           | type-check and run a program         | Phase 5 (interpreter) |
| `nex check`         | type-check without running           | Phase 6 (type checker) |
| `nex fmt [--check]` | reformat source files                | Phase 15 (formatter) |
| `nex test [filter]` | run the tests in a program           | Phase 10 (stdlib + test runner) |

## Building the docs site

This site is a [Starlight](https://starlight.astro.build) project under
`docs/`. To render it locally:

```sh
cd docs
npm install
npm run dev
```

Then open http://localhost:4321.

### Deploying to Vercel

The docs site lives in `docs/`, so the Vercel project must be rooted there:

1. Push the repo to GitHub and import it at https://vercel.com/new.
2. Set the root directory to `docs`. This is the one that matters.
3. Framework preset: **Astro** (auto-detected from `package.json`).
4. Deploy.

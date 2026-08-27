# Contributing

Thanks for taking a look. Nex is pre-alpha and moving fast, so read this
before you spend real time on a change.

## The one thing that's unusual here

Nex is built in small numbered steps, one per commit, tracked in
[`progress.txt`](progress.txt). Each step has an acceptance check, and steps
land in order.

That means a pull request implementing five steps at once, or jumping ahead to
a phase that isn't started yet, is hard to review and probably won't be merged
as-is. If you want to work on a step, open an issue first so we don't both
write it.

Fixes, tests, docs and bug reports don't have this constraint. Go ahead.

## Getting set up

You need a stable Rust toolchain, 1.83 or newer.

```sh
git clone https://github.com/c4ist/nex
cd nex
cargo build --workspace
cargo test --workspace
```

The docs site is a separate Astro project:

```sh
cd docs
npm install
npm run dev
```

## Before you open a pull request

Run the same three checks CI runs:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

Or all at once:

```sh
just check              # if you have `just`
./scripts/check.ps1     # windows
```

All three have to pass. Clippy is deny-warnings, including a cognitive
complexity limit, so a function that's grown too tangled will fail the build
rather than just look bad.

## House style

- Comments are lowercase and short. Explain the thing that isn't obvious from
  reading the code, and skip the rest. If a function name says it, don't
  repeat it in a doc comment.
- Every compiler error carries a `Span`. Diagnostics point at source.
- No `unwrap()` outside tests.
- Every language feature needs tests: lexer or parser snapshots now, checker
  and interpreter tests once those exist.

The lexer and AST modules are the reference for tone if you're unsure.

## Tests

Unit tests sit next to the code. Integration tests live in each crate's
`tests/` directory. Snapshot tests use [insta](https://insta.rs):

```sh
cargo test --workspace              # run everything
INSTA_UPDATE=always cargo test      # accept new snapshots, then read the diff
```

Read the snapshot diff before you commit it. Accepting a snapshot you haven't
looked at is how a bug becomes the expected output.

## Reporting bugs

Include the input that broke it. For a compiler that usually means a short
`.nex` file plus what you ran and what you expected. `cargo run -p nex-driver
-- lex yourfile.nex` output helps if it's a lexing problem.

## Where things are

| Path | What it is |
| ---- | ---------- |
| `crates/nex-lexer` | source text to tokens |
| `crates/nex-syntax` | AST types and the parser |
| `crates/nex-driver` | the `nex` CLI |
| `docs/` | the documentation site |
| `examples/` | sample `.nex` programs used by tests |
| `progress.txt` | which step is done, and what's next |

[docs/src/content/docs/internals/architecture.md](docs/src/content/docs/internals/architecture.md)
explains how the pieces fit together.

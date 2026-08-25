---
name: run-nex
description: Build, run, and smoke-test the nex compiler CLI (the `nex` binary in crates/nex-driver). Use when asked to run nex, build nex, lex a .nex file, screenshot/verify a nex CLI change, or check that nex still works after an edit.
---

Nex is a Rust CLI tool (`crates/nex-driver`, binary name `nex`), not a GUI or
web app. It's driven by invoking the binary and reading stdout/stderr/exit
code — no browser or REPL needed. All paths below are relative to the repo
root (`C:\Users\boroc\CascadeProjects\nex`).

As of writing (roadmap phase 2, AST design), only the `lex` subcommand is
implemented. `build`, `run`, `check`, `fmt`, and `test` are stubs that print
`` `<name>` is not implemented yet; it arrives in Phase N `` to stderr and
exit 1. This is expected, not a bug — see `progress.txt` for the current
phase before assuming a stub command is broken.

## Prerequisites

Stable Rust toolchain 1.83+ (`rustc --version` / `cargo --version` to check;
pinned in `rust-toolchain.toml`). Nothing else — the workspace has no other
system dependencies. `just` is optional (not required — see below).

## Build

```bash
cargo build --workspace
```

## Run (agent path) — the driver

Use `.claude/skills/run-nex/smoke.sh`. It builds the workspace, runs the
real `lex` subcommand against both example programs, checks that the stub
subcommands fail the correct way, and runs the test suite:

```bash
bash .claude/skills/run-nex/smoke.sh
```

Expect `all smoke checks passed` at the end. This is the fastest way to
confirm a change hasn't broken the CLI surface or the lexer.

To drive the CLI directly instead of through the smoke script:

```bash
cargo run -q -p nex-driver -- lex examples/hello.nex   # dump token stream
cargo run -q -p nex-driver -- lex path/to/file.nex      # any .nex file
cargo run -q -p nex-driver --                            # no args -> usage, exit 2
```

`lex` output is one token per line: `Kind(payload)@start..end` (byte
offsets), terminated by an `Eof@n..n` line. Lexer errors (if any) print
after the token stream and cause a non-zero exit; see
`crates/nex-driver/src/main.rs::cmd_lex` for the exact format.

## Direct invocation (no CLI, just the lexer)

For lexer-only changes you don't need the CLI at all — call the crate
directly, e.g. in a test or a `cargo run --example`:

```rust
let (tokens, errors) = nex_lexer::tokenize(source_str);
```

This is what `crates/nex-driver/src/main.rs` itself does, and what
`crates/nex-lexer` and `crates/nex-syntax`'s own test suites do
(`golden.rs`, `literals.rs`, `expr_debug.rs` under each crate's `tests/`).

## Run (human path)

```bash
cargo run -p nex-driver -- lex examples/hello.nex
```

Same as the agent path, just without `-q`. There's no long-running server
or window to worry about — the binary runs and exits.

## Test

```bash
cargo test --workspace
```

Or, if `just` is installed (`cargo install just`; not installed by default
on this machine):

```
just check     # fmt-check + clippy + test, same as CI
just lex FILE   # shorthand for the lex command above
```

Windows without `just`: `./scripts/check.ps1` runs the same three steps.

## Gotchas

- **Stub subcommands are supposed to fail.** `nex run`, `nex build`,
  `nex check`, `nex fmt`, `nex test` all currently exit 1 with a
  "not implemented yet; it arrives in Phase N" message. Don't treat this
  as a regression — check `progress.txt` for which phase is current before
  concluding something broke.
- **No-args invocation exits 2**, not 0 or 1 (clap's usage-error convention),
  because `arg_required_else_help = true` in `main.rs`.
- **`just` isn't installed** in this environment (`cargo install just` if
  you want it) — use the plain `cargo` commands or `scripts/check.ps1`
  instead; don't assume `just check` works out of the box.
- Token stream lines are `Debug`-formatted (`{:?}` on `TokenKind` and
  `Span`), so the exact text is sensitive to any `Debug` impl changes —
  don't hardcode full lines in new tests, match on substrings/prefixes like
  the smoke script does.

## Troubleshooting

- `cannot read `<file>`: The system cannot find the file specified. (os
  error 2)` — the path passed to `lex` doesn't exist relative to the cwd
  you ran `cargo run` from (usually the repo root, not `crates/nex-driver`).

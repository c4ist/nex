# Task runner for the Nex repository. Requires `just` (cargo install just).
# On Windows without `just`, use scripts/check.ps1 instead.

default: check

# Everything CI runs, in CI order.
check: fmt-check lint test

test:
    cargo test --workspace --all-targets

lint:
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

# Regenerate the insta snapshots, then review the diff before committing.
snapshots:
    INSTA_UPDATE=always cargo test --workspace

# Dump the token stream for a source file.
lex file:
    cargo run -q -p nex-driver -- lex {{file}}

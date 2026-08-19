# task runner. needs `just` (cargo install just)
# on windows without it, use scripts/check.ps1

default: check

# everything ci runs, in ci order
check: fmt-check lint test

test:
    cargo test --workspace --all-targets

lint:
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

# regen the insta snapshots, then check the diff before committing
snapshots:
    INSTA_UPDATE=always cargo test --workspace

# dump the token stream for a file
lex file:
    cargo run -q -p nex-driver -- lex {{file}}

#!/usr/bin/env bash
# build the mdbook docs site for deployment (vercel, ci, whatever).
#
# downloads a pinned prebuilt mdbook binary instead of compiling it, so the
# build does not need a rust toolchain. keep MDBOOK_VERSION in sync with the
# mdbook you use locally (cargo install mdbook --version 0.4.52).
#
# usage: ./scripts/build-docs.sh [book-root]
#
# book-root defaults to `docs`, so from the repo root you can run
# ./scripts/build-docs.sh. vercel runs it as `bash ../scripts/build-docs.sh .`
# with the project root directory set to `docs`.

set -euo pipefail

MDBOOK_VERSION="0.4.52"
BOOK_ROOT="${1:-docs}"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/mdbook"
BIN="$CACHE_DIR/mdbook-$MDBOOK_VERSION"

if [ ! -x "$BIN" ]; then
    case "$(uname -s)-$(uname -m)" in
        Linux-x86_64)  asset="x86_64-unknown-linux-gnu" ;;
        Linux-aarch64) asset="aarch64-unknown-linux-gnu" ;;
        Darwin-x86_64) asset="x86_64-apple-darwin" ;;
        Darwin-arm64)  asset="aarch64-apple-darwin" ;;
        *) echo "build-docs.sh: unsupported platform $(uname -s)-$(uname -m)" >&2; exit 1 ;;
    esac

    mkdir -p "$CACHE_DIR"
    echo "==> downloading mdbook $MDBOOK_VERSION ($asset)"
    curl -fsSL \
        "https://github.com/rust-lang/mdBook/releases/download/v$MDBOOK_VERSION/mdbook-v$MDBOOK_VERSION-$asset.tar.gz" \
        | tar -xz -C "$CACHE_DIR"
    mv "$CACHE_DIR/mdbook" "$BIN"
fi

echo "==> building docs with mdbook $MDBOOK_VERSION"
"$BIN" build "$BOOK_ROOT"

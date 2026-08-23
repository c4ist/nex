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

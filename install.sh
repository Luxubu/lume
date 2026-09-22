#!/usr/bin/env bash
# Builds the compiler and puts `lume` where you can run it.
#
#   ./install.sh                 -> ~/.local/bin/lume
#   ./install.sh /usr/local/bin  -> somewhere else
#
# Needs a Rust toolchain: https://rustup.rs

set -eu
cd "$(dirname "$0")"
DEST="${1:-$HOME/.local/bin}"

command -v cargo >/dev/null || {
  echo "lume is compiled by rustc, so it needs a Rust toolchain."
  echo "install one from https://rustup.rs, then run this again."
  exit 1
}

echo "building the compiler (a minute or two the first time)..."
cargo build --release --manifest-path compiler/Cargo.toml

mkdir -p "$DEST"
cp compiler/target/release/lume "$DEST/lume"
echo "installed $("$DEST/lume" --version | head -1) -> $DEST/lume"

case ":$PATH:" in
  *":$DEST:"*) echo "try it: lume run examples/fib.lume" ;;
  *)
    echo
    echo "$DEST is not on your PATH yet. add this to your shell profile:"
    echo "    export PATH=\"$DEST:\$PATH\""
    echo "or run it by full path: $DEST/lume run examples/fib.lume"
    ;;
esac

#!/usr/bin/env bash
# Builds tools/lumecheck — the suite's checks of the docs, the compiler's
# messages and the editor grammar, written in Lume — and runs it:
#
#   tests/lumecheck.sh docs [-v] [-j N]
#   tests/lumecheck.sh coverage [--check N]
#   tests/lumecheck.sh keywords
#   tests/lumecheck.sh suite [--update] [-j N] [-x] [-v]   (what tests/run.sh runs)
#
# The build is cached, so only the first run after a change compiles.
set -u
cd "$(dirname "$0")/.."
if [ ! -x compiler/target/release/lume ]; then
  echo "building compiler..."
  (cd compiler && cargo build --release 2>&1 | grep -E "^error" -A 8) && true
fi
bin=compiler/target/lumecheck
built=$(compiler/target/release/lume build tools/lumecheck/main.lume -o "$bin" 2>&1) || { echo "lumecheck did not build:"; echo "$built"; exit 1; }
exec "$bin" "$@"

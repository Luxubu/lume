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
# on Windows (Git's bash), the folder of Git's Unix tools, as Windows names
# it: the suite runs `bash`, `diff` and `find` from there, not Windows' own
if command -v cygpath >/dev/null 2>&1; then
  export LUME_UNIX_BIN="$(cygpath -m /usr/bin)"
  export LUME_ROOT="$(pwd -W)"
  # and the temporary folder in a form both Git's tools and Lume read the
  # same way (`C:/...`): Git's `/tmp` is not the `\tmp` a Windows program sees
  export TMPDIR="$(cygpath -m "${TMPDIR:-/tmp}")"
fi
bin=compiler/target/lumecheck
built=$(compiler/target/release/lume build tools/lumecheck/main.lume -o "$bin" 2>&1) || { echo "lumecheck did not build:"; echo "$built"; exit 1; }
exec "$bin" "$@"

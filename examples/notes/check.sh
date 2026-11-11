#!/usr/bin/env bash
# notes, served on this machine to its own client, and its tests. Prints
# "skipped" where no local connection is allowed (some sandboxes do not).
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")" || exit 1
if ! python3 -c "
import socket
s = socket.socket(); s.bind(('127.0.0.1', 0)); s.listen(1)
socket.create_connection(s.getsockname(), timeout=2)" 2>/dev/null; then
  echo "skipped: no local connection allowed here"; exit 0
fi
echo "== tests"
"$LUME" test 2>/dev/null; echo "exit: $?"
echo "== demo"
"$LUME" run -- demo 2>/dev/null; echo "exit: $?"
echo "== usage"
"$LUME" run -- nothing 2>&1 | grep -v "^warning\|^ *[0-9]* *|\|^  -->\|^  help"; echo "exit: ${PIPESTATUS[0]}"
rm -rf .lume

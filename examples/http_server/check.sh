#!/usr/bin/env bash
# http_server answers its own requests on this machine. Prints "skipped"
# where no local connection is allowed (some sandboxes do not).
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")" || exit 1
if ! python3 -c "
import socket
s = socket.socket(); s.bind(('127.0.0.1', 0)); s.listen(1)
socket.create_connection(s.getsockname(), timeout=2)" 2>/dev/null; then
  echo "skipped: no local connection allowed here"; exit 0
fi
"$LUME" run 2>&1 | grep -v "^warning\|^ *|\|^  -->\|^  help"; echo "exit: ${PIPESTATUS[0]}"
rm -rf .lume

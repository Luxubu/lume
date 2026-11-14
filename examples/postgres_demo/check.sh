#!/usr/bin/env bash
# The postgres package against a real server: the one LUME_PG_URL names,
# or the one `docker run ... -p 127.0.0.1:5440:5432 postgres:16-alpine`
# with user, password and database `lume` gives (as CI starts it). Prints
# "skipped" where there is none, so the suite can tell that from a failure.
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")" || exit 1
url="${LUME_PG_URL:-postgresql://lume:lume@127.0.0.1:5440/lume}"
hostport=$(printf '%s' "$url" | sed -E 's#^[a-z]+://([^@]*@)?([^/:]+)(:([0-9]+))?.*#\2 \4#')
set -- $hostport
if ! python3 -c "import socket,sys; socket.create_connection((sys.argv[1], int(sys.argv[2] or 5432)), timeout=2)" "$1" "${2:-5432}" 2>/dev/null; then
  echo "skipped: no PostgreSQL to try (set LUME_PG_URL)"; exit 0
fi
echo "== package tests"
(cd ../../packages/postgres && "$LUME" test 2>/dev/null; echo "exit: $?"; rm -rf .lume)
echo "== against the server"
LUME_PG_URL="$url" "$LUME" run 2>/dev/null; echo "exit: $?"
rm -rf .lume

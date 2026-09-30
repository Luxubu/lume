#!/usr/bin/env bash
# healthcheck against a server on this machine: a fast page, three slow
# ones, one that fails, one that fails once and then works, and one with the
# wrong body. Prints "skipped" where no local connection is allowed.
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")" || exit 1
tmp=$(mktemp -d)
cat > "$tmp/server.py" <<'PY'
import http.server, socketserver, time, threading
seen = {}
class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def answer(self, code, body):
        b = body.encode()
        self.send_response(code); self.send_header("Content-Length", str(len(b))); self.end_headers(); self.wfile.write(b)
    def do_GET(self):
        p = self.path
        seen[p] = seen.get(p, 0) + 1
        if p == "/": self.answer(200, "welcome home")
        elif p.startswith("/slow"): time.sleep(0.3); self.answer(200, "slow but fine")
        elif p == "/boom": self.answer(500, "broken")
        elif p == "/flaky": self.answer(503 if seen[p] == 1 else 200, "ok now")
        elif p == "/text": self.answer(200, "hello")
        else: self.answer(404, "no")
class S(socketserver.ThreadingMixIn, socketserver.TCPServer): daemon_threads = True
with S(("127.0.0.1", 0), H) as s:
    print(s.server_address[1], flush=True); s.serve_forever()
PY
python3 "$tmp/server.py" > "$tmp/port" 2>/dev/null &
pid=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do [ -s "$tmp/port" ] && break; sleep 0.2; done
port=$(cat "$tmp/port")
if ! python3 -c "import socket,sys; socket.create_connection(('127.0.0.1', int(sys.argv[1])), timeout=2)" "$port" 2>/dev/null; then
  echo "skipped: no local connection allowed here"; kill $pid; wait $pid 2>/dev/null; rm -rf "$tmp"; exit 0
fi
cat > "$tmp/checks.json" <<JSON
{"base": "http://127.0.0.1:$port",
 "checks": [
   {"name": "home", "path": "/", "contains": "welcome"},
   {"name": "slow-a", "path": "/slow-a", "within_ms": 2000},
   {"name": "slow-b", "path": "/slow-b"},
   {"name": "slow-c", "path": "/slow-c"},
   {"name": "boom", "path": "/boom"},
   {"name": "flaky", "path": "/flaky"},
   {"name": "text", "path": "/text", "contains": "welcome"},
   {"name": "strict", "path": "/slow-d", "within_ms": 100}
 ]}
JSON
"$LUME" build -o "$tmp/hc" >/dev/null 2>&1
start=$(python3 -c 'import time; print(time.time())')
"$tmp/hc" "$tmp/checks.json" --json 2>&1; echo "exit: $?"
python3 -c "import time; print('the slow checks ran together:', time.time() - $start < 1.5)"
printf '{"base": "x", "checks": [{"name": "a", "path": "/", "status": "200"}]}' > "$tmp/bad.json"
"$tmp/hc" "$tmp/bad.json" 2>&1 | sed "s#$tmp#<tmp>#"; echo "exit: ${PIPESTATUS[0]}"
kill $pid; wait $pid 2>/dev/null; rm -rf "$tmp" .lume

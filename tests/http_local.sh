#!/usr/bin/env bash
# The http package against a real server on this machine: starts one on a
# free port, runs examples/http_demo against it, stops it. Prints
# "skipped" when this machine does not allow a local connection (some
# sandboxes do not), so the suite can tell that apart from a failure.
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")/.." || exit 1
tmp=$(mktemp -d)
cat > "$tmp/server.py" <<'PY'
import http.server, json, socketserver, sys
class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def send(self, code, body, ctype):
        b = body.encode()
        self.send_response(code); self.send_header("Content-Type", ctype); self.send_header("Content-Length", str(len(b))); self.end_headers(); self.wfile.write(b)
    def do_GET(self):
        if self.path == "/hello.json": self.send(200, '{"ok": true}', "application/json")
        else: self.send(404, "not here", "text/plain")
    def do_POST(self):
        n = int(self.headers.get("Content-Length", 0)); body = self.rfile.read(n).decode()
        self.send(200, json.dumps({"type": self.headers.get("Content-Type"), "body": json.loads(body)}), "application/json")
with socketserver.TCPServer(("127.0.0.1", 0), H) as s:
    print(s.server_address[1], flush=True); s.serve_forever()
PY
python3 "$tmp/server.py" > "$tmp/port" 2>/dev/null &
pid=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do [ -s "$tmp/port" ] && break; sleep 0.2; done
port=$(cat "$tmp/port")
if ! python3 -c "import socket,sys; s=socket.create_connection(('127.0.0.1', int(sys.argv[1])), timeout=2)" "$port" 2>/dev/null; then
  echo "skipped: no local connection allowed here"
  kill $pid 2>/dev/null; wait $pid 2>/dev/null; rm -rf "$tmp"; exit 0
fi
(cd examples/http_demo && "$LUME" run -- "http://127.0.0.1:$port" 2>&1 | grep -v "^warning\|^ *|\|^  -->\|^  help"; echo "exit: ${PIPESTATUS[0]}")
kill $pid 2>/dev/null; wait $pid 2>/dev/null; rm -rf "$tmp" examples/http_demo/.lume

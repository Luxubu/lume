#!/usr/bin/env python3
"""Drives `lume lsp` the way an editor does and prints what comes back,
with the temporary folder written as <tmp>, for tests/run.sh to compare
with tests/lsp_test.expected."""
import json, os, subprocess, sys, tempfile

lume = os.path.abspath(sys.argv[1])
tmp = os.path.realpath(tempfile.mkdtemp())

def write(rel, text):
    p = os.path.join(tmp, rel)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    open(p, "w").write(text)
    return p

one = write("one/hello.lume", "def main:\n  puts 1\n")
helper = write("mods/helper.lume", "pub def twice(n: Int) -> Int = n * 2\n")
main = write("mods/main.lume", "import helper\n\ndef main:\n  puts helper.twice(2)\n")
write("pkg/lume.toml", '[package]\nname = "pkg"\nversion = "0.1.0"\n')
write("pkg/main.lume", "import util\n\ndef main:\n  puts util.hi\n")
util = write("pkg/util.lume", 'pub def hi -> Str = "hi"\n')

proc = subprocess.Popen([lume, "lsp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, cwd=tmp)
nid = [0]

def send(method, params, request=False):
    msg = {"jsonrpc": "2.0", "method": method, "params": params}
    if request:
        nid[0] += 1
        msg["id"] = nid[0]
    body = json.dumps(msg).encode()
    proc.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    proc.stdin.flush()

def recv():
    n = None
    while True:
        line = proc.stdout.readline().decode().strip()
        if not line:
            break
        if line.startswith("Content-Length:"):
            n = int(line.split(":")[1])
    return json.loads(proc.stdout.read(n))

def show(msg):
    s = json.dumps(msg, sort_keys=True).replace("file://" + tmp, "<tmp>")
    print(s)

def uri(p):
    return "file://" + p

def diagnostics_until(target):
    """Messages until the one for `target`; prints each."""
    while True:
        m = recv()
        show(m)
        if m.get("params", {}).get("uri") == uri(target):
            return

print("--- initialize")
send("initialize", {"processId": None, "rootUri": uri(tmp), "capabilities": {}}, True)
show(recv())
send("initialized", {})

print("--- a file with an error, then fixed")
send("textDocument/didOpen", {"textDocument": {"uri": uri(one), "languageId": "lume", "version": 1, "text": "def main:\n  puts nope + 1\n"}})
diagnostics_until(one)
send("textDocument/didChange", {"textDocument": {"uri": uri(one), "version": 2}, "contentChanges": [{"text": "def main:\n  puts 1 + 1\n"}]})
diagnostics_until(one)

print("--- a module, checked through the main.lume that imports it")
send("textDocument/didOpen", {"textDocument": {"uri": uri(helper), "languageId": "lume", "version": 1, "text": "pub def twice(n: Int) -> Int = n * \"2\"\n"}})
diagnostics_until(helper)

print("--- a module of a package, unsaved text")
send("textDocument/didOpen", {"textDocument": {"uri": uri(util), "languageId": "lume", "version": 1, "text": "pub def hi -> Str = 42\n"}})
diagnostics_until(util)
send("textDocument/didClose", {"textDocument": {"uri": uri(util)}})
diagnostics_until(util)

print("--- a request it does not answer, then shutdown")
send("textDocument/hover", {"textDocument": {"uri": uri(one)}, "position": {"line": 0, "character": 0}}, True)
show(recv())
send("shutdown", None, True)
show(recv())
send("exit", None)
print("exit:", proc.wait(timeout=30))

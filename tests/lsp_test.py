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

print("--- the module fixed again")
send("textDocument/didChange", {"textDocument": {"uri": uri(helper), "version": 2}, "contentChanges": [{"text": "pub def twice(n: Int) -> Int = n * 2\n"}]})
diagnostics_until(helper)

print("--- hover and go to definition")
nav_text = """import helper

struct Person:
  name: Str
  age: Int

  def greet -> Str = "hi #{name}"

def older(p: Person, by: Int) -> Person = Person(name: p.name, age: p.age + by)

def main:
  ann = Person(name: "ann", age: 30)
  var later = older(ann, 1)
  puts later.greet
  puts later.name.upcase
  puts helper.twice(2)
"""
nav = write("mods/nav.lume", nav_text)
send("textDocument/didOpen", {"textDocument": {"uri": uri(nav), "languageId": "lume", "version": 1, "text": nav_text}})
diagnostics_until(nav)
lines = nav_text.split("\n")

def at(token, nth=1, line_has=None):
    """The position of the `nth` `token` on the first line containing `line_has`."""
    for i, l in enumerate(lines):
        if line_has is None or line_has in l:
            k = -1
            for _ in range(nth):
                k = l.index(token, k + 1)
            return {"line": i, "character": k}
    raise KeyError(token)

probes = [
    ("a local", "ann", 1, "older(ann"),
    ("a var", "later", 1, "puts later.greet"),
    ("a parameter", "by", 2, "def older"),
    ("a function", "older", 1, "var later"),
    ("a constructor", "Person", 1, "ann = Person"),
    ("a method", "greet", 1, "puts later.greet"),
    ("a field", "name", 1, "later.name"),
    ("a field read bare in a method", "name", 1, "hi #{name}"),
    ("a built-in method", "upcase", 1, "upcase"),
    ("a function of another module", "twice", 1, "helper.twice"),
]
for what, tok, nth, line_has in probes:
    pos = at(tok, nth, line_has)
    send("textDocument/hover", {"textDocument": {"uri": uri(nav)}, "position": pos}, True)
    h = recv().get("result")
    shown = h["contents"]["value"].replace("```lume\n", "").replace("\n```", "") if h else None
    send("textDocument/definition", {"textDocument": {"uri": uri(nav)}, "position": pos}, True)
    d = recv().get("result")
    where = None
    if d:
        where = "%s:%d:%d" % (d["uri"].replace("file://" + tmp, "<tmp>"), d["range"]["start"]["line"] + 1, d["range"]["start"]["character"] + 1)
    print("%-30s hover %s | definition %s" % (what, json.dumps(shown), where))

print("--- a request it does not answer, then shutdown")
send("textDocument/completion", {"textDocument": {"uri": uri(one)}, "position": {"line": 0, "character": 0}}, True)
show(recv())
send("shutdown", None, True)
show(recv())
send("exit", None)
print("exit:", proc.wait(timeout=30))

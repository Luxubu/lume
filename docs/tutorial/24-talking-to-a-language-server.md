# Lunch 24: Talking to a language server

> **A preview.** This lunch talks to `lume lsp`, Lume's language server,
> whose answers are a preview: it works as shown — every example is checked
> — but what it says may change before 1.0. See [what is
> settled](../stability.md).

**In this lunch you will build `lspeek`**, a small program that drives a
**language server** — the program behind an editor's error underlines and
hovers — and asks it questions, as an editor would. The server is Lume's
own, `lume lsp`. The last lunch puts nearly everything together: a program
running alongside (lunch 18), messages framed by their length, JSON from a
package, structs, `?`, and async code.

> This lunch's program needs `lume` on your `PATH`, which the test suite cannot assume,
> so it was run by hand. Lume's own suite drives `lume lsp` the same way,
> with a longer version of this program: `tools/lumecheck/lsp.lume`.

## 24.1 How editors and servers talk

An editor and a language server talk over the server's input and output,
in the **Language Server Protocol**. Each message is a JSON document, and
before it comes a header saying how many bytes it is:

```text
Content-Length: 58

{"jsonrpc":"2.0","id":1,"method":"shutdown","params":null}
```

A header line, a blank line, then exactly that many bytes — with no line
ending after them, so the next header starts straight after. That is why
lunch 18 had `read(n)`.

The editor sends **requests**, which have an `id` and get an answer with
the same `id`, and **notices**, which have none and get no answer. The
server, for its part, sends notices of its own whenever it likes — such as
the errors it found in a file — so an answer may arrive after some of
those.

## 24.2 The package

`lspeek` uses the `json` package that comes with Lume, so it is a package
(lunch 19):

```toml
[package]
name = "lspeek"
version = "0.1.0"

[dependencies]
json = { git = "https://github.com/Luxubu/lume", tag = "v0.7.0" }
```

`json.obj([...])` makes a JSON object from pairs, `x.to_json` turns a
value into JSON, `json.show` writes a document as text and `json.parse`
reads one. A parsed document answers `doc.path("result.contents.value")`,
giving the value at that path if there is one. **New here**, all from the
`json` package: a JSON value is a `Json`, an enum whose variants include
`Null` and `Arr(items)` — a JSON array, made from a list; and `.str`,
`.int` and `.items` read a value as text, a whole number or a list, each
giving it if that is what the value is (a `T?`).

## 24.3 Sending and receiving

A `Server` holds the running program. Sending writes a header and a body;
receiving reads header lines up to the blank one, then exactly the body:

```lume-skip
import json
import json.Json

struct Server:
  child: Child

  def send(msg: Json) -> () or Error:
    body = json.show(msg)
    child.write("Content-Length: #{body.len}\r\n\r\n#{body}")?

  async def receive -> Json or Error:
    var size = 0
    while true:
      line = (await child.read_line).or_error("the server stopped")?
      break if line.trim.empty?
      if line.starts_with?("Content-Length:"):
        size = line.split(":").at(1).trim.to_int?
    body = (await child.read(size)).or_error("the server stopped in a message")?
    json.parse(body)
```

`Content-Length` counts bytes, and `body.len` counts characters (lunch 3).
They agree as long as the JSON is plain ASCII — true for everything this
program sends. (Lume's own suite writes every other character as a
`\u` escape, to keep it so.) Headers end in `\r\n`, as the protocol
says; `read_line` drops the `\r` along with the `\n`.

Requests and notices are JSON objects of a set shape:

```lume-skip
def request(id: Int, method: Str, params: Json) -> Json =
  json.obj([("jsonrpc", "2.0".to_json), ("id", id.to_json), ("method", method.to_json), ("params", params)])

def notice(method: Str, params: Json) -> Json =
  json.obj([("jsonrpc", "2.0".to_json), ("method", method.to_json), ("params", params)])
```

## 24.4 Waiting for an answer

Because the server sends notices of its own, the answer to a request is
the first message that comes back **with that request's `id`**. Anything
before it is handed to a block, so the caller can look at the notices too:

```lume-skip
  async def answer(id: Int, others: (Json) -> ()) -> Json or Error:
    while true:
      msg = (await receive)?
      if msg.path("id").map { |v| v.int == Some(id) }.or(false):
        return msg
      others(msg)
    Error("unreachable")
```

That method goes inside `Server`. A notice the server sends after a
change — **`textDocument/publishDiagnostics`** — carries the errors it found
in a file, which is what an editor underlines.

## 24.5 A conversation

`main` starts the server, says hello (`initialize`), opens a file with a
mistake in it, prints what the server finds, asks what the name under the
cursor is (`hover`), and says goodbye:

```lume-skip
DIR = "/tmp/lume-lunch-24"
# the program the editor has open, before and after its mistake is fixed;
# raw strings (lunch 22), so `#{` in them stays as written
TEXT = r"""
  def double(n: Int) -> Int = n * 2

  def main:
    puts double(4) + "!"
  """

FIXED = r"""
  def double(n: Int) -> Int = n * 2

  def main:
    total = double(4)
    puts "#{total}!"
  """

def show_diagnostics(msg: Json):
  if msg.path("method").map { |m| m.str == Some("textDocument/publishDiagnostics") }.or(false):
    found = msg.path("params.diagnostics").map { |x| x.items }.or([])
    puts "no problems" if found.empty?
    for d in found:
      line = d.path("range.start.line").map { |x| x.int.or(0) }.or(0)
      text = d.path("message").map { |x| x.str.or("") }.or("")
      puts "line #{line + 1}: #{text.lines.first.or("")}"

async def main -> () or Error:
  Dir.make(DIR)?
  file = "#{DIR}/hello.lume"
  File.write(file, TEXT)?
  uri = "file://#{file}"
  s = Server(child: Process.spawn_in(DIR, "lume", ["lsp"])?)

  s.send(request(1, "initialize", json.obj([("processId", Null), ("rootUri", "file://#{DIR}".to_json), ("capabilities", json.obj([]))])))?
  hello = (await s.answer(1) { |m| () })?
  name = hello.path("result.serverInfo.name").map { |x| x.str.or("?") }.or("?")
  version = hello.path("result.serverInfo.version").map { |x| x.str.or("?") }.or("?")
  puts "talking to #{name} #{version}"
  s.send(notice("initialized", json.obj([])))?

  doc = json.obj([("uri", uri.to_json), ("languageId", "lume".to_json), ("version", 1.to_json), ("text", TEXT.to_json)])
  s.send(notice("textDocument/didOpen", json.obj([("textDocument", doc)])))?

  # the mistake fixed, as an editor sends each change: the whole new text
  change = json.obj([("textDocument", json.obj([("uri", uri.to_json), ("version", 2.to_json)])), ("contentChanges", Arr([json.obj([("text", FIXED.to_json)])]))])
  s.send(notice("textDocument/didChange", change))?

  # where `double` is called in the fixed text: line 4, which counted from 0
  # is 3, and its `d` is character 10 of `  total = double(4)`
  at = json.obj([("line", 3.to_json), ("character", 10.to_json)])
  s.send(request(2, "textDocument/hover", json.obj([("textDocument", json.obj([("uri", uri.to_json)])), ("position", at)])))?
  hover = (await s.answer(2) { |m| show_diagnostics(m) })?
  shown = hover.path("result.contents.value").map { |x| x.str.or("") }.or("nothing")
  # the server sends Markdown; the answer is the code inside its fence
  puts "hover: " + shown.replace("```lume\n", "").replace("\n```", "")

  s.send(request(3, "shutdown", Null))?
  _ = (await s.answer(3) { |m| () })?
  s.send(notice("exit", Null))?
  puts "the server exited with #{await s.child.wait}"
  Dir.remove(DIR)?
```

With `lume` on your `PATH`:

```sh
lume run
```

```text
talking to lume 0.7.0
line 4: `+` cannot combine an `Int` and a `Str`
no problems
hover: def double(n: Int) -> Int
the server exited with 0
```

The second line is the server's diagnostic: the same message `lume check`
would print, found in the text the editor sent, not read from disk. The
change that fixes it brings `no problems`. Then comes what an editor shows
when the pointer rests on `double` — asked after the fix, because the
server answers hovers from the last version of the file that checked
cleanly.

## 24.6 What you built

`lspeek` is about a hundred lines, and it does what an editor's language
support does: start a server, frame messages, match answers to requests,
and handle notices that arrive in between. The same shape — a program
alongside, messages with a length, requests matched by an id — is how
programs talk to databases, debuggers and many network services.

## Underneath, in Rust

The server, `lume lsp`, is itself written in Rust, inside Lume's compiler:
it runs the same checking `lume check` does, on the text it is sent. This
program's side is lunch 18's `Child` — a thread reading each of the
server's outputs, `read(n)` handing out exactly the bytes asked for — and
the `json` package, written in Lume. Lume's test suite runs a longer
version of this conversation against every change to the compiler.

## Try it

1. Ask for `textDocument/definition` at the same position, and print the
   line the answer points to (`result.range.start.line`, counted from 0).
2. Send a second `textDocument/didChange` that breaks the program again
   in another way, and print the diagnostics that come back.
3. Send a change whose last line is `  puts total.`, and ask for
   `textDocument/completion` at the end of that line — the methods an
   `Int` has — and print the first five labels. (The text does not parse
   while you type `total.`, so the server answers from the last version
   that checked cleanly, where `total` was an `Int`.)
4. Send a request the server does not support, such as
   `textDocument/rename`, and print the error it answers with.

## Recap

- A language server speaks JSON messages, each framed by a
  `Content-Length` header, over its input and output.
- `read_line` reads headers and `read(n)` exactly one body; `write` sends
  both.
- Requests carry an `id`, and their answer the same one; notices may
  arrive in between, and are handed to whoever wants them.
- That is the whole of what an editor does to show errors and hovers —
  and you have now built one, end to end, in Lume.

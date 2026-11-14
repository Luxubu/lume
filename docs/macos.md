# Lume on macOS, for a real app

This guide goes from a Mac with nothing installed to a Lume service built
for release and running under `launchd`. Every step was done on macOS 26 on
Apple silicon. [Installing](install.md) has the commands in full;
[the tour](tour.md) and [the tutorial](tutorial/README.md) teach the
language.

## 1. What you need

Lume compiles your program to Rust, and Rust's compiler makes the native
binary. So you need:

- **Apple's command line tools**, for the linker:

  ```sh
  xcode-select --install
  ```

  `xcode-select -p` prints a path when they are there. A full Xcode works
  too.
- **Rust**, from [rustup.rs](https://rustup.rs):

  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

  Open a new terminal, and `rustc --version` answers.
- **git**, which the command line tools include.

## 2. Install Lume

Take a release, not whatever `master` holds today:

```sh
git clone https://github.com/Luxubu/lume ~/lume
cd ~/lume
git checkout v1.2.1
./install.sh
```

`install.sh` builds the compiler, a minute or two the first time, and puts
`lume` in `~/.local/bin`. macOS's shell is zsh; if that folder is not on
your `PATH` yet, add it once and open a new terminal:

```sh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
```

Check it:

```sh
lume --version
lume run ~/lume/examples/fib.lume
```

**To move to a newer release later**, take its tag and install again:

```sh
cd ~/lume && git fetch --tags && git checkout v1.3.0 && ./install.sh
```

[What is settled](stability.md) promises that a program that works on 1.2
keeps working on every 1.x, unless a release before warns about the form it
uses.

## 3. An editor

The VS Code extension gives highlighting, Lume's errors as you type, hover,
go to definition and completion. It needs Node.js, for packaging it:

```sh
cd ~/lume/editors/vscode
npm install
npx @vscode/vsce package
code --install-extension lume-1.2.1.vsix
```

It runs `lume lsp`, so `lume` must be on the `PATH` VS Code sees, or its
full path set in the setting `lume.path`. Any editor that speaks the
Language Server Protocol can run `lume lsp` the same way.

## 4. Start the project

```sh
mkdir -p ~/apps && cd ~/apps
lume new todo_api
cd todo_api
```

That makes a package: `lume.toml`, which names it and what it uses, and
`main.lume`. (`lume new --lib` makes a library, with `lib.lume`.) A package
name is lower case letters, digits and `_`.

Lume's `http` and `json` packages come from its repository, at the release
you installed:

```toml
[package]
name = "todo_api"
version = "0.1.0"

[dependencies]
http = { git = "https://github.com/Luxubu/lume", tag = "v1.2.1" }
json = { git = "https://github.com/Luxubu/lume", tag = "v1.2.1" }
```

The first build fetches them and writes **`lume.lock`**, the exact commit of
each: commit it, so every machine builds the same code. Build output goes
in `.lume/` beside your files; leave it out of git:

```sh
git init
echo '.lume/' > .gitignore
```

A Rust crate is a dependency too, under `[rust]`, and is used with
`import rust.<crate>`: see [Rust crates](reference/crates.md).

## 5. The app

A small JSON service: a list of to-dos, which clients add to and finish.
Each request is answered in a task of its own, and the to-dos are one
`shared var` that every task works on. [Lunch 27](tutorial/27-a-small-web-service.md)
walks through each part.

```lume-skip
# todo_api — a small JSON service.
#
#   lume run -- 127.0.0.1:8080
#   curl -d 'title=milk' http://127.0.0.1:8080/todos

import http
import json
import json.Json

struct Todo:
  id: Int
  title: Str
  done: Bool

  def to_json -> Json =
    json.obj([("id", id.to_json), ("title", title.to_json), ("done", done.to_json)])

struct Board:
  todos: [Todo]
  next_id: Int

  def add(var self, title: Str) -> Todo:
    t = Todo(id: next_id, title: title, done: false)
    todos.push(t)
    next_id += 1
    t

  def finish(var self, id: Int) -> Bool:
    found = todos.any? { |t| t.id == id }
    todos = todos.map { |t| if t.id == id: Todo(id: t.id, title: t.title, done: true) else: t }.to_list
    found

enum Route:
  List
  Add
  Finish(id: Int)
  Nowhere

def route(method: Str, path: Str) -> Route:
  match (method, path.split("/").reject(_.empty?).to_list):
    ("GET", ["todos"])              -> List
    ("POST", ["todos"])             -> Add
    ("POST", ["todos", id, "done"]) -> id.to_int.map { |n| Finish(n) }.or(Nowhere)
    _                               -> Nowhere

def answer(board: shared var Board, req: http.Incoming) -> http.Reply:
  match route(req.method, req.path):
    List -> http.reply_json(200, board.todos.map { |t| t.to_json }.to_list.to_json)
    Add ->
      match http.value(req.form.or([]), "title"):
        Some(title) -> http.reply_json(201, board.add(title).to_json)
        None        -> http.reply(400, "a todo needs a title")
    Finish(id) -> if board.finish(id): http.reply(200, "done") else: http.reply(404, "no todo #{id}")
    Nowhere    -> http.reply(404, "not found")

async def main -> () or Error:
  address = Env.args.first.or("127.0.0.1:8080")
  server = http.listen(address)?
  warn "todo_api on http://#{server.address}"
  shared var board = Board(todos: [], next_id: 1)
  for req in server.incoming:
    _ = spawn: req.respond(answer(board, req))
  ()

test "routes":
  assert route("GET", "/todos") == List
  assert route("POST", "/todos/3/done") == Finish(3)
  assert route("DELETE", "/todos") == Nowhere

test "a board finishes what it has":
  var b = Board(todos: [], next_id: 1)
  _ = b.add("milk")
  assert b.finish(1)
  assert not b.finish(2)
  assert b.todos.at(0).done
```

`warn` writes to standard error, which is where a service's log belongs.
Lume warns that `board` is a `shared var` each time it compiles; here,
sharing it is the point.

## 6. The daily loop

Inside the package, the commands need no file name:

```sh
lume run -- 127.0.0.1:8080    # build and run; what follows -- is the program's
lume check                    # type-check only: the quickest answer
lume test                     # the `test` blocks of every file
lume fmt                      # the canonical layout, in place
```

`lume run` rebuilds only what changed, in under a second for a program this
size. The first build of a program that uses `async` or a package fetches
and compiles their crates, tokio among them, once per machine. With it
running, from another terminal:

```sh
curl -d 'title=milk' http://127.0.0.1:8080/todos
curl -X POST http://127.0.0.1:8080/todos/1/done
curl http://127.0.0.1:8080/todos
```

```text
{"id":1,"title":"milk","done":false}
done
[{"id":1,"title":"milk","done":true}]
```

Keep the parts that decide things — `route`, `Board` — as plain functions
and methods, and `lume test` tries them with no server at all.

## 7. Build for release

```sh
lume build -o todo_api
./todo_api 127.0.0.1:8080
```

`lume build` compiles fully optimised: this service is a 1.3 MB binary that
needs nothing but macOS's own libraries, so it can be copied to another Mac
with the same kind of chip and run there. It does not check `Int`
operations for overflow, as `cargo build --release` does not; `lume build
--checked` keeps the checks `lume run` has.

To build for an Intel Mac or for Linux, build on one: Lume does not cross
compile.

## 8. Run it as a service

macOS's way to keep a program running is `launchd`. Save this as
`~/Library/LaunchAgents/dev.todo-api.plist`, with your own paths:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.todo-api</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Users/you/apps/todo_api/todo_api</string>
    <string>127.0.0.1:8080</string>
  </array>
  <key>WorkingDirectory</key>
  <string>/Users/you/apps/todo_api</string>
  <key>EnvironmentVariables</key>
  <dict>
    <key>TODO_DATA</key>
    <string>/Users/you/apps/todo_api/data</string>
  </dict>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>/Users/you/apps/todo_api/todo_api.log</string>
  <key>StandardErrorPath</key>
  <string>/Users/you/apps/todo_api/todo_api.log</string>
</dict>
</plist>
```

Then start it, and see that it is running:

```sh
plutil -lint ~/Library/LaunchAgents/dev.todo-api.plist
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.todo-api.plist
launchctl print gui/$(id -u)/dev.todo-api | grep state
```

`KeepAlive` starts it again if it stops. To stop it for good:
`launchctl bootout gui/$(id -u)/dev.todo-api`. After building a new binary,
`launchctl kickstart -k gui/$(id -u)/dev.todo-api` restarts it.

**Settings** come from the environment: `Env.get("TODO_DATA")` gives
`Str?`, so a program can fall back to a default with `.or(...)`.
`Env.args` gives the arguments.

## 9. What to know before you depend on it

- **The server speaks plain HTTP.** For `https://`, put it behind a reverse
  proxy such as Caddy or nginx, which holds the certificate. The client,
  `http.get` and the rest, speaks both.
- **What a service keeps**, it keeps itself: the to-dos above are gone when
  it stops. `examples/notes/` saves every change to a JSON file and reads
  it back on start; for a database, the `postgres` package, a preview in
  1.3, talks to PostgreSQL — [PostgreSQL](reference/postgres.md).
- **There is no package registry yet.** Packages are shared from git, at a
  tag, as `http` and `json` are.
- **If the compiler says "the generated Rust did not compile"**, that is a
  gap in Lume, not in your program, unless the program has a `rust:` block.
  It is worth reporting, with the file.
- **Disk:** builds share one cache, `~/.cache/lume/target`, and each program
  keeps its build in `.lume/`. Both are safe to delete at any time.
  `LUME_CACHE_KEEP=50` keeps the shared cache smaller, and `lume clean`
  removes a program's own build.

## Where to go from here

- [The tutorial](tutorial/README.md): twenty-seven lunches, the last of them
  a small web service.
- [HTTP](reference/http.md): every part of the `http` package.
- [JSON](reference/json.md), [Packages](reference/packages.md),
  [Rust crates](reference/crates.md), [Concurrency](reference/concurrency.md).
- [The examples](examples.md): `examples/notes/` is a whole service with
  its own tests and client.

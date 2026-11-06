# Changes

## Unreleased

## 1.1.0 — serving

Six milestones after 1.0.0 (122–127), the first release under the 1.x
promise. Nothing that worked in 1.0 stops working. The `http` package can
answer requests now, as a preview, and has the URL and form helpers a
request and a server both need; the tutorial's longest lunch is three;
and a log parsed and summarised runs a fifth faster.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v1.1.0
./install.sh
lume run examples/fib.lume
```

It needs a Rust toolchain (<https://rustup.rs>). On Windows, run it from
Git's bash. See `docs/install.md`. Take the packages with
`tag = "v1.1.0"`; the `http` package is at 1.1.0.

### Preview

**Serving HTTP**, the first preview since 0.8: its shape may change in a
1.x release, with each change listed here under **Preview:**.
`http.listen(address)` gives a `Server`; `accept`, `requests` and, in async
code, `incoming` give each request as an `Incoming` — its method, path,
query, headers and body, and `params` and `form` decoded — and
`req.respond(reply)` answers it once. `http.reply`, `http.reply_json`,
`.header` and `.with_body` build a `Reply`. Built on the Rust crate
tiny_http; `examples/http_server/` is a server and its own client.

### Settled in this release

- **URLs and forms** in the `http` package: `http.encode` and
  `http.decode` for a piece of a URL; `http.form` and `http.parse_form` for
  `a=1&b=two+words`; the request steps `.with_form(pairs)` and
  `.with_query(pairs)`, the second before a `#fragment`. Lunch 21 teaches
  them.

### What is new

- **The tutorial is twenty-six lunches.** Lunch 12 is three: 12,
  Generics; 13, Modules and packages, which makes a library package of
  your own and takes one from git; 14, Tests, which tests work that may
  fail and a package's tests. Lunches 13–24 are 15–26.
- **Faster, by copying less.** `match f(x):` takes apart the value the call
  gave; a list made in a chain — `to_list`, `sort_by`, a call's result — is
  walked without a copy of each item; `group_by`, `partition`, `sort_by`
  and `min_by` take their items one at a time; `first` and `last` of a
  list made just now give its item up; a map not needed again gives its
  entries to `to_list`. `bench/b8_logq` went from 200% of the same program
  in Rust to 155%.
- **A `for` over a list of tasks** walks the tasks themselves, so each can
  be awaited in turn; using the list after the loop is refused, saying so.
- **A method an optional or a result lacks** is named as missing, with the
  one meant — `is_error?`, `is_ok` and Rust's `is_err` for `error?` and
  `ok?`, `unwrap_or` for `or`, `map_err` for `map_error` — not that the
  value must be unwrapped first.
- **`http`: of two headers with one name, in any case, only the later is
  sent**, as `http.md` now says; `headers` keeps both, as you built it.
  ureq sent the later of two in the same case, both of two in different
  cases, and both of any beginning `X-`.

### Fixed

- A `for` over a list of tasks, `xs[i % xs.len] = xs.at(i) + 1`, and a
  list pattern inside another pattern each reached rustc.
- `await t` in a loop said `t` was used again below, when a later `for t
  in ...` named its own `t`.
- A stray `Error(...)` at the end of an `if` in a loop that ends a `() or
  Error` function was taken as the function's value, and left it. A loop's
  body is not the function's end; the line is refused, as such a line is
  everywhere else, by the rule for bugs in `stability.md`.
- Alternatives of `|` of different shapes point at the one that differs.

### How it was checked

- **The suite**: `tests/run.sh`, 1408 checks and 703 docs examples, byte
  for byte, on Linux, macOS and Windows (where ten that drive Unix programs
  as Unix has them are left out).
- **Review round twenty-five** (`corpus/m126/`): thirty programs by three
  reviewers from the docs alone, fourteen of them a server and its own
  client; 35 guesses, no wrong answer, and the one leak to rustc fixed.
- **The benchmarks**, all within budget.
- Installed from a clean checkout and run.

## 1.0.0 — the promise, for all of 1.x

Four milestones after 0.8.0 (118–121). Every part of Lume is settled, and
`docs/stability.md`'s promise now lasts across every 1.x release: a program
that builds on 1.0 builds, and does the same, on each 1.x after it, unless
a release before warns it about the form it uses. The means of that
warning was built and checked before anything needs it. A last review
round, the twenty-fourth, aimed at everything 1.0 promises rather than one
release's changes.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v1.0.0
./install.sh
lume run examples/fib.lume
```

It needs a Rust toolchain (<https://rustup.rs>). On Windows, run it from
Git's bash. See `docs/install.md`.

### Settled

Everything `docs/stability.md` lists: the language, collections,
generators, files and folders, running programs and talking to them as
they run, tasks and async sequences, tests, packages, the `json` and
`http` packages, Rust crates, the commands, and what `lume check --json`
and `lume lsp` give editors and tools. **There are no previews.** Outside
the promise, for good: the VS Code extension, a tool beside the language;
the Rust Lume writes, and `rust:` blocks beside it; the words of messages;
speed.

The `json` and `http` packages are at 1.0.0 too; take them with
`tag = "v1.0.0"`.

### What is new

- **Deprecation warnings, ready before they are needed**: a method or a
  namespace's function on its way out is one line in
  `compiler/src/deprecated.rs`; while it is listed, a call of it works as
  the call of its new name and is warned about wherever it is written, with
  what to write and the release it goes in. Nothing is listed: nothing is
  going. The suite checks the warning with a test form.
- **A list pattern goes inside any other pattern**, as `patterns.md` always
  said: `Say([w])`, `Some([first, ..rest])`, `(cmd, ["hi" | "yo", ..])`,
  `[[a, b], [c]]`. Alternatives of `|` that name nothing may differ in
  shape and hold literals, as `("a", _) | (_, 2)`.
- The VS Code extension, the last preview, is outside the promise: a tool
  beside Lume, versioned with it, built on the protocol `lume lsp` speaks,
  which is settled. Lunch 24 is no longer a preview.

### Fixed

- By the rule for bugs in `stability.md`: in a `-> () or Error` function,
  an `Error(...)` on a line of its own that does not end the function left
  it, unlike in every other function, where it is refused; `errors.md`
  said it was thrown away. It is refused there too now, with the same
  message and fix: `return Error(...)`. One that ends a branch at the
  function's end is still its value, nested `if`s and `match`es included.
- A list pattern inside anything but a list matched on its own reached
  rustc.
- `xs.push(1)` inside `spawn:` on an outside `var` said `xs` was not a
  `var`; it says, as `xs = ...` does, that the task changes its own copy.
  A chain's list of methods in an error left out `at`.
- Two examples in the I/O reference used one folder, and the docs check,
  which runs examples side by side, could have one remove it while the
  other read it — as 0.8.0's own check on Linux did.

### How it was checked

- **The suite**: `tests/run.sh`, 1374 checks and 697 docs examples, byte
  for byte, on Linux, macOS and Windows (where ten that drive Unix programs
  as Unix has them are left out).
- **Review round twenty-four** (`corpus/m120/`): thirty programs by three
  reviewers from the docs alone, one for each settled part among them;
  42 guesses, no wrong answer, and the one leak to rustc fixed.
- **An audit** of every settled part against its page, and about fifteen
  places where two pages disagreed brought to one answer.
- **The benchmarks**, all within budget; `bench/BASELINE` holds 1.0's.
- Installed from a clean checkout and run.

## 0.8.0 — the last previews

Six milestones after 0.7.0 (112–117). Every part of Lume that was a
preview is settled now but one, the VS Code extension: the `http` package,
Rust crates, and what `lume lsp` and `lume check --json` give editors and
tools. On the way the language server learned to answer an editor on
Windows by the editor's own names for its files, integers crossing to a
crate learned to be checked, and a review round, the twenty-third, checked
it all.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v0.8.0
./install.sh
lume run examples/process_spawn.lume
```

It needs a Rust toolchain (<https://rustup.rs>). On Windows, run it from
Git's bash. See `docs/install.md`.

### Settled

`docs/stability.md` promises that a settled part does not break a working
program without a warning release first. Settled in this release:

- **The `http` package.** A `Request` and an `Opened` may gain fields: read
  them, but build a request with `http.request` and its steps.
- **Rust crates**: `import rust.<crate>`, `[rust]` in `lume.toml`, how a
  crate's types are read — now a table in `docs/reference/crates.md` —
  builders, and `lume crate`. **A `rust:` block stays outside the promise
  for good**, as the Rust Lume writes does.
- **For editors and tools**: the fields of `lume check --json`, written down
  field by field, and the protocol `lume lsp` speaks. What a hover or a
  completion says is not promised, as no message's words are.

Lunches 19 and 23 are no longer previews. The one preview left is the VS
Code extension.

### What is new

- **A body sent as it is made**: `.with_file(path, content_type)` sends a
  file as it is read, never held whole; `req.send_parts(parts)` sends a
  sequence's parts in chunks as it makes them.
- **An async sequence has `join`, `max`, `min`, `max_by`, `min_by`, `sort`,
  `sort_by`, `last` and `reverse`**: what needs every item keeps them, as
  `.to_list` would, and gives what a list's would.
- **`LUME_LSP_LOG=<file>`** makes the language server write what it reads,
  checks and publishes, and why it stops.
- **Warnings come in the order they are written**, whatever found them.

### Fixed

- On Windows, `lume lsp` published diagnostics under a uri of its own
  making, not the one the editor opened a file by (`file:///d%3A/...` from
  VS Code); it answers by the editor's own now. This is what kept the
  suite's language-server test off Windows, which runs there again.
- An `Int` handed to a crate's narrower integer was cast — 300 for a `u8`
  became 44 — and a returned `u64` past `Int`'s largest wrapped; both are
  checked now, as Rust's `TryFrom` is, a list of them item by item, and a
  value that does not fit stops the program on its line.
- `xs == []` with a list of `Str` reached rustc.
- `lume check --json main.lume`, the file after the option, said there was
  no file.
- A request given two bodies sent two `Content-Type` headers; a missing
  file's `Error` did not name the method and URL.
- `lume lsp` stopped at a message it could not read; it passes over it.
- A program's output whose reading failed ended without a word.
- `lume fmt` moved a comment between or after a chain's steps.
- `.uniq` on an async sequence said it could not be printed, not that it
  has no `uniq`.

### How it was checked

- **The suite**: `tests/run.sh`, 1338 checks and 696 docs examples, byte
  for byte, on Linux, macOS and Windows.
- **Review round twenty-three** (`corpus/m116/`): thirty programs by three
  reviewers from the docs alone, with nothing run; no wrong answer, and
  what reached rustc fixed.
- Installed from a clean checkout and run.

### Previews

No preview changed shape. Three were settled; the VS Code extension
remains.

## 0.7.0 — settling

Six milestones after 0.6.0 (106–111). Lume runs on Windows now, warns
about names it is given and never reads, and settles what 0.5 introduced:
async sequences — which an interface may now require — and programs run
alongside, with `Child`'s edges tried and written down. A review round,
the twenty-second, checked it all.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v0.7.0
./install.sh
lume run examples/async_interfaces.lume
```

It needs a Rust toolchain (<https://rustup.rs>). On Windows, run it from
Git's bash. See `docs/install.md`.

### Settled

`docs/stability.md` promises that a settled part does not break a working
program without a warning release first. Settled in this release, with
nothing in their shape changed:

- **Async sequences**: async generators, their chains and the methods that
  wait for their items, and `async Iterator[T]`. Lunch 17 is no longer a
  preview.
- **`Process.spawn`, `Process.spawn_in` and `Child`**, after each of
  `Child`'s edges was tried and written down: a write after the program
  ended is an `Error`; `wait` gives the same code each time; `kill` after
  the end keeps the code it ended with; a `Child` let go of runs on, as in
  Rust. Lunch 18 is no longer a preview.
- **`async test`.**

What is left a preview: the `http` package, the Rust bridge, and editor
support.

### What is new

- **Tested on Windows**: the suite runs on Linux, macOS and Windows for
  every push. Ten checks that need Unix programs as only Unix has them —
  Windows' own `sort` comes first, it has no signals, and Git's tools drop
  a newline inside an argument — are left out on Windows, and the suite
  says so.
- **Unused names are warned about**, as Rust's `unused_variables` lint
  does: a local given a value and never read, a parameter, a `for`
  variable, a block's parameter, a name a pattern binds. A name beginning
  with `_` says it is meant, and `_` may stand more than once among a
  block's, a `for`'s or a function's names (`{ |_, v| v }`). Warnings never
  stop a program; Lume's own examples and docs have none.
- **An interface may require an async generator**: `async def items ->
  Iterator[T]`, or `def items -> async Iterator[T]`, written by each type
  that fits — the gap 0.5.0 listed. A default one written in the interface
  is refused, with what to write instead.
- **`await c.read(n)` never cuts a character in two.**

### Fixed

- On Windows: a built program is `name.exe`; messages write paths with
  `/`; a missing file is told in the same words as elsewhere, `No such
  file or directory (os error 2)`; `kill` makes a program give `-1`;
  fetching a package from git no longer fails on the `\\?\` Windows puts
  before a path; the language server reads and writes `file:///C:/...`;
  the build cache is found without `HOME`; `http` says "timed out".
- A pipe read that was only interrupted ended a program's output.
- A type with an async generator method did not fit a generic interface
  requiring one, and reached rustc; a generic interface whose `T` appears
  only in `async Iterator[T]` was refused as never using it.
- A `() or Error` function could not end with a call giving `() or Error`,
  as `= child.write(...)`.
- `await c.receive?` was refused, where the `?` belongs to the `await`.
- A program with a generator could not `extend [Str]` with an `items` of
  its own: Lume's own `Iterable` was given to lists unasked.
- `lume fmt` wrote `{ |_| x }`, a block that ignores its item, as the
  shorthand `(x)`, which means something else.
- A program with its own type named `Fn` hid Rust's `Fn` trait from the
  code Lume writes, and reached rustc.

### How it was checked

- **The suite**: `tests/run.sh`, 1307 checks and 695 docs examples, byte
  for byte, on Linux, macOS and Windows.
- **Review round twenty-two** (`corpus/m110/`): thirty programs by three
  reviewers from the docs alone, with nothing run; no wrong answer, no
  rustc error, all seven refusals predicted word for word, and 27 of 30
  as predicted after the fixes.
- Installed from a clean checkout and run.

### Previews

No preview changed shape. Settling moved three out of the list; the
`http` package, the Rust bridge and editor support remain.

## 0.6.0 — kinder when it breaks

Eight milestones after 0.5.0 (98–105). Lume got ready to be taught: a
tutorial, *Lume, a Lunch at a Time*, tried by three beginners; a promise of
what is settled; and a program that goes wrong now says where, in Lume's
words. Tests got the care the docs get, and can test async code. Two
review rounds — the tutorial's dry run and round twenty-one — checked it.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v0.6.0
./install.sh
lume run examples/chains_over_lines.lume
```

It needs a Rust toolchain (<https://rustup.rs>). See `docs/install.md`.

### What is new

- **A tutorial**: *Lume, a Lunch at a Time*, in `docs/tutorial/` — 24
  chapters, each about an hour, from a first program to a language-server
  client, with exercises and answers. Every example in it is run by the
  test suite, its tests included.
- **What is settled**: `docs/stability.md` says which parts of Lume are
  settled and which are previews, and promises that a settled part does not
  break a working program without a warning release first. From here on, a
  change to a preview is listed in an item beginning **Preview:**, and a
  coming change to a settled part in one beginning **Changing:**.
- **A program that stops says where**: `xs.at(9)`, `!` on a failure, a
  division by zero, an overflow or an `assert` name the file and line they
  stopped on, and show the line, as a compile error does — the line it
  happened on, in a method, a block, a generator, a module, a task, or a
  step of a chain. Rust's "index out of bounds" reads `no item at 7 — the
  list has 3`, and a failed `assert` outside a test `assert failed: ...`.
- **A task that stops is told where it is awaited**, in the form of any
  other stop, so what a program prints does not depend on when its tasks
  stop. One never awaited is told when the program ends.
- **A chain over several lines**: a line starting with `.name` continues the
  line above, so a long chain can put one step on each line, with comments
  and blank lines between them. `lume fmt` keeps it so.
- **Structs with no fields**: `struct Marker` alone, or a struct whose block
  has only methods — a type that is all behaviour, made with `Marker()`, as
  Rust's unit structs.
- **`async test`** *(preview)*: a test that can `await`, run on its own
  tokio runtime, as Rust's `#[tokio::test]`.
- **`lume test file.lume name`** runs only the tests with `name` in theirs,
  as `cargo test name` does. A failing test names its file as it was
  given, and its line for any stop, not only an `assert`.
- **A Tests page** in the reference.
- **Tested on Linux**: the suite runs on Ubuntu and macOS for every push,
  with the newest stable Rust.

### Refused, with a message

- `price = price`, which changes nothing — the help says `self.price =
  price` when a field has that name. It compiled before.
- A keyword used as a name, called one.
- A line starting with `.` that begins a block, after `:`, `do` or `do |x|`.
- `struct Marker:` with nothing under it — the help says to drop the `:`.
- `await` in a plain `test`, with the `async test` to write.

### Fixed

- A list or map literal of values behind an interface reached rustc when
  given to a field, a parameter or a function's result.
- With Rust 1.99, a program whose interface has an `async` method that can
  fail did not compile: the prelude's types were private.
- An `async def` call printed, put in text or in a list without `await`
  reached rustc.
- A `Char` as the key of a `{Str: Int}` map reached rustc.
- `take` and `skip` with a negative count wrapped around.
- `-> async Iterator[T]` on an async generator broke its callers.
- Each side of an `assert` comparison is worked out once, as `assert_eq!`
  does: a failed `assert it.next == Some(1)` showed the item after the one
  compared, and an `await` on a side did not compile.
- Better help for a missing `await` inside a task or a plain `def`, a
  statement after a one-line `spawn:`, `+=` on a task's copy of a name, and
  a field named as an interface's method.

### How it was checked

- **The suite**: `tests/run.sh`, 1273 checks and 693 docs examples, byte
  for byte, on Linux and macOS. The docs check now also runs the indented
  examples under list items, and `lume test` on every example with tests.
- **The tutorial's dry run** (milestone 99): three reviewers worked through
  all 24 lunches as Python programmers new to Lume, typing every example
  and doing every exercise; what they found is fixed (`corpus/m99/`).
- **Review round twenty-one** (`corpus/m104/`): thirty programs by three
  reviewers from the docs alone, with nothing run; after the fixes all
  thirty print what their authors predicted.
- Installed from a clean checkout and run.

### Previews

Async generators, `Process.spawn`, the `http` package, the Rust bridge,
editor support and now `async test` are previews; see `docs/stability.md`.
None of the previews from 0.5.0 changed shape in this release.

## 0.5.0 — talking to the world

Five milestones after 0.4.0 (92–96). A Lume program can now drive another
program while it runs and read an HTTP body as it arrives, both as async
sequences, which gained the methods that wait for their items. The
language-server test moved to Lume on top of it, and a review round checked
it all.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v0.5.0
./install.sh
lume run examples/process_spawn.lume
```

It needs a Rust toolchain (<https://rustup.rs>). See `docs/install.md`.

### What is new

- **`Process.spawn(program, args)`** and `Process.spawn_in(dir, ...)`: a
  program running alongside, as Rust's `tokio::process::Child`. Its output
  is an async sequence of lines (`lines`, `error_lines`), or read a line
  or a number of bytes at a time (`await read_line`, `await read(n)`); its
  input is written to (`write`, `close_input`); `await wait` gives its exit
  code, and `kill` stops it.
- **The methods that wait for an async sequence's items**: `first`,
  `count`, `sum`, `any?`, `all?`, `find`, `each` and `fold`, as Rust's
  `StreamExt` has them. Each item is handled as it arrives; `find`, `any?`,
  `all?` and `first` stop at their answer, so they work on an endless
  sequence; their block may change the names around it.
- **`async Iterator[T]` as a type**: a `def` may hand on an async sequence
  without `yield`, and a parameter takes one by move, as Rust takes a
  `Stream`.
- **`http`, streamed**: `http.request(method, url)` builds a request with
  `.header`, `.with_body` and `.timeout(ms)`. `send` waits for the whole
  answer, `await fetch` waits in async code as other tasks go on, and
  `open` gives the status and headers at once and the body as it arrives,
  as `lines` or `chunks`. `get`, `post` and the rest are as before.
- **`==` on a `Child`**, or on a value holding one, is refused, as on a
  block: compare what it gives.

### Fixed

- The shared build cache, `~/.cache/lume/target`, kept every program's
  build for good, and reached 13 GB on one machine before the disk filled.
  It now keeps the 300 programs built most recently.
- A bare call of an `async` sibling method could not be awaited.
- A struct holding a `Child` did not compile.
- An async generator taking an `async Iterator[T]` reached rustc.
- A `next_done` example in the docs depended on a 40 ms margin.

### How it was checked

- **The suite**: `tests/run.sh`, 1220 checks and 424 docs examples, byte for
  byte. Its language-server test is `tools/lumecheck/lsp.lume` now, a Lume
  program on `Process.spawn` that prints what the Python one did, byte for
  byte; the only Python left is three local HTTP servers the tests start.
- **Error messages**: 375 of the compiler's 391 messages are shown by a test.
- **Review round twenty**: 30 programs written from the docs alone, 34
  guesses, 20 of 22 right on the first run, no wrong output; all six whole
  programs — driving `sort`, `tr`, `sh` loops and framed output — right on
  the first run.
- **Streaming**: against a local server writing a line every 400 ms, the
  lines arrive 400 ms apart.
- **Benchmarks**: eight programs against the Rust a person would write; six
  are within 10% of it, as at 0.4.0.

### Known limits

- **HTTP**: a request's own body is sent whole, and Lume cannot serve HTTP.
- **Async generators** cannot be declared in an interface.
- **VS Code**: packaged, and its language server checked with VS Code's
  messages, but not yet looked at in an editor window.
- **Time zones**: dates and times are UTC only.
- **No registry**: packages come from paths and git.

## 0.4.0 — sequences that wait

Eight milestones after 0.3.0 (83–90). Sequences learned to wait: an
`async def` may `yield`, the sequence it gives has the chains a generator
has, a method may be one, and tasks can be taken as they finish. The test
suite was rewritten in Lume and runs on these features, and two review
rounds checked them.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v0.4.0
./install.sh
lume run examples/async_generators.lume
```

It needs a Rust toolchain (<https://rustup.rs>). See `docs/install.md`.

### What is new

- **Async generators**: an `async def` that gives `Iterator[T]` and has
  `yield` may `await` in its body. It runs as a task never more than one
  item ahead, is walked with `for`, `next` and `.to_list` in async code,
  and letting go of it ends it.
- **Chains over them**: `map`, `filter`, `reject`, `take_while`, `take`,
  `skip` and `enumerate`, lazy as Rust's `StreamExt`, so a chain over an
  endless sequence ends. A chain's block copies in the names it uses.
- **Async generator methods**, on structs, enums and generic types.
- **`await tasks.next_done`**: whichever task in a `var` list finishes
  first, taken out of the list, as tokio's `JoinSet::join_next`. With it
  a fixed number of tasks can work through a long list.
- **Generators in an `extend`**, of your own types and of built-in ones
  such as `[T]` and `Str`.
- **Generators that keep a block or an interface value**, plain or async,
  functions or methods; an interface method that gives a sequence keeps
  its block, so a generator can implement it.
- **`Process.run_in(dir, program, args)`** runs a program in a folder.
- **A task takes what cannot be copied**: a generator, an async sequence
  or another task it uses is moved into it, and using it after is refused.

### The test suite, in Lume

- **`tools/lumecheck/`** holds the suite's checks, written in Lume: the
  plan of every check, the docs examples, message coverage and the editor
  grammar. Only the language-server driver, `tests/lsp_test.py`, is still
  Python.
- **`tests/run.sh`** runs it, a check starting as each finishes, through
  an async generator and `next_done`: about 2½ minutes, from 15. It tells
  each failure as soon as it is known, stops at the first with `-x`, and
  names each pass with `-v`.

### Fixed

- A block for `filter`, `reject`, `take_while`, `count`, `any?`, `all?`,
  `find` or `partition` that did not give a `Bool`.
- `for w in split(" ")` inside an `extend Str`.
- A method building a chain over a block an interface keeps, and one
  keeping a block an interface lends: both refused now, with their own
  messages.
- Two programs with the same file name, built at the same time, could run
  each other's binary; a generator from another module lost the type of
  its items.
- A program's own `struct Box` could shadow the one the runtime uses.

All of these reached rustc before; each now runs or gets a Lume message.

### How it was checked

- **The suite**: `tests/run.sh`, 1185 checks and 422 docs examples, byte for
  byte.
- **Error messages**: 372 of the compiler's 389 messages are shown by a test;
  the rest are listed in `examples/errors/UNREACHED.md` with why.
- **Review rounds**: rounds eighteen and nineteen, 60 programs written from
  the docs alone. Round nineteen: 51 guesses, 20 of 22 programs right on the
  first run, no wrong output, and after its fixes all thirty print what
  their authors predicted.
- **Benchmarks**: eight programs against the Rust a person would write; six
  are within 10% of it, and none is slower than at 0.3.0.

### Known limits

- **Async sequences**: no `count`, `fold` or the other methods that wait for
  every item (keep the items with `.to_list`); an async generator cannot be
  declared in an interface.
- **VS Code**: packaged and installed, and its language server checked with
  VS Code's messages, but not yet looked at in an editor window.
- **`http` package**: no streaming bodies, and a fixed 30-second timeout.
- **Time zones**: dates and times are UTC only.
- **No registry**: packages come from paths and git.

## 0.3.0 — interfaces all the way

Eight milestones after 0.2.0 (75–81). Generators became complete enough
to build programs around: methods may be generators, interfaces may give
sequences and take and give each other, and every function that says
`-> Iterator[T]` hands one back. Two programs were written on them, two
review rounds checked them, and the VS Code extension was packaged for
the first time.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v0.3.0
./install.sh
lume run examples/interface_signatures.lume
```

It needs a Rust toolchain (<https://rustup.rs>). See `docs/install.md`.

### What is new

- **Generator methods**: a method of a struct or enum may have `yield`. It
  walks its own copy of the value, taken at the call.
- **Interfaces in signatures**: an interface method may take an interface
  value (`def bigger?(o: Shape)`), give one back (`-> Shape`), and give a
  sequence (`-> Iterator[T]`); a default may be a generator built from the
  interface's other methods. The trait stays usable through a pointer.
- **`-> Iterator[T]` on any function**, with `yield` or without: a function
  may hand back a generator, a chain, a list or a sequence of your own, as
  Rust's `-> impl Iterator` does.
- **Lazy into generators**: a chain that starts from a generator, a range
  or a sequence of your own stays lazy when given to a generator, so an
  endless one works: `evens(naturals().map { |n| n * k })`.
- **Blocks over tuples**: a block names one argument for each part of a
  tuple, three or more as well as two, as `for` does.
- **VS Code**: `editors/vscode/` packages to a `.vsix` and installs; `yield`
  is coloured; completion after `.` on a generator, and go to definition on
  an interface's default method.
- **`examples/deps/`**: a dependency-graph tool whose every answer is a
  generator method.

### Faster

- **Reading text in place**: a name bound to an item of a split line, and a
  `.slice` read by a text method, are slices of the line, not copies.
  `logq`'s parse and summary of a 500,000-line log is 2.0 times the
  hand-written Rust, from 2.2.

### Fixed

- A struct or variant built at the head of an `if`, `while`, `for` or
  `match`.
- `-> Shape or Error` giving a plain value.
- A list literal, or a `match` head, moving a value the same statement
  reads again.
- `enumerate` and `zip` over split text, and `text.split(sep).at(i)`
  followed by a text method.

### How it was checked

- **The suite**: `tests/run.sh`, 1110 checks and 415 docs examples, byte for
  byte, with the language server driven by VS Code's own messages and the
  editor grammar checked against every keyword.
- **Error messages**: 365 of the compiler's 380 messages are shown by a test.
- **Review rounds**: rounds sixteen and seventeen, 59 programs written from
  the docs alone. Round seventeen: 12 guesses, 26 of 27 programs right on the
  first run, no wrong output.
- **Benchmarks**: eight programs against the Rust a person would write; six
  are within 10% of it.

### Known limits

- **Generators**: not yet `async`, nor in an `extend`, nor taking a block.
- **VS Code**: packaged and installed, and its language server checked with
  VS Code's messages, but not yet looked at in an editor window.
- **Time zones**: dates and times are UTC only.
- **No registry**: packages come from paths and git.

## 0.2.0 — sequences

Nine milestones after 0.1.0 (65–73). The theme is sequences: types of your
own that `for` and every list method work on, lazy and endless ones,
generators written with `yield`, and functions that take any of them. Along
the way came the `http` package, Rust builders through the bridge, and two
programs written to find what was missing.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v0.2.0
./install.sh
lume run examples/generators.lume
```

It needs a Rust toolchain (<https://rustup.rs>). See `docs/install.md`.

### What is new

- **Sequences of your own** ([Collections](docs/reference/collections.md)):
  - a type with `def items -> [T]` works with `for` and every list method
    (`Iterable[T]`, as Rust's `IntoIterator`);
  - a type with `def next(var self) -> T?` is a lazy sequence, possibly
    endless, and compiles to a Rust `impl Iterator` (`Iterator[T]`);
  - bounds `[C: Iterable[T]]` and `[S: Iterator[T]]` take them, and an
    interface's required method may now take `var self`.
- **Generators**: a function that gives `Iterator[T]` hands out its items
  with `yield`, one at a time, so it may be endless. In Rust it is an
  `impl Iterator` whose body rustc turns into a state machine. A generator
  moves as a Rust iterator does: walking it uses it up.
- **Sequence parameters**: `xs: Iterator[T]` takes a generator, a lazy
  chain, a list, a set or a type with `next`, by move, so generators chain:
  `evens(naturals()).take(3)`.
- **`zip`** stays lazy on a chain and takes a sequence of your own.
- **`packages/http`**: `get`, `post`, `post_json` and `send`, with a
  `Response` that has `ok?`, headers and `.json`. A 404 is a response; only
  a request that could not be made is an `Error`. Depend on it with
  `http = { git = "https://github.com/Luxubu/lume", tag = "v0.2.0" }`.
- **Rust builders**: crate methods that take `self` by value can be called
  through the bridge, which is how most Rust builders work.
- **Blocking tasks**: a `spawn:` body that awaits nothing runs on Tokio's
  pool for blocking work, as `spawn_blocking` does, so requests, program
  runs and computations in tasks run at once.
- **Two programs written to find what was missing**:
  - `examples/healthcheck/` checks services at once, on `http` and `json`;
  - `examples/logq/` answers questions about an access log, on generators.
- **Layout**: lists, maps, calls and parameter lists may be written across
  lines, one item to a line.
- **Docs**: new pages [HTTP](docs/reference/http.md) and
  [Rust crates](docs/reference/crates.md); sections on sequences,
  generators and sequence parameters.

### Faster

- **Generators** run at 1.3 times a hand-written Rust iterator, and
  sequences of your own level with it.
- **`split` and `lines`** give a list of slices of the text, not a copy of
  each piece, when the list is only read by position.
- **`Time.parse`** no longer allocates: 2.7 times faster.
- **`logq`'s parse and summary** of a 500,000-line log went from 4.7 to 2.2
  times the hand-written Rust.

### Fixed

- A `var`, field, list position or map key held as an interface could not
  be given a value of another type.
- `_ = ...` worked once per block.
- A task put in a list was copied; it now moves, and using it afterwards is
  refused with a message.
- A function ending in `Env.exit` with no return type could not be typed.
- Messages: "an `Ones`"; built-in interfaces named `lume.Iterable`; `Time`
  used as a type; help lines for a `next` of the wrong shape and for a
  `var self` call on a parameter.

### How it was checked

- **The suite**: `tests/run.sh`, 1045 checks and 411 docs examples, byte for
  byte.
- **Error messages**: 364 of the compiler's 379 messages are shown by a test.
- **Review rounds**: rounds fourteen and fifteen, 59 programs by six
  reviewers, written from the docs alone. Round fifteen: 37 guesses, 20 of 21
  programs right on the first run, no wrong output.
- **Benchmarks**: eight programs against the Rust a person would write; six
  are within 10% of it.

### Known limits

- **Generators**: a method cannot be one yet, nor an `async def`, and a
  lazy chain given to a generator is worked out at the call, so an endless
  one never returns.
- **Time zones**: dates and times are UTC only.
- **Build caching**: a whole program is one Rust crate.
- **VS Code**: the extension has still not been tried in VS Code itself.
- **No registry**: packages come from paths and git.

## 0.1.0 — the first release

Lume is a language with Ruby's feel that compiles to Rust and runs at Rust's
speed. This is its first tagged release, after 64 milestones. `HISTORY.md`
has every one, with what building it found.

### Install

```sh
git clone https://github.com/Luxubu/lume && cd lume && git checkout v0.1.0
./install.sh
lume run examples/fib.lume
```

It needs a Rust toolchain (<https://rustup.rs>). See `docs/install.md`.

### What is in it

- **The language**:
  - values without an ownership word: moves at the last use, copies where
    needed;
  - structs, enums, `match` checked for every case;
  - interfaces that a type fits by having the methods, with defaults and
    `extend`;
  - generics with bounds, and Rust's coherence and orphan rules;
  - blocks, kept blocks and blocks as values;
  - `T?` and `T or Error` with `?`, `map_error`, and failures that cannot be
    dropped;
  - `async def`, `spawn:`, `await` and `shared var` for concurrency;
  - a `rust:` block, and `import rust.<crate>` for any crate, typed from its
    signatures.
- **Built in**:
  - `File`, `Dir`, `Path`, `Env` and `Time`;
  - `Process.run` to run another program;
  - dates and times in UTC: `Time.format`, `Time.parse`, `Time.date`.
- **Packages**:
  - `lume.toml`, with dependencies by path or git and `lume.lock`;
  - `lib.lume` as a package's public surface;
  - `lume new` and `lume update`.
- **The first package**: `packages/json/`, for parsing, typed reads,
  decoding into your own types with errors that name the field, and writing
  through `ToJson`. Depend on it from anywhere with
  `json = { git = "https://github.com/Luxubu/lume", tag = "v0.1.0" }`.
- **Tools**:
  - `lume fmt`, `lume test` and `lume check --json`;
  - `lume lsp`, a language server with errors as you type, hover, go to
    definition and completion;
  - `editors/vscode/`, a VS Code extension.
- **Speed**: level with hand-written Rust on four of five benchmarks, 107% on
  the fifth. Since milestone 64, overflow checks follow Rust:
  - `lume run` and `lume test` stop the program when an `Int` overflows;
  - `lume build` does not check, unless it is given `--checked`.

### How it was checked

- **The suite**: `tests/run.sh` runs every example, every error example, the
  docs' examples and the review programs, and compares each output byte for
  byte.
- **Error messages**: 346 of the compiler's 359 messages are shown by a test.
- **Review rounds**: thirteen rounds of reviewers wrote programs from the docs
  alone, without the compiler, and predicted every output. The last round had
  31 guesses across 27 programs, and no wrong output.

### Known limits

- **Time zones**: dates and times are UTC only.
- **Build caching**: a whole program is one Rust crate, so a 27,000-line
  program rebuilds in about 1.6 s after an edit.
- **VS Code**: the extension has not yet been tried in VS Code itself.
- **No registry**: packages come from paths and git.

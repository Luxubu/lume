# Changes

## Unreleased

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

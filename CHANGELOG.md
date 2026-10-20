# Changes

## Unreleased

- **`Process.spawn`**: a program running alongside, as Rust's
  `tokio::process::Child` — its output as an async sequence of lines, or
  read by line or by byte count; its input written to; its exit code
  awaited.
- **Async sequences wait for their items**: `first`, `count`, `sum`,
  `any?`, `all?`, `find`, `each` and `fold`, as Rust's `StreamExt` has
  them; `find`, `any?`, `all?` and `first` stop at their answer.

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

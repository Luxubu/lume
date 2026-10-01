# Changes

## Unreleased

- **`packages/http`**: `get`, `post`, `post_json` and `send`, with a
  `Response` that has `ok?`, headers and `.json`. A 404 is a response; only a
  request that could not be made is an `Error`.
- **Crate builders**: methods that take `self` by value, which is how most
  Rust builders work, can be called through the bridge. `lume crate` marks
  them.
- **Docs**: new pages [HTTP](docs/reference/http.md) and
  [Rust crates](docs/reference/crates.md).
- **Blocking tasks**: a `spawn:` body that awaits nothing runs on a pool for
  blocking work, as Rust's `spawn_blocking` does, so requests, program runs
  and computations in tasks all run at once.
- **`examples/healthcheck/`**: a service checker built on `http` and `json`.
- **Iterating your own types**: a type with `def items -> [T]` works with
  `for` and every list method, and a bound `[C: Iterable[T]]` takes it.
- **Lazy sequences of your own**: a type with `def next(var self) -> T?`
  works with `for` and every chain method, one item at a time, so it may be
  endless; it compiles to a Rust `impl Iterator`. A bound
  `[S: Iterator[T]]` takes it.
- **Interfaces**: a required method may take `var self`. A type fits only
  when its method agrees about `var self`.
- **`zip` on a lazy chain** stays lazy.

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

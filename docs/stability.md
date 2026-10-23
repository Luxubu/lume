# What is settled, and what may change

Lume is at 0.5, and still growing. This page says which parts of it you can
build on today — write programs in, teach, write tutorials about — and
which are previews that may still change. It also says how a settled part
is allowed to change, so that what you write keeps working.

It follows the spirit of Rust's own promise, *stability without
stagnation*: the language keeps getting better, and a program that works
keeps working.

## The promise

**A settled part of Lume does not change in a way that stops a working
program from compiling, or makes it behave differently, without warning
first.** When a settled part must change, the release before the change
makes Lume warn wherever a program uses the old form, saying what to write
instead; the old form keeps working for at least that one release. The
`CHANGELOG.md` entry for each of those releases says so, in an item that
begins **Changing:**.

**A preview may change in any release**, with no warning release. Every
change to one is listed in that release's `CHANGELOG.md` entry, in an item
that begins **Preview:**, so you can see what to update. A preview becomes settled when
a release says so; from then on the promise above covers it.

## Settled

| part | where it is described |
|---|---|
| The language: values, names and `var`, functions, `if`, `match`, loops, blocks, structs, enums, interfaces, generics, `T?` and `T or Error`, `?`, modules, `pub`, `import` | [the reference](README.md), lunches 1–12 |
| Collections: lists, maps, sets and their methods; lazy chains | [Collections](reference/collections.md), [Every method](reference/methods.md) |
| Generators: `yield`, `Iterator[T]`, generator methods | [Collections](reference/collections.md#generators-yield), lunch 14 |
| Files, folders, paths, the environment, time: `File`, `Dir`, `Path`, `Env`, `Time` | [Input and output](reference/io.md), lunch 13 |
| Running a program and waiting for it: `Process.run`, `Process.run_in` | [Input and output](reference/io.md#process), lunch 15 |
| Tasks: `async def`, `await`, `spawn:`, `shared var`, `next_done` | [Concurrency](reference/concurrency.md), lunch 16 |
| Tests: `test` blocks, `assert`, `lume test` | lunch 12 |
| Packages: `lume.toml`, dependencies by path and git, `lume.lock` | [Packages](reference/packages.md) |
| The `json` package | [JSON](reference/json.md) |
| The commands: `lume run`, `build`, `check`, `test`, `fmt`, `new`, `clean`, `update` | [Install](install.md#the-commands) |

## Previews

| part | since | where it is described |
|---|---|---|
| Async generators and their chains and methods; `async Iterator[T]` as a type | 0.4, 0.5 | [Collections](reference/collections.md#async-generators-async-def-with-yield), lunch 17 |
| A program running alongside: `Process.spawn`, `Process.spawn_in`, `Child` | 0.5 | [Input and output](reference/io.md#a-program-running-alongside-processspawn), lunch 18 |
| The `http` package | 0.2, streamed in 0.5 | [HTTP](reference/http.md), lunch 19 |
| The Rust bridge: `import rust.crate`, `rust:` blocks, `[rust]` in `lume.toml`, `lume crate` | 0.1 | [Rust crates](reference/crates.md) |
| Editor support: `lume lsp`, `lume check --json`, the VS Code extension | 0.1 | [Install](install.md#editors) |

The previews work, and every documented example of them is checked on
every change, as the settled parts' are. What is not settled is their
*shape*: a method's name, what a call gives back, how a request is built.
Use them freely in your own programs; expect to update those programs now
and then.

## Not covered by the promise

- **The wording of messages.** Errors and warnings are improved all the
  time, and a better message is never a breaking change — though which
  programs compile is covered, so a program Lume accepted is not refused
  without warning first.
- **The Rust that Lume writes.** `lume emit` shows it, and it changes
  freely; nothing should depend on its names or shape. A `rust:` block is
  a preview for this reason.
- **Speed.** Lume tries never to get slower, and the benchmarks in
  `bench/` guard that, but a program's exact speed is not a promise.
- **Bugs.** A program that leaked an error from rustc, or did something
  other than what the docs say, may be fixed to do what they say. Lume 0.5
  refusing `price = price` — which compiled before and did nothing — is
  such a fix.
- **What `lume lsp` offers an editor**, beyond its being a language server.

## How the promise is kept

Every example in `docs/` — the reference and the tutorial — every program
in `examples/`, and every refused program in `examples/errors/` is run on
every change, on Linux and macOS, and must print exactly what it shows.
A change that alters a settled behaviour fails those checks, so it cannot
happen unnoticed: it has to be made on purpose, with its warning release
and its `CHANGELOG.md` entry, by the rules in `CONTRIBUTING.md`.

## Towards 1.0

At 1.0, each preview above will be settled or taken out, and the promise
will last across every 1.x release, as Rust's has since its 1.0.

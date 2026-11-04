# What is settled, and what may change

Lume is at 1.0, and still growing. This page says which parts of it you can
build on — write programs in, teach, write tutorials about — and
what the promise does not cover. It also says how a settled part is allowed
to change, so that what you write keeps working.

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

Such a warning looks like this, and the program still runs:

```
warning: `Str.shout` is going away in Lume 1.2: write `upcase`
  --> menu.lume:4:28
  |
4 | def loud(s: Str) -> Str = s.shout
  |                            ^
  help: it still works until then, as `upcase` does; `CHANGELOG.md` says why, under **Changing:**
```

(`shout` is made up: nothing is going yet. `tests/deprecated/` shows the
warning with a test form, so the means of keeping the promise is checked
before it is needed.)

**A preview may change in any release**, with no warning release. Every
change to one is listed in that release's `CHANGELOG.md` entry, in an item
that begins **Preview:**, so you can see what to update. A preview becomes settled when
a release says so; from then on the promise above covers it.

## Settled

| part | where it is described |
|---|---|
| The language: values, names and `var`, functions, `if`, `match`, loops, blocks, structs, enums, interfaces, generics, `T?` and `T or Error`, `?`, modules, `pub`, `import` | [the reference](README.md), lunches 1–13 |
| Collections: lists, maps, sets and their methods; lazy chains | [Collections](reference/collections.md), [Every method](reference/methods.md) |
| Generators: `yield`, `Iterator[T]`, generator methods | [Collections](reference/collections.md#generators-yield), lunch 16 |
| Files, folders, paths, the environment, time: `File`, `Dir`, `Path`, `Env`, `Time` | [Input and output](reference/io.md), lunch 15 |
| Running a program and waiting for it: `Process.run`, `Process.run_in` | [Input and output](reference/io.md#process), lunch 17 |
| A program running alongside: `Process.spawn`, `Process.spawn_in`, `Child` — settled in 0.7 | [Input and output](reference/io.md#a-program-running-alongside-processspawn), lunch 20 |
| Tasks: `async def`, `await`, `spawn:`, `shared var`, `next_done` | [Concurrency](reference/concurrency.md), lunch 18 |
| Async sequences: async generators, their chains and the methods that wait for their items, `async Iterator[T]`, an interface requiring one — settled in 0.7 | [Collections](reference/collections.md#async-generators-async-def-with-yield), lunch 19 |
| Tests: `test` blocks, `assert`, `lume test` and its name filter, `async test` (settled in 0.7) | [Tests](reference/testing.md), lunch 14 |
| Packages: `lume.toml`, dependencies by path and git, `lume.lock` | [Packages](reference/packages.md), lunch 13 |
| The `json` package | [JSON](reference/json.md) |
| The `http` package's requests — settled in 0.8. A `Request` and an `Opened` may gain fields: read them freely, but build a request with `http.request` and its steps rather than by naming every field, and do not depend on how one prints; `Opened.body_id`, the number a body not yet read is kept under, is Lume's own | [HTTP](reference/http.md), lunch 21 |
| The commands: `lume run`, `build`, `check`, `test`, `fmt`, `new`, `clean`, `update` | [Install](install.md#the-commands) |
| Rust crates: `import rust.<crate>`, `[rust]` in `lume.toml`, how a crate's types are read, builders, `lume crate` — settled in 0.8 | [Rust crates](reference/crates.md) |
| For editors and tools: the fields of `lume check --json`, and the protocol `lume lsp` speaks — settled in 0.8; what a hover or a completion says is not, as no message's words are | [Install](install.md#editors) |

## Previews

A part added before it has been used enough to be sure of its shape is a
preview, listed here; its changes are listed under **Preview:**.

| part | since | where it is described |
|---|---|---|
| Serving HTTP: `http.listen`, `Server`, `Incoming`, `Reply` | 1.1 | [HTTP](reference/http.md#serving-a-preview) |

## Not covered by the promise

- **The VS Code extension**, in `editors/vscode/`: a tool beside Lume,
  versioned with it, as Rust's editor extensions are beside Rust. It is
  built on what `lume lsp` speaks, which is settled; what the extension
  itself offers — its settings, how it shows things — may change in any
  release.
- **The wording of messages.** Errors and warnings are improved all the
  time, and a better message is never a breaking change — though which
  programs compile is covered, so a program Lume accepted is not refused
  without warning first.
- **The Rust that Lume writes.** `lume emit` shows it, and it changes
  freely; nothing should depend on its names or shape. **A `rust:` block**
  sits next to it, and so stays outside the promise for good, as Rust keeps
  `unsafe` code and its nightly features apart from what it promises; the
  helpers a `rust:` block may call, such as `lume_park`, are outside it
  with the block.
- **Speed.** Lume tries never to get slower, and the benchmarks in
  `bench/` guard that, but a program's exact speed is not a promise.
- **Bugs.** A program that leaked an error from rustc, or did something
  other than what the docs say, may be fixed to do what they say. Lume 0.6
  refusing `price = price` — which compiled before and did nothing — is
  such a fix.
- **What `lume lsp` offers an editor**, beyond its being a language server.

## How the promise is kept

Every example in `docs/` — the reference and the tutorial — every program
in `examples/`, and every refused program in `examples/errors/` is run on
every change, on Linux, macOS and Windows, and must print exactly what it
shows (a few that drive Unix programs Windows lacks run on the other two).
A change that alters a settled behaviour fails those checks, so it cannot
happen unnoticed: it has to be made on purpose, with its warning release
and its `CHANGELOG.md` entry, by the rules in `CONTRIBUTING.md`.

## Across 1.x

The promise lasts across every 1.x release, as Rust's has since its 1.0:
a program that builds on 1.0 builds, and does the same, on every 1.x after
it, but for the warning releases above and the bugs this page names. New
parts may come in a 1.x release — as a preview first, if their shape is not
yet sure — and a settled part may gain methods and fields.

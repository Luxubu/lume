# Reviewer C — round twenty-four: whole programs

## What I read

- `corpus/m120/BRIEF.md`.
- `docs/stability.md`, `docs/tour.md`, `docs/README.md`, `docs/install.md`
  (the commands, warnings, `check --json`), `docs/examples.md`.
- Every reference page used here, whole: `printing.md`, `strings.md`,
  `collections.md` (lazy chains, generators, async generators and their
  chains, `Iterator[T]` parameters), `errors.md`, `io.md` (`File`, `Dir`,
  `Path`, `Env`, `Process`, `Child`), `interfaces.md`, `structs-and-enums.md`,
  `concurrency.md`, `methods.md`, `functions-and-blocks.md`, `modules.md`,
  `names.md` (including the unused-name warnings), `control-flow.md`,
  `patterns.md`, `numbers.md`, `operators.md`, `generics.md`, `json.md`,
  `crates.md`, `testing.md`, `packages.md`.
- Tutorial lunches 12 (first half), 18, 21 and 22.
- `packages/json/lib.lume` whole (for what `pretty`, `show` and the
  `need_…` errors print exactly).
- Examples: `crate.lume`/`.expected`, `crates.lume`/`.expected`,
  `process_spawn.lume`/`.expected`, `async_interfaces.lume`/`.expected`,
  `async_generators.lume`/`.expected`, `unused_names.lume`/`.expected`,
  the top of `async.expected`, `json_config/` (all files),
  `packages/tally/` (all files) and `packages/new.expected`,
  `tests/parse.lume`/`.expected`, `tests/async_tests.lume`/`.expected`,
  `csvq/queries.sh` and the `lume test` part of `csvq/queries.expected`.

Nothing else: no compiler source, no `tests/`, no other `corpus/` file, no
`.lume` folder. Nothing was compiled or run.

## The programs

| file | what it is | how it is checked | settled parts it leans on |
|---|---|---|---|
| `c01_units/` (`lume.toml`, `lib.lume`, `convert.lume`, `main.lume`) | a library package: amounts with units, parsed, added, scaled, sorted | `lume test corpus/m120/c01_units/main.lume`, 13 tests, all pass | packages, `lib.lume`, `pub`, `pub import`, items imported by name, bare variants of another module's enum, `T or Error` with `map_error`, `?`, list patterns, `def <` for `Ordered`, tests, `async test` with `spawn:` |
| `c02_inventory/` (`lume.toml`, `main.lume`) | a stock keeper writing and reading JSON under `/tmp/lume-review-c/inventory` | `lume run corpus/m120/c02_inventory/main.lume` | the `json` package by path (`parse`, `pretty`, `obj`, `need_…`, `to_json` on lists of own types), `File`, `Dir`, `Path`, enums with payloads, `var self`, `for var`, `map_error`, `or_error`, `sort_by`, `sum` with a block, `pad`/`pad_right`, `decimals` |
| `c03_jobs.lume` | a job runner: five `sh -c` jobs at once, in tasks | `lume run`; one `shared var` warning on stderr | `Process.spawn`, `Child.lines`/`error_lines`/`wait`/`write`/`close_input`/`read_line`/`kill`, `Process.run`, `spawn:`, `await` on a list of tasks, `shared var`, `Task[T or Error]`, raw strings, `for … where` |
| `c04_logscan.lume` | a log scanner on the `regex` crate | `lume run` | `import rust.regex = "1"`, a crate type in fields, `captures`/`get`/`find`/`find_iter`/`replace_all`/`escape`/`is_match`, `?` on `T?`, a generator of pairs, maps of lists, `max_by` on a map, `"""` text |
| `c05_feeds.lume` | an async pipeline on an interface requiring an async generator | `lume run` | `async def readings -> Iterator[Int]` in an interface, written in a struct and in an `extend`, a plain default, async chains (`filter`, `map`, `take`, `enumerate`), `def … -> async Iterator[T]`, async generators over others, `next` on a `var`, the waiting methods (`count`, `sum`, `max`, `any?`, `find`, `fold`, `join`, `sort`, `each`, `last`), tasks |
| `c06_todo.lume` | a to-do command-line tool with its own failure enum and exit codes, under `/tmp/lume-review-c/todo` | `lume run`; stderr lines; exits 2 | generic enum `Outcome[T]`, list patterns with literals, guards, `..rest`, `\|`, a block parameter of type `(Int) -> Command`, `T?` returns, `return` from a match arm, `Env.args`, `Env.exit`, `warn`, `File`/`Dir` |

## Guesses

1. **Which tests `lume test` runs when it is given a package's
   `main.lume`, and how they are named.** I put every test in `main.lume`
   and none in `lib.lume` or `convert.lume`, so that whichever files are
   included, the same 13 tests run; I assumed the entry file's tests are
   named bare (no `main:` prefix). `testing.md` ("Tests in a package") says
   what happens with *no* file named, and `csvq/queries.expected` shows a
   library's root tests bare and a module's prefixed (`query: …`); neither
   says what naming one file of a package does.
   Touches: `c01_units/main.lume`. Looked: `testing.md`, `packages.md`
   (Commands), `install.md` (the commands), `csvq/queries.sh`.

2. **Naming a package's file from outside the package.** I assumed
   `lume run corpus/m120/c02_inventory/main.lume` and `lume test
   corpus/m120/c01_units/main.lume`, run from the repository root, find the
   `lume.toml` next to the file (so `import json` and `import units`
   resolve). `packages.md` says "Naming a file still works in a package:
   `lume run tools/report.lume`", but that example is run from inside the
   package; with no file named it "looks upward for a `lume.toml`" from the
   current folder.
   Touches: `c01_units/`, `c02_inventory/`. Looked: `packages.md`
   (Commands), `modules.md` (where a path starts), `install.md`.

3. **A package's own `main.lume` gets what its `lib.lume` passes on.**
   `main.lume` writes `convert.unit_of(...)` after only `import units`,
   relying on `pub import convert` in `lib.lume`. `packages.md` spells this
   out for a *dependency* ("`import tally` alone brings what `lib.lume`
   passes on"); for `main.lume` beside `lib.lume` in the same package it
   only says "`main.lume` imports the library by the package's own name".
   Touches: `c01_units/main.lume`. Looked: `packages.md` (`main.lume` and
   `lib.lume`; What carries over), `modules.md` (`pub import`).

4. **`return` from inside a `for var` loop in a `var self` method, with the
   change kept.** `Stock.change` does `g.qty += by` on a `for var` item and
   then `return ()` from inside the loop. Each piece is documented
   (`for var` in a `var self` method in `structs-and-enums.md`, `return`
   inside loops in `errors.md`), not the two together; I assumed the change
   made before the `return` stays.
   Touches: `c02_inventory/main.lume`. Looked: `structs-and-enums.md`
   (`var self`), `control-flow.md` (`for var`), `errors.md`.

5. **`()` on a type's own no-argument function in its own file.**
   `Scanner.build()?`. `functions-and-blocks.md` allows `()` on a method,
   `modules.md` allows it through a module prefix (`items.Item.blank()`),
   and `structs-and-enums.md` only shows the bare form `Temp.freezing`.
   Touches: `c04_logscan.lume`. Looked: those three pages.

6. **`next` on a `var` holding an interface's async sequence, inside another
   async generator, with no `await`.** `zipped` does `var xs = a.readings`
   and `x = xs.next`. The docs show `puts t.next` on a `var` holding an async
   generator called directly, in `async def main`
   (`examples/async_generators.lume`), and say "`s.next` on a `var` …
   wait[s]"; not on one an interface value hands out.
   Touches: `c05_feeds.lume`. Looked: `collections.md` (async generators),
   `interfaces.md` (an interface may require an async generator),
   `examples/async_interfaces.lume`.

7. **`each` with a `do |v|` block on an async sequence.** The docs show
   `words().each { |w| total += w.len }`; I used the `do` form with two
   statements, both changing outer `var`s, relying on "the three block forms
   mean the same thing" and "their block runs during the call … so it may
   change the names around it".
   Touches: `c05_feeds.lume`. Looked: `collections.md` (methods that wait),
   `functions-and-blocks.md`.

8. **A generic enum's variants built with no type written.**
   `Good(Help)`, `Bad(Usage("add TEXT"))`, `Bad(f)` as the value of a
   function declared `-> Outcome[Command]`, and `Good(i)` for
   `Outcome[Int]`. `generics.md` shows a generic enum built where a binding
   says the type (`t: Tree[Int] = Node(...)`) and says the `T` is read "off
   the type the result goes into"; `Bad(...)` has nothing else to read `T`
   from.
   Touches: `c06_todo.lume`. Looked: `generics.md`, `structs-and-enums.md`
   (functions of a type).

9. **How to write an exit code that is not 101.** The brief gives
   `EXPECTED EXIT: 101` for a program that stops; `c06_todo` ends with
   `Env.exit(2)`, and I wrote `EXPECTED EXIT: 2` in the same form.
   Touches: `c06_todo.lume`. Looked: the brief, `io.md` (`Env.exit`).

**Total: 9 guesses.**

### Where two pages disagree, or a page promises what no example shows

- **How far ahead an async generator runs.** `docs/reference/collections.md`
  says "it is never more than two items ahead of whoever walks it";
  `examples/async_generators.lume`'s opening comment says "never more than
  one item ahead". (It changes nothing my programs print.)
- `docs/tutorial/22-a-job-runner.md` §22.3 shows
  `done.sort_by { |i, o| i }.map { |i, o| o }`, which by `names.md` gives two
  unused-block-parameter warnings; §22.5, the version that is run, writes
  `_`. The skipped snippet teaches the warned form.
- `errors.md` says a stray `Error(...)` statement in a `-> () or Error`
  function is "currently accepted and silently discarded". That reads as a
  bug the 1.0 promise would then cover as behaviour; it left me unsure
  whether an `Error(...)` as the *last line* of such a function is returned
  or dropped, so `c02_inventory` writes `return Error(...)` there.
- `testing.md` documents `lume test` for one file and for a package with
  no file named, but not for one file named inside a package (guess 1).

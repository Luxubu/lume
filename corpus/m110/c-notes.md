# Reviewer C — notes (round twenty-two, whole programs)

## What I read

- `corpus/m110/BRIEF.md`.
- `docs/reference/`: `io.md` (all of it, with "A program running alongside"
  read line by line), `names.md` ("A name that is never used"),
  `testing.md`, `interfaces.md`, `collections.md` (generators, async
  generators, chains, waiting methods), `concurrency.md`, `errors.md`
  (the `!` warning, "Where the program stopped"), `methods.md`,
  `strings.md`, `printing.md`, `control-flow.md`,
  `functions-and-blocks.md`, `structs-and-enums.md` (first 400 lines),
  `generics.md`, `patterns.md` (first 200 lines).
- `docs/stability.md`.
- `docs/tutorial/`: lunches 17 (async sequences), 18 (programs alongside),
  22 (a job runner, first 200 lines), 24 (a language server), the test
  part of 12, and the warning paragraph of 9.
- `examples/`: `unused_names.lume` + `.expected`, `async_interfaces.lume`
  + `.expected`, `process_spawn.lume` + `.expected`, `async_generators.lume`
  + `.expected`, and every file in `examples/tests/`.

Nothing outside `docs/`, `examples/`, `packages/` and the brief. Nothing was
compiled or run; every expected output was worked out by hand (string
padding checked with a calculator-style script, not with Lume).

## The programs

| file | what it is | probes | outcome |
|---|---|---|---|
| `c01_logpipe.lume` | a log pipeline: lines through `sort`, `uniq`, `wc -l`, a `sh` splitter, `tr`, `cat`, `wc -w` | `spawn`, `write`, `close_input`, `lines` as list and async chains (`map`, `enumerate`, `sum`), `error_lines`, `read_line` to the end, `wait`, a failing stage as `Error` | RUN, 17 lines |
| `c02_plugins.lume` | a plugin system: four types fit `Plugin` (requires `async def items(n) -> Iterator[Str]`), a driver collects each in a task and merges round-robin; a second merge interleaves two async sequences with `next`; a generic `Scores[T]` fitted by a struct and an `extend` | async-generator interfaces both spellings, `extend` of a struct and of `[Str]`, generic interfaces, functions taking the interface, defaults, a chain handed on as `async Iterator` | RUN, 18 lines |
| `c03_lenproto.lume` | a REPL-like client for a `sh` loop that answers with a length line and a body | `Child` in a struct field, `read_line` + `read(n)` with bodies holding line breaks and no trailing newline, `read(n)` past the end, `None` after the end, `wait`, `write` after `wait` | RUN, 18 lines |
| `c04_jobs.lume` | a job runner whose jobs are `Job` interface values: shell jobs, a fixed answer, `Str` via `extend`, a job holding two jobs in fields | `[Job]` binding (warning predicted), interface fields, defaults overridden, interface values into `spawn:`, `await` on a list of tasks | RUN, 8 lines + 1 warning |
| `c05_tests.lume` | async helpers with ten tests, plain and `async test`, run with `lume test` | awaiting an `async def`, tasks, async generators and chains, a `Child` in a test; a failed comparison, a failed non-comparison `assert`, a test that stops inside a helper | `lume test`, exit 1 |
| `c06_wordtool.lume` | a word-count tool with names left over | each kind of unused-name warning (parameter, local, `for`, block parameter, pattern), and the cases that must not warn (`_name`, `_` twice in a block and in a `for`, a `var` parameter only changed, a `for` variable read only by `where`) | RUN, 9 lines + 5 warnings |

## Guesses

1. **A type may meet an interface's `async def items(n) -> Iterator[Str]`
   with a plain `def items(n) -> async Iterator[Str]` that hands on a chain**
   (`Evens`). The docs say the two spellings "say the same" for an
   interface, and `async_interfaces.lume` shows the interface written with
   `def ... -> async Iterator` and the type with `async def`, but never a
   type whose method has no `yield`. — `c02`; `interfaces.md`,
   `collections.md` "Async generators", `examples/async_interfaces.lume`.
2. **An `extend` of a built-in type may hold an async generator method**
   (`extend [Str] with Plugin`). Shown: an async generator in an `extend` of
   a struct (`Pantry`), and a plain generator in an `extend Str`. Not shown
   together. — `c02`; `interfaces.md`, `examples/async_interfaces.lume`.
3. **A parameter `a: async Iterator[Str]` can be moved into a `var`
   (`var xs = a`) and walked with `xs.next`**, which waits without `await`.
   Docs show `next` on a `var` holding an async generator, and that a
   parameter takes the sequence by move; not the two together. — `c02`;
   `collections.md`, `examples/async_generators.lume`, lunch 17.
4. **A list literal of `spawn:` expressions may span lines with a trailing
   comma.** Docs show only one-line lists of `spawn:` items. — `c02`;
   `concurrency.md` ("next_done"), `collections.md` ("A literal may span
   lines").
5. **The `write` error names the program as it was given**:
   ``cannot write to `sh`: its input is closed``. The reference shows only
   `sort`. — `c03`; `io.md` table on `Child`.
6. **`read(n)` gives `None` once the output has ended and nothing is
   left.** The reference says what is left comes back when the program ends
   first (relied on too: header says 9, body is 3), and that `read_line`
   gives `None` at the end, but not what `read` gives then. — `c03`;
   `io.md`, lunch 18.5.
7. **A `Str` literal is accepted as an item of a `[Job]` list literal**
   through `extend Str with Job`. Docs show a list literal (`[10, 20]`) in
   a `[Walk]` list, and `extend Str with ...` methods, but not a `Str` held
   as an interface value. — `c04`; `interfaces.md`.
8. **An interface value from a `[Job]` list can be copied into a `spawn:`
   body** (`jobs.map { |j| spawn: j.run }`). `concurrency.md` says anything
   can be copied in, and `collections.md` says interface values are copied
   through `lume_box`; no example sends one to a task. — `c04`.
9. **The interface-binding warning for a binding whose list literal spans
   lines shows only the first line** (`  jobs: [Job] = [`), caret at the
   name. The only example is one line. — `c04`; `interfaces.md`.
10. **A failed `assert` that is not a comparison prints only the
    `file:line: assert ...` line**, with nothing after it. `testing.md`
    says it "shows the line and the `assert`" but gives no text block. —
    `c05`; `testing.md`.
11. **A `for` variable read only by the loop's `where` counts as used**
    (no warning for `for w in words where w.len > 2: c += 1`). Not
    mentioned anywhere. — `c06`; `names.md`, `control-flow.md`.
12. **A block parameter's warning points at the block's own `{`** even when
    an earlier `{` (a map type) is on the line: `71:62`, not `71:21`. The
    example has only the block's brace on its line. — `c06`;
    `examples/unused_names.expected`.
13. **The ordinary tools behave the same on every machine for plain
    lowercase ASCII**: `sort` orders these lines bytewise whatever the
    locale, `wc` numbers are trimmed (macOS pads them), `read cmd arg` in
    `sh` puts the rest of the line in `arg`. Not Lume's docs, but each
    expected output leans on it. — `c01`, `c03`, `c04`, `c05`.

Where the docs promise more than the reference shows, or disagree:

- `collections.md`, "Generators: `yield`", still lists **"Not yet: an
  `async` generator in an interface."**, while the "Async generators"
  section below it, `interfaces.md` and `stability.md` say an interface may
  require one, settled in 0.7.
- `stability.md` opens with "Lume is at 0.6" while its table says three
  parts are "settled in 0.7".
- `stability.md` settles `Child`, but `io.md` does not say what `read(n)`
  gives once the output has ended (guess 6), nor whether an interface
  value held in a list may call a required async generator (I kept every
  call on a parameter, since `interfaces.md` refuses `async` methods on
  held values).
- Lunch 24's `Server` struct (a `Child` in a field, awaited from methods)
  is `lume-skip`, so the shape `c03` relies on is documented but not one of
  the checked examples.

**Total: 13 guesses.**

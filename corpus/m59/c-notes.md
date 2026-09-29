# Reviewer C — round twelve notes

## What I read

- `corpus/m59/BRIEF.md`.
- `docs/README.md`, `docs/tour.md`, `docs/install.md`, `docs/examples.md`.
- `docs/reference/`: `concurrency.md`, `io.md` (all of it, "Process"
  twice), `errors.md`, `names.md` (the `Task` section), `packages.md`,
  `functions-and-blocks.md` (kept blocks), `control-flow.md` (`for (i, x)
  in`), `structs-and-enums.md`, `interfaces.md`, `methods.md`,
  `collections.md`, `printing.md`, `strings.md`, `patterns.md`, and
  `modules.md`.
- `examples/jobr/` (all of it: both packages, `jobs.txt`, `check.sh`,
  `queries.expected`, `FINDINGS.md`), `examples/csvq/` (all three packages,
  `queries.sh`, `FINDINGS.md`), `examples/packages/`, `examples/async.lume`,
  `examples/kept_blocks.lume`, `examples/match/`, and the start of
  `examples/port/app_async.lume`.
- I did not read `docs/QUESTIONS.md`, the compiler, `tests/`, or any other
  file in `corpus/`. I ran nothing: no compiler and none of the programs.

## The programs

All seven are Meant to RUN. Each one's output depends only on what it
starts and in what order, never on which task finishes first. Where it
matters, a command sleeps a second so that it finishes last, and it is
still reported in its place. Every file a program makes goes under
`/tmp/lume-m59-cNN`, and the program removes that folder before it ends
(and also at the start, if an earlier run left it behind). The commands
use only `sh -c`, `echo`, `printf`, `true`, `false`, `cat`, `wc`, `sort`,
`sleep` and `test`. `wc` output is always trimmed, because macOS pads it
with spaces and Linux does not.

| file | what it is | leans on |
|---|---|---|
| `c01_checkrun/` + `c01_checks/` (package + library) | A parallel test runner. It runs six `sh -c` checks in tasks and reports PASS/FAIL, with the exit code and the output, in the order the checks were listed. | a library `pub async def` that uses `spawn:`; `await` through a package prefix; `Process.run` exit code, stdout and stderr; an enum with payloads; `Ok((0, _, _))` patterns; `for (i, r) in` |
| `c02_pipeline.lume` | A build pipeline. Its stages are structs that hold a kept `(Str) -> Str or Error` block. Stages in the same wave run at once. One stage writes files, the others run commands in a scratch folder. A failed stage stops the pipeline after its wave. | kept blocks in struct fields, including a function's name used as one; a struct holding a block copied into `spawn:`; `(code, out, err) = Process.run(..)?`; `for (s, r) in xs.zip(ys)`; `break` |
| `c03_fetcher/` + `c03_fetch/` (package + library) | A retrying fetcher. Three sources are fetched at once, with three tries each. `sh -c` and marker files simulate the refusals. At the end the program lists the markers and removes the folder. | `map_error` around `Process.run`, then `?` and tuple destructuring; `return` from a `match` arm inside a `while` inside a `pub async def`; `await Time.sleep`; a constant inside `spawn:`; `Dir.list` order |
| `c04_jobgraph/` + `c04_graph/` (package + library) | A job graph like jobr's, written in a `name: needs \| command` format. It runs level by level, one task per job, and skips a job whose need failed. Results go into a `shared var` map that is printed sorted. Three bad graphs are then checked with the library's `plan`. | `match` arms that end in `Env.exit(2)`; a `shared var` map written by key inside and outside tasks; a `"""` constant; `for (i, level) in`; `for (name, ok) in names.zip(..)`; `Process.run` in a plain `def` inside a task |
| `c05_own_task.lume` | A to-do list whose items are the program's own `struct Task`. Each item's command runs in a task. The outputs go into a `shared var` map and a `shared var` counter through a helper, and the map is printed sorted by title. | milestone 57: `struct Task` beside `spawn:`, with the task type never written; `shared var` map and `Int` parameters used from a `spawn:` inside `.map { }` |
| `c06_steps.lume` | An interface `Step` with four conforming structs. One of them, `Shell`, pipes its input through `sort -u` using a scratch file. The step list runs on three inputs at once, and the third fails at `keep 3`. | structural conformance and a default; `[Step]` copied into `spawn:`; `map_error` inside a method; `?` through a list of steps; `File.write`/`File.remove` in tasks |
| `c07_linecount/` + `c07_shell/` (package + library) | Counts file lines in parallel through a library whose `run`, `sh`, `lines` and `count` return `T or Error` built with `map_error` around `Process.run`. The counts are added to a `shared var` total. The run also covers a missing file, a non-number, and a program that cannot start. | `map_error` in the library and in the program; `?` passing a library failure through unchanged; `await t?`; `?` inside a `spawn:` body; `for (name, body) in` over a constant list of tuples |

There are four packages with a library beside them: c01, c03, c04 and c07.

## Guesses

1. **Where a library goes.** I looked at the brief ("dependencies beside it
   by `path`"), `packages.md` and `examples/jobr/`. I assumed the library
   is a sibling folder in `corpus/m59/` with its own `lume.toml` and no
   `main.lume`, found as `path = "../cNN_lib"`. I gave each folder the same
   name as its package (`c01_checks`) so that nothing depends on whether a
   folder name must match `name`. I also assumed a package name may contain
   digits after its first letter; `packages.md` says "lower case letters,
   digits and `_`".
2. **`Process.run` in a plain `def` that runs inside a task** (c01 `run`,
   c02 `sh`, c04 `run_node`, c05 `record`, the c07 library). `io.md` shows
   it in a plain `def main`, and jobr calls it in an `async def` with no
   `await`. Neither says what happens in a plain `def` called from
   `spawn:`. I assumed it needs no `await` and only holds up that thread,
   as `Time.sleep` does in a plain `def`.
3. **A struct holding a kept block, copied into `spawn:`, with its block
   called there** (c02: `wave.map { |s| spawn: s.act(ROOT) }`).
   `functions-and-blocks.md` says a kept block "may go to another task",
   and `concurrency.md` says "anything can be copied in". Neither shows a
   struct with a block field crossing into a task.
4. **A function's name as a block-typed field in a constructor** (c02:
   `Stage(name: "sources", wave: 1, act: write_sources)`). The docs show
   `g: (Int) -> Int = double` and a list `[double, adder(1)]`, but not a
   constructor argument.
5. **A list of interface values, `[Step]`, copied into `spawn:`** (c06).
   `interfaces.md` says such values sit behind a pointer. Nothing says
   whether that pointer can go to another task. I assumed it can, by the
   "no special rule for what may cross" line.
6. **Brackets around two names in a `for` over `zip` or over a list of
   tuples**: `for (s, r) in wave.zip(results)` and `for (name, body) in
   FILES` (c02, c04, c06, c07). `control-flow.md` shows the bracket form
   only with `.enumerate`. `concurrency.md` shows `zip` and tuple lists only
   with bare names (`for n, r in`, `for who, cents in`). I assumed "means
   the same" holds for every pair.
7. **`shared var` values handed to a plain `def` from a `spawn:` inside a
   `.map { }` block** (c05: `todo.map { |t| spawn: record(t, outputs, ran)
   }`; c07 the same with `total`). The docs show `spawn:` inside `.map { }`
   only with read-only `shared`, and `shared var` helpers only from a
   `spawn:` in a `for` loop. I needed the `.map` form in c05 because my own
   `Task` hides the name for `[Task[Bool]]`.
8. **A `shared var {Str: Str}` parameter, assigned by key through the
   parameter** (c05: `outputs[t.title] = text` in `record`).
   `concurrency.md` says "a parameter can be typed `shared var T`" but
   shows only `shared var Int`. It shows key assignment only on a
   `shared var` local.
9. **What `EXPECTED OUTPUT` covers.** The brief does not say. I listed
   standard output only. The `shared var` warnings (c04, c05, c07) and any
   `warn` go to standard error, according to `concurrency.md`, "Where the
   warnings go".

I did not have to guess the following, but I stepped around them:

- **The message when a program cannot be started is not in the docs.**
  `io.md` shows only `Process.run("no-such-program", []).error?` giving
  `true`. So c07's library drops the system's reason and uses its own text,
  and the `Error(e)` branches of c01, c04 and c05, which would print
  `e.message`, never run.
- **Shell messages differ between shells.** A redirect from a missing file
  prints something different in dash and in bash. c07 therefore checks
  with `test -f` and prints its own "no such file".

## Verdict

These programs were less guesswork than earlier rounds. `io.md`'s Process
section, the jobr and csvq examples, and `concurrency.md`'s notes on
ordering and copying answered almost everything a real runner needs:
results in start order, failures that come home, `Env.exit` in an arm,
`map_error`, and `?` inside a task. Every remaining guess is about **what
may cross into a task**, and about where the docs' examples stop short of
a combination:

- kept blocks inside structs;
- interface values;
- `shared var` maps as parameters;
- `Process.run` from a plain `def` inside a task;
- the bracket form of `for` beyond `enumerate`.

Two short additions to the docs would remove most of them:

- a line in `concurrency.md` saying whether a struct holding a block, and a
  `[SomeInterface]`, can go into `spawn:`;
- one example in `io.md` of `Process.run` inside a plain helper called from
  a task.

The start-failure message of `Process.run` should also be shown, so that a
program can print it and still promise its output.

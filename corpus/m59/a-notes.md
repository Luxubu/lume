# Reviewer A: everyday tools and scripts

## What I read

- `corpus/m59/BRIEF.md`.
- `docs/README.md`, `docs/tour.md`, `docs/install.md` (commands, "Editors"),
  `docs/examples.md`.
- `docs/reference/`: `io.md` (all of it, "Env" and "Process" twice),
  `errors.md`, `names.md` (the `Task` section), `control-flow.md` ("`for`
  over pairs"), `concurrency.md`, `printing.md`, `structs-and-enums.md`,
  `functions-and-blocks.md`, `patterns.md`, `methods.md`, `strings.md`,
  `collections.md`, `operators.md`, the start of `numbers.md`.
- `examples/`: `jobr/` (both packages, `FINDINGS.md`, `queries.expected`,
  `check.sh`), `async.lume`, `enumerate_blocks.lume`, `check_json.expected`,
  `errors/own_task_args.*`, parts of `csvq/csvq/main.lume`,
  `site/main.lume`, `errors_and_results.lume` and `stdlib.lume`. I grepped
  `examples/` for `Process`, `Env.`, `map_error` and `Task`.

I did not read `docs/QUESTIONS.md`, the compiler, `tests/`, or anything else
in `corpus/`. I ran nothing. The only command I ran on my own files was a
Python check that the padded columns in the a06 and a10 headers are as wide
as I meant them to be.

## The programs

All ten are single files, Meant to RUN. Each header gives the exit status
too, because three of them end through `Env.exit`.

| file | what it is | leans on |
|---|---|---|
| `a01_shell_report.lume` | runs `echo`, `printf`, `sh -c`, `true` and a missing program, then prints each exit code and each output line | `Process.run`, `(code, out, err) = ..?`, `Ok((code, out, _))`, `.error?` on a failed start, a `() or Error` helper called with `?` |
| `a02_make.lume` | a small "make": a recipe of `name: command` lines, run in order, stopping at the first failure with that step's code (exits 2) | `Process.run("sh", ["-c", ..])`, `Env.exit(code)` inside an `if` in a `match` arm, a triple-quoted recipe, `split(": ")` / `skip(1)` |
| `a03_config.lume` | a `key = value` config reader in which every failure becomes one clear message | `map_error { \|e\| Error("port: #{e.message}") }?`, `.or_error(..)?`, `f(..)?.to_int`, a `(Str, Str) or Error` function, list patterns on `split("=")`, `for (i, raw) in text.lines.enumerate` |
| `a04_todo.lume` | a to-do list with its own `struct Task`, and no concurrency | own `Task`, `var self` methods, `for var t in tasks` with `return ()`, a `() or Error` method called with `?`, printing a `Task` |
| `a05_task_spawn.lume` | its own `Task` (a named command), with one `spawn:` task per `Task` and then one more task that uses `?` | own `Task` beside `spawn:` with the task type never written, `await` on a list in start order, `Process.run` inside a task, a raw string for the shell |
| `a06_report.lume` | an invoice report: numbered lines, a total, a ranking and a legend | `for (i, it) in` over a list, over a `sort_by` result and over `split.enumerate`, `pad` / `pad_right`, `decimals(2)`, `sum` |
| `a07_args.lume` | reads `--loud`, `--count N` and a name from `Env.args`, with defaults; then runs the same parser on argument lists written in the program | `Env.args`, `args[i].or_error(..)?`, `map_error(..)?` in a `match` arm inside `while`, `opts = match .. Error -> Env.exit(2)`, `Env.get(..).or(..)` |
| `a08_exit_arm.lume` | functions whose last expression is a `match` or an `if`/`else` with a branch that warns and runs `Env.exit`. That branch is never taken | `Env.exit` as a branch of a function's *result*, not only of a binding |
| `a09_wordcount.lume` | counts words by running `printf '%s' '<text>' \| wc -w` and compares the count with `split`, then lines, bytes against characters, and a `sort` pipe | `sh -c` pipes, `out.trim.to_int.map_error(..)` as a function's result, a helper that turns a non-zero exit into an `Error` |
| `a10_harness.lume` | a test harness: `true`, `false`, `test`, `sh -c 'exit 3'` and a missing program, each against the code it should exit with. Prints a tally and exits 1 | `Ok((code, _, _))`, `Error(_)` giving BROKEN, an enum outcome, a `var self` tally, `Env.exit(1)` after all output |

## Guesses

1. **An `Env.exit` arm in a `match` that is a function's result.**
   `mode_from(s) -> Mode` ends in a `match` whose last arm is
   `warn ..` then `Env.exit(2)` (a08). `io.md` "Env" shows this only as the
   value of a binding (`n = match "12".to_int: ..`). `jobr/FINDINGS.md` item
   2 speaks generally ("such a branch now has no value to agree on"), but
   every example I found binds the `match` to a name. I assumed that the
   function-result position works in the same way.
2. **An `else` branch that ends in `Env.exit`, as a function's result.**
   `pick(xs, i) -> Str` is `if .. xs.at(i) else: warn ..; Env.exit(4)` (a08).
   `io.md` says "can end a branch whose other branches give a value", but its
   only example is a `match`. I looked in `control-flow.md` ("`if` as a
   value") and found nothing on it. I assumed that an `if` branch counts as
   "a branch".
3. **A function you declare as returning a tuple or an `Error`.**
   `parse_line(..) -> (Str, Str) or Error`, where one arm gives a bare
   `(k.trim, v.trim)` as the implied `Ok` (a03). The only `(..) or Error`
   type in the docs is the one `Process.run` returns (`io.md`). `errors.md`
   says a bare `T` is wrapped for any `T`, but every example there uses
   `Int`. I assumed that a tuple `T` is wrapped in the same way.
4. **`?` on a call to your own `() or Error` function or method, as a line
   of its own.** Examples are `show(..)?` (a01) and `todo.finish(1)?` on a
   `var self` method (a04). The docs show this only for built-ins
   (`Dir.make(dir)?`, `File.write(..)?` in `io.md`), and `csvq` uses it only
   with a value bound. I assumed that the statement form works for any
   `() or Error` call.

These are not about Lume, so I did not count them: `false` and
`test 1 -gt 2` exit 1; `test` exists as a program on `PATH`; `wc` may pad
its number with spaces (so every count is trimmed); `wc -c` counts bytes;
`sort` puts lowercase ASCII words in the same order in any locale; and
`LUME_A07_USER` is not set. I also chose how to record things the brief
does not say how to record: each header states the exit status and says
whether standard error is empty. `warn` lines are not in EXPECTED OUTPUT,
because EXPECTED OUTPUT is standard output only and no `warn` runs in these
programs anyway.

Some questions I checked and settled:

- `Process.run` needs no `await` inside async code: `jobr/jobr/main.lume`
  calls it bare in an `async def`.
- `for (i, x) in chain.enumerate` means the same as `for i, x in`
  (`control-flow.md`, `methods.md` "Lazy chains").
- `f()?.method` parses: `io.md` has `Env.stdin?.lines`.
- `map_error` followed by `?`: `jobfile/lib.lume` does exactly this.
- A last-line `Error(..)` in a `() or Error` function: `errors.md` says a
  stray one there is silently discarded. It is unclear whether that
  includes the last line, so I wrote `return Error(..)`.
- `pad` fills with spaces, never zeros, so money goes through `decimals`.
- Own `Task` and `spawn:`: the pattern in `names.md` is followed exactly,
  and I never write the task type.

## Verdict

The m55–m58 surface was quick to write against. `io.md` "Process" answers
the questions that matter for a script:

- what an `Error` is from `Process.run`: only a program that fails to
  start;
- what a non-zero exit is: a value;
- what a signal gives: `-1`;
- that no shell is involved.

The `(code, out, err)` tuple takes apart cleanly both with `?` and in a
`match`. `map_error` did what `errors.md` and `jobfile` show, and adding
context to a parse error took one line each time. With `names.md` I had no
doubt about a `struct Task` of my own, alone or beside `spawn:`.

All four guesses are about how general a feature is, not how to spell it.
Two are about `Env.exit` in a position other than `x = match ..`. One line
in `io.md` would settle both: "a function's last `match` or `if` may end a
branch in `Env.exit`", with an example. The other two would be settled by
one example each: a user function returning `(A, B) or Error`, and a
`() or Error` method of your own called with `?` on a line by itself.
Nothing I needed from `Process`, `Env`, `map_error`, own `Task` or
`for (i, x) in` was missing from the docs.

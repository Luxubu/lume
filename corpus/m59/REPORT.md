# Milestone 59 review corpus — REPORT (twelfth review round)

This round tested milestones 55–58:
- `Process.run`, which runs another program;
- `map_error`;
- `Env.exit` ending a branch;
- a `Task` of your own;
- `for (i, x)`;
- the error messages, which `lume lsp` now shows in the editor as the
  program is typed.

Three reviewers wrote twenty-nine programs from `docs/` and `examples/`
alone, with nothing run, and counted what they had to guess. The brief is
in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a10` | everyday tools: shell reports, a small make, a config reader using `map_error`, a to-do app with its own `Task`, a test harness |
| `b01`–`b12` | the edges of the new features, seven of them meant to fail with the exact message, and three with the exact `lume check --json` output |
| `c01`–`c07` | whole programs: parallel test runners, pipelines, a retrying fetcher, a job graph, an interface over several steps; four of them packages |

## The numbers

| | round eleven | round twelve |
|---|---|---|
| guesses | 18 | **38** (A 4, B 25, C 9) |
| meant to run, right on the first run | 28 of 30 | **21 of 22** |
| refused as predicted | — | 7 of 7 refused, **2 word for word** |
| refused although correct | 0 | **1** |
| rustc leak | 1 | **0** |
| wrong output | 0 | 0 |

After the fix, **22 of 22** programs print exactly what their authors worked
out by hand, and each ends with the exit status its author gave it: 0, or
2, 1 and 5 for the four that end on purpose through `Env.exit` or `?`.

The count of guesses rose, and the notes say why. Reviewer B spent sixteen
of its twenty-five guesses on the wording of misuse messages for features
no page or example showed misused. Five of those messages were also worse
than they needed to be. The other two reviewers guessed four and nine
times, about combinations of features the docs show only one at a time.

## What the programs found

1. **`Env.exit` ending a function's result was refused** (b07). Milestone 58
   let a branch end in `Env.exit` when the `match` or `if` was a binding's
   value. When it was the function's result, the branch was checked against
   the return type as `()`. Now `Env.exit` passes there as well, as Rust's
   `!` does.

## Messages that now say what to write

2. **A result taken apart as a tuple** (b02),
   `(code, out, err) = Process.run(..)`, now says the tuple is inside a
   result and shows `?`.
3. **`map_error` on a `T?`** (b06) used to say "only lists and ranges can"
   take a block. Now it says `map_error` is for a result, and that
   `.or_error` turns a `None` into a failure.
4. **Two different types both printed `Task[Int]`** (b09). In a program with
   a generic `Task[T]` of its own, pushing a `spawn:` task into a
   `[Task[Int]]` said a `Task[Int]` is not a `Task[Int]`. Now it says which
   is which, and to leave the task's type out.
5. **`var shared [Int]`** (b12) was "expected `)`". Now it says `shared var`
   is written in that order.
6. **`Process.run("sh", "-c true")`** now shows the arguments as a list.

Each of these, and `map_error` given a block that is not an `Error`, has an
example in `examples/errors/`. That was the gap behind B's sixteen guesses.

## Where the other guesses went

- **What may cross into a task** (C, 5): a struct holding a kept block, a
  list of interface values, a plain `def` that runs a program. All work.
  `io.md` now says that a plain `def` holds its thread while its program
  runs.
- **`Env.exit` as a function's result** (A, 2): now fixed, and said.
- **The message when a program cannot start** (B and C): now quoted in
  `io.md`.
- **A tuple-or-`Error` of your own, and `?` on your own `() or Error`
  call** (A, 2): both work as guessed.

## Verdict (condensed from the three notes)

A: the new features are learnable, and the few guesses were about using
two of them at once. B: none of the new features came with an example of
getting it wrong, so every message was a guess. C: every remaining guess
was about what may cross into a task.

One defect, in the newest milestone. No output was ever wrong, and nothing
leaked into rustc. What the round mostly found was messages, which matter
more now that they appear while the program is being typed.

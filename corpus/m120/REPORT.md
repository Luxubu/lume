# Milestone 120 review corpus — REPORT (twenty-fourth review round)

The last round before 1.0, aimed at what 1.0 promises rather than at one
release: each settled part of `docs/stability.md`, as someone meeting Lume
for the first time would find it. Three reviewers wrote thirty programs
from `docs/`, `examples/` and `packages/` alone, with nothing run. The
brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | a newcomer's first weeks, learnt from the tutorial in order: a word tally, a struct-and-enum model, `?` and `T or Error`, files under /tmp, a generator pipeline, tasks, a test file, a stop, a refusal |
| `b01`–`b15` | one program for each row of the settled table, taking its reference page at its word: nested patterns, a chain's methods, a generator used up, each I/O edge, `Process.run` and `spawn` edges, a task's copy, async `uniq`, failing tests, `import lib`, `json`, `http`, `lume fmt`, a crate's `usize`, warnings in `--json` |
| `c01`–`c06` | whole programs: a units library with its own tests, a stock keeper on `json`, a job runner, a log scanner on `regex`, an async pipeline on an interface, a to-do tool with exit codes |

## The numbers

| | round twenty-three | round twenty-four |
|---|---|---|
| guesses | 48 | **42** (A 4, B 29, C 9) |
| meant to run, stop, test, format or check as JSON, right on the first run | 18 of 24 | **22 of 24** |
| refused as predicted | 3 of 6 | **3 of 6**, word for word (the other three by their message, not their help) |
| rustc leak | 3 | **1** (b01) |
| wrong output | 0 | 0 |

No program printed a wrong answer. All six whole programs ran as their
author predicted on the first run, and so did everything reviewer A, who
learnt Lume from the tutorial alone, meant to run.

## What the programs found

1. **A list pattern inside another pattern reached rustc** (b01):
   `(Say([]), _)`, `Some([a, ..rest])`, `[[a, b], [c]]`. `patterns.md`
   promises that every pattern goes inside every other, to any depth, and
   a list did only when matched on its own. Inside anything else it is bound,
   tested as a slice in the arm's guard, and taken apart at the top of the
   arm now, as a recursive enum's field already was. Alternatives of `|`
   that name nothing may differ in shape and hold literals now
   (`("a", _) | (_, 2)`, refused before); one that would name what is inside
   a list is refused with its own message.
2. **`seen.push(1)` inside `spawn:` on an outside `var` said `seen` was
   not a `var`** (b07), and offered to make it one. It says what `seen =
   ...` does: the task changes its own copy, and `shared var` is the fix.
3. **A chain's methods, listed in an error, left out `at`** (b02), which a
   chain has and `methods.md` says so.

## What the audit found

Before the round, every settled row was checked against its page:

- **A stray `Error(...)` in a `-> () or Error` function** returned from it
  — a rule from milestone 7 that no page described — while in every other
  function the same line is refused with "this failure is thrown away";
  `errors.md` said it was silently dropped. Reviewer C flagged the page
  too. It is refused the same way now, as a bug fixed by the rule
  `stability.md` gives for bugs; an `Error` that ends a branch at the
  function's end is still its value, nested `if`s and `match`es included
  (which, mixed with `()` branches, were refused before).
- **Stale numbers** in the README and the docs: twenty review rounds (twenty-four
  with this one), 662 review programs (782), 394 refused
  programs in one place and 106 in another (445 now), 5,000 lines of
  examples (7,000). `tests/corpus.sh`, which classified only the first seven
  rounds and was run by nothing, is gone with its paragraph; the suite
  checks every round.
- `stability.md` said the promise is checked on Linux and macOS; Windows
  too, since milestone 106.
- The "Not yet" sections left (a registry; serving HTTP, cookies) are about
  what may be added, and hedge nothing settled.
- **Speed**: every benchmark within budget; `bench/BASELINE` holds the 1.0
  numbers (b8 from 230% of Rust to 200%, b3 113% to 107%).

## What the docs now say

- `patterns.md`: a list pattern inside any other, with an example; which
  alternatives may differ in shape (B).
- `errors.md`: the stray `Error` refused in a `() or Error` function too;
  the second wording of a dropped failure, "whether it did"; exit status 1
  for a failure out of `main` and 101 for a stop, not "non-zero" (A, B, C).
- `concurrency.md`: `await` covers the call and the method steps to its
  right, and an operator ends it — `assert await t == 9` (A).
- `collections.md`: the nine methods an async sequence keeps every item
  for, by name; `examples/async_generators.lume` says two items ahead, as
  the page does, not one (B, C).
- `methods.md`: a tuple's method error is a proper one (B).
- `io.md`: `Path.dir("/top")` is `/` (B).
- `names.md`: the whole help for a name changed in a nested block (A).
- `install.md`: an option may come before the file or after it (B).
- `testing.md`: `spawn:` in an `async test`; naming a package's file runs
  the package's tests. `packages.md`: a file finds its `lume.toml` from
  wherever the command is run (B, C).
- Lunch 10: `!` is warned about outside tests. Lunch 21 and 22: unused
  block parameters written `_` in the fragments, as in the whole programs.
  Lunch 23: `Time.sleep` holds up its thread, which in 23.4 is the check's
  own (A, C).

## Left

- Lunch 3 calls `split`'s result a list; it is a chain that prints and
  measures as one, which `methods.md` says. Left as the tutorial's plain
  word.
- The version lines ("Lume is at 0.8", "Written for Lume 0.8") are the
  release's to change: milestone 121, 1.0.0.

# Milestone 53 review corpus — REPORT (eleventh review round)

This round tested milestones 50–52: values that **move** instead of being
copied where they are not needed again. A move never changes what a
program prints; a wrong one makes the program fail to build. So every
program here was meant to run, and its worth was in how close it walked to
the edge. Three reviewers wrote thirty programs from `docs/` and
`examples/` alone — nothing run — and counted what they had to guess. The
brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a10` | everyday code: reports built line by line, records filtered and re-sorted into the same name, grouping, struct pipelines, word counts, recursion returning lists |
| `b01`–`b12` | breaking the move rules on purpose: `while` conditions, a name twice in one statement, a struct read after its fields went, the same field twice, guards, `?`, `.or`, tuples, `a.b.c` paths, kept blocks, `shared var`, tasks, generics, early `return` |
| `c01`–`c08` | whole programs: a stack machine replacing its state every step, generic containers, interfaces building new values, enums with payloads, tasks with `shared var` maps, kept blocks in structs; four folders of modules and two packages |

## The numbers

| | round ten | round eleven |
|---|---|---|
| guesses | 38 | **18** (A 4, B 6, C 8) |
| right on the first run | 12 of 18 | **28 of 30** |
| rustc leak | 0 | **1** |
| refused although correct | 5 | 0, and one reviewer's slip |
| wrong output | 0 | 0 |

After the fix: **30 of 30** print exactly what their authors worked out by
hand. None of the eighteen guesses was about when a value moves. Reviewer B
set out to break the rules in twelve programs and named five places a
compiler would most likely get wrong. It got none of them wrong.

## What the programs found

1. **Taking a part out of a tuple moved it out even when the tuple was
   used later** (b06). `inner = nest.0` followed by `puts nest` did not
   build. This is older than milestones 51–52, which never touched tuples,
   but it is the same question, and it now has the same answer: a tuple part
   is taken out only when the tuple is finished with and the statement names
   it once; otherwise it is copied.

a08 named a struct `Task`, which is a built-in name (`Task[T]`, what
`spawn:` gives). The compiler refused it with a message saying so, and the
program was changed to `Chore`. **It is the second round running in which a
reviewer reached for `Task`.** Rust has no such reserved name, and a user's
own `struct Task` shadowing the built-in one where `Task[..]` is never
written is worth considering. That is left as an open question.

## Where the guesses went

- **Changing what lives inside a struct** (A, 3): replacing a whole field
  with `=` in a `var self` method, `for var` over a field, taking apart and
  then replacing a destructured name. All work. `structs-and-enums.md` now
  shows the first two, and the moves section says a destructured name is a
  local like any other.
- **`shared var` outside a task** (B, 3): `=`, replacing from itself,
  handing to a plain parameter. All work, and `concurrency.md` now says so.
- **A kept block and the `var` it copied** (B, 1): replacing the `var`
  afterwards leaves the block's copy alone. This is now shown in
  `functions-and-blocks.md`.
- **The wording "writes `w` onto the end of `acc`"** (B, 1) suggested
  `acc` changes partway through the statement. The docs now say everything
  on the right is read first.
- **Spelling two documented features together** (C, 8): a typed multi-line
  constant, a three-part destructure, a generic method returning its own
  type at another parameter, a block bound by a pattern. Every guess was
  right.

## Verdict (condensed from the three notes)

A: moves are invisible, which is the point; my guesses were about changing
things inside structs. B: the rules were enough to work every output out,
and the gaps were about `shared var` and kept blocks, not moves. C: none of
my guesses was about moves.

One defect, older than what this round was aimed at, and zero in the move
analysis itself. For the first time, a round aimed at a change to how the
compiler generates code found nothing in that change.

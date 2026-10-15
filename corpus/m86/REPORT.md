# Milestone 86 review corpus — REPORT (eighteenth review round)

This round tested what milestones 82–85 added: async generators, tasks that
take a generator, a stream or another task instead of copying it, and
`Process.run_in`. The brief asked the reviewers for programs that wait
inside a sequence, hand sequences to tasks, and run programs in folders,
and for the refusals around them.

Three reviewers wrote thirty programs from `docs/` and `examples/` alone,
with nothing run. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday async code: a paged API, a poller, a job runner, retries, a pipeline of async generators, a fan-out over tasks, a note reader whose items may fail, two refusals |
| `b01`–`b15` | the edges: generators of every shape, generic and `Iterator` parameters, walks, captures in `spawn:`, `Process.run_in` alone and in tasks, a static async generator, nine refusals |
| `c01`–`c06` | whole programs: a build orchestrator, a crawler, an event-sourced bank, chat rooms with rules, a log tailer, a health monitor |

## The numbers

| | round seventeen | round eighteen |
|---|---|---|
| guesses | 12 | **34** (A 10, B 16, C 8) |
| meant to run, right on the first run | 26 of 27 | **16 of 19** |
| refused as predicted | 3 of 3 | **9 of 11**, word for word |
| rustc leak | 1 | **1** (b02) |
| wrong output | 0 | 0 |

No program printed a wrong answer. c04's output was right; as in round
seventeen, its author did not predict the warning for a list of values
behind an interface — this time the docs say it.

The two refusals that differed were not bugs: b07, a function of a type
written as an async generator, compiles and runs (it was meant to; the
docs now say so), and b09's message shows the call as `nums(...)`, as every
message does for a call with arguments.

## What the programs found

1. **An async generator that took an `Iterator` reached rustc** (b02). Its
   body runs as a task, and a task may only hold what lasts; the parameter
   was an `impl Iterator` that might borrow. Such a parameter is now
   `+ 'static`, and a chain handed to one is collected first, as it is for
   a plain generator.
2. **`yield Error(..)` into an `Iterator[T or Error]` was refused** (a07).
   A function giving `T or Error` reads `Error(..)` as the failure and a
   `T` as the success; `yield` now reads its value the same way, in plain
   and async generators alike, and `yield Ok(x)` still works.
3. **The help for `yield` inside a `spawn:` was out of date** (b15): it
   said an async generator could not exist. It now says to make the
   generator an `async def`, where `yield await work()` waits.
4. **Reviewers read runtime code from a package's build folder** (A, B).
   `packages/http/.lume/` held generated Rust left by a test run; the suite
   now cleans `packages` as it cleans `examples` and `corpus`.

## What the docs now say

- `collections.md`: a function of a type may be an async generator, a
  method of a value cannot yet (B); in an `Iterator[T or Error]`,
  `yield Error("..")` is the failed item (A).
- `io.md`: the text of the `Error` from `Process.run_in` when the folder is
  missing (B).
- `interfaces.md`: a binding that holds interface values gives a warning,
  with the warning as it prints (C, and C of round seventeen).

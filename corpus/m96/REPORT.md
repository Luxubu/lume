# Milestone 96 review corpus — REPORT (twentieth review round)

This round tested what milestones 92–95 added: the methods that wait for an
async sequence's items, `Process.spawn` and `Child`, `async Iterator[T]` as
a type, and the `http` package's request builder and streamed bodies (with
no server: requests built and looked at, and requests that cannot be made).

Three reviewers wrote thirty programs from `docs/` and `examples/` alone,
with nothing run. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday code: the waiting methods, pipes through `sort`, `tr`, `wc` and `head`, a conversation with a `sh` loop, exit codes and standard error, programs in tasks, functions taking and giving async sequences, the request builder, two refusals |
| `b01`–`b15` | the edges: which waiting methods stop, empty sequences, `each` changing names, a `Child` after its end, in lists, fields and tasks, `==` on it, a program's own `Child`, `async Iterator[T]` in every place, the builder leaving the old request alone, unreachable hosts, six refusals |
| `c01`–`c06` | whole programs: a build tool streaming its steps, a REPL client for `sh`, a pipeline of external programs, a bounded job pool, an HTTP request planner, a client for framed answers as a language server writes them |

## The numbers

| | round nineteen | round twenty |
|---|---|---|
| guesses | 51 | **34** (A 4, B 22, C 8) |
| meant to run, right on the first run | 20 of 22 | **20 of 22** |
| refused as predicted | 6 of 8 | **4 of 8**, word for word |
| rustc leak | 1 | **1** (a06) |
| wrong output | 0 | 0 |

No program printed a wrong answer, and all six of C's whole programs —
driving `sort`, `tr`, `sh` loops and framed output — printed what their
author predicted on the first run. After the fixes every program but one
prints what its author predicted; that one, b05, is a prediction the docs
now answer the other way.

## What the programs found

1. **An async generator that took an `async Iterator[T]` reached rustc**
   (a06). A generator copies what it is given, and a stream cannot be
   copied; it is now moved in, as an `Iterator[T]` is.
2. **A `def` giving an `async Iterator[T]` could not use a `Child`'s
   `lines`** (b11), though milestone 95 lets such a `def` start async
   sequences: the check for `lines` outside async code missed that case.
3. **`==` on two `Child` values compiled** (b12), comparing them as the
   same running program, which milestone 94 gave `Child` so that a struct
   holding one compiles. Rust's `Child` has no `==`, and a block has none in
   Lume; now neither has a `Child` or a value holding one, with the help
   the reviewer predicted, word for word.
4. **`c.lines.count { .. }` in a plain `def` named the `count` first**
   (b08), where a `for` over `c.lines` names the `lines`: what is wrong
   with the sequence itself is now said first.
5. **A plain `def` walking an `async Iterator[T]` parameter said "loop over
   it"** (a09) where every other message says "walk it"; that message was
   listed as unreachable, and a parameter of the type now reaches it.

`count` with no block on an async sequence (b05, predicted refused, as a
list's is) runs and counts every item, as Rust's `StreamExt::count` does: an
async sequence has no `len`. The docs now say so.

## What the docs now say

- `collections.md`: `count` with no block counts every item of an async
  sequence (B).
- `examples/errors/UNREACHED.md`: the message for walking an async
  sequence outside async code is reachable, and comes off the list (A).

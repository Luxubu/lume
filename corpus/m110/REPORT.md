# Milestone 110 review corpus — REPORT (twenty-second review round)

This round tested milestones 103–109 and what 0.7 settles: warnings for
unused names, an interface requiring an async generator, `Process.spawn`
and `Child` with the edges the reference now states, `async test` and
`lume test FILE NAME`, and programs leaning on `docs/stability.md`. Three
reviewers wrote thirty programs from `docs/` and `examples/` alone, with
nothing run. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday code: words through `tr` and `sort`, a generic interface met by a type and an `extend`, line and byte counts through `wc`, a `sh` calculator kept in a field, `async test` with a name filter, a stop, two refusals |
| `b01`–`b15` | the edges: which names warn (`x?`, `x!`, a later block, `+=` only, `push` only, `\|` alternatives, tuples, `shared var`), `_` twice, every form of an interface requiring an async generator, `Child`'s edges, `read(n)` through a character, a test filter matching none, a task stopping in an `async test`, five refusals |
| `c01`–`c06` | whole programs: a log pipeline through `sort`, `uniq` and `wc`; plugins merged from an interface's async generators; a length-prefixed protocol over a `sh` loop; a job runner of interface values; a file of plain and `async` tests; a word tool with its unused names |

## The numbers

| | round twenty-one | round twenty-two |
|---|---|---|
| guesses | 39 | **45** (A 12, B 20, C 13) |
| meant to run, stop or test, right on the first run | 17 of 23 | **16 of 23** |
| refused as predicted | 6 of 7 | **7 of 7**, word for word |
| refused, though they should run | — | **4** (a02, b08, c02, c03) |
| rustc leak | 3 | **0** |
| wrong output | 0 | 0 |

No program printed a wrong answer and none reached rustc. All seven
refusals were predicted word for word — B's included the help line that
names the method, `posts`. After the fixes, 27 of the 30 print what their
authors predicted; the other three are guesses the docs now answer the
other way (below).

All three reviewers noticed the same thing in the docs: `collections.md`
still said "Not yet: an `async` generator in an interface" in its section
on generators, though its own section on async generators, `interfaces.md`
and `stability.md` said an interface may require one. Milestone 108 had
removed one such line and missed the other.

## What the programs found

1. **A generic interface whose `T` appears only in `async Iterator[T]` was
   refused** as "never uses `T`" (a02, b08): the check that a type
   parameter is used did not look inside sequences, async or plain.
2. **A function giving `() or Error` could not end with a call that gives
   one** (c03): `def send(line: Str) -> () or Error = child.write(line +
   "\n")` was refused as dropping the failure, though the call is the
   function's result — such a function always ended with an `Ok(())` of
   its own. The last statement is the result now when it gives a `() or
   E`, as any other type's last expression is. (The first way of doing
   this hid the last statement from the check of which names are used
   later, and two older review programs reached rustc; the suite caught
   it before anything else did.)
3. **`await c.receive?` was refused** (c03), though the docs say the `?` of
   `await t?` belongs to the `await`: that held for a name, not for a
   method called without parentheses, nor a call. It holds for all three.
4. **`extend [Str] with Plugin` could not give an `items` method** (c02): a
   program that names `Iterator` — any program with a generator — was
   given Lume's own `extend [T] with Iterable[T]`, whose `items` it never
   asked for. Lists and sets get that `extend` only in a program that
   names `Iterable` now.
5. **The warning for an unused name of `(a, b) = pair` pointed at the
   line's start** (a03); it points at the name, as Rust's does.

The three predictions the docs now answer the other way: B expected a list
only pushed to to be warned about (b03) — calling a method on a value is a
use, as in Rust, and `names.md` says so; B expected the caret of a tuple's
unused name at the line's start (b04), which A expected at the name and
Lume now gives; and B guessed the summary `lume test` prints when its name
matches no test (b13), which `testing.md` now quotes.

## What the docs now say

- `collections.md`: the second "Not yet: an async generator in an
  interface" is gone (A, B, C).
- `names.md`: the caret is on the name, even one of several in a tuple;
  any use of a name counts, `xs.push(3)` included (A, B).
- `testing.md`: what a name filter matching nothing prints, and that it
  exits with 0 (A, B).
- `io.md`: `read(n)` gives `None` once nothing is left (B, C).

## Left

- `docs/stability.md` opens with "Lume is at 0.6" beside parts "settled in
  0.7" (A, B, C); release 0.7.0, milestone 111, makes that true.

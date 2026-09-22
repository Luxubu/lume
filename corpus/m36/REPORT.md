# Milestone 36 review corpus — REPORT (seventh review round)

This round tested the **documentation**, not the compiler. Six earlier rounds
each ended with about ninety-five things the reviewers had to guess, and
milestone 35 rewrote the documentation to answer them. Three reviewers wrote
thirty programs from `docs/` and `examples/` alone — no README, no compiler
source, no `docs/QUESTIONS.md`, nothing run — and counted what they still had
to guess. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a10` | everyday programs: text, collections, printing and laying out reports — where earlier rounds guessed most |
| `b01`–`b10` | types and abstractions: structs, enums, results, interfaces, `extend`, generics, blocks of your own |
| `c01`–`c10` | whole programs, including **modules** and **concurrency**, which had no reference page, deliberately, to measure what that cost |

## The number

| | guesses |
|---|---|
| reviewer A | 22 |
| reviewer B | 36 |
| reviewer C | 40 (12 modules, 13 concurrency, 15 other) |
| **total** | **98** |

Earlier rounds: about 95 each. **The count did not fall.** That is the
honest headline, and it is worth saying before explaining it.

What changed is *which* questions. The top of `docs/QUESTIONS.md` — how a
value prints, which side `pad` fills, whether `join` works on numbers, how
`split` treats the ends, what `decimals` does, what the character predicates
give on empty text, how `?` behaves — produced **no guesses at all**. Reviewer
A, aimed squarely at the area where every earlier round guessed most, made
none about printing or layout and wrote that the reference pages were
"strong". Reviewer C made none about printing, strings, padding, `split`,
ranges or errors.

The 98 fall into a handful of clusters, and several were asked independently
by all three reviewers:

- **25 — the two pages that did not exist.** Modules and concurrency, which
  reviewer C had to reverse-engineer from examples, error-message fixtures and
  a `FINDINGS.md`. That is a quarter of the round, from a third of one
  reviewer's code.
- **Which methods exist on a list, a map, a set, a number.** All three. The
  docs index promised the collections page covered "every method"; it had no
  method table.
- **Which words are reserved.** All three. `QUESTIONS.md` listed this as
  answered; the answer was a sentence, not a list.
- **Does an overwritten map key keep its place.** All three.
- **Who wins a `max_by` tie.** Two, and one in an earlier round. It was not a
  documentation gap but an undecided rule: Rust's `max_by` keeps the last of a
  tie and `min_by` the first, and Lume had inherited both.
- **Patterns** (list patterns with literals, a catch-all that binds, whether
  the first matching arm wins), **floats** (ordering, rounding exact halves,
  `to_int`, how an inexact float prints) and **changing things in place**
  (`var` parameters, `for var`, calling a sibling method by bare name).

Every one of these now has an answer in `docs/`, checked by `tests/docs.sh`.

## The programs

| class | first run | after the fixes |
|---|---|---|
| prints exactly what its author worked out by hand | 19 of 27 | **27 of 27** |
| refused, with the error its author predicted | 3 of 3 | 3 of 3 |
| rustc leak | 3 | 0 |
| refused although correct | 1 | 0 |
| wrong output | 4 | 0 |

**Nineteen of twenty-seven on the first run** printed exactly the output
worked out by hand. The previous round managed eleven of thirty. A reviewer
can now predict what a program prints, which was the whole of milestone 32's
point and most of 35's.

**All three refusals said, word for word, what their authors predicted** —
`` `a` is immutable, but `add_interest` changes it ``, `` `peak_multiplier`
exists in module `fares` but is not `pub` `` — so the error messages are
predictable from the documentation too.

Of the four wrong outputs, one was the reviewer's arithmetic: `c06` applied
its last transfer backwards. Worked again by hand the compiler is right, and
the program is concurrent, which made it the one worth checking most
carefully. The header now carries a note. The other three were real:

1. **A number in a pair from a map or a lazy chain arrived as a reference**
   (`for w, n in ranked.take(5)`, `for region, amount in by_region`). The
   declaration said it was a value, so using it as one reached rustc — three
   of the four leaks and a wrong line. One rebinding line per numeric part,
   which works whatever shape the pairs came in.
2. **A function returning an interface could not choose between two types.**
   `def pick(big: Bool) -> Shape` with a `Rect` in one branch and a `Circle`
   in the other was refused as "branches give different types" — the one
   thing returning an interface is for. Branches are now normalised to the
   declared interface, as they already were for `T?` and `T or E`.
3. **`max_by` kept the last of a tie**, `min_by` the first. Both now keep the
   first, as Ruby and Python do. Two old corpus expectations changed; neither
   was hand-computed.

## What writing the answers found

The reference pages this round asked for — methods, numbers, patterns,
modules, concurrency, names, input and output — were written by running every
claim, and that found more compiler defects than any review round has:

- **a compiler crash**: a struct used as a pattern (`P(a, b) -> ...`)
  indexed past the end of the exhaustiveness checker.
- **`==` was never type-checked.** The operand check trims a trailing `=` so
  that `+=` is checked as `+`, and that turned `==` into `=` and `!=` into
  `!`, which matched nothing. Every comparison between mismatched types went
  straight to rustc. All 502 tests and 283 corpus programs passed unchanged
  under the fixed check, so the hole had not been fallen into — only
  approached.
- **two silent wrong answers**: `lume fmt` dropped the `pub` from
  `pub import`, which changes what a library exports; and `import lib.one`
  beside a `def one` of your own called the imported one without a word.
  Both are the worst class of bug — no error, different behaviour — and
  neither was reachable by a program a person would write to test a feature.
- **a hang**: `Time.sleep(0 - 5)` compiled into a sleep of eighteen
  quintillion milliseconds, because namespace arguments were never checked.
- **a crash at run time**: `spawn:` in a program whose `main` is not
  `async` compiled, then stopped with a message about Tokio.
- **fourteen leaks**: a constant rebound or changed; `count` with no block;
  `sum` of text; a built-in method on a tuple; a `Float` as an inferred map
  key, set item, `group_by` key or tuple sort key; a variable called `Err`;
  type names the generated Rust uses (`Default`, `Eq`, …); wrong argument
  types to `File`, `Dir`, `Path`, `Env` and `Time`.
- **three wrong rules**: `b = 7` on a `shared var` inside `spawn:` was
  refused though `+=` was not; `var x` in a nested block hid an outer `x`
  though `x = ...` there was an error; a field named `match` or `next` could
  be declared and read but not set by name.

All are fixed, and each has a program in `examples/errors/` or a checked
example in `docs/` that keeps it fixed.

Two things were deliberately *not* done. `Dir`, `Path` and `Env` stay
available as type names although a type of that name hides the built-in:
two independent reviewers in two different rounds named an enum `Dir` for a
direction, and reserving it would break the natural program to protect the
rare one. And `?` inside `spawn:` stays unsupported, documented with the
pattern to use instead.

## Left open

- Error-message quality in about a dozen places the new pages recorded:
  `await f()?`, a `pub` function returning a non-`pub` struct, `2.pow(-1)`
  reporting overflow, `<` on tuples refused although sorting them works, a
  field not hiding a module of the same name, `None + 1` quoting the wrong
  expression.
- The method list in an error's help line leaves out some block methods on
  maps and sets, and says nothing for `T?` and `T or Error`. `methods.md`
  says so.

## Newcomer's verdict (condensed from the three notes)

A: "A new user could learn everyday Lume from these docs." B: "A new user
could learn to design their own types from these docs." C: the reference
pages are strong; modules and concurrency could only be learned by
reverse-engineering, and a wrong concurrency guess — unlike a wrong module
guess — can compile and behave differently, so that page mattered most.

The documentation now has every page the three asked for. Whether that is
enough is what the next round is for — and its number is the one to watch,
since this round shows the count can stay flat while everything it counts
changes.

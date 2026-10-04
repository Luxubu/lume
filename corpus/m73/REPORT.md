# Milestone 73 review corpus — REPORT (fifteenth review round)

This round tested milestones 70–72:
- generators: `yield`, what they can do, how they move, where they are
  refused;
- sequence parameters, `xs: Iterator[T]`;
- `split` and `lines` giving slices of the text;
- lists, maps and calls written across lines;
- the error messages.

Three reviewers wrote thirty programs from `docs/`, `examples/` and
`packages/` alone, with nothing run, and counted what they had to guess.
The brief is in `BRIEF.md`. This time they were asked to count only real
uncertainties, not things they had confirmed, so the count is comparable
with rounds before fourteen.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday programs: a sales report, an RPN calculator fed by a tokenizer generator, a pager, date schedules, word frequencies, an INI reader, prime and Fibonacci pipelines, a ledger, one refusal |
| `b01`–`b15` | the edges: every place `yield` may go, generic and recursive generators, `zip` either way, each kind of sequence into a parameter, when a chain into a generator is worked out, slices that would show if wrong, multi-line layout, eight refusals |
| `c01`–`c06` | whole programs: an INI reader whose lexer and parser are generators, a stack machine, an inventory reconciler, a two-week agenda over an endless sequence of days, a text adventure, a log report with a task per service |

## The numbers

| | round fourteen | round fifteen |
|---|---|---|
| guesses | 179, counted finely | **37** (A 12, B 14, C 11) |
| meant to run, right on the first run | 16 of 21 | **20 of 21** |
| refused as predicted | 8 of 8, 5 word for word | 9 of 9, **6 word for word** |
| rustc leak | 2 | **1** (c06) |
| wrong output | 0 | 0 |

Reviewer A was right on all nine programs, including what goes to standard
error. c01's output was right; the reviewer did not predict the documented
warning for a list of mixed values behind an interface. The three refusals
that were not word for word differ in wording or in where the caret sits
(b10: on the generator inside the tuple, not on the tuple), and b11's help
is better now (below).

## What the programs found

1. **A `match` on an item of a list of slices reached rustc** (c06, a leak
   from milestone 72). `match parts.at(1):` with text patterns borrowed the
   item a second time. Fixed; and every other place an item can go — a map
   key, `contains?`, `index_of`, a function's argument, `+`, `<`, a tuple, a
   `push`, a pattern binding — was tried, and all of them work.
2. **A generator given to a field typed `Iterator[Int]`** (b11) was refused
   with advice meant for a function's result ("make this a generator too").
   The help now says where a generator can live: a local or a parameter
   `xs: Iterator[T]`, or `.to_list` to keep the items.

## What the docs now say

- `methods.md`: `zip` takes a list, a range or a sequence, and on a chain it
  gives a chain, so an endless sequence on either side is fine (A).
- `collections.md`: a `def`'s parameters may be written across lines, and
  the indentation inside brackets is free (A, B); a generator that has ended
  gives `None` to every `next` (B); a `var` holding a generator carries on
  where it stopped (A); an endless chain given to a generator never returns,
  because a chain into a generator is worked out at the call (A).
- `io.md`: the "has days 1 to N" error, for any month (A, C).

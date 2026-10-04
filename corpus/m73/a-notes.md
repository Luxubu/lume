# Reviewer A — notes

## What I read

- `corpus/m73/BRIEF.md`.
- `docs/tour.md`, `docs/examples.md`, `docs/README.md` (listing only).
- `docs/reference/`: `collections.md` (all of it, especially "Generators:
  `yield`", "Taking a sequence" and "When a list is copied, and when it
  moves"), `strings.md`, `printing.md`, `errors.md`, `control-flow.md`,
  `methods.md`, `functions-and-blocks.md`, `structs-and-enums.md`,
  `patterns.md` (first half), `names.md` ("Binding a name again"),
  `numbers.md` (arithmetic, converting, printing), `io.md` ("Dates" and
  "When I/O fails").
- `examples/`: `generators.lume`/`.expected`, `split_slices.lume`/`.expected`,
  `iterators.lume`/`.expected`, `dates.lume`/`.expected`, `logq/main.lume` and
  `logq/FINDINGS.md`, and in `examples/errors/` every `generator_*`,
  `yield_*`, `iterator_param_var`, `next_wrong_shape`, `dropped_failure`,
  `missing_arm`, `unknown_function` and `typo` (program and expected text).
- I did not open `packages/` (http and json); none of these programs uses them.

Header convention in every program: each output line is the comment line
with its leading `# ` removed; a line that is just `#` would be a blank line
(none of these programs prints one).

## The programs

| file | what it is | leans on |
|---|---|---|
| `a01_sales_report.lume` | CSV-ish sales report from a `"""` text: totals by region, best sale, units per rep | generator over `lines.enumerate` with `next if`, `split(",")` read by position, `warn` from a generator, `xs: Iterator[Sale]` into a map, generator called twice (warnings twice), multi-line constructor call with `?` in it and a trailing comma, multi-line call, `group_by` then map `.map` to pairs, `sort_by` on pairs, `pad`/`pad_right` |
| `a02_rpn_calc.lume` | a tokenizer generator feeding an RPN evaluator | `yield` inside a `match` arm and after the loop, `Str + Char`, enum tokens, `toks: Iterator[Token]`, `pop.or_error(..)?`, `match` as a value, `return` from an arm, a `var` generator moved on by `next` and then handed to the evaluator, multi-line list |
| `a03_pager.lume` | paging through search results | endless generator zipped with a list, generator of `Page` structs with a multi-line `yield Page(...)`, `skip(1).first` on a generator, `next` on a `var` generator then a chain over the rest, empty `[]` argument, printing a list of structs |
| `a04_schedule.lume` | every Nth day, weekdays only, the 31st of each month | `Time.date`/`format`/`weekday`/`parse`, endless generator stopped by `take`, `take_while`, `find`; generator over `xs: Iterator[Int]`; generator yielding from both arms of a `match` on `Int or Error`; top-level constant used in a generator |
| `a05_word_freq.lume` | word-frequency pipeline | nested `for` over `lines` and bare `split` in a generator, `without(xs: Iterator[Str], stop: {Str})`, multi-line set literal, tally map, `sort_by` on a tuple key over a map, `max_by` tie, a chain over a generator handed to a generator |
| `a06_config.lume` | INI-style config reader with `[sections]` | generator keeping a `var section`, `split("=")` by position, `warn`, pairs into `Iterator[(Str, Str)]`, map keeps first place on overwrite, `or_error`, `map_error`, `?` in `main -> () or Error` ending the run with `error: ...` |
| `a07_primes.lume` | prime and Fibonacci pipelines | endless generators three deep through `xs: Iterator[Int]` generators, `zip` of two endless sequences then `take`, chains over a generator / a range and a plain list into a generator, `_ = ps.next` on a `var` generator |
| `a08_ledger.lume` | bank statement with running balance and overdraft check | enum with methods, generator of `(Txn, Int)` pairs with its parameters written across lines, `() or Error` function over `Iterator[(Txn, Int)]`, `decimals` on negatives, `min_by` on pairs printed |
| `a09_reused_readings.lume` | **FAIL**: a generator handed to an `Iterator[Int]` parameter and then used again | move into a sequence parameter; the "used up" error |

## Guesses

1. **a02** — a `var` generator that has been moved on with `g.next` can then
   be handed to an `xs: Iterator[T]` parameter, and the function gets only
   the items left (`Num(8)`, `Op("*")`), so the result is
   `Error("`*` needs two numbers")`. Looked in: collections.md "Generators"
   (`next` on a `var`) and "Taking a sequence" ("A generator moves in"),
   generators.lume (`first_and_rest` calls `next` on a parameter, not on a
   caller's `var`). No example hands on a partly walked generator.
2. **a02** — `Bad(text) -> return Error("...")`: a one-line `match` arm whose
   body is a `return`. control-flow.md shows statement arms (`puts`) and
   errors.md shows value arms, but never `return` after `->`.
3. **a03** — after `pager.next` on a `var` generator, `pager.map { .. }`
   starts from the second page (the generator is moved, not copied as a
   `def next(var self)` type would be). Looked in: collections.md ("A loop
   or chain walks a copy" is said for types of your own; "It is used up as
   it is walked" for generators). Only shown for a parameter
   (`first_and_rest`), not for a local `var`.
4. **a04** — `Time.date(2024, 2, 31)` and `Time.date(2024, 4, 31)` give
   `February 2024 has days 1 to 29, not 31` and `April 2024 has days 1 to
   30, not 31`. Extrapolated from the one example
   (`February 2023 has days 1 to 28, not 29`) in io.md and dates.lume.
5. **a04** — `%b` writes `Oct` and `Nov`. io.md's table shows only `Sep`,
   and dates.lume parses `Dec`; I assumed the usual English three-letter
   abbreviations. (`%a` was confirmed for all seven days by dates.expected.)
6. **a04** — `opt.map(show)`: a function's name handed as the block to
   `map` on a `T?`. functions-and-blocks.md shows function names as blocks
   for list methods, and methods.md shows `T?.map` with `_`, never both
   together.
7. **a05, a07** — a lazy chain whose source is a generator
   (`words(text).filter { .. }`, `fibs().take(12)`) or a range
   (`(1..50).filter { .. }`) is accepted by a generator's `Iterator[T]`
   parameter and "worked out first" with no visible difference. The doc
   sentence is about "a lazy chain" in general; its only example passes a
   range (`evens(1..6)`) to a plain function. (Note it also implies that an
   *endless* chain handed to a generator never returns, which I avoided.)
8. **a07** — `primes(count_from(2)).zip(fibs()).take(4)`: `zip` of two
   endless sequences stays lazy so `take` can stop it. collections.md says
   `zip` pairs a sequence with another sequence and that `zip` stops an
   endless one; iterators.lume only zips an endless sequence with a range.
   methods.md's table says `zip` gives a `[(T, U)]` (a list), which would
   never return here.
9. **a08** — a `def`'s parameter list may be written across lines with a
   trailing comma, like a call's. collections.md says only "the parentheses
   of a call"; nothing mentions a definition.
10. **a08** — `Ok(_)` matches the success of a `() or Error`. patterns.md
    and errors.md show `Ok(n)` on valued results only.
11. **a08** — `txns.filter(_.fee?)`: the `_` shorthand with a method whose
    name ends in `?`. functions-and-blocks.md shows `_.upcase`, `_.len`,
    never a `?` method.
12. **a09** — handing a generator to an `xs: Iterator[T]` parameter and
    using it after gives the same message as `generator_used_twice`, with the
    caret on the argument where it moved (`24:16`). That example only shows
    the generator walked by `.sum`; no error example covers a move into a
    parameter, and the docs do not quote a message for it.

## Summary

The generator and sequence-parameter sections are clear about what a
generator is and where it is refused, and the error catalogue covers the
refusals well, but the docs are thin on how a generator behaves *between*
those cases: what a local `var` generator looks like after `next` (shown only
for a parameter), what happens when a partly walked one is handed on, which
chains count as "a lazy chain" when handed to a generator (and the trap that
an endless one would hang), and whether `zip` on sequences is lazy when the
methods table says it gives a list. The multi-line rule names lists, maps and
calls but leaves function definitions, and `match` arms holding a `return`,
unsaid. `Time.date`'s errors and `%b` are each shown by a single example, so
any other month is extrapolation.

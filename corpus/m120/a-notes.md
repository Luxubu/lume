# Reviewer A — a newcomer's first programs

## What I read

In this order, all of it unless marked:

- `corpus/m120/BRIEF.md`
- `docs/tutorial/README.md`, then lunches 1–23 whole, and lunch 24 up to
  24.5 (the server conversation; nothing in my set uses it)
- `docs/stability.md`, `docs/install.md`, `docs/README.md` (the index)
- Reference pages read whole: `errors.md`, `printing.md`, `testing.md`,
  `names.md`, `io.md`, `concurrency.md`, `collections.md`, `methods.md`,
  `strings.md`, `structs-and-enums.md`
- Reference pages read in part: `patterns.md` (to "Enums, nested to any
  depth"), `numbers.md` ("Converting", "What stops the program"),
  `control-flow.md` (`match`, "Doing nothing"); the headings of
  `functions-and-blocks.md`
- Examples: `examples/unused_names.lume` and `.expected`,
  `examples/tests/parse.lume` and `.expected`, and from `examples/errors/`
  the `.lume` and `.expected` of `arity`, `missing_arm`, `dropped_result`,
  `dropped_failure`, `spawn_changes_copy`, `immutable`, `typo`,
  `none_arith`, `missing_error_arm`, `nonexhaustive_nested`,
  `nonexhaustive_nested_deep`, `var_hides_outer`

**One slip to declare.** A `grep -rn "left:" examples docs`, looking for how
a failed `assert` prints text, also matched files in
`examples/mini/.lume/` (build output, which the brief puts off limits). The
lines shown were generated Rust; one was
`msg.push_str(&format!("\n  left:  {}\n  right: {}", l, r));`. It says
nothing about whether text is quoted, and I did not use it: guess 2 is
made from the docs alone. Later greps left out `.lume` folders.

## The programs

| file | probes | meant to | checked with |
|---|---|---|---|
| `a01_word_tally.lume` | a word tally: bare `split`, `map(fn)`, map counting, `to_list` + `sort_by` on pairs, `take`, `pad_right`, `repeat`, `max_by`, map `filter`/`keys`, `to_set`, `Char == "a"` | RUN | `lume run` |
| `a02_cafe_model.lume` | structs and enums: payload variants, an enum inside a variant, `match self` with nested variant, literal and guard patterns, `Int` ranges, `def self.`, `var self`, `sum` with a block, printing and `==` | RUN | `lume run` |
| `a03_guest_list.lume` | `T or Error`, wrapping in `Ok`, `return Error`, `?` on `to_int` and own functions, list pattern on `split`, `or_error`, `map`, `map_error`, `?.method`, `main -> () or Error` ending in failure (exit 1) | RUN (exit 1) | `lume run` |
| `a04_notes_folder.lume` | `File`, `Dir`, `Path` under `/tmp/lume-review-A/a04`: make, write, append, read, size, list, walk, a failed read's message, remove, exists | RUN | `lume run` |
| `a05_prime_pipeline.lume` | generators: endless, chained through `Iterator[Int]` parameters, a range as a sequence, `take`, `next if` in a generator, printing inside one to show the order of work, `break`, `next` on a `var` then `to_list` | RUN | `lume run` |
| `a06_kitchen_tasks.lume` | tasks: `spawn:` and `await`, a list of tasks from `map`, blocking work in a task, `next_done`, `shared var` and its warning, tasks giving `T or Error`, a task's copy | RUN (1 warning) | `lume run` |
| `a07_fizz_tests.lume` | `test` and `assert`, `map(fn)`, `[]` to a typed parameter, `async test`, one failure with text on both sides | RUN (exit 1) | `lume test` |
| `a08_seating_stops.lume` | the `!` warning, then a stop from `.at` past the end inside a one-line function | STOP (101) | `lume run` |
| `a09_tea_count_fails.lume` | a counter without `var` re-bound inside `for` and `if` | FAIL | `lume check` |

## Guesses

1. **A `match` whose arms name every inner variant is complete.** In
   `Item.price`, the arms `Coffee(Small)`, `Coffee(Medium)` and
   `Coffee(Large)` cover `Coffee` with no `Coffee(_)` or `_` arm, and I
   expect no "does not cover" error. — `a02_cafe_model.lume`. Looked:
   `patterns.md` ("The forms at a glance", "Enums, nested to any depth":
   its example ends in `other -> other`), lunch 9.2,
   `examples/errors/nonexhaustive_nested.expected` and
   `nonexhaustive_nested_deep.expected`, which show nested analysis of what
   is missing, but no page or example shows a nested match that is
   complete without a catch-all.
2. **`left:` and `right:` show text without quotes.** The failing test
   compares two `Str`s; I expect `left:  Buzz` and `right: Fizz`, by the
   printing rule that a value on its own prints as itself. —
   `a07_fizz_tests.lume`. Looked: `testing.md` ("What a failure says": only
   `Int`s, and `Some("starter")`, where the quotes come from being inside
   `Some` either way), lunch 12.5 (`Int`s), `examples/tests/parse.expected`
   (`Int`s), `printing.md` ("The rule").
3. **Where the caret goes for a re-bind in a nested block.** `names.md`
   gives the message and help as `#!` lines but no location; I put it at
   the first character of the statement, `count`, column 7, as
   `immutable.expected` (for `+=`) and `var_hides_outer.expected` do. —
   `a09_tea_count_fails.lume`. Looked: `names.md` ("Binding a name
   again"), lunch 2.3, those two `.expected` files.
4. **Two blocks deep gives the same message.** `names.md` shows the
   re-bind one block deep (`for`); mine is inside an `if` inside a `for`. I
   expect the same message, naming line 18, where `count` was made, and
   the same help. — `a09_tea_count_fails.lume`. Looked: `names.md`,
   lunch 2.3, lunch 5.

Things I worked out rather than guessed, because a page says so: `"-2"`
reads as `Ok(-2)` (`numbers.md`, "Converting": a sign is allowed); the
stop names the line inside the called function (`testing.md`, the `share`
example; `errors.md`, "Where the program stopped"); `main`'s failure prints
one `error:` line with no location and exits 1 (`io.md`, "When I/O fails";
lunch 10.6); the `!` warning's caret is on the `!` (`errors.md`); the
`shared var` warning's text and caret (`concurrency.md`, "Where the
warnings go"); a two-digit line number widens the gutter (`errors.md`;
`examples/unused_names.expected`); `Dir.walk` lists files only, sorted
(`io.md`); after `g.next`, `g.to_list` carries on (`collections.md`).

**Total: 4 guesses.**

## Where the tutorial and the reference disagree

1. **The message for a dropped failure has two wordings.** Lunch 10.5 and
   `errors.md` give ``can fail, and nothing here looks at the result``;
   `io.md` and `examples/errors/dropped_result.expected` give ``can fail,
   and nothing here looks at whether it did``. Perhaps one is for a
   `T or Error` and the other for a `() or Error`, but no page says so.
2. **`!` and its warning in tests.** Lunch 10.8 says Lume warns "wherever
   you use it"; `testing.md` says a test may use `!` "without the warning
   it gets in other code".
3. **What a transform may be.** Lunch 2.3 says `name = name.…` works "on
   the next lines of the same block"; `names.md` says "in the same block",
   with no word on the lines between, and adds that the right side must
   begin with the name followed by a method call, an operator, a range or
   `?`.
4. **What `await` covers.** `concurrency.md` says "`await` covers
   everything to its right"; `testing.md` writes
   `assert await [pie, tea] == ["pie is ready", "tea is ready"]`, which by
   that rule would await the comparison. Comparison operators are
   evidently not covered, but no page says where `await` stops. (I named
   the awaited value first, in `a07`, to keep clear of it.)
5. **What `Time.sleep` holds up outside async code.** Lunch 23.3 says it
   "simply pauses the task it is in"; `concurrency.md` and `io.md` say it
   "holds up the whole thread". In a blocking-pool task these may be the
   same thing, but a newcomer cannot tell.
6. **What `split` gives.** Lunch 3.6 says it "cuts text into a list";
   `methods.md` says it gives a chain of `Str`. They print the same, so
   this matters only to someone reading both.

Inside the tutorial itself: the fragments in lunch 21.4
(`sort_by { |p, avg| 0 - avg }`) and lunch 22.3
(`done.sort_by { |i, o| i }.map { |i, o| o }`) name block parameters they
never use, which `names.md` says is warned about; the whole programs in
21.5 and 22.5 write `_` there instead. Someone typing the fragment, as
lunch 1 asks, gets warnings the page does not show.

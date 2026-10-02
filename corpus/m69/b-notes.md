# Reviewer B — round fourteen (m69): the edges of the newest features

## What I read

- `corpus/m69/BRIEF.md`.
- `docs/reference/collections.md` (all, especially "Types of your own:
  `items`" and "Types of your own: `next`"), `interfaces.md` (all),
  `numbers.md` (all), `concurrency.md` (all), `control-flow.md`,
  `printing.md`, `methods.md` (the list, map, set, optional and tuple
  tables), `generics.md`, `structs-and-enums.md` (`var self` and below),
  `names.md` (reserved names; `next` as a keyword and a field name),
  `functions-and-blocks.md` (`var` parameters), `operators.md` (overflow),
  `errors.md` (how `!` stops the program), `docs/install.md` (`lume run` and
  overflow).
- `examples/iterators.lume` + `.expected`, `examples/iterable.lume` +
  `.expected`, `examples/async.lume` + `.expected`,
  `examples/tasks_blocking.lume` + `.expected`, a grep of `examples/` for
  `var self`/`next`/`Iterator`.
- `examples/errors/`: `items_and_next`, `items_not_a_list`,
  `iterable_bound_unmet`, `iterator_bound_var_self`, `next_wrong_shape`,
  `interface_var_self`, `extend_var_self`, `for_var_items`, `for_var_next`,
  `for_var_pair`, `for_var_immutable`, `immutable`, `immutable_receiver`,
  `push_immutable`, `readonly_self`, `reassign`, `shared_readonly`,
  `chain_no_items`, `loop_no_items`, `iface_mismatch`, `iface_missing`,
  `iface_arity`, `iface_generic_unmet`, `iface_pass_held_value`,
  `iface_unused_param`, `generic_bound_unmet`, `generic_unbound`,
  `operator_var_self`, `var_param_immutable`, `var_param_copy`,
  `var_arg_expression`, `dropped_result`, `overflow_constant`,
  `reserved_type`, `iface_reserved`, `spawn_without_async_main`,
  `spawn_changes_copy`, `spawn_uses_self`, `own_task_args`, `task_ok`,
  `task_awaited_twice`, `async_not_awaited`, `interface_method_no_return`,
  `method_twice`, and `UNREACHED.md`.
- `packages/`: listed both; grepped `packages/json/lib.lume` for
  `items`/`var self` (its `Json` enum has a `def items -> [Json]`).
- Not read: compiler source, `tests/`, other corpus folders, `README.md`,
  `HISTORY.md`, `CHANGELOG.md`, `docs/QUESTIONS.md`. Nothing was run.

## The programs

| file | probes |
|---|---|
| `b01_countdown_ends.lume` (RUN) | a `next` sequence that ends: `len`, `sort`, `first`, `max`, `join`, `count`, `map`, `reverse`, `for`; the loop walks a copy; `next` by hand on a `var` past the end; chains on an exhausted `var`; a sequence empty from the start |
| `b02_endless_stops.lume` (RUN) | an endless sequence with `take`, `take_while`, `find`, `first`, `any?`, `skip`+`first`, `enumerate`+`skip`+`first`, `zip` with a list, `all?`, `take(0)`, `find(..).map`, `break` with `where`, `for i, x in .enumerate` + `break`; a chain on a hand-moved `var` starts where it is and leaves it there |
| `b03_cycle_shapes.lume` (RUN) | generic `struct Cycle[T]` over structs, strings and tuples; a sequence of `(Str, Int)` with `for k, v`, two-name `map`/`max_by`; `for` over a stopped chain of structs; `next` by hand on a generic `var` |
| `b04_iterator_bound_held.lume` (RUN) | `[S: Iterator[Int]]` with `for`, `find`, `filter.take`; a `var` bound parameter whose `next` moves the caller's value; a plain parameter walking a copy; a value held as `Iterator[Int]`: `next`, `to_list`, `next` again, `take`, reassignment to another type |
| `b05_items_shadow_bounds.lume` (RUN) | a field (`sum`) and an own method (`first`, other return type) shadowing list methods; list methods through `items`; a chain's `sum`; `c.sum` inside `[C: Iterable[Int]]`; a set with a duplicate and a list passed to `Iterable[Int]` / `Iterable[Str]` bounds |
| `b06_ticker_var_self.lume` (RUN) | an interface with a required `var self` method, a required read-only method and defaults; `[T: Ticker](var x: T)` changes the caller's value; `for var` over a `var [Ticker]` list calling the `var self` method |
| `b07_overflow_in_next.lume` (RUN, stops) | `Int` overflow under `lume run` inside an endless sequence's `next`, reached through `for` over `skip(61)`; `skip(60).first` asks no further than it needs |
| `b08_spawn_sequences.lume` (RUN) | `spawn:` with and without `await` in the body, awaited out of start order; a `[Task[[Int]]]` list; sequences copied into tasks; a `var` copy of an outside sequence advanced inside a task |
| `b09_next_not_var_self.lume` (FAIL) | `def next -> Int?` without `var self`: callable by hand, `for` refused; help line |
| `b10_next_takes_param.lume` (FAIL) | `def next(var self, step: Int) -> Int?`: callable by hand, `for` refused; help line |
| `b11_next_then_items.lume` (FAIL) | `next` written before `items`, never looped: where the caret goes |
| `b12_ticker_bound_unmet.lume` (FAIL) | an own interface asking `tick(var self)`, a type with a read-only `tick`, at a bound |
| `b13_extra_var_self.lume` (FAIL) | the reverse: interface asks read-only `size`, type's `size` takes `var self`, used as a parameter typed as the interface |
| `b14_tick_on_plain_param.lume` (FAIL) | calling a required `var self` method on a non-`var` parameter of a bound type |

## Guesses

1. **b01** — that `len`, `sort`, `max`, `join`, `count`, `reverse` all exist on a `next` sequence. The docs name `to_list`, `len`, `sum`, `sort` as methods "that need every item", and say "every chain method" works; `max`, `join`, `count`, `reverse` I took from "every list method that does not change the list works directly on a chain" (methods.md) by analogy. Looked: collections.md "Types of your own: `next`", methods.md "Lazy chains".
2. **b01** — that `c.sort` on a sequence prints as a list (`[1, 2, 3]`) and `c.reverse` gives the reversed *sequence order* `[1, 2, 3]` (sequence is 3,2,1). Looked: methods.md (`sort`/`reverse` give `[T]`).
3. **b01** — that `next` called again after it has given `None` gives `None` again (my `next` checks `n == 0` every time, so this follows from the code, but whether Lume fuses the iterator or calls my `next` again is not stated). Looked: collections.md ("ends at the first `None`"), iterators.expected (only one `None` shown).
4. **b01** — that a chain on a `var` that was moved to its end by hand (`d.to_list`, `d.first`, `d.sum`) starts from the current state (empty), not from the original. Inferred from iterators.lume's held `s.take(5)` giving `[1]` after one `next`. Looked: iterators.lume/.expected.
5. **b01** — that `for` over a sequence empty from the start just prints nothing. Looked: collections.md (nothing explicit).
6. **b02** — that `all?` stops an endless sequence at the first failing item. The docs' list of stoppers is `take`, `take_while`, `find`, `first`, `any?`, `zip`, `break`; `all?` is not in it. Looked: collections.md "Endless is fine".
7. **b02** — that `zip` on a sequence accepts a **list** argument and stops at the list's end (the docs show only `zip(1..5)`; methods.md says list `zip` takes `[U]` and gives `[(T, U)]`), and that the result needs/accepts `.to_list` and prints as a list of tuples. Looked: iterators.lume, methods.md.
8. **b02** — that `enumerate.skip(2).first` gives `Some((2, 3))` — positions are counted before `skip`, and a tuple inside `Some` prints with double parentheses. Looked: methods.md (`enumerate`, `max_by` printing `Some(("ann", 3))`).
9. **b02** — that `take(0)` on an endless sequence gives `[]` without calling `next` (and so terminates). Looked: methods.md (`take` never stops the program); nothing on 0.
10. **b02** — that `.find {..}.map {..}` on the `Int?` result works (`T?.map` with a block). Looked: methods.md `T?` table.
11. **b02** — that `for x in seq where cond:` works over a `next` sequence (docs show `where` over ranges and lists only) and that `break if` inside it stops an endless loop. Looked: control-flow.md "`for x in xs where cond`".
12. **b02** — that `for i, x in nat.enumerate:` takes the pairs apart over an endless sequence and `break` stops it. Looked: control-flow.md "for over pairs", methods.md.
13. **b02/b04** — that a chain on a `var` sequence (`m.take(2)`) does not move `m` (so the next `m.next` is `Some(2)`). The docs say "a loop or chain walks a copy" with `fib` (not `var`) as the example. Looked: collections.md.
14. **b03** — that `Cycle(xs: [...], i: 0)` infers `T` from the field argument for struct, `Str` and tuple items. Looked: generics.md (generic structs), iterators.lume (only `Str`).
15. **b03** — that `for c in cards.take(2):` over a stopped chain of struct items works and `c.suit` reads a field. Looked: methods.md ("`for` walks a chain too").
16. **b03** — that `def next(var self) -> (Str, Int)?` is accepted (a tuple inside an optional as a return type) and that `Some(("p#{k}", k * k))` builds it. Looked: methods.md tuples; nothing on tuple optionals as return types.
17. **b03** — that `for name, sq in Pairs(k: 0):` takes pairs apart for a `next` sequence of tuples. Docs say this only for `items -> [(Str, Int)]` ("Pairs: ... as over a map"). Looked: collections.md "Types of your own: `items`".
18. **b03** — that two-name blocks (`map { |name, sq| .. }`, `max_by { |name, sq| .. }`) work on a sequence of tuples. Looked: methods.md ("on every method that takes a block").
19. **b03** — that `max_by` on a sequence gives `T?` printed `Some(("p1", 1))`. Looked: methods.md.
20. **b03** — that `puts w.i` reads a field of a generic `var` struct after two `next`s and gives `2`. Follows from code; the guess is that `next` on the `var` really mutates (only shown for non-generic `Countdown`). Looked: iterators.lume.
21. **b04** — that `s.find { .. }` inside a function bounded by `Iterator[Int]` works with the bound parameter (the docs say "`for` and chains work on the parameter"; `find` is a terminal method, not a chain). Looked: collections.md "Generic code".
22. **b04** — that a `var` parameter of a generic bounded type (`var s: S`) is allowed (var_param_copy says `var` needs "a list, map, string or struct") and that it changes the caller's own value, so `c.n` is `3` afterwards. Looked: functions-and-blocks.md (`var` parameters), interfaces.md (`twice[T: Ticker](var x: T)` — but the docs never print `c` afterwards), errors/var_param_copy.
23. **b04** — that a plain (non-`var`) bound parameter given to `total` walks a copy, so `c.n` is unchanged after `total(c)`. Looked: collections.md ("A loop or chain walks a copy").
24. **b04** — that `puts held.to_list` on a value held as `Iterator[Int]` walks a copy, so the following `held.next` gives `Some(2)` (the docs show `next` then `take`, but never `next` after a chain on a held value). Looked: iterators.lume/.expected.
25. **b04** — that a `var` binding typed `Iterator[Int]` can be reassigned to a value of another conforming type (`held = Nat(i: 100)`). Looked: interfaces.md "Where an interface can be used" (says a binding works; no reassignment shown).
26. **b04** — that `s.next.or(0)` chains `.or` directly on the result of a `var self` call. Looked: methods.md `T?` table.
27. **b05** — that a **field** named `sum` shadows the list method on the value (docs say "a method or field ... (`len`, `first`)", example only shows a method `len`). Looked: collections.md, iterable.lume.
28. **b05** — that an own method `first` with a *different* return type (`Int`, not `Int?`) is accepted and used. Looked: collections.md (silent on signatures).
29. **b05** — that `s.map { .. }.sum` is the chain's `sum` (18), unaffected by the field. Looked: collections.md.
30. **b05** — that inside `[C: Iterable[Int]]`, `c.sum` is the list method over `items` (9), not the type's own field (100). The docs are silent on what a bound parameter sees. Looked: collections.md "Generic code", generics.md.
31. **b05** — that a set literal with a duplicate passed to an `Iterable[Int]` bound sums its distinct items (6). Looked: collections.md (sets drop duplicates; bounds take sets).
32. **b05** — that `c.max` inside `[C: Iterable[Str]]` works (needs `Ordered` on `Str`) and gives `Some("pear")` for a set and `Some("c")` for a list. Looked: collections.md, methods.md.
33. **b05** — that `items` is called fresh for each use, so `vals.sort` gives `[1, 3, 5]` every time and `last` is `Some(5)`. Looked: iterable.lume.
34. **b06** — that an interface can mix required methods (`tick`, `now`) with defaults in any order, and that a default (`describe`) may call another default (`label`) and a required read-only method. Looked: interfaces.md "Required methods and defaults".
35. **b06** — that `x.tick` on its own line (result dropped) is fine for an `Int` result (the docs' `twice` does this; dropped_result is only about `or Error`). Looked: interfaces.md, errors/dropped_result.
36. **b06** — that `run3(c)` changes the caller's `c` (so `c.t` is `3`). Looked: interfaces.md (does not print `c` after `twice(c)`), functions-and-blocks.md.
37. **b06** — that `var all: [Ticker] = [...]` is allowed for an interface with a `var self` method, and `for var x in all: x.tick` changes each held value in place (`[11, 1]`). Nothing in the docs shows `for var` over a list of interface values, or a `var self` method called through a held interface value. Looked: interfaces.md, control-flow.md "`for var`".
38. **b06** — that an own method overriding a default (`label` on `Metronome`) needs no keyword and wins. Looked: interfaces.md (shown for `name`).
39. **b07** — the exact stderr text for an overflow: `error: Int overflow in `*`` for `*` (the docs show only the `+` form), followed by the `  (set LUME_BACKTRACE=1 to see where in the generated Rust)` hint line (shown in errors.md only for `!`, not for overflow). Looked: numbers.md "What stops the program", errors.md "`!`".
40. **b07** — the exit status: "a failure status" — I could not tell whether it is 1 or 101 (Rust's panic code). Looked: numbers.md, concurrency.md (exit 1 for a `main -> () or Error` failure only).
41. **b07** — that the lines printed before the stop (`puts` to stdout) all appear (are flushed) before the error. Looked: nothing says.
42. **b07** — that `skip(60).first` calls `next` exactly 61 times (does not ask for a 62nd), so it does not overflow. Follows from "Nothing is worked out before it is asked for". Looked: collections.md.
43. **b07** — that `big = 9223372036854775807` followed by `big - 1` is not folded into an error and prints `9223372036854775806` (overflow_constant shows literal arithmetic is checked at compile time; a binding read is the docs' own run-time example). Looked: numbers.md, errors/overflow_constant.
44. **b07** — that `v = v * 2` inside `next` is not caught at compile time (field value unknown). Looked: errors/overflow_constant.
45. **b07** — docs contradiction noted: operators.md says overflow stops the program "in a built binary as much as under `lume run`", numbers.md and install.md say `lume build` wraps unless `--checked`. Irrelevant to `lume run`, which both agree stops. Looked: operators.md, numbers.md, install.md.
46. **b08** — that `spawn: nat.take(4).sum` (no `await` in the body) is fine and gives `10`, and that awaiting `b` before `a` changes nothing. Looked: concurrency.md "A task that waits without `await`".
47. **b08** — that a `next` sequence (a struct) is copied into a task like any other local, and that the outside `nat.i` stays `0`. Looked: concurrency.md "What a task sees: copies".
48. **b08** — that `var tasks: [Task[[Int]]] = []` parses (a list type inside `Task[...]` inside a list type). Looked: concurrency.md (only `Task[Str]`, `Task[Int]`, `Task[()]`).
49. **b08** — that `nat.skip(w).first.or(0)` after `w = await pause(5)` in a task body is fine. Looked: concurrency.md.
50. **b08** — that inside `spawn:`, `var local = mine` (copying an outside `var`) and `local.next` are allowed — the task changes only its own new `var`, not `mine`, so the "is a copy" error does not fire. Looked: concurrency.md ("A task may declare its own `var`s"), errors/spawn_changes_copy.
51. **b08** — that a task whose last line is `local.next` is a `Task[Int?]` and prints `Some(11)`; and that `?`-less optional results are not turned into anything else. Looked: concurrency.md (`?` makes `T?`; nothing about a plain optional last line).
52. **b08** — that `async def pause` with `await Time.sleep(n)` taking an `Int` parameter works. Looked: concurrency.md `Time.sleep`.
53. **b09** — that a `next` with no `var self` (and no explicit `self`) whose body only reads is accepted as an ordinary method, and `o.next` by hand prints `Some(1)` before the error is reached (the error is from `lume check`, so only the error is printed). Looked: collections.md ("A `next` of another shape is just a method").
54. **b09** — the help line for a missing `var self`: `` `Ones`'s `next` does not take `var self`; to loop over it, write `def next(var self) -> T?` ``. Only the wrong-return help is documented (`` `Counter`'s `next` gives an `Int`; ... ``), and iterator_bound_var_self uses "its `next` is `-> Int?`". Looked: errors/next_wrong_shape, errors/iterator_bound_var_self.
55. **b09** — the first line and caret for this case are the same as next_wrong_shape (`cannot loop over a `Ones``, caret at the expression after `in`). Looked: errors/next_wrong_shape, errors/loop_no_items.
56. **b10** — that a `next(var self, step: Int)` called by hand as `s.next(5)` compiles as an ordinary method. Looked: collections.md.
57. **b10** — the help line for an extra parameter: `` `Stepper`'s `next` takes `step`; to loop over it, write `def next(var self) -> T?` ``. Undocumented. Looked: errors/next_wrong_shape.
58. **b10** — that the error is not instead the generic "`for` needs a list, a set, a map, a range, or a type with `def items -> [T]`" help of loop_no_items. Looked: errors/loop_no_items vs next_wrong_shape.
59. **b11** — that the both-`items`-and-`next` refusal happens for a type that is never looped over (the docs' example also never loops, so this is fairly safe), and — the real guess — that the caret is on the `next` line even when `next` comes first (not on whichever of the two is second). Looked: errors/items_and_next.
60. **b11** — that the help line is word for word the same as items_and_next regardless of order. Looked: errors/items_and_next.
61. **b11** — that `cards.pop` in a `var self` method giving `Str?` is accepted as the body of `next`. Looked: errors/items_and_next (same body).
62. **b12** — the whole message for an own interface with a `var self` requirement not met: modelled on iterator_bound_var_self (`` `name_of` needs `T: Ticker`, and `Sundial` is not Ticker ``, help `` its `tick` is `-> Int`, and `Ticker` asks for `(var self) -> Int` ``). The docs only say "a type fits only if its own method takes `var self` too". Looked: interfaces.md, errors/iterator_bound_var_self, errors/iface_generic_unmet.
63. **b12** — that the bound check, not something about `x` being unused, is the only error; and that `"#{x}"` on a bounded `T` is fine. Looked: generics.md ("interpolation is available on every type").
64. **b13** — that an extra `var self` (interface asks read-only, type's method takes `var self`) is refused at all — the docs are silent on this direction. Looked: interfaces.md "Required methods and defaults".
65. **b13** — its wording, modelled on iface_mismatch: `` `Bag` is used as a `Sized`, but its `size` is `(var self) -> Int` and `Sized` needs `-> Int` ``, no help line, caret at the argument. The way a signature with `var self` is written inside a message (`(var self) -> Int`) is from iterator_bound_var_self. Looked: errors/iface_mismatch, errors/iterator_bound_var_self.
66. **b13** — that the error is reported at the call that uses `Bag` as a `Sized` (the parameter typed as the interface), not at the struct or at `x.size` inside `show`. Looked: errors/iface_mismatch.
67. **b14** — the message for a `var self` method called on a non-`var` parameter: `` `x` is immutable, but `tick` changes it `` (from immutable_receiver), caret on the `.`. Looked: errors/immutable_receiver, errors/push_immutable.
68. **b14** — its help line: `declare the parameter `var x: T` to change the caller's value`. Every documented help is for a local (`declare it with `var c = ...` on line 7`); none is for a parameter. Looked: errors/immutable_receiver, functions-and-blocks.md.
69. **b14** — that the call to an interface's required `var self` method through a bound is checked for mutability the same way as a concrete struct's method (and not, for example, by refusing the interface or the bound). Looked: interfaces.md.
70. **all FAIL programs** — that the path in ` --> ` is printed as given on the command line, `corpus/m69/<file>`, as examples/errors show `examples/errors/<file>`. Looked: examples/errors/*.expected.
71. **all FAIL programs** — that the gutter widens to three spaces for a two-digit line number (`   |`) while ` --> ` keeps two spaces. Looked: errors/iterator_bound_var_self, errors/next_wrong_shape (both two-digit).
72. **all programs** — that the `#` comment header lines are ignored and counted in line numbers. Looked: examples/errors/*.lume (they start with comment lines and the line numbers count them).

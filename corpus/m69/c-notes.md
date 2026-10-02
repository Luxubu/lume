# Reviewer C — notes

## What I read

- `corpus/m69/BRIEF.md`.
- `docs/README.md`, `docs/tour.md`, `docs/examples.md`.
- `docs/reference/`: `collections.md`, `interfaces.md`, `concurrency.md`,
  `control-flow.md`, `printing.md`, `structs-and-enums.md`, `errors.md`,
  `generics.md`, `strings.md`, `methods.md`, `numbers.md`, `patterns.md`,
  `operators.md`, `functions-and-blocks.md`, `names.md` — all in full.
- `examples/iterators.lume` + `.expected`, `examples/iterable.lume` +
  `.expected`, `examples/async.lume` + `.expected`,
  `examples/interface_methods.lume` + `.expected`,
  `examples/tasks_blocking.lume` + `.expected`,
  `examples/mini/lexer.lume` (first 80 lines), `examples/mini/parser.lume`
  (lines 40–200).
- `examples/errors/`: the file list, and `next_wrong_shape`,
  `items_and_next`, `iterator_bound_var_self`, `interface_var_self`,
  `extend_var_self`, `iterable_bound_unmet`, `for_var_next`,
  `for_var_items`, `items_not_a_list`, `immutable_receiver` (`.lume` and
  `.expected`).
- A grep for `var self` and for `Iterator[` / `Iterable[` across
  `examples/`, `packages/` and `docs/`.

Not read: `docs/QUESTIONS.md`, compiler source, `tests/`, other corpus
folders, README/HISTORY/CHANGELOG. Nothing was run.

## The programs

| file | what it is | leans on |
|---|---|---|
| `c01_calc.lume` | arithmetic calculator: a lexer that is an `Iterator[Tok]`, a recursive-descent parser with `var self` methods giving `Int or Error`, error reporting | `def next(var self) -> Tok?`, `for` walking a copy, `count`/`map`/`filter`/`to_list` on a sequence, bound `[S: Iterator[Tok]]`, a held `var it: Iterator[Tok]`, nested literal patterns `Some(Op("+"))`, `?`, a map tally, a set |
| `c02_signals.lume` | a traffic light as an endless sequence with cars queueing at it, and a turnstile fed by an endless cycling script of events | endless `next`, `take`/`find`/`take_while`/`to_set`/`enumerate`/`zip` with a list, bound `[S: Iterator[Snap]]`, moving a `var` by hand, enum map keys and set items, `match (state, e)` in a `var self` method, tuple destructuring |
| `c03_numbers.lume` | number theory: primes, twin primes, Collatz, prime factors, divisors, perfect/abundant classification | endless and self-ending sequences, two of your own zipped together, `skip`/`first`/`len`/`max`, an `items` type in a bound `[C: Iterable[Int]]`, bounds `[S: Iterator[Int]]`, map of lists through missing keys, `map` over map pairs, set `intersect` |
| `c04_ledger.lume` | a bank ledger: accounts as an interface with a required `var self` method, moves and transfers with refunds, a statement of failures | interface with required `def apply(var self, ..) -> () or Error` and a default, `[A: Account]` with `var acct: A`, `[Account]` list with `push` and `for var`, `var` list parameter, enum + `match`, `?`, an `items -> [(Str, Int)]` type with two-name `for` and blocks, `pad`/`decimals` |
| `c05_workqueue.lume` | jobs planned onto workers (longest first, least-loaded worker), run as tasks, results combined; batches of a sequence run as tasks, some failing | `async def main`, `spawn:` indented and one-line, `await` on a list in start order, `?` inside `spawn:`, `async def` awaited inside a task, an `items` type and a list both given to `[C: Iterable[Int]]`, a sequence of lists with `[S: Iterator[[Int]]]`, `min_by` on a map with ties |
| `c06_replay.lume` | **meant to FAIL**: a text-adventure log replayer whose reader's `next` has no `var self`, handed to `[S: Iterator[Event]]` | the bound check error of `examples/errors/iterator_bound_var_self.lume`, plus list patterns over `split`, an enum of events, a `var self` world with a set and a map |

## Guesses

1. **c01** — `s.count { ... }` works directly on a generic parameter
   `s: S` with `S: Iterator[Tok]`. collections.md says "`for` and chains
   work on the parameter" and lists `count` as a chain method, but no
   example calls `count` on an `Iterator` parameter (iterators.lume uses
   `for` and `filter`).
2. **c01** — `.to_list` called straight on a finite sequence of your own
   (`Lexer.of(src).to_list`) gives the whole list. Docs and
   `examples/iterators.lume` only show `.to_list` after `take`/`map`/
   `filter`/`zip`; collections.md lists `to_list` among methods that "need
   every item", which implies it works. Also relied on in c03
   (`c6.to_list`, `Factors(...).to_list`) and c05 (`batcher.to_list`).
3. **c01** — a held interface value of a user enum item type,
   `var it: Iterator[Tok]`, works as `Iterator[Int]` does in
   iterators.lume (only `Int` is shown).
4. **c01** — after two `it.next` calls, `it.take(5).to_list` continues from
   where `it` is (`[Num(value: 4)]`), by analogy with the last line of
   `examples/iterators.expected` (`[1]`).
5. **c01** — a statement `match` inside a value `match` arm whose arms are
   `Some(RParen) -> advance()` and `_ -> return Error("missing )")`, and
   the outer arm then ends with `inner`. patterns.md shows `return` in an
   arm, and mini/parser.lume shows `advance()` in arms; the combination
   (unit arm + returning arm as a statement) is assumed fine.
6. **c01** — the arm `_ if c.digit? ->` with an indented block that
   declares a `var` and loops, inside a `var self` method. Guards on `_`
   are in patterns.md; a multi-line guarded `_` arm is assumed.
7. **c01** — `ops.add(t.show)` as the last statement of a `for` body (its
   `Bool` dropped silently). collections.md has `seen.add(x * 10)` as a
   loop body, so I treated it as checked for that shape, but it is inside
   a nested `for` over a `filter` chain of a sequence here.
8. **c01** — the set prints in insertion order as `{"+", "*", "-", "/"}`
   (methods.md: a set keeps the order items were added; printing.md shows
   `{"one", "two"}`).
9. **c02** — `zip` on an endless sequence with a **list** argument
   (`sig.zip(ARRIVALS)`), taken apart by `for snap, cars in ...`.
   iterators.lume only zips with a range (`colors.zip(1..5)`); methods.md
   says `zip` takes `[U]`.
10. **c02** — `_ = sig.next` to throw away a `Snap?`. errors.md shows
    `_ = ...` only for dropping a failure; I did not know whether a bare
    `sig.next` statement is allowed, warned about, or refused, nor whether
    `_ =` is accepted for an optional.
11. **c02** — inside `def feed(var self, e: Ev)`, `match (state, e):` and
    then assigning `state = Open` inside an arm. No doc shows a field that
    is being matched on being reassigned in the arm (fine in Lume terms;
    a borrow question in the Rust).
12. **c02** — an enum value as a map key prints bare: `{Red: 6, Green: 4,
    Yellow: 2}`; and a set of enum values prints `{Red, Green}`.
    structs-and-enums.md shows a struct key (`{P(x: 1, y: 2): "here"}`),
    printing.md shows `Dot` bare; enums as keys are only implied by
    "a value made of hashable parts can be a set item or a map key".
13. **c02** — `simulate(sig: Signal)` takes a sequence as a plain
    (non-generic, read-only) parameter and calls `sig.zip(...)` on it; I
    assumed the chain walks a copy of the borrowed value as it does for a
    local ("A loop or chain walks a copy"), with no `var` needed.
14. **c02** — `find`, `take_while`, `to_set` and `enumerate` on a struct
    sequence give what they give on a list (`Some(Snap(tick: 3, light:
    Red))` etc.) — `find`/`take_while`/`enumerate` are shown on `Fib`/
    `Cycle`, `to_set` on a sequence is not shown.
15. **c03** — `found.any? { |p| ... }` on an **empty** list is `false`
    (so 2 is prime). methods.md does not say what `any?`/`all?` give on an
    empty list; I chose `any?` because "none passes" seemed the safer
    reading.
16. **c03** — `if not found.any? { |p| k % p == 0 }:` parses: `not` in
    front of a method call taking a `{ }` block, as an `if` condition.
    operators.md gives `not` its precedence, but no example puts a block
    call in an `if` condition.
17. **c03** — `while true:` that only leaves by `return`, followed by an
    unreachable `None` so the function has a last value. I assumed the
    trailing `None` is needed or at least accepted, with no error.
18. **c03** — zipping two sequences of your own:
    `Naturals(k: 0).zip(primes)`. Docs only show `zip` with a range or a
    list argument.
19. **c03** — `c6.max` on a finite sequence gives `Some(16)`; `max` is on
    the chain list in collections.md but not shown on a sequence.
20. **c03** — `Factors(...).to_set.intersect(small)` keeps the receiver's
    order (`{2, 3, 5}`). methods.md says sets keep insertion order but not
    which side's order `intersect` keeps (here both agree anyway, so only
    the print form is at stake).
21. **c03** — to avoid an unknown I copied fields into locals before using
    them inside a block in a method (`m = n` in `Divisors.items`, `k = n`
    in `Primes.next`). Whether a field can be read inside a non-kept block
    in a method (and inside a `var self` method) is not stated;
    functions-and-blocks.md only says a *kept* block cannot use `self`.
    I record it as a guess because I could not check it, even though the
    program steers round it.
22. **c04** — `for var a in all:` over a `var all: [Account]` (a list of an
    interface type) and calling the `var self` method `a.apply(...)` on
    each held value. interfaces.md shows reading through held values and
    `var self` only through a generic bound; calling a `var self`
    interface method through the pointer is not shown.
23. **c04** — a `var` parameter whose type is a list of an interface,
    `def deliver(var all: [Account], ...)`, passed a `var` local.
24. **c04** — `return a.apply(amount)` from inside a `for var` loop, the
    `() or Error` passing through unwrapped (errors.md: "a value that is
    already a `T or Error` passes through").
25. **c04** — errors.md warns that in a `-> () or Error` function a stray
    `Error(...)` *statement* is silently discarded. I could not tell
    whether a last-line `Error(...)` in such a function counts as the
    function's value or as a stray statement, so I wrote
    `return Error(...)` everywhere in `() or Error` functions.
26. **c04** — `match m:` as the last expression of a `() or Error`
    function with arms like `Deposit(w, c) -> deliver(all, w, c)?` (value
    `()`) and an indented arm ending in a statement `match`; assumed to
    type-check as `()` and count as success.
27. **c04** — interpolating an enum value whose fields are `Str` puts
    quotes round them: `ok   Deposit(who: "ana", cents: 2500)`.
    printing.md: interpolation is the "value as itself" form, which only
    differs for text; a variant inside interpolation is not itself text,
    so its fields print "as you would write" them. Not shown directly.
28. **c04** — on an `items -> [(Str, Int)]` type, `st.len`,
    `st.map { |who, n| n }.sum` and `st.filter { |who, n| n > 1 }` with
    two-name blocks. collections.md says the type "takes every method a
    list has" and methods.md says two names take a pair apart "on every
    method that takes a block"; the combination is not shown.
29. **c04** — formatting a negative balance: `-500` cents gives
    `(-500).to_float / 100.0` = `-5.0`, and `.decimals(2)` → `-5.00`; `decimals` of a negative is only
    shown for rounding (`-0.125` gives `-0.13`).
30. **c04** — no warning, or only a stderr warning, for `var all:
    [Account] = []` filled by `push`; interface_methods.expected shows a
    stderr warning for an annotated literal of mixed types. Expected
    output lists stdout only.
31. **c05** — the annotation `var checks: [Task[Int or Error]] = []`:
    concurrency.md writes `Task[T or Error]` in prose but never as an
    annotation; I assumed `or` is allowed inside the brackets.
32. **c05** — a bound whose type argument is itself a list:
    `[S: Iterator[[Int]]]`, and a sequence whose items are lists
    (`def next(var self) -> [Int]?`).
33. **c05** — a struct field named `cost` alongside a top-level function
    `cost`, both used in one `spawn:` body
    (`Summary(..., cost: spent)` next to `cost(n)`); assumed no clash.
34. **c05** — `for b in bad:` (a sequence of your own) with a `spawn:`
    body inside the loop that mentions the loop variable `b`; copying in
    is documented for list loops (`BATCHES` example) but not for a
    sequence loop.
35. **c05** — `batcher.len` and `short_batches(batcher, 3)` after
    `batcher.to_list` each start from the beginning (an immutable binding;
    each chain walks a copy).
36. **c05** — `Time.sleep(5 - i)` with a varying, small argument inside an
    `async def` called from tasks; only the order of `await` matters for
    the output, which the docs guarantee.
37. **c06** — the exact error for a `next` with **no** `self` at all
    (`def next -> Event?`). `examples/errors/iterator_bound_var_self` has
    `def next(self) -> Int?` and says
    ``its `next` is `-> Int?`, and `Iterator[Int]` asks for
    `(var self) -> Int?` ``; I assumed the same wording, with `Event` in
    place of `Int`.
38. **c06** — that this is the only error reported and nothing else is
    printed by `lume check` (no summary line, no other error first, e.g.
    from type-checking `Replay.next` or `World.apply`).
39. **c06** — the caret column (11, under `replay`) and the gutter widths
    (`   |` for a two-digit line), copied from the shape of the error
    examples, where the caret sits under the called function's name.
40. **c06** — the path printed as `corpus/m69/c06_replay.lume`, i.e. as
    given on the command line, by analogy with `examples/errors/*.expected`.
41. **c06** — a `def next -> Event?` that only reads fields and returns
    `parse(line)` (already an `Event?`) is accepted as a plain method; the
    docs say a `next` of another shape "is just a method".
42. **c06** — list patterns with a literal first item and an unnamed or
    named rest, `["#", ..]` and `["say", ..words]`, over `line.split`;
    patterns.md shows `[cmd, ..rest]` and literals inside, but not a
    literal followed by `..`. (Only matters if the compiler checks it
    before the call — see 38.)
43. **all** — that a long single-line list literal is fine and that a list
    literal may **not** be broken over lines: I found no doc saying a
    bracketed literal may continue onto the next line (operators.md says
    `|>` is "the one operator that may continue an expression onto the
    next line"), so I kept every list on one line.
44. **all** — EXPECTED OUTPUT is standard output only. Any compiler
    warning (c04's `[Account]` list) goes to standard error per
    concurrency.md/errors.md and is left out.

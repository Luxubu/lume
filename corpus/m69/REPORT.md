# Milestone 69 review corpus — REPORT (fourteenth review round)

This round tested milestones 64–68:
- iterating types of your own: `items` (`Iterable[T]`) and
  `next(var self) -> T?` (`Iterator[T]`), endless sequences, bounds and
  values held as the interface;
- interfaces whose required methods take `var self`;
- integer overflow under `lume run`;
- `spawn:` with and without `await`;
- the error messages.

Three reviewers wrote twenty-nine programs from `docs/`, `examples/` and
`packages/` alone, with nothing run, and counted what they had to guess.
The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday programs: a pager, a tokenizer, a ring buffer, ID minting, a playlist that cycles, running averages, a folder tree, a word stream, one refusal |
| `b01`–`b14` | the edges: sequences that end and that never do, every stopper, generic and tuple items, bounds and held values, own names shadowing list methods, `var self` interfaces both ways, overflow inside `next`, sequences in tasks, six refusals |
| `c01`–`c06` | whole programs: a calculator whose lexer is an `Iterator[Tok]`, traffic signals as endless sequences, number theory with zipped sequences, a ledger over a `var self` interface, a work queue of tasks, one refusal |

## The numbers

| | round thirteen | round fourteen |
|---|---|---|
| guesses | 31 | **179** (A 63, B 72, C 44) |
| meant to run, right on the first run | 20 of 21 | **16 of 21** |
| refused as predicted | 6 of 6, 3 word for word | 8 of 8, **5 word for word** |
| rustc leak | 0 | **2** (b04, c05) |
| wrong output | 0 | 0 |

The guess count is not comparable with earlier rounds: this round's reviewers
listed every inference, including ones they then confirmed from an example
(B counted the gutter width and the path in ` --> `). About thirty point at
real gaps in the docs; those are filled (below). After the fixes, all 29
programs print what their authors worked out, except b14, whose help line
now says more than the reviewer predicted.

No program printed a wrong answer. Every failure was a refusal or a leak.

## What the programs found

1. **A `var` held as an interface could not be given a new value** (b04,
   rustc leak). `var s: Iterator[Int] = Countdown(..)` and then
   `s = Nat(..)` reached rustc without the value being boxed. This was
   older than milestone 68 and held for every interface, and for a field
   (`h.shape = Circle(..)`, also inside a `var self` method), a list
   position and a map key. All four box now.
2. **`zip` could not take a sequence of your own** (a06, c03). `feed.zip(r)`
   and `Naturals(k: 0).zip(primes)` were refused with "cannot take a
   block". Both work now and stay lazy: a list zipped with an endless
   sequence stops with the list. Finding it showed that the check for a
   list receiver also caught lazy chains, since `materialized` turns a
   chain into a list; fixed.
3. **`_ = value` worked once per scope** (c02). The second `_ = sig.next`
   said "`_` is immutable and cannot be reassigned". `errors.md` promises
   `_ = ...` as the way to drop a value; it is now a discard every time.
4. **A task handle was cloned** (c05, rustc leak). `tasks.push(t)` copied
   `t` when a later loop also had a `t`, and a `JoinHandle` has no clone. A
   task now always moves, as in Rust. Awaiting it after it was given away
   is refused with its own message (`examples/errors/task_used_after_move`);
   a later `t = spawn: ..` in another block is another `t` and is fine.
5. **"an `Ones`"** (b09). The article rule took the `O` for a vowel sound;
   names starting with "one" or "once" now take "a".
6. **The help lines for a `next` of the wrong shape** (b09, b10) now use the
   reviewer's wording: "does not take `var self`", and "takes `step`",
   naming the parameters.
7. **A `var self` method called on a parameter** (b14) said "declare it with
   `var x = ...`", which is advice for a local. It now says to declare the
   parameter `var x: T`, or to copy it into a `var` of another name.
8. **A warning named `lume.Iterable`** (a03). Built-in interfaces are named
   as a program writes them everywhere now.

## What the docs now say

- `collections.md`: which list methods a sequence has (all but the ones that
  change a list in place), that `skip`, `enumerate` and `all?` stop an
  endless one too, what `zip` takes, pairs from `next`, that a walk copies
  the value as it is now (a parameter, a held value and a task's copy too),
  and that nothing remembers a sequence has ended.
- `interfaces.md`: a `var self` mismatch is refused both ways, and what a
  `var self` method can be called on.
- `operators.md` contradicted `numbers.md` on overflow in a built binary
  (b07). It now points to `numbers.md`, which says which operators the
  message names and that the exit status is 101.
- `errors.md`: `_ = ...` may be written any number of times.
- `concurrency.md`: putting a task in a list gives it away.

## Left open

- The guess about a `var self` default (a04: an interface with a reading
  default and a `var self` requirement) was right; a default that changes
  the value stays refused.
- Whether a list literal may span lines (c, "all") was not settled by any
  doc the reviewers read; they kept lists on one line.

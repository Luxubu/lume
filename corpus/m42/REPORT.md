# Milestone 42 review corpus — REPORT (eighth review round)

This round tested milestones 37–41, which changed the language: functions of
a type (`def self.name`), blocks as values, a module's function as a block,
tuple ordering, `await f()?`, and Rust's coherence rule for `extend`. Three
reviewers wrote thirty programs from `docs/` and `examples/` alone — nothing
run — and counted what they had to guess. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a10` | everyday programs leaning on the new features |
| `b01`–`b10` | types and abstractions where the new features meet them: generics, interfaces, `extend` and its coherence rule |
| `c01`–`c10` | whole programs, seven of them over several modules, and concurrency with kept blocks |

## The numbers

| | round seven | round eight |
|---|---|---|
| guesses | 98 | **88** (A 21, B 33, C 34) |
| meant to run, right on the first run | 19 of 27 | 16 of 23 |
| refused as predicted | 3 of 3 | 6 of 7, and the seventh refused for the right reason in other words |
| rustc leak | 3 | 2 |
| refused although correct | 1 | 5 |
| wrong output | 4 | **0** |

After the fixes: **23 of 23** print exactly what their authors worked out by
hand, and all seven refusals say what their authors predicted.

The count fell for the first time in eight rounds, and it fell while the
round was aimed at five new features. No guess was about printing, strings or
the everyday library.

## What the programs found

Every problem was in the new features, and every one was where two of them
meet. None was in a feature used alone — each milestone's own tests had that
covered.

1. **A block written in a map, a tuple or a `Some(...)`** was not told the
   type the binding declared, so `ops: {Str: (Int, Int) -> Int} = {"add": { |a,
   b| a + b }}` was refused (a02, a10). A list already was.
2. **A generic method's argument** was built with the type the method
   *declared* (`T`) rather than the one the call filled in, so
   `nested.push(Stack.empty)` could not tell what to make (b02).
3. **A generic call could not read `T` off a block made by an expression**
   — `then(times(2), times(5))` — only off a literal block or a name (b05).
4. **One kept block type had two Rust spellings.** A `(Str) -> Str` kept in a
   binding took `&str`; one kept in a field declared `(T) -> T` with `T = Str`
   took `&String`, so a block made for one could not go in the other — a
   rustc leak (b05). A kept block now always takes text as `&String`, and one
   lent to a parameter that hands out `&str` gets a small adapter.
5. **A literal block's parameter types were fed back into inference**, so
   the `T` of `FnHandler(f: { |e| ... })` "worked out" to itself (b10).
6. **A generic type's methods were compared unfilled** when checking an
   interface: `FnHandler[Event]` was said to have `handle(x: T)`, not
   `handle(x: Event)` (b10). The same fix turned two programs from earlier
   rounds, refused since rounds five and six and both meant to run, into runs
   that print exactly what their authors wrote: `corpus/m28/x04` and
   `corpus/m30/r07`.
7. **A kept block holding an interface value** captured the generic
   reference the function was lent — a rustc leak (b04). It now keeps the
   boxed value, and interfaces and type parameters carry `Send + Sync`, as
   every Lume value is, so such a block can still go to a task.
8. **Milestone 38's coherence rule was only half built.** Disjoint extends of
   a *built-in* were allowed; of the program's own generic type at concrete
   arguments — `Box[Int]` and `Box[Str]`, `Pair[Int, Str]` and `Pair[Str,
   Int]` — they were still filed under one name and refused (b06). Rust allows
   them; now so does Lume. The refusal side already held: `Cell[T]` against
   `Cell[Str]` and the bounds clash were refused word for word as predicted
   (b07, b09).
9. **A function of a type that keeps its block argument**, imported from
   another module, was passed a lent block instead of a kept one — a rustc
   leak (c07). The pass that works out which parameters are kept updated the
   function table but not the copy that travels with the type, and skipped
   methods.

And one message: `==` on a struct holding a block ended in a stray comma; it
now reads `` `==` cannot compare `Button` values: they hold a block ``.

## Where the guesses went

- **Where the new features meet** (about 40): a block in a map, a tuple or
  an `Option`; a block in a generic container; a type function that keeps a
  block; a kept block holding an interface value, a `shared var`, a list of
  blocks; `extend` of the program's own generic type. The docs described
  each feature on its own and rarely two together.
- **Modules** (13, all from reviewer C): whether `()` goes on a no-argument
  type function through a prefix, whether a type function or a re-exported
  function hands over as a block through a prefix, where a cross-module
  clash is reported.
- **Contradictions in the docs** (asked by all three): `methods.md` said `<`
  between tuples is refused, and that a tuple holding a `Float` cannot be
  sorted — both untrue since milestone 37; `modules.md` said a function
  reached through a prefix is "always a call, never a function value",
  which milestone 37 made untrue; the method table left out `sum` with a
  block. All four are fixed, with a checked example where one fits.
- **Exact error wording** for the programs meant to fail (7): the authors
  predicted all of them from `examples/errors/`, and missed only on how an
  expression is quoted (`ops[op]`, not `ops[...]`).

## Left open

- `pub async def` — the order is right but no page shows it.
- A kept block reading a `shared var` sees later changes; the concurrency page
  says so for tasks but not for blocks.
- Two more type-function spellings through a prefix (`model.User.guest()`
  with `()`) work and are not shown.

## Verdict (condensed from the three notes)

A: a new user could learn the new features from the docs; the weak spot was
tuples, where the pages disagreed. B: the core of each feature is well
documented; the guesses came from combinations the docs never show
together. C: modules and concurrency are now learnable from their pages; the
cost of a wrong guess in concurrency is still the highest.

The round did what review rounds do: every new feature worked on its own, and
nine defects sat where two of them met.

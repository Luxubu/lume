# Milestone 28 review corpus — REPORT (fifth review round)

Thirty-six programs by three independent reviewers working from `README.md`
and the top-level `examples/` only — no compiler source, nothing run, every
expected output computed by hand before the compiler saw a line.

| set | reviewer aimed at |
|---|---|
| `x01`–`x12` | generic `extend` and interfaces with type parameters — the newest feature |
| `y01`–`y12` | the seams: features meeting each other, rather than features alone |
| `z01`–`z12` | whole programs with a purpose — a ledger, a league table, a diff, a router, a TOC |

Alongside them, `x-notes.md`, `y-notes.md` and `z-notes.md`: what each program
probes, and ninety-six numbered places the README left the reviewers guessing.
Those notes are the most useful thing this round produced.

## First run

| class | count |
|---|---|
| ok | 17 |
| lume error | 13 |
| rustc leak | 6 |
| crash | 0 |
| wrong output | 2 of 17 |

Of the seventeen that ran, fourteen matched the hand-computed output exactly.
The two that did not were both the reviewer's own: one mis-added a
`pad_right(8)` and an explicit space, and one printed `Int.to_char` without
knowing it gives a `Char?`, so `Some(A)` appeared where `A` was wanted. The
README now says `to_char` is optional.

## After the fixes

| class | files |
|---|---|
| ok (27) | x01–x03, x05–x09; y01–y07, y09, y10; z01–z09, z11 |
| lume error (9) | x04 x10 x11 x12 y08 y11 y12 z10 z12 |
| rustc leak (0) | — |
| crash (0) | — |

Twenty-four of the twenty-seven print exactly what their author worked out by
hand, including the aligned money columns in `z01`, the league table with
tie-breaks in `z03`, the LCS diff in `z07`, the Dijkstra route in `z08` and
the wrapped Markdown in `z11`.

Five of the nine rejections are the reviewers' deliberate-error programs, and
four of those five now give the error their author asked for, at the line
they named. The fifth (`y11`) is stopped earlier by an unrelated rule, which
is itself a finding — see 11 below.

## Ranked problems, and what was done

The six leaks were six different bugs, not one idea seen six ways. That is
the opposite of the last round, and it is what a corpus aimed at the seams
rather than at one feature is supposed to find.

1. **A name used in two arguments of one constructor was moved by the
   first.** `Heading(level: n, title: t, slug: slug(t))` moved `t` into the
   field and then borrowed it for `slug`. *Fixed:* the arguments still to
   come count as later uses, so the earlier one copies — the same rule the
   last round applied to `if` branches, now applied to argument lists.

2. **A `T or Error` used after `.or(default)` was moved.** The guard asked
   whether the *inner* value was copyable; an `Error` never is, so a
   `Int or Error` is never copyable whatever `T` is. *Fixed.*

3. **A value held as an interface could not be copied,** so `[Priced]` with
   `sort_by`, and a struct field typed `Bag[Int]`, both failed. A pointer to
   a trait cannot be cloned on its own. *Fixed:* every interface carries a
   `lume_box`, implemented by the value inside, and the pointer copies
   through it; the interface also requires `Debug`, since a value held as an
   interface still ends up inside a struct that derives it, and a struct
   holding one compares that field by what it shows.

4. **A generic type conforming by having the methods emitted an impl that
   spoke of a `T` it never introduced.** Structural conformance built the
   impl against the bare name, so `Chain[T]` became `impl Compact<T> for
   Chain`. *Fixed:* a generic type conforms at its own parameters. Its own
   methods are then reached by path, which needs a turbofish.

5. **An `extend` whose interface argument contradicted its target was
   accepted,** and failed later at a call site. `require_conforms` re-checked
   against the bare interface name, which let the arguments be inferred
   again — so `Bag[Str]` was re-verified as `Bag` and matched whatever the
   type happened to say. *Fixed:* the check keeps the arguments it was asked
   about. `x12` now gets exactly the message it asked for.

6. **Writing through a missing map key aborted at run time** — `grid[0][1] =
   "x"` on an empty `{Int: {Int: Str}}` stopped with *no such key in map*.
   The README already promised *"a missing map key starts from an empty
   value"*; the compiler did not do it. *Fixed* where the value has an empty
   form to start from (a map, set, list, string or number).

Four more, found among the rejections:

7. **A generic `extend`'s parameter was not substituted at a call site,** so
   `[1, 2, 3].accepts?(2)` said *takes `x: T`, but this is a `Int`* — a
   parameter that appears only in an argument, never a result. *Fixed*, along
   with the same substitution for a method reached through a bound that
   carries arguments (`P: Pairish[Int, Int]`).

8. **A bare built-in method of `self` did not resolve inside an `extend`.**
   `def accepts?(x: T) = contains?(x)` on `[T]` said *unknown function
   `contains?`*, while `self.contains?(x)` worked. *Fixed:* inside an
   `extend`, a bare name means `self`, the way it already does inside a
   struct.

9. **Tuples and optionals could not be `extend` targets in practice.** The
   rule said any shape, but only the named containers had a shape to register
   under, so `extend (A, B) with Pairish[A, B]` compiled and then failed
   conformance. *Fixed:* a tuple's shape is its arity.

10. **A block declared `(Str) -> T or Error` could not choose between the
    error and the value.** An `if` giving `Error(...)` on one side and a value
    on the other was rejected, though the same code in a function is fine;
    and inference took `Error` as what `T` stood for. *Fixed:* a declared
    block ends the way a function does, and an error branch says nothing
    about `T`.

11. **`x?` on a value was read as a name ending in `?`.** The lexer folds a
    trailing `?` into an identifier, so `raw_scores?` was one token and the
    propagation was lost. Two reviewers hit this independently. *Fixed:*
    where nothing is called `raw_scores?` and `raw_scores` is a value here,
    the `?` is propagation. `y12` now reports the error its author wanted.

12. **`Result` was reserved with no Lume reason.** A reviewer named a struct
    `Result` for a football result; Lume spells that concept `T or E` and has
    no `Result` of its own, so this was the target language leaking into the
    author's namespace — the same finding as `Box` in the last round, and
    fixed the same way: the generated code qualifies its own.

## Left open, and why

- **`extend [[T]]` and `extend {Str: [T]}`** (x04). Nested targets register
  under the same shape as their outer container, so `[[Int]]` matches both
  `extend [T]` and `extend [[T]]` with no rule saying which wins. The
  reviewer flagged the absence of any specificity rule; picking one is a
  design decision, not a fix.
- **A method with its own type parameters inside an interface** (x10). The
  compiler says plainly that this is not supported yet, which is the right
  answer until it is.
- **`puts (x + y).method`** (y08, y11). The space-before-parenthesis rule is
  deliberate and documented, but two reviewers wrote this shape naturally and
  one of them had a deliberate-error program masked by it. Worth revisiting.
- **How an `extend` crosses a module boundary.** Raised first in the x notes
  and unanswered anywhere: an `extend` has no name, so nothing can be marked
  `pub`, and `extend [T] with Bag[T]` is now a claim about every list in the
  program. This is the largest open question the round produced.

## What the reviewers had to guess

Ninety-six numbered points across the three notes files. Raised by more than
one reviewer:

- **Which side `pad` and `pad_right` fill,** and what happens on overflow.
  Never stated; inferred from one line of an example. Two reviewers said
  every column in every program rested on it.
- **How a value prints** — an optional, a list, a map, a set, a tuple, a
  struct, a `Float`, a `T or E`. Raised in every round so far, including
  this one.
- **`join` on a non-`Str` list**, listed with no element-type restriction and
  only ever demonstrated on `[Str]`.
- **Whether `Str.len` counts characters or bytes.** The README promises
  character counting for `s[i]` and says nothing about `len`.
- **Nested patterns through a recursive enum's field,** which the README
  describes as working in one paragraph and as an error in another. One of
  those two sentences is wrong, and it has now been reported twice.
- **`next`, `break` and `Time`**, which appear in examples but not in the
  README.

## Newcomer's verdict (condensed)

The newest feature held: of the twelve programs aimed squarely at generic
`extend`, the failures were in how its types were *carried* — substituted at a
call site, resolved through a bound, given a shape to register under — and
not in the idea. The seams were where the round paid off: four of the six
leaks came from the set that deliberately made features meet, and three of
those were ownership, in places a person would reach naturally and never
suspect. The whole-program set found the two that a feature probe would never
find: a nested map write that the README already promised worked, and a
struct innocently named `Result`.

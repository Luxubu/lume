# Milestone 26 review corpus — REPORT (fourth review round)

Thirty-six programs written by an independent reviewer from `README.md` and
`examples/` only — no compiler source, nothing run. Expected output for every
program was worked out by hand and written into the file's header before the
compiler saw it.

Three sets, because two big features had landed since the last round:

| set | what it aims at |
|---|---|
| `g01`–`g12` | user-defined generics (m22) and interfaces that take type parameters (m25) |
| `b01`–`b12` | blocks of your own (m23) |
| `m01`–`m12` | whole programs, mixing the new features with everything older |

Alongside them, three notes files (`g-notes.md`, `b-notes.md`, `m-notes.md`)
recording what each program probes and what the reviewer found
under-specified. Those lists are the most useful part of the round and are
condensed at the end.

## First run

| class | count |
|---|---|
| ok | 19 |
| lume error | 11 |
| rustc leak | 6 |
| crash | 0 |
| wrong output | 0 |

Six leaking programs, all in the two newest features. No program produced
wrong output and none crashed — where the compiler was wrong, it was wrong
loudly.

## After the fixes

| class | files |
|---|---|
| ok (30) | b01–b06, b08–b11; g01–g04, g06–g10; m01–m10, m12 |
| lume error (6) | b07 b12 g05 g11 g12 m11 |
| rustc leak (0) | — |
| crash (0) | — |

Every one of the thirty running programs printed exactly the output the
reviewer had computed by hand, character for character, including the padded
tables in `m01`/`m04`, the money columns in `m08`/`m12`, and the observable
block-call count in `b11` (four, not two).

Of the six rejections, four are deliberate — `b12` (a block parameter
shadowing an outer name), `g11` (a bare `T` used as a map key and with `<`),
`g12` (`Cell` is a `Comparable[Int]`, not a `Comparable[Cell]`) and `m11`
(assigning an immutable binding from inside an `if` inside a `for`) — and
each produced the message the reviewer asked for, at the definition rather
than at the instantiation. The other two are the reviewer meeting a wall:
`b07` guessed a positional enum variant field, and `g05` named an interface
`Sized`.

## Ranked problems, and what was done

1. **A block's parameters travelled differently depending on their type.**
   Lent for a struct, passed by value for an `Int` — so a block declared
   `(Int) -> Int` and one declared `(T) -> U` compiled to different Rust
   shapes and could not be handed to each other. This is the
   declared-versus-instantiated split again, and it broke every program that
   reused a block. *Fixed:* everything but `Str` is lent, so every block of
   the same arity has the same shape.

2. **A block used inside its own argument leaked** — `map_of(map_of(xs, f), f)`
   held the block twice at once. *Fixed:* the inner result is taken into a
   binding first.

3. **A forwarded block was moved**, so a function that gave the same block to
   two calls failed on the second. *Fixed:* a forwarded block is lent, except
   into a recursive call, where it is moved so every level has one type. The
   reviewer had asked (g-notes 10) whether a recursive handoff was supportable
   at all; it is.

4. **A conformance at an argument other than `Self` leaked.** Where the
   interface declared a parameter `T`, the trait lends it; the `extend` or
   structural method underneath took the copied type by value. `Ruler`
   (`compare(other: Int)`, so a `Comparable[Int]`) and every `Renders[Int]`
   hit it. *Fixed:* arguments travel the way the trait declared them, not the
   way the `extend` block spelled them.

5. **A generic enum boxed the wrong field.** Whether a recursive field needs
   a pointer was decided from what the instantiation filled in rather than
   from what the declaration said, so `Chain[Chain[Int]]` disagreed with
   `Chain[Int]`. *Fixed:* the declared type decides.

6. **`m[k] += v` leaked for a map of lists or of strings**, and borrowed a key
   the map then had to keep. *Fixed:* a list extends, a string pushes, and the
   key is owned.

7. **An `if` chain whose condition consumed a name a later branch still
   needed** moved it instead of copying. *Fixed:* the branches still to come
   count as uses of what the condition mentions.

8. **A block declared `(T) -> U or Error` could not use `?`,** and a bare
   value at its end was not the implied `Ok` — so the one documented way to
   fail inside a block did not work (b-notes 14 is exactly this question).
   *Fixed:* such a block ends the way a function does. A block handed to a
   built-in still cannot, and now says so without suggesting a declaration the
   author cannot write.

9. **A block parameter could not be handed to a built-in.** `xs.sum(f)` and
   `xs.map(f)` worked for a top-level function's name but not for a block the
   function had itself been given — the obvious way to write
   `def total(xs, f) = xs.sum(f)`. *Fixed.*

10. **The error for a positional enum variant field suggested a wrong fix.**
    `Num(Int)` said *write `Num(Int: Float)`*. Lume constructs and matches
    variants positionally but declares them with names, so this is a fair
    thing to get wrong. *Fixed:* it now says a field needs a name, not just
    the type, and suggests `Num(value: Int)`.

11. **`Box` was reserved with no Lume reason.** Two of the twelve
    generics/blocks programs independently named a one-field container `Box`,
    and `Box` is not a Lume word — the target language was leaking into the
    author's namespace. The third review round flagged this and it was left.
    *Fixed:* the generated code qualifies its own `Box`, so the name belongs
    to the author. `Vec`, `String`, `Rc`, `Arc`, `Mutex`, `Sized`, `Clone`,
    `Copy`, `Iterator` and `Ordering` are still held back (g05), and the
    message no longer explains itself in terms of Rust.

## Checked by hand and correct

Inference for one, two and three type parameters — from an argument, from a
nested generic call, from two arguments that must agree, from a tuple's parts,
from a typed binding where `T` appears only in the return type, and from a
block's body where the parameter appears nowhere else. Generic structs and
enums, including recursive ones, ones holding themselves (`Box[Box[Int]]`,
`Chain[Chain[Int]]`, `Seq[Seq[Int]]`), generic methods that add their own
parameters, and bounds on a generic type's parameters. `Ordered` and
`Hashable` at scalars, user structs and tuples. Parameterised interfaces at
two different arguments in one program, interfaces as field types, defaults
reached through structural conformance with no `extend`, and defaults reached
on a built-in extended after the fact. All four block forms, blocks nested in
blocks, blocks on methods, chained trailing blocks, `_` shorthand on a method,
a block whose value is bound, and `f(x) { }.field`. Generics crossed with
`T?`, `T or Error`, maps, sets and modules.

The formatter was idempotent on every file and every formatted program printed
what the original did.

## Where the reviewer had to guess

Forty-odd under-specified points are listed in the three notes files. The ones
that came up in more than one set:

- **How values print.** `puts` of an optional, list, map, set, tuple, struct,
  `Float` or `T or E` has no documented spelling, and the examples that print
  one record no expected output. Every reviewer routed around it; all three
  raised it first or near-first.
- **`join` on a non-`Str` list** is listed with no element-type restriction
  and only ever shown on `[Str]`.
- **Two bounds on one parameter** (`[T: Ordered + Hashable]`) has no syntax
  and was wanted for sort-then-dedup.
- **`enum` declarations are never spelled out** — the README shows uses and
  one line of prose. `b07` guessed, and guessed wrong.
- **`var self` is named but never written,** so no program could change its
  own fields from inside a block — arguably the most likely closure corner in
  real code.
- **A block parameter that is not last** can only be reached by `_` or a
  function's name; this is never said and quietly constrains API design.
- **Zero-argument blocks in brace form,** and **zero-parameter generic
  functions** (`def none_of[T] -> T?:`), have no shown spelling.
- **`Char` has no literal,** so `s[i].or("?")` reads as a type error and
  `m06`/`b09` slice instead.

One design question the round settled in the compiler's favour: `m01` prints
`(9/8).decimals(2)`, and the reviewer noted that Rust's and Python's own
formatters give `1.12` where "halves away from zero" demands `1.13`. Lume
gives `1.13`, which is what the milestone-24 findings promised and what money
wants.

## Newcomer's verdict (condensed)

Generics and parameterised interfaces held up under everything thrown at
them — inference never needed a hint that the README had not already
described, and the deliberate-rejection programs were all refused at the
definition with the right message. Blocks held up too, but every leak in the
round was in how a block *travels*: reused, forwarded, handed to a built-in,
or carrying a failure out. Those four are one idea, and the round's real
result is that it is now one implementation.

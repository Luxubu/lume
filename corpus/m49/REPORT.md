# Milestone 49 review corpus — REPORT (tenth review round)

This round tested milestones 47–48: packages. Three reviewers wrote
twenty-five programs from `docs/` and `examples/` alone — nothing run — and
counted what they had to guess. Every program is several packages side by
side: an `app/` and the libraries it uses, some of them using each other.
The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a08` | everyday programs: a library of helpers, what `lib.lume` passes on and keeps, `extend`s shipped by a library, tests in a library, a package with both `main.lume` and `lib.lume`, a subfolder inside a library |
| `b01`–`b10` | the rules at the edges: privacy, the orphan rule in each direction, `extend` travelling through a dependency, crates, a misnamed dependency, `import lib` |
| `c01`–`c07` | whole programs of three or more packages, four of them diamonds, combining packages with generics, interfaces, enums, kept blocks, `?`, `async` and constants |

## The numbers

| | round nine | round ten |
|---|---|---|
| guesses | 115 | **38** (A 11, B 15, C 12) |
| meant to run, right on the first run | 17 of 22 | **12 of 18** |
| refused as predicted | 6 of 8 | 7 of 7 refused; **6 of 7 word for word** |
| accepted, then stopped at run time | 2 | **0** |
| rustc leak | 3 | **0** |
| refused although correct | 2 | **5**, and one reviewer's slip |
| wrong output | 0 | 0 |

After the fixes: **18 of 18** print exactly what their authors worked out by
hand, and a05's library prints the `lume test` report its author predicted.
The seventh refusal, b10's `import lib`, was refused as predicted; only its
wording was invented, because no page showed it. It is shown now.

The count fell by two thirds on a feature two milestones old. The notes say
why: a package is a folder of modules, and everything reviewers knew about
modules held. Nearly every guess was the same question in different forms:
*does this module rule still hold across a package boundary?* Every time,
the answer was yes.

## What the programs found

1. **A subfolder module passed on by a library could not be named by its
   path** (c02, c06). After `pub import model.event` in `lib.lume`,
   `import core.model.event` was called private; only `import core.event`,
   the name it is passed on under, worked. Both work now.
2. **A module of the app named like a package deeper in the tree** (b07).
   The app depends on `labels`, which depends on `tally`, and the app has
   its own `tally.lume`. Both got the same internal name, and the app's
   calls went to the wrong one. The app's module now has a name of its own.
3. **A bounded generic struct from another module** (c01).
   `pricing.Basket[catalog.Book]` said `Book` was not `Priced`: the bound on
   `Basket[T: Priced]` travelled unqualified, so the importing file looked
   for its own `Priced`. Functions already qualified their bounds, and now
   structs and enums do too. This is an older defect: packages only made it
   easy to reach.
4. **A generic struct built with a block field, where only the declared
   type says what `T` is** (c03). `Stage(label: .., run: { |n| n * 2 })`
   returned from a `-> Stage[Int]` function, or put in a
   `[Stage[Int]]` list, read `n` as `T`. The wanted type now fixes the
   parameters before the arguments are read, and the items of a typed list
   literal are each wanted as the item type. Also an older defect.

a06 named a struct `Task`, which [names.md](../../docs/reference/names.md)
lists as a built-in name. The compiler refused it with a message saying so,
and the reviewer's program was changed to `Chore`. That is a slip, not a
defect.

## Where the guesses went

- **Does a module rule hold across packages?** (asked 20 times, by all
  three): passed-on modules arriving by `import pkg` alone, a package prefix
  on a bound, on a type with arguments, on an `extend` target, three-part
  item imports, structural conformance to another package's interface, how
  a value of another package's type prints. All of these are now answered
  in one section of [packages.md](../../docs/reference/packages.md), "What
  carries over from modules".
- **The edges of the orphan rule**: is `[Bean]` your own type (no, as in
  Rust), and does an `extend` reach you through a package you cannot
  import (yes). Both are now stated.
- **Where errors land**: an error in a dependency's file is reported there,
  and `lume.toml` mistakes come first. Both are now stated.
- **Language questions unrelated to packages**: `"05".to_int`,
  `[Int or Error]` as a type, `for var` in a `var self` method. Every
  reviewer's guess was right.

## Verdict (condensed from the three notes)

A: packages are learnable from the docs; one sentence about what
`import pkg` brings would have removed a third of my guesses. B: the rules
are Rust's and clear once stated, and every rule with an example in
`examples/errors/packages/` cost nothing. C: the guesses were about
spelling two documented features together across a boundary, never about
what a package is.

Four defects. Two sat in the new package code; two were older defects in
generics and modules that multi-package programs reached first. None was
the worst kind: nothing compiled and then did the wrong thing.

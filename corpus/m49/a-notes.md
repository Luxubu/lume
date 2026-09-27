# Reviewer A — everyday programs built from packages

## What I read

- `corpus/m49/BRIEF.md`
- `docs/README.md`, `docs/tour.md`, `docs/install.md`, `docs/examples.md`
- `docs/reference/packages.md`, `modules.md`, `interfaces.md`, `generics.md`,
  `structs-and-enums.md`, `functions-and-blocks.md`, `errors.md`,
  `control-flow.md`, `patterns.md`, `collections.md`, `methods.md`,
  `strings.md`, `printing.md`, `names.md`, `numbers.md`, `operators.md`
- `examples/packages/` (shelf, tally, labels, `new.expected`,
  `tally/test.expected`)
- `examples/errors/packages/*` (every package error, with its `expected`),
  plus `examples/errors/import_twice.expected`, `import_cycle.expected`,
  `missing_arm.expected` for the error layout
- `examples/tests/` (`parse`, `generics`, and the `.expected` files),
  `examples/travel/`, `examples/shipping/`, `examples/lib/README.md`,
  `examples/lib/report.test.expected`

Not read: `docs/QUESTIONS.md`, `docs/reference/concurrency.md`,
`docs/reference/io.md` (not needed), and anything outside `docs/` and
`examples/`. The other reviewer's folders in `corpus/m49/` were not opened.

## The programs

| file | what it is | RUN/FAIL | leans on |
|---|---|---|---|
| `a01_textkit/` | a library of text helpers (`slug`, `title`, `initials`, `truncate`, `rule`) used by a blog-post app | RUN | `import pkg`, `import pkg.item` and then `item()`, a `pub` constant through the package prefix, `split`/`capitalize`/`slice`/`repeat` |
| `a02_inventory/` | a `stock` library that passes `items` and `money` on with `pub import` and keeps `rounding` private; the app builds a shelf | RUN | `pub import` in lib.lume, passed-on modules used after only `import stock`, `import stock.items.Kind`, `stock.empty` without `()`, bare and qualified variants, a struct printed across packages |
| `a03_shapes/` | a `geometry` library that has an interface and helpers but no shapes; the app's own structs conform | RUN | `import geometry.Shape`, `[Shape]`, a `[T: Shape]` bound, structural conformance across packages, an interface default called directly on an app struct |
| `a04_durations/` | a `clock` library that ships `extend Int`, `extend Str` and `extend [Int]` with its own `Minutes` interface | RUN | extends that travel with `import clock`, defaults on built-ins, an app struct as a `clock.Minutes`, a mixed `[clock.Minutes]` list |
| `a05_ledger/` | a `ledger` library with `test` blocks in lib.lume (the header also gives the output of `lume test` in the library folder); the app reads lines and prints a statement | RUN | `T or Error` from a library, `?` in match arms, list patterns with literals, `import ledger.Entry`, `ledger.Kind.Debit`, `pad`/`pad_right` |
| `a06_todo/` | a package `app` that has both `lib.lume` and `main.lume`, plus a `columns` library for layout | RUN | `import app` from main.lume, `var self` methods, `for var` over a field, bare variants from a parameter's type, a dependency used by both halves |
| `a07_badges/` | an app that writes `extend Str with badges.Badge` | FAIL | the orphan rule, and the exact error layout from `examples/errors/packages/orphan/expected` |
| `a08_settings/` | a `settings` library whose parser lives in the subfolder `parse/` (`parse/value.lume` passed on, `parse/line.lume` private) | RUN | module paths from the package root inside a subfolder, `pub import parse.value` arriving as `value`, a qualified variant of a passed-on enum in a pattern, a `"""` constant, `?` through a private module |

## Guesses

1. **`import pkg` alone gives you the modules the library passes on.** In
   a02 the app uses `items.Item` and `money.show`, and in a08 it uses
   `value.Value`, after writing only `import stock` or `import settings`.
   Where I looked: packages.md "Importing a package" says `pub import counts`
   means "`import tally.counts` works", and `shelf/main.lume` writes *both*
   `import tally` and `import tally.counts`, so it never shows whether the
   first is enough. modules.md "`pub import`" says a file that imports yours
   "gets it too", but only between modules of one program. Assumed that the
   modules.md rule holds across packages. For a08 I also assumed that a
   subfolder module passed on without `as` (`pub import parse.value`)
   arrives by its last part, `value`. modules.md implies this but never shows
   it.
2. **Three-part item import through a package: `import stock.items.Kind`.**
   packages.md shows `import tally.counts` (a passed-on module) and
   `import tally.total` (an item of lib.lume), but never an item of a
   passed-on module. Assumed it works as `import shop.items.Kind` does in
   modules.md.
3. **A no-argument function through a package prefix needs no `()`:
   `stock.empty`, `app.board`.** modules.md says this for a *module*
   prefix. packages.md only ever calls package functions that take
   arguments (`tally.total(xs)`). Assumed a package name counts as a module
   prefix.
4. **How a struct from another package prints.** a02 prints
   `Item(name: "apples", …)`. printing.md says your own types print "with
   their fields named". modules.md says a type is *named* `shop.items.Item`
   in error messages, but no page prints a value whose type comes from
   another module or package. Assumed the bare type name.
5. **An app type that conforms structurally to a library's interface gets
   that interface's defaults as its own methods.** a03 has
   `Circle(2.0).describe` and a04 has `m.hm` and `m.longer_than?(45)`, with
   no `extend` in the app. interfaces.md says "every conforming type gets
   it" and calls `Sq(2.0).report` directly, but only inside one file. In the
   packages example, `l.empty?` works only because `labels` wrote an explicit
   `extend Label with tally.Sized`. Assumed that the defaults reach across
   packages without an extend.
6. **Built-in values held as an imported interface.** In a04,
   `day: [clock.Minutes] = [Meeting(…), 90, "2:30"]` holds an `Int` and a
   `Str` as the interface, and the conformances for both come from `clock`'s
   extends. interfaces.md does this with `[Walk] = [Bag(…), [10, 20]]` in one
   file. Assumed it works the same when the extend is in another package.
7. **`"05".to_int` is `Ok(5)`.** numbers.md says a sign and surrounding
   spaces are allowed and `_` is not. It says nothing about leading zeros.
   Assumed they are accepted, as Rust's `parse` accepts them.
8. **The exact output of `lume test` in a library folder.** I copied the
   format from `examples/packages/tally/test.expected` and
   `examples/tests/*.expected`: `test NAME ... ok`, a blank line, then
   `4 tests: 4 passed, 0 failed`. I took the `exit: 0` line in those files
   to be added by the test harness, not printed by `lume`. I assumed nothing
   else is printed (no "compiling …" line).
9. **The orphan error's wording for `Str`, and how it names the
   interface.** The only example is for `[T]`, and it says "`[T]` is built
   in". Assumed the same template with `Str` in place of `[T]`. I also
   assumed the interface appears as written, `badges.Badge`; the example's
   `tally.Sized` is both the written form and the full name, so it does not
   tell the two apart.
10. **Only the orphan error is reported.** Once that extend is refused,
    a07's later line `"beta".framed` would also fail, because `Str` has no
    `framed`. I found nothing that says whether `lume check` stops at the
    first error or goes on to list more. Every expected file shows one
    error. Assumed one.
11. **`for var t in tasks` inside a `var self` method, assigning a field of
    `t`.** a06 writes `t.status = to` in `Board.move`. control-flow.md shows
    `for var c in cs` on a local `var` list and calls a `var self` method
    (`c.bump`). It shows neither a field assignment on the loop variable nor
    a loop over a field of `self`. Assumed both work.

Things I checked and found settled, so they are not counted: which paths
appear in `-->` and `help:` lines when run from the repository root (from
`examples/errors/packages/*/expected`); the gutter layout for a two-digit
line number (modules.md); that a library needs no `[dependencies]` section
(`tally/lume.toml`); that `main.lume` imports its own package's lib.lume by
the package name (packages.md); that module paths in a subfolder start at
the package root; that `test` blocks are stripped by `lume run`; that a
library's `pub` constants are items like any other; bare variants when the
type is known; `extend Int`, `extend Str` and `extend [Int]` of one interface
not overlapping.

## Verdict

Mostly yes. The first program was easy to write with `packages.md` and
`examples/packages/` open side by side: `lume.toml`, `lib.lume`,
`import pkg`, the private-module error and the orphan rule are all stated
plainly, and `examples/errors/packages/` gives the exact messages. Most of
my eleven guesses sit in one gap. `packages.md` says the module rules carry
over, but never shows the everyday forms working across a package boundary:
whether `import pkg` alone brings the passed-on modules, the three-part
`pkg.module.Type`, a zero-argument call through a package name, interface
defaults on a type that conforms structurally, and how a foreign struct
prints. The shelf example imports `tally` and `tally.counts` both. That
makes guess 1 feel like a trap, and one sentence would settle it. The
remaining guesses (`lume test` output, several errors or one, leading zeros)
are small and have nothing to do with packages.

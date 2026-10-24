# Reviewer C — round twenty-one (m103) notes

## What I read

- `corpus/m103/BRIEF.md`.
- `docs/reference/`: `errors.md` (all, especially "`!` — unwrap or stop" and
  "Where the program stopped"), `structs-and-enums.md` (all, especially "A
  struct with no fields"), `collections.md` (all, especially "A chain too
  long for one line"), `operators.md`, `printing.md`, `strings.md`,
  `numbers.md` (all of the parts on printing, `decimals` and what stops a
  program), `methods.md`, `control-flow.md`, `functions-and-blocks.md`,
  `interfaces.md`, `patterns.md`, `modules.md` (first 330 lines),
  `packages.md` (first 120 lines), `concurrency.md` (first 200 lines),
  `names.md` (keyword table).
- `docs/tutorial/03-numbers-and-text.md` (division by zero stop) and
  `12-generics-modules-and-tests.md` (tests and packages); `docs/install.md`
  (when warnings are shown).
- `examples/`: `chains_over_lines.lume`/`.expected`,
  `unit_structs.lume`/`.expected`, `pipes.lume`/`.expected`,
  `port/orders.expected` and `crate.expected` (the `!` warning, two-digit
  gutters), `packages/shelf/lume.toml`, `packages/new.expected`, and in
  `examples/errors/`: `assert_with_message`, `assert_in_expression`,
  `bang_on_int`, `keyword_as_name`, `leading_dot_block`, `unit_struct_bare`,
  `iface_unused_param`, `iface_pass_held_value`,
  `iface_generic_through_pointer`, and `UNREACHED.md`.

I did not run the compiler or any program. Every output was worked out by
hand.

## Programs

| id | file | what it is | meant to | stops at |
|---|---|---|---|---|
| c01 | `c01_pricing.lume` | order-pricing engine: discount strategies as field-less structs in a `{Str: Discount}` field, picked by order code; best-seller tally, revenue report, `\|>` mixed with `.` steps, chains in nested `for` headers | RUN | — |
| c02 | `c02_gradebook.lume` | grade book: grading schemes as field-less structs in a `[Scheme]` catalog, one per course in a field; report chain in a method | STOP | `scores.at(i)` in method `Student.score`, line 66: `no item at 3 — the list has 3` |
| c03 | `c03_door.lume` | door state machine: states as field-less structs whose `on` gives the next state, replaced in a `var self` method; log enum with a field-less `Beep` payload, printed, compared, counted; `[State]` binding warning | RUN (with a warning) | — |
| c04 | `c04_units.lume` | length converter: units as field-less structs with interface defaults; `Request.parse` validates with `!` | STOP | `to_float!` in type function `Request.parse`, line 79: `` `abc` is not a number `` (plus the `!` warning) |
| c05 | `c05_stack_calc.lume` | command dispatcher for a stack calculator: commands as field-less structs in a `{Str: Command}` field; script lines from a generator; chains over the map | RUN | — |
| c06 | `c06_payroll/` (`lume.toml`, `main.lume`, `rates.lume`) | weekly payroll package: pay policies as field-less structs in another module behind a `pub interface`, in a map field with a fallback | STOP | `paid / e.hours` in `rates.effective_rate`, `rates.lume` line 32: `division by zero` |

## Guesses

1. **Gutter of a run-time stop on a two-digit line.** The docs show stops
   only on one-digit lines (`  |`). I used `   |` (three spaces, the width of
   the line number plus one), as compile errors and warnings on two-digit
   lines do. Files: c02, c04, c06. Looked: `errors.md` "Where the program
   stopped", `tutorial/03`, `examples/port/orders.expected`,
   `examples/errors/iface_pass_held_value.expected`.
2. **Gutter of a warning on a three-digit line** (c03, line 126): `    |`,
   extrapolated from the one- and two-digit cases; the `  -->` line keeps
   two spaces as in every example. Looked: as in 1, and `interfaces.md`
   (the pointer warning on line 29).
3. **A plain interface-typed local gives no pointer warning.** `d =
   engine.pick(o.code)` (c01, in `main` and in `Engine.total`), `p`/`cmd`/`d`
   bound by `Some(..)` patterns, and `for` variables over interface lists
   are taken to be silent; only the four forms the docs list (`[Shape]`,
   `Shape?`, map of `Shape`, tuple holding one) warn. Files: c01, c02, c03,
   c05, c06. Looked: `interfaces.md` "Where an interface can be used".
4. **A map literal of different field-less structs where a field's type
   says `{Str: Discount}`** is accepted, as the docs show for a list
   literal in a `[Shape]` field. Files: c01, c05, c06 (written over several
   lines inside the call's parentheses in c05 and c06). Looked:
   `structs-and-enums.md` "A field may be an interface type",
   `interfaces.md`.
5. **Handing back an interface value taken from a field's map or list**
   (`Some(d) -> d`, `return s` inside `for s in schemes`, and `return u` in
   a `Unit or Error` method, where it is the implied `Ok`) compiles and
   copies the value. Files: c01, c02, c04, c06. Looked: `interfaces.md`
   (`biggest` returns an item of a `[Shape]` parameter; nothing from a
   field or map, nothing in a `T or Error`).
6. **One `-> State` method giving two different structs**, one with
   `return Closed()` and one as its last expression, converts both to the
   interface. Files: c03. Looked: `interfaces.md` ("`scaled` gives one").
7. **Unused parameters give no warning**: `cents`/`qty` in
   `NoDiscount.off`, `arg` in `Add.apply` and its siblings, and unused block
   names (`|sku, n|`, `|name, c|`, `|s| 1`). Files: c01, c05. Looked: nowhere
   documents an unused-name warning either way; only an unused interface
   type parameter is an error (`examples/errors/iface_unused_param`).
8. **A field used inside an ordinary (not kept) block in a method**,
   `scheme.grade(s.avg)` inside `.map { .. }` in `Course.report`, is allowed.
   Files: c02. Looked: `functions-and-blocks.md` forbids `self` only in a
   kept block; `collections.md` forbids it only in an async chain's block.
9. **A `|>` step directly after a `.name` step**, `orders` / `.map { .. }` /
   `|> .sum`, continues the same expression. Files: c01. Looked:
   `collections.md` ("`|>` ... may also begin a line"), `operators.md`,
   `examples/pipes.lume` (which uses `|>` after `?`, never after a `.` step).
10. **A stop inside a type function** (`def self.parse`) names the line in
    the type function, as for a method. Files: c04. Looked: `errors.md`
    lists a method, a block, a generator, another module and a task, but not
    a `def self.` function.
11. **How the package is run and named.** I wrote the header for `lume run
    corpus/m103/c06_payroll/main.lume` from the repository root, with the
    stop naming `corpus/m103/c06_payroll/rates.lume`. Files: c06. Looked:
    BRIEF.md, `packages.md`, `install.md` (a package needs no file name when
    run from inside it; nothing says what a path from outside looks like).
12. **An interface default in another module calls that module's private
    function** (`Policy.line` calls `cents`) when `main.lume` calls it on a
    value. Files: c06. Looked: `modules.md` says a method of a `pub` type may
    call a private function of its module, and an interface's defaults are
    public with it; an interface default is not named there by itself.

12 guesses.

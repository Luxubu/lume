# Reviewer C — round eight notes

## What I read

All of it from `docs/` and `examples/` only; no compiler, tests, corpus
(other than the brief), README.md or HISTORY.md, and nothing was run.

- `docs/tour.md`, `docs/examples.md`
- `docs/reference/modules.md`, `concurrency.md`, `functions-and-blocks.md`,
  `structs-and-enums.md` (closely); `generics.md` (the generic `extend` and
  "the Rust rule" sections), `interfaces.md` (extend, where interfaces go,
  `pub`), `methods.md`, `collections.md`, `printing.md`, `control-flow.md`
  (if/for/ranges), `strings.md` (ordering, slicing, split), the keyword
  table in `names.md`
- `examples/modules/` (app, blocks, users/model, users/store),
  `examples/lib/` (README, seq, table, report), `examples/match/`,
  `examples/travel/`, `examples/shipping/`, `examples/port/app_async.lume`
- `examples/kept_blocks.lume`, `type_functions.lume`, `extend_coherence.lume`,
  `async.lume`, `ordering.lume`, `blocks_of_your_own.lume` and their
  `.expected`
- `examples/errors/`: the extend overlap ones (single-file, `overlap/`,
  `travel/`), `block_kept_*`, `block_returned`, `block_value_untyped`,
  `value_call`, `static_*`, `type_call_method`, `spawn_uses_self`,
  `shared_lock_in_block`, `call_optional_block`, `private_access`,
  `reexport/`

## The programs

| path | what it is | RUN/FAIL | leans on |
|---|---|---|---|
| `c01_users/` (main.lume, data/model.lume) | parse "name: age" lines into users with a type's own `parse` | RUN | `model.User.parse(..)?`, `import data.model.User` then `User.parse`, a type function handed over as a block both ways, `User.guest` / `model.User.guest()`, `model.Tier.Senior` |
| `c02_shop/` (main.lume, shop/item.lume, shop/pricing.lume) | a basket priced by rules whose blocks come from another module | RUN | `Item.make`, `item.Item.make(sku:, cents:)`, `pub def` returning a block, a struct field holding blocks made in two other files, `f(x)(y)` through a prefix, printing a struct with blocks |
| `c03_text/` (main.lume, lib/fmt.lume, lib/text.lume) | a module's functions handed around as blocks | RUN | `words.map(text.shout)`, `text` passed on by `pub import`, a module function in a typed binding and a list of blocks, `fmt.twice(text.shout)("hi")` |
| `c04_extend/` (main.lume, report.lume, geo/*.lume ×6) | conformances shipped by a library, reaching main through a chain | RUN | extend travelling up two imports, extend of another module's struct, `extend [T: Sized]`, a `[Pt]` passed as an interface value, `[Int]`/`[Str]` extends of one interface in two files |
| `c05_overlap/` (main.lume, kinds/*.lume ×3) | two modules that each extend lists with one interface | FAIL | the coherence rule across imports: `extend [T]` and `extend [Int]` of one interface, error at the second import |
| `c06_fetch/` (main.lume, net/fetcher.lume) | a pretend fetcher: fan out, collect, fail | RUN (then exit 1) | `async def main -> () or Error`, `await module.fn(..)?`, `spawn:` of an imported async def, start-order results, `await t?` failing out of main |
| `c07_jobs/` (main.lume, work/job.lume, work/steps.lume) | jobs whose steps are blocks from another module, run on tasks | RUN | `Job.of(..)` keeping a block, blocks from `steps` called in tasks, a kept block changing a `shared var` called from every task |
| `c08_ledger.lume` | one task per account, problems recorded through a kept block | RUN | `def self.parse_move -> (Str, Int) or Error`, a kept block with a `shared var` passed into a helper inside `spawn:`, `shared var total +=` |
| `c09_grader.lume` | a grader built from kept blocks and type functions | RUN | `Check.at_least(n)` keeping `n` in a block field, list of blocks folded, `curve(10)(95)`, `boosts.at(1)(30)`, `.map(Grade.from_score)`, tuple sort and `<` |
| `c10_limits.lume` | a method that hands back a block using its fields | FAIL | "a block that is kept cannot use `self` or its fields" |

Seven of the ten are multi-file. c06 is meant to run to its last line and
then stop with exit 1; its header gives standard output only.

## GUESSES — 34 in total

Confidence is how sure I am the guess matches what the compiler does.

### Modules (13)

1. **Is `()` allowed on a no-argument type function through a prefix,
   `model.User.guest()`?** Looked: modules.md "Calling a function through a
   prefix" (a module function may take or drop `()`), structs-and-enums.md
   (type functions shown only bare: `Temp.freezing`, `Light.all`),
   examples/modules/blocks.lume (`model.User.guest("Eve")`, with an argument).
   Guessed: yes, same as a method or a prefixed function. 60%. (c01)
2. **Is a no-argument type function bare after importing the type by name,
   `User.guest`?** modules.md says a *function* imported by name needs `()`;
   a type function is not a function imported by name, and locally
   `Temp.freezing` has none. Guessed: bare works. 75%. (c01)
3. **Can a type function be handed over as a block through the module
   prefix, `LINES.map(model.User.parse)`?** modules.md: "A module function
   reached through its prefix is always a call, never a function value" —
   while functions-and-blocks.md says `words.map(text.shout)` hands it over,
   and type_functions.lume does `.map(Temp.from_f)` locally. The
   three-part path is shown nowhere. Guessed: works. 65%. (c01)
4. **Is `LINES.map(User.parse)` (type imported by name) handed over?** Same
   places; the local form is documented. 80%. (c01)
5. **May a type function call a private helper of its own module
   (`Item.make` → `guess_kind`)?** modules.md says a *method* of a `pub` type
   may. Guessed: a type function may too. 85%. (c02)
6. **Can one type's function call another type's function by type name in
   the same module (`Tier.for_age(n)` inside `User.parse`)?** Not shown;
   only bare calls to the same type's functions are. 90%. (c01)
7. **Does a function passed on by `pub import` hand over as a block
   (`words.map(text.shout)` where `text` came through fmt.lume)?** modules.md
   shows passed-on modules *called*; functions-and-blocks.md shows
   `text.shout` handed over from a direct import. 85%. (c03)
8. **Can a module's function be kept in a typed binding or a list literal
   (`loud: (Str) -> Str = text.shout`, `[text.shout, text.initial, {..}]`)?**
   functions-and-blocks.md shows `g: (Int) -> Int = double` for a local
   function and "passing `text.shout` to a block parameter of your own",
   not a binding. 75%. (c03)
9. **Is a `[Pt]`, Sized only through `extend [T: Sized] with Sized` in another
   module, accepted where the parameter type is the interface `Sized`?**
   interfaces.md shows structs as interface values; generics.md shows
   generic extends used through methods and generic bounds (`[B: Bag[Str]]`),
   never passed as a plain interface-typed parameter. Also guessed that
   `[T: Sized]` works when `Sized` is imported by name into the extending
   file. 60%. (c04)
10. **Which message form does `extend [T]` + `extend [Int]` from two modules
    take?** Two forms exist in examples/errors: "`X` overlaps `Y`" (for
    `[T]` vs `[[T]]`) and "`[T]` is already a `one.Bag[T]`" (identical
    targets, examples/errors/travel). Guessed "overlaps", naming the second
    extend first, with `help: both would apply to `[Int]``. 55%. (c05)
11. **Where is the cross-module clash reported?** modules.md: "at the
    import that brings the second in"; extend_overlap_modules.expected
    agrees. But errors/travel reports at the extend itself (there the second
    file imports the first). Guessed: main.lume:23:1, the `import
    kinds.int_list` line, with paths relative to the directory run from.
    60%. (c05)
12. **Does a bare variant work as an argument when the parameter's type
    comes from a function of another module (`pricing.of_kind(Tool)`, the
    parameter being `item.Kind`)?** modules.md says bare works wherever the
    type is known from a parameter; this is one step further. 90%. (c02)
13. **Can a `pub struct`'s field be a block type naming an imported type
    (`applies: (Item) -> Bool`)?** Fields holding blocks and imported types
    are each documented, not together. 90%. (c02)

### Concurrency (7)

14. **What does main print, and where, when `await bad?` fails out of an
    `async def main -> () or Error`?** concurrency.md: "stops with the
    message on standard error and exit status 1". The shape of that message
    is not shown anywhere. Guessed `error: refused ftp://x: not https` on
    standard error; standard output up to that point is kept. 40%. (c06)
15. **Is it `pub async def`, in that order?** Not shown; every async def in
    the docs is in the file that uses it. Also guessed an async def is
    awaited through a prefix as `await fetcher.fetch(..)`. 80%. (c06)
16. **Does a kept block that mentions a `shared var`, copied into many
    tasks, still change the one value?** functions-and-blocks.md: a kept
    block "may go to another task" and a `shared var` "is the one thing it
    changes where it lives"; the two are not shown together. Guessed yes.
    75%. (c07, c08)
17. **Can a struct with a block field be copied into a `spawn:` and its
    block called there (`j.run`)?** concurrency.md: "Anything can be copied
    in"; functions-and-blocks.md: a kept block is an `Arc<dyn Fn>` that may
    go to another task. 80%. (c07)
18. **Can that kept block be passed as a block argument to a plain `def`
    from inside a task (`replay(owner, amounts, note)`)?** Not shown. 70%.
    (c08)
19. **Is `await Time.sleep(0)` fine?** Time.sleep takes "a whole number of
    milliseconds"; zero not mentioned. 90%. (c07)
20. **Is a task whose indented body ends in a tuple literal a
    `Task[(Str, Int)]` that can be pushed into a list typed that way?**
    port/app_async.lume has `Task[(Int, Str)]` from an async def, not from a
    body ending in `(a, b)`. 90%. (c07)

### New features: blocks as values and type functions (10)

21. **Can a type function keep a block it is given in the value it builds
    (`Job.of(.., step)`)?** "A function may keep the blocks it is given" —
    said of functions, not type functions. 85%. (c07)
22. **Does a block type with a list-of-tuples parameter parse,
    `([(Str, Int)]) -> Int`, and may the kept block take the pairs apart
    with `rs.map { |n, v| v }`?** Not shown. 70%. (c07)
23. **Is `{ |i| true }` (a block ignoring its parameter) fine as a kept
    block?** Not shown; no unused-name rule is documented. 90%. (c02)
24. **Does a struct with two block fields print
    `Rule(name: "cap", applies: <block>, adjust: <block>)`?** Documented for
    one field. 90%. (c02)
25. **Can a block returned from a module function be called at once through
    the prefix, `pricing.percent_off(50)(999)`, `fmt.twice(text.shout)("hi")`?**
    Documented for local functions (`adder(1)(5)`). 85%. (c02, c03)
26. **Is an *enum's* type function handed over as a block,
    `.map(Grade.from_score)`?** Documented for a struct's (`Temp.from_f`).
    90%. (c09)
27. **Is a method's returned block using fields refused with the kept-block
    message, at the `{` (c10_limits.lume:24:34)?** block_kept_self shows a
    `do` block in a no-arg method; mine is a `{ }` after `=`. Guessed same
    message, same help, column of the `{`. 75%. (c10)
28. **Is `<` on tuples accepted?** methods.md "Tuples": "`<` and `>` between
    two tuples are refused"; examples/ordering.lume uses `a < b` on tuples
    and its .expected prints `true`. The brief names tuple ordering as new,
    so I took methods.md to be stale. 80%. (c09) — the docs contradict each
    other here.
29. **Can `?` sit inside a tuple expression in a match arm,
    `(who, amount.to_int?)`?** `?` inside a constructor call is shown
    (modules example `User(.., parse_role(role.trim)?)`); inside a tuple not.
    85%. (c08)
30. **Is a result type written `(Str, Int) or Error`?** Tuples and `T or
    Error` are each documented; the combination is not. 85%. (c08)

### Other (4)

31. **Does `"-700".to_int` give -700?** to_int "reads a whole number"; a sign
    not mentioned. 85%. (c08)
32. **Can a `match` arm be an assignment, `Ok(updated) -> acct = updated`?**
    Arms that push are shown; an assignment arm is not. 80%. (c08)
33. **Do `(1999.to_float / 100.0).decimals(2)` and `(5 / 100.0)` round to
    "19.99" and "0.05"?** printing.md documents half-away rounding of
    `decimals`, not binary float noise; I relied on the nearest double being
    within rounding. 90%. (c02)
34. **How does a FAIL program's header show "no output"?** The brief only
    says what a blank line is. I wrote `# EXPECTED OUTPUT: none; it does not
    compile.` 70%. (c05, c10)

Things I could check and so did not count: import paths start from
main.lume's directory, `import a.b` together with `import a.b.T`, qualified
and bare variants, extends travelling up imports and interface defaults
coming along, `await` on a list keeping start order, reading `tasks.len`
before the `await`, `for` over a `shared var` list, a `shared var` method
under one lock, `?` belonging to the `await`, writing through a missing map
key, keyword arguments on a type function, and `f(x)(y)`.

## Verdict

The docs carried me most of the way: modules.md and concurrency.md now
answer the questions that used to be guesses (where paths start, `()` through
a prefix, what `await t?` means, what a task copies). The guesses left
are nearly all about **combinations**. Each new feature is documented alone,
but not where it meets the module system or tasks: a type function through
a module prefix used as a value (with the modules.md sentence "always a call,
never a function value" pulling the other way), a module's function kept in
a binding, a kept block with a `shared var` running on many tasks, and a
generic extend's list passed as an interface value. Three things need
fixing in the docs: the stale "tuples refuse `<`" line in methods.md, which
contradicts ordering.lume; the missing text of the message a failing
`async def main -> () or Error` prints; and a sentence on which of the two
overlap messages ("overlaps" or "is already a") you get, and where it is
reported, when the extends live in different modules.

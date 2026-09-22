# m30 review round — reviewer R

Ten multi-file programs, written from `README.md` and the permitted examples
only (`examples/*.lume`, `examples/modules/`, `examples/travel/`). The
compiler, the tests and the rest of the corpus were not read, and nothing
here was run. Every number, column width and sort order was worked out by
hand and then recomputed independently.

My brief was **real programs laid out across files the way a person would
actually lay them out** — not module-feature probes. So the file split in
each program is the one the task wants (a format layer, a domain layer, an
algorithm, a renderer), and the features fall where they fall.

## 1. What each program probes

| # | Program | Files | Runs? | What it is for |
| --- | --- | --- | --- | --- |
| r01 | `r01_expenses` | main, money, expense, parse, report | runs | An expense tracker. A `Money` struct with `+`/`<` crossing a module boundary; a `pub enum` with a method used from two other files; `?` through two module hops; reading a `pub struct`'s field from an importing file; a method (`Money.show`) called in a file that never imports that module. Two report tables, one padded, one not. |
| r02 | `r02_testsummary` | main, cases, parse, suite | runs | A test-result summariser. A `pub enum` and a `pub struct` in one module with the struct delegating to the enum; a list pattern with a leading literal and a rest binding (`["fail", ..rest]`) in an imported module; `group_by` over an imported struct type; per-line errors reported instead of aborting. |
| r03 | `r03_recipe` | main, units, recipe, format | runs | A recipe scaler. An enum and a struct from one module used together across a boundary and rebuilt inside a third; a method returning a fresh value of its own type; a block inside a method reading a field through a local binding; integer-only arithmetic so no Float is ever printed. |
| r04 | `r04_deps` | main, manifest, graph, order | runs | A dependency resolver (Kahn). A struct holding `{Str: [Str]}` crossing a boundary with its methods called from a third file; `next` inside a `for` in an imported module; a `{Str}` built in one module and consumed in another; a block closing over two surrounding names inside a `while`; `T or Error` matched twice in main, once for a real cycle. |
| r05 | `r05_changelog` | main, commit, parse, group, render | runs | A changelog builder. An optional field (`scope: Str?`) on a `pub struct`, matched in its own method and built elsewhere; `{Int: [Str]}` grown with `body[r].push(...)` on a missing key, in an imported module; `.or_error(...)?`; string slicing across a boundary; a three-deep import chain. |
| r06 | `r06_forms` | main, rules, field, check | runs | A form validator. A rule is *data* (an enum) because behaviour cannot be stored in a field; an enum variant holding a `{Str}`; a struct field holding `[rules.Rule]`; a method returning `Str?` matched from another file; a block parameter passed across a module boundary and handed straight to the built-in `map`, with `_` shorthand supplied from main. |
| r07 | `r07_seating` | main, seating, guests, plan | runs | **AWKWARD.** A seating planner arranged so the interface, the `extend` for the program's own generic type, the algorithm and the consumer are four different files. Does an `extend` travel transitively? May an `extend` live in a file other than its interface's? Does one bounded generic function take a `Table[T]`, a `[T]` and a `{T}` in the same program? Module-qualified generic type with a module-qualified argument (`guests.Table[guests.Guest]`). |
| r08 | `r08_sheet` | main, expr, sheet, parse | runs | **AWKWARD.** A spreadsheet formula evaluator. A recursive `pub enum` matched from *outside* its module, five module-qualified variant patterns in one exhaustive `match`, two of them binding recursive fields; two methods of one struct in mutual recursion, reached through `?` inside a `match` arm that also runs a `for`. |
| r09 | `r09_units_clash` | main, scale, metric, imperial | **REFUSED** | **AWKWARD, deliberate error.** A unit converter split one file per unit system. Both `metric.lume` (line 5) and `imperial.lume` (line 6) write `extend Int with Scaled[Str]`. The README promises "two `extend`s claiming the same type and the same interface anywhere in one program is an error naming both files" — this checks that it fires across files, names *both*, says which type and which parameterised interface, and does not instead surface as a duplicate method, an ambiguous call in `main.lume`, or a rustc conflicting-`impl`. |
| r10 | `r10_gitlog` | main, record, reader, stats, table | runs | A git-log analyser. One aggregation function used three times by handing it a different key block across a module boundary, with `_` supplied as a field twice and a method once; three parallel `{Str: Int}` maps; column widths computed in a file that sees the data only through one imported struct; `?` unwinding out of a `for` in an imported module. |

Awkward: r07, r08, r09. Deliberate error: r09.

## 2. Ambiguous, surprising or under-specified in the README

Ordered roughly by how much of my writing it affected.

1. **`pad` and `pad_right` — which side does each pad?** The README lists
   both under `Str` (`"...`pad`, `pad_right`, `reverse`...") and never says
   which adds spaces where. My only evidence is
   `examples/collections.lume`'s `"#{w.pad_right(6)} ..."`, which produces a
   left-aligned column, so I took `pad_right` = pad on the right and `pad` =
   pad on the left. Every column in r01, r03, r08 and r10 depends on this.
   I hedged by using only `pad_right` in r03 and r08 and both in r01 and r10,
   so one wrong guess does not invalidate all four. Nothing says what the
   fill character is either, or what happens when the string is already
   wider than the width (I assumed: unchanged, not truncated).

2. **Are a `pub struct`'s *fields* readable from an importing file?** The
   README says "only `pub` items cross a file boundary". A field is not a
   top-level item and there is no `pub` on fields anywhere. `examples/modules`
   shows the *constructor* used across a boundary
   (`model.User("Ed", "ed@x.com", model.Role.Guest(7))`) and *methods* called
   across one (`store.count`, `store.find`), but never a bare field read.
   Six of my ten programs read a field across a module boundary
   (`x.amount.cents`, `r.total.cents`, `i.qty.show`, `g.names`, `s.order`,
   `t.number`, `c.added`). If that is not allowed the layout I wrote is not
   writable at all, since the alternative is an accessor method per field.
   The same question applies to *writing* a field across a boundary
   (`p.x = v`), which I avoided everywhere.

3. **Does an `extend` travel transitively?** Milestone 29 says "An `extend`
   travels with the import: a library ships its conformances, and a consumer
   that imports it writes none of its own". Every example of this
   (`examples/travel/`, `generic_extend.lume`) is one hop, interface and
   `extend` in the same file. Unanswered: if A imports B and B imports C,
   does A get C's `extend`s? r07 leans on this — `main.lume` gets
   `extend Table[T] with Seating[T]` only through `import guests`, and the
   interface itself reaches `guests.lume` as a single imported *name*
   (`import seating.Seating`), not as a module.

4. **May an `extend` live in a file other than its interface's?** Every
   example writes `interface X` and `extend T with X` in one file. r07 splits
   them. Related and unstated: when the interface arrives via `import seating`
   (module form) rather than `import seating.Seating` (name form), must the
   target be written `extend Table[T] with seating.Seating[T]`? I used the
   name form to sidestep it, which is itself a guess about whether
   `import a.b.Name` works for an `interface` and not just a struct/enum/def.

5. **When does a lazy chain need `.to_list`?** The README says chains "are
   lazy and compile to one fused Rust iterator" and lists `sort`, `sort_by`,
   `uniq`, `group_by`, `partition`, `flat_map` and `join` among the methods —
   but a sort cannot be lazy, so some of these must already materialise.
   The examples do both: `travel/shapes.lume` writes
   `entries.map { |x| "#{x}" }.to_list.join(" + ")` and
   `collections.lume` writes `idx[w].or({}).to_list.sort.join(", ")`, while
   `blocks.lume` writes `nums.sort.take(3)` and `users.sort_by(_.age).reverse.map(_.name)`
   with no `to_list` at all, and `stdlib.lume` writes `xs.uniq` bare. I could
   not derive a rule, so I inserted `.to_list` before every `join`, `sort`,
   `first`, `max` and `sum` that follows a block method. If a redundant
   `.to_list` is an error ("a method `[T]` does not have"), several programs
   fail on a line that is about nothing.

6. **Multi-line calls.** No example anywhere spreads a call's arguments over
   more than one line — `examples/collections.lume` shows a multi-line *list
   literal*, which is not the same thing. Since the language is
   indentation-significant and `|>` is called out specially as the one thing
   that continues an expression across lines, I assumed a call may not be
   broken. That cost me: `r01/parse.lume` and `r02/suite.lume` both have
   intermediate bindings that exist only to keep a constructor on one line,
   and several lines are longer than I would normally write.

7. **What does `split` do at the ends of a string, and what does the
   no-separator form drop?** `"a  b   c".split.to_list` is shown, so runs of
   spaces collapse. Unstated: `"util:".split(":")` — two fields or one?
   `"".split` — `[]` or `[""]`? `"a;b;".split(";")` — trailing empty kept?
   This changed a line of r04's output, so `manifest.read` now rejects empty
   dependency names explicitly rather than trusting the answer.

8. **What do `max_by` and `min_by` give back?** They are listed among the
   chain methods, but the sentence that pins down optionality names only
   "`first`, `last`, `find`, `max`, `min`, `pop`". I wanted
   `runs.max_by { |c| c.ms }` in r02 and wrote
   `runs.sort_by { |c| 0 - c.ms }.to_list.first` instead. The same doubt
   applies to `fold`'s result on an empty list and to `group_by`'s key order
   (I sort the keys rather than trust it).

9. **Sort stability is never mentioned.** Not for `sort`, not for `sort_by`,
   not for `sort` on a type with only `<`. I removed every tie from every
   sort key in all ten programs so that no expected line depends on it. That
   is a real constraint on writing reports, which are mostly ties.

10. **`Str` ordering.** `sort` "follows from `<`", but `<` on `Str` is never
    defined. r10's area table puts `(root)` before `compiler`, which needs
    code-point order (`(` is 0x28). If `<` on `Str` is locale- or
    case-insensitive, that line is wrong.

11. **Descending sort, and sorting on more than one key.** There is no
    `sort_by` with a direction and no comparator form. `enums.lume` uses
    `sort_by(_ * -1)`, which is the trick I copied (`0 - r.total.cents`) —
    but it only works when the key is a number. Sorting a report by
    "section rank, then name" would need `sort_by` to take a tuple, and
    nothing says a tuple is `Ordered`. I designed around it.

12. **No way to store behaviour.** "behaviour cannot be stored in a field or
    returned yet" is stated plainly, and it is the single biggest thing that
    changed a design: r06's validator wants `Rule(message, test)` with a
    predicate in a field, and had to become an enum plus an interpreter
    method. Worth recording as the gap a real program hits first.

13. **No number formatting.** There is `x.decimals(n)` for Floats and `pad`
    for widths, but nothing zero-pads an integer. r01 writes
    `if rest < 10: "#{whole}.0#{rest}"` by hand to print cents. There is also
    no format string of any kind, so every table in this round is built from
    `pad`/`pad_right` and interpolation.

14. **Does `sum` follow from `+` the way `sort`/`max`/`min` follow from `<`?**
    The README says explicitly that `!=` follows from `==` and that
    `<=`, `>`, `>=`, `sort`, `max`, `min` follow from `<`. It says nothing
    about `sum` and a user type with `def +`. r01 wanted
    `good.map(_.amount).to_list.sum`; I summed cents and used `max` (which
    *is* documented to follow from `<`) for the largest-expense line.

15. **Iterating a map in a `for`.** Only `m.each { |k, v| ... }` is shown.
    `for k, v in m` is never shown, and `m.to_list` is listed without saying
    it gives `[(K, V)]`. I used `m.keys.sort` plus `m[k].or(default)`
    everywhere, which forces an `.or` at every read even where the key is
    certainly present — five of my programs contain `.or(0)` or `.or([])`
    that exist only because of this.

16. **`xs[i].field.method(...)`.** The README documents `xs[i].method(...)`
    and `xs[i].field = v` as the forms that reach the item itself, and warns
    that "a change that would land on a temporary copy" is an error. It does
    not say which side of that line `tables[i].seats.push(g)` falls on. I
    restructured `r07/plan.lume` to build each list before constructing the
    item, which is not how I would have written it.

17. **Empty literals in a constructor call.** "an empty `[]` binding with no
    type" is an error, and `var out: [U] = []` is the documented fix. Is a
    constructor's declared field type "the type the result is going into"?
    I did not want to find out inside a should-run program, so
    `r08/sheet.lume`'s `empty` binds `[]` and `{}` to typed `var`s first.
    `examples/sets.lume` does write `Graph(edges: {}, seen: {})`, which
    suggests it is fine — but that is keyword form, and mine would be
    positional.

18. **List patterns composed with literals and with an empty tail.**
    `[]`, `[x]`, `[first, ..rest]` are documented, and literal patterns are
    documented, but `["fail", ..rest]` (r02) is never shown. Nor is it stated
    whether `[hash, ..rest]` matches a one-element list with `rest == []`
    (r05 depends on it) — `enums.lume` carefully writes `[only]` before
    `[first, ..]`, which could be read either way.

19. **Qualified variant patterns as a group.** `app.lume` shows
    `model.Role.Guest(d)` as one arm beside a `_`. r08 writes a whole
    exhaustive `match` of five module-qualified variants with no wildcard,
    and I do not know what the exhaustiveness checker names as a missing
    witness in that case — `Add(_, _)` or `expr.Expr.Add(_, _)`.

20. **`next` in a `for`.** The README mentions `next` only obliquely, in the
    list of "spellings other languages use (`continue`, ...) each with the
    Lume form". It is never shown; `chars.lume` uses `next` inside a `while`.
    r04 and r10 use it inside a `for`.

21. **`return` inside a `match` arm.** The warnings list says "`return`
    inside a block" is warned about. Is a `match` arm a block for that
    purpose? `r08/parse.lume` and `r01/parse.lume` both `return Error(...)`
    from inside a `match` arm, which is the natural way to write a parser.
    If that warns, a lot of ordinary code warns.

22. **`Str.digit?` on the empty string** is unspecified (`"123"` and `"12a"`
    are shown). r06 is arranged so the one field with a `Digits` rule is
    never empty.

23. **Method visibility.** There is no `pub` on a method anywhere, and
    `examples/modules/users/store.lume`'s methods are reachable from
    `app.lume`, so methods evidently travel with a `pub` type. But the README
    never says so, and it does say `pub def` is a milestone-24 addition for
    *top-level* functions — which implies methods are a separate rule that is
    written down nowhere.

24. **Importing a module only for its side effects.** If a file imports a
    library purely to receive its `extend`s and never writes the module name,
    is that a dead import? Nothing says whether unused imports warn, error or
    pass. This matters directly for the "ship your conformances" story. It
    also bit me in `r10/main.lume`, where `record` would otherwise be
    imported and never named: I wrote `cs: [record.Change] = reader.changes(log)?`
    — a typed immutable binding, in the shape `examples/modules/app.lume`
    uses for `s2: Store = st.empty` — purely so the import is used.

25. **Re-export.** If `parse.lume` imports `commit`, does a file that imports
    `parse` see `parse.commit` or `commit`? I assumed not, and every file
    imports what it names itself (r05's `main.lume` imports `commit` even
    though `parse`, `group` and `render` all already do). Never stated.

26. **Module names colliding with ordinary names.** r04 has a module
    `order.lume`; I renamed a `Graph` field from `order` to `names` because I
    could not tell whether a field, a local binding or a parameter may share
    a name with an imported module. (r08 keeps a field called `order` in a
    program with no such module.) Also unstated: whether a module may share a
    name with a type, as `r07`'s `seating`/`Seating` nearly does.

27. **Calling a method on a value whose module is not imported.** `r01`'s
    `main.lume` calls `.show` on a `money.Money` reached through
    `report.Row`, without `import money`. The README says imports bring names
    "in expressions, types and patterns"; a method call is none of those, so
    I assumed no import is needed. If it is needed, the error will be about
    the wrong thing.

28. **Two imports of the same module in different forms.**
    `examples/modules/app.lume` does `import users.store.Store` and
    `import users.store as st`, so it is evidently legal. r07 does
    `import seating` in one file and `import seating.Seating` in another,
    and r09 does `import scale.Scaled` in two files. Fine, presumably, but
    it is worked out from an example rather than stated.

29. **How anything prints.** This is the biggest thing I had to design
    around rather than guess. The README never shows the text form of a
    struct, an enum, a list, a set, a map, an optional or a Float — it says
    printing is "derived" and leaves it there. So **none of my ten programs
    prints any of those directly**: every line is `Str` or `Int`, built with
    interpolation, `join`, or a hand-written formatter. `examples/modules/app.expected`
    is the one place I could see the answer for a `[Str]` (`admins: [Ana]` —
    no quotes, no spaces after `[`), and I still avoided depending on it.
    The cost is visible: the expense tracker keeps whole cents, and the
    recipe scaler keeps tenths, purely so that no Float is ever rendered.

30. **Integer division.** Truncation towards zero is never stated (only that
    `10 / 0` on literals is caught, and that overflow stops the program).
    r03's sugar row (`750 / 4 = 187`, `250 / 4 = 62`) is the one place my
    expected output depends on it; every value there is positive, so only
    "truncate, don't round" matters, not the direction for negatives.

31. **Precedence.** There is no precedence table. `done.contains?(d) or not g.known?(d)`
    (r04) and `amount * num / den` (r03) both assume the ordinary reading.
    `**` is listed with no associativity.

32. **`#` immediately before `#{`.** r05 prints Markdown, so it writes
    `"# #{version}"` and `"## #{s.heading} (#{s.lines.len})"`. Obviously `#`
    should not start a comment inside a string literal, but the README's
    lexer notes mention only "INDENT/DEDENT, string interpolation", so I am
    flagging it rather than assuming.

33. **A set as an enum variant's payload.** r06's `OneOf(allowed: {Str})`.
    The README says which *values* may be set items and map keys, and that
    struct/enum fields may be `shared var`, but never that a `{T}` may be a
    variant's field.

34. **An interface default with a block parameter.** I wanted
    `def roll(show: (T) -> Str) -> Str` as a default on `interface Seating[T]`
    in r07, so every conforming container would get it. Nothing says an
    interface method may be typed `(A) -> B`, and blocks are described
    entirely in terms of *functions* and *methods*, so I backed off and made
    it a bounded `pub def` instead. This is the one thing I reached for and
    could not tell whether it exists.

35. **Small README inconsistencies.** The milestone table runs 1–18
    ascending then 29, 28, 27, 26, 25, 24, ..., 18r descending, so the
    newest rules are in the middle of the table; "Status: milestones 1–29
    done" sits above a table where 19 is eleven rows below 29. `18` appears
    as two different milestones (`18` = sets and slicing, `18r` = the third
    review round). The corpus paragraph says "223 programs from five review
    rounds" and then lists six directories (`corpus/`, `corpus/edge/`,
    `m17`, `m18`, `m26`, `m28`). None of this affects a program, but all of
    it cost me time working out which rules are current.

36. **`xs.at(i)` out of range.** Documented as "the item when the position is
    already known to be good", with no word on what happens when it is not —
    presumably the same stop-the-program as `xs[i].method`. `r08/parse.lume`
    guards with `empty?` and an explicit `i + 1 >= parts.len` check.

37. **What a *near-miss* `extend` reports.** The error list has both "an
    `extend` that leaves a method out" and "a value used as an interface it
    does not satisfy (naming the missing method or the signature that
    differs)". For r09 the interesting failure mode is the compiler reporting
    one of *those* instead of the duplicate-claim error — e.g. complaining
    about `unit_name` being defined twice on `Int`, or about the call site in
    `main.lume` being ambiguous. The message naming both files is the whole
    point of milestone 29, so it is the thing to check.

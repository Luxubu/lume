# m30 review notes — module mechanics

Ten multi-file programs, written from `README.md`, `examples/modules/` and
`examples/travel/` only. The compiler was not read and was not run: every
`# EXPECTED OUTPUT:` block was worked out by hand.

Eight programs are meant to run. Three are meant to be refused (p02, p07, p09)
— one more than the brief asked for, because the collision cases in p09 turned
out to have no documented answer at all and a refusal is the only outcome I can
argue for.

## 1. What each program probes

| # | Files | Probes |
| --- | --- | --- |
| `p01_import_forms` | 3 | Every import form in one file: whole module; `import a.b.Name`; the same module under its plain name *and* an alias; the same module under **two** aliases; a `pub` **constant** imported by name; and `import a.b.name as x` — `as` renaming a single name rather than a module, which the README never shows. Runs. |
| `p02_private_refused` | 2 | **Deliberate error.** Five private items reached from outside one module: a function, a constant, a struct, an enum and an interface. Checks that each message says *private* and points at the declaration in `ledger.lume`, rather than "unknown name" with a did-you-mean. Also checks that a private function still works *inside* its own file (`Entry.total` calls it), so the `pub` parts are not collateral damage. |
| `p03_deep_nesting` | 4 | Three-segment paths (`store.csv.rows`) and a straight three-hop chain `main → store.report → store.csv.reader → store.csv.rows`. The deepest module is imported from two different depths; `store/report.lume` binds `[Row]` (imported by name) to a value typed `[rows.Row]` (imported as a module) to check they are one type. Runs. |
| `p04_diamond` | 4 | A diamond: `core/money.lume` imported by `billing/invoice.lume` (as `money`), by `billing/receipt.lume` (as `m`), and by `main.lume` (as `cash`) — one module, three names, loaded once. A `Money` produced by the left arm is handed to the right arm with no conversion, and `due.text` is called in `main.lume` on a type `main.lume` never spells. Runs. |
| `p05_qualified_everywhere` | 3 | A qualified name in every position I could find one: parameter type, return type, type in a binding, struct field, enum variant in an expression, enum variant in a `match` pattern, constant, keyword constructor, generic **bound** (`[S: shape.Sized]`), and an interface default (`c.blurb`) inherited structurally by an imported type. Runs. |
| `p06_no_reexport` | 3 | The absence of re-export. `table` is built on `seq`; a consumer that wants `seq`'s helpers must `import seq` itself, and `table` can only offer hand-written forwarders. The header spells out the two spellings I expect to be refused (`table.seq.tally`, bare `tally`) and why the real cost is `seq` silently becoming part of `table`'s public contract. Also uses a **qualified function name as a block** (`xs.map(seq.shout)`). Runs. |
| `p07_cycle` | 3 | **Deliberate error.** `graph/node.lume` ↔ `graph/edge.lume`, the shape a person writes by accident the first time they split a graph across files. Checks the cycle error names both files and both import lines, does not degrade into "unknown type `edge.Edge`", does not leak a rustc error, and — the point — ends with the fix, since the error message is the only documentation this rule has. |
| `p08_consts_tests_ops` | 3 | `pub` top-level constants of three types (Int, Str, `{Str}`) across a boundary; a private constant reachable only through a `pub` function; hand-written operator methods `+ - * <` called from another file; and the four operators the README says *follow* (`!=` from the derived `==`, and `>`, `sort`, `max` from `<`) which are written nowhere — so this asks whether a **derived** operator survives an import. Plus a `test` block in an imported module and another in `main.lume`. Runs. |
| `p09_collisions` | 4 | **Deliberate error.** Three collisions in one file: two modules whose last path segment is both `config`, so the bare name is claimed twice; a module named `str`, one case-fold away from the built-in `Str`; and a local binding named after an imported module. Each has an "if accepted" line in the header so the finding is recorded either way. |
| `p10_extend_two_hops` | 4 | How far an `extend` travels. `kitchen/scale.lume` ships `interface Weighed` plus `extend Int` and `extend [Int]`; `pantry` imports it (one hop), `recipe` imports `pantry` (two hops), and `main.lume` imports `recipe` and `pantry` but **never `scale`** — then calls `.kilos_text` on a plain `Int`. The README says an extend "travels with the import" and never says whether it keeps travelling. Runs, betting on the program-wide answer. |

## 2. Ambiguous, surprising or under-specified in the README

Numbered roughly by how much they cost me while writing.

1. **`as` has no stated grammar.** The whole specification of renaming is four
   words: "`import users.model.User` for one name; `as` to rename". It is not
   said whether `as` attaches only to a module path (`import a.b as x`), which
   is the only form `examples/modules/app.lume` shows, or also to a single
   imported name (`import a.b.Name as X`). I used the second form in
   `p01_import_forms` because a language that offers `import a.b.Name` and
   `as` in the same breath ought to compose them, but it is a guess. If it
   does not exist there is no way at all to resolve a clash between two names
   imported from two modules — you must fall back to module-qualifying one of
   them, which is a silent downgrade in readability for a problem `as` exists
   to solve.

2. **Importing a *constant* by name is nowhere shown.** The modules paragraph
   says "`import users.model.User` for one name" and only ever renames a type.
   The constants paragraph says constants are "computed once, `pub` to share
   them with other files" and shows no import at all. So
   `import geo.units.METRES_PER_MILE` (p01) and `import core.money.CURRENCY`
   (p04) are both guesses. The same doubt covers importing a bare `pub def` by
   name; `examples/modules/app.lume` imports `Store`, a type, and never a
   function. I used `import geo.units.miles_to_metres as to_metres` anyway.

3. **There is no re-export, and nothing says so.** I went looking for
   `pub import`, `pub use`, `export`, `import a.b.* `, `import a.b.{X, Y}` —
   none of them appear. The consequence is structural and the README never
   mentions it: when `table` is built on `seq`, a consumer of `table` has to
   `import seq` too, which makes `seq` part of `table`'s public contract
   without a single line in either file saying so, and means swapping `table`'s
   internals for a different helper library breaks every consumer even though
   `table`'s own surface never changed. `p06_no_reexport` is built entirely
   around this. It is the single biggest thing I would add a paragraph for.

4. **How far an `extend` travels is unanswerable from the text.** Two
   sentences in the same paragraph, both quoted exactly: "An `extend` travels
   with the import, so a library ships its conformances and a consumer that
   imports it writes none of its own" and "it has no name, so there is no
   `pub extend`". Together they leave the transitive case — A holds the
   extend, B imports A, C imports B — with no answer *and no syntax to force
   either answer*. If conformances are import-local, a wrapper library is
   stuck: it cannot re-export the extend (no name) and cannot re-export the
   module (item 3). If they are program-wide, then a method appears on `Int`
   in a file with no visible cause, and adding an import three files away
   changes what a call here means. `p10_extend_two_hops` bets on program-wide.
   Either way the README needs one sentence.

5. **Two modules whose paths end in the same segment.** `import app.config`
   and `import db.config` in one file both want the bare name `config`.
   Nothing in the README says whether that is an error, whether the last one
   wins, or whether the bare name simply becomes unusable. `as` is the obvious
   fix and is never connected to this problem. `p09_collisions` probes it. A
   silent last-wins would be the worst outcome, because which `config` you got
   would depend on the order of two adjacent lines and be invisible at every
   call site.

6. **Module names and local names: same namespace or not?** The README is firm
   that Lume has no shadowing ("nested rebinding is an error", "rebinding an
   outer name inside a loop or branch ... the error says to declare it `var`"),
   and separately that "a user type of the same name wins over any of these
   namespaces" for `File`/`Env`/`Path`. Neither statement covers a *module*
   name. `config = "local"` after `import app.config` (p09) is therefore
   undefined: an error, a shadow, or a rebinding error with a confusing
   message about a thing that is not a binding.

7. **A module named after a built-in.** `str.lume`, `path.lume`, `env.lume`,
   `file.lume` are all plausible file names. The README forbids "a user type
   named after a built-in (`struct Option`)" — a *type*, capital-letter, and a
   module is neither. I assumed `import str` is fine (p09), but if it is not,
   the message must say that case matters, because "`str` is already a type"
   would be false; the type is `Str`.

8. **A module name colliding with a name inside it.** I *avoided* this rather
   than probe it, which is itself worth reporting: the natural spelling of
   p04 was `billing/invoice.lume` with a `pub def invoice(...)` in it, giving
   `invoice.invoice(...)` at the call site, exactly as a person would write it.
   I renamed the function to `build` because I could not guess whether the
   resolver reads `invoice.invoice` as module-then-function or trips over
   itself. Somebody should probe it deliberately.

9. **`import rust.<crate>` shares a keyword with `import a.b`.** Nothing says
   `rust` is a reserved first path segment. A project with a `rust/` directory
   containing `rust/util.lume` would write `import rust.util` and get — what?
   A crate lookup for a crate called `util`, presumably, with a confusing
   error. Not stated anywhere.

10. **Import paths appear to be program-absolute, never relative.** In
    `examples/modules/`, `users/store.lume` writes `import users.model`, not
    `import model` — so a module names its own sibling through the full path
    from the entry file's directory. That is a real and unremarked cost:
    a subtree cannot be moved or renamed without editing every file inside it,
    and there is no `import .sibling` / `import ..parent` form. I followed the
    absolute convention everywhere (p03 has `store/csv/reader.lume` writing
    `import store.csv.rows`), but the README states the rule only as
    "`import users.model` loads `users/model.lume`", which is ambiguous about
    what the path is relative to when the importing file is not the entry file.

11. **The entry file has no described status.** `examples/modules/` uses
    `app.lume`, `examples/travel/` uses `main.lume`. Is the entry file a module
    like any other — can another file `import main` and reach a `pub` item in
    it? p08 puts a `test` block in `main.lume` alongside imports, which assumes
    the entry file is at least ordinary enough to hold one.

12. **The word "private" never appears.** The rule is "only `pub` items cross a
    file boundary", stated once. There is no vocabulary for the error, and the
    long list of error kinds the README enumerates — which is otherwise very
    complete, down to "`name (` with a space (ambiguous call)" — does not
    include a privacy violation at all. That is why `p02_private_refused`
    exists: a reader who knows the item is right there in the other file needs
    to be told it is not `pub`, not that it is unknown.

13. **Field visibility is not discussed.** There is no `pub` on a field
    anywhere, and `examples/modules/app.lume` reads `guest.role` from outside
    the declaring module, so every field of a `pub struct` is public. That is a
    defensible design, but it means `pub struct` is an all-or-nothing decision
    and a module cannot keep an invariant; it deserves a sentence rather than
    being inferred from one example line.

14. **Enum variants of an imported enum.** `pub enum Role` plus
    `model.Role.Guest(7)` is shown, so variants ride along with the enum's
    `pub`. But the README also allows "bare `Circle(1.0)` where unambiguous" —
    is `Guest(7)` bare available in an importing file? `app.lume` always
    qualifies, including in patterns (`model.Role.Guest(d) ->`). I qualified
    everywhere in p05 and never tested the bare form; if it works, the rule for
    when a name is "unambiguous" across modules needs writing down.

15. **A qualified name as a block argument.** "a function name works as the
    block for all of these: `xs.map(parse)`, `xs.group_by(kind)`" — both
    examples are bare. `xs.map(seq.shout)` (p06) is a guess. I avoided it in
    p03 and used `{ |l| rows.row(l) }` there instead, so p03 does not depend on
    it.

16. **A qualified interface name as a bound.** `[T: Ordered]`, `[T:
    Comparable[T]]`, "any interface name for its methods", "`pub interface`
    crosses modules" — all four statements exist, but no example writes
    `[S: shape.Sized]`. p05 does. If dotted bounds do not parse, a `pub
    interface` is of very limited use across a boundary.

17. **A typed top-level constant.** The only example is `MAX = 100`, untyped.
    Whether `BEST: tags.Tag = tags.Tag.Ranked(9)` is allowed is not stated. I
    wrote the untyped form in p05 rather than risk the program on it — so this
    is a gap nothing in the corpus covers.

18. **Constant initialisation order across modules.** Constants are "computed
    once". If `a.lume` has `pub LIMIT = 10` and `b.lume` has
    `pub DOUBLE = a.LIMIT * 2`, is that legal, and in what order do the two
    files' constants run? p08's `SECRET`/`secret_doubled` stays inside one file
    precisely because I could not answer this.

19. **`lume test` on a multi-file program.** "`test "name":` blocks with
    `assert`; `lume test` runs them" and the CLI shows `lume test
    examples/tests/parse.lume` — one file. Does `lume test main.lume` run the
    tests in the modules `main.lume` imports, or only its own? Both are
    reasonable and a library author's habits depend entirely on which. p08 has
    a test in `vec/vec2.lume` and another in `main.lume` so the answer shows up
    either way.

20. **The cycle rule has no documented fix.** Milestone 9's row says "cycle
    detection" and that is the entire treatment. But a cycle is not always a
    mistake: two mutually referring types (`Node` holds `[Edge]`, `Edge` holds
    `Node`) are the normal shape of a graph, and the only remedy is "put both
    types in one file", which appears nowhere. p07 is built on that shape
    deliberately. If the compiler refuses it, the error message is carrying the
    whole documentation burden for this rule and should say the fix out loud.

21. **Errors inside imported modules.** The error paragraph is long and
    detailed but is written entirely in terms of one file. Nothing promises
    that an error in `users/model.lume` is reported with *that* file's path and
    line rather than the entry file's, which is the first thing a multi-file
    user needs to trust.

22. **`Int.pad` is listed with no semantics.** "on `Int`/`Float` ... `pad`" —
    pad with what, to what, and is the argument a width? `Str` has `pad` and
    `pad_right`, which suggests spaces and alignment, but zero-padding
    `"#{(cents % 100).pad(2)}"` is the obvious use for money. I could not pin
    the output so I rewrote p04 around `.decimals(2)`, which *is* pinned
    ("halves away from zero"). Any program using `Int.pad` is guessing.

23. **How a list renders under `puts`.** `examples/modules/app.expected` has
    `admins: [Ana]` — square brackets, no quotes around the `Str`. The
    separator for two or more elements is not shown anywhere I am allowed to
    look, so I used `.join(...)` on every list in all ten programs rather than
    guess. Worth one line in the README next to "`==` and printing derived".

24. **Interface defaults on an imported type, discovered in the importing
    file.** "a type conforms by having the methods, with nothing to declare,
    and then has the defaults as its own methods (`q.describe`)". When the
    interface and the type both come from module `kit.shape` and the *call*
    `c.blurb` happens in `main.lume` (p05), the conformance has to be visible
    across the boundary. It surely is, but this is the same question as item 4
    wearing different clothes, and the two are never connected in the text.

25. **Calling an inherited default from inside the conforming type.**
    `examples/travel/shapes.lume` calls a sibling default bare (`def bare? ->
    Bool = total == 0`) but that is inside the *interface*. Inside a `struct`
    that conforms structurally, is the default reachable as a bare name, or
    only as `self.kilos_text`? I wrote `self.` in `p10/kitchen/pantry.lume` to
    be safe, so the bare form is untested.

26. **`import a.b` and `import a.b as x` together.** `app.lume` pairs
    `import users.store.Store` with `import users.store as st` — a name plus an
    alias. It never binds the plain module name *and* an alias for the same
    module, which p01 does. Harmless if it works; if it is an error, that is a
    surprising restriction given the file it would most resemble is in
    `examples/`.

27. **Nested list types.** `[[Str]]` (p06's `Table.rows`) is implied by `[T]`
    but never written in the README. Same for a qualified type inside a
    container, `[pantry.Item]` and `[rows.Row]` — `examples/modules/store.lume`
    does have `users: [model.User]`, so that one is evidenced; the nested case
    is not.

28. **`lume fmt` and imports.** "one canonical layout, no options; keeps
    comments, blank lines". Does it sort or group `import` lines? If it sorts
    them, a program like p09 whose behaviour might depend on import order would
    have its meaning changed by the formatter. Unstated, and the combination of
    items 5 and 28 is the reason it matters.

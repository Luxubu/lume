# Reviewer C — whole programs, modules, concurrency (round seven)

Everything below was written from `docs/` (not `QUESTIONS.md`) and `examples/`
only. Nothing was compiled or run; every expected line was worked out by
hand. The padded lines were checked with a few lines of Python that apply
`pad`/`pad_right` as `printing.md` defines them.

## 1. The programs

| # | files | lines | kind | what it probes |
|---|---|---|---|---|
| c01 | `c01_ledger/` main, accounts, money | 106 | modules, RUN | `import m` + `import m.Type` together; `pub struct`/`pub enum`/`pub def`/`pub` constant (`money.CENTS`); `Entry.Deposit(...)` after importing the enum by name; a `var self` method returning `() or Error` that falls off its end; `?` on failures from two modules down; list patterns on `split` |
| c02 | `c02_inventory/` main, catalog/items, catalog/stock | 109 | modules, RUN | modules in a subdirectory; `as` alias; a module importing **only** one item from a sibling (`import catalog.items.Item`); a `pub interface` with a default used as a type from another file (`[items.Priced]`); keyword construction through a module prefix; a no-argument module function called with `()` |
| c03 | `c03_pipeline/` main, report, stats, textutil | 99 | modules, RUN | `pub import stats` and `pub import textutil as tx` re-exported through `report`, so `main` imports only `report`; a private helper called from a method of a `pub struct`; `T?` across modules; `[]` as a constructor argument |
| c04 | `c04_deps/` main, manifest, graph | 110 | modules, RUN | a dependency resolver (Kahn's algorithm); item-only import; top-level `"""` constants; `for a, b in` a list of tuples; a `() or Error` helper matched by the caller |
| c05 | `c05_crawler.lume` | 61 | concurrency, RUN | `async def` + `Time.sleep`; `spawn:` with a two-line body capturing a loop variable; `[Task[Page]]` awaited as a list, inside a `while` that spawns every time round; `shared var` counter |
| c06 | `c06_bank/` main, ledger | 86 | modules **and** concurrency, RUN | a cross-module `pub struct` held in `shared var` and changed through its `var self` method from tasks; two shared vars in one task; a module constant used inside `spawn:`; a task-local `var`; walking a map field of a shared struct afterwards |
| c07 | `c07_jobs.lume` | 69 | concurrency, RUN | an `async def -> Str or Error` using `?` and `return Error`; `Task[Str or Error]`; a spawn body that awaits, bumps a shared var, then yields; results paired back with `zip` (spawn order); list patterns with string literals |
| c08 | `c08_stackvm.lume` | 120 | interpreter, RUN | a stack machine: ten-variant enum, exhaustive `match` in a `while` in a `var self` method, `self.take()?`, `A \| B \| C` arms with an inner `match`, trailing `if` on assignment, a method's `T or Error` used as a scrutinee then the struct read |
| c09 | `c09_grades.lume` | 90 | report generator, RUN | `group_by` (block and `_`), `avg` on a chain, an enum as a map key, `pad`/`pad_right`/`decimals`/`repeat`, `max_by`, map → tuples → `sort.reverse`, nested string literal in a block in an interpolation |
| c10 | `c10_fares/` main, fares, zones | 68 | modules, **FAIL** | calling a non-`pub` function through the module prefix; also `.or_error(...)?` in a module, `import zones.Zone` then bare variant patterns across the boundary |

Totals: 10 programs, 6 multi-file (c01, c02, c03, c04, c06, c10), 3 concurrent
(c05, c06, c07), 9 meant to run, 1 meant to fail.

For c05, c06 and c07 the EXPECTED OUTPUT is standard output only. The
`shared var` lock warnings go to standard error (that is a guess, #14).

---

## 2. GUESSES

**40 guesses**: [modules] 12 · [concurrency] 13 · [other] 15

### Modules

1. **Does a module's no-argument function need `()` when called through the
   prefix?** Looked in: `functions-and-blocks.md`, which says *"A function
   call always carries `()`, even with no arguments — a bare name is a value,
   not a call"*, and `examples/modules/app.lume`, which writes
   `var store = st.empty` and `s2: Store = st.empty` with no `()`. The two
   disagree. I guessed that `st.empty()` (declared `def empty() -> Shelf`)
   works. **medium** [modules] (c02)

2. **Can a file import one item from a module without importing the module
   itself?** Looked in: every running example imports both (`app.lume`,
   `shipping/main.lume`, `lib/report.lume`, `mini/eval.lume`). Only
   `errors/missing_item.lume` has a lone `import mods.secret.gret`, and it is
   an error case. Guessed yes: `stock.lume` and `graph.lume` have only
   `import catalog.items.Item` / `import manifest.Package`. **medium** [modules]
   (c02, c04)

3. **Is an import that is never used as a prefix an error or a warning?**
   c01's main has `import accounts` beside `import accounts.Account` and
   `import accounts.Entry` and never writes `accounts.`. c03's main imports
   `report` only so that `report`'s `pub import`s reach it. Looked in: all of
   docs, `examples/errors/` (there is `import_twice`, no "unused import").
   Guessed that it is accepted and says nothing. **medium** [modules]

4. **After `import accounts.Entry`, is a variant written `Entry.Deposit(...)`?**
   Looked in: `structs-and-enums.md` ("Bare variant names and qualified
   ones", single file only), `modules/app.lume` (`model.Role.Guest(7)`, with
   the module imported and not the enum), `mini/eval.lume`. Guessed that
   `Entry.Deposit` works once `Entry` is imported by name. I did not dare the
   bare `Deposit(...)` in a value position. **medium** [modules] (c01)

5. **Do bare variant *patterns* work on an enum from another module?**
   `fares.lume` does `import zones.Zone`, then `match z: Central -> … Outer(r) -> …`.
   Looked in: `mini/eval.lume` (bare `Num(v)` after `import parser.Expr`) and
   `mini/FINDINGS.md` ("patterns already resolve by scrutinee type"). The
   example suggests yes but never says whether importing the enum is what
   makes it work. **medium-high** [modules] (c10)

6. **With `pub import textutil as tx`, does the importer see it as `tx`?**
   Looked in: `shipping/` (only a plain `pub import units`) and
   `errors/reexport/` (`pub import a as core`, but only as the failing
   case). No running example has an aliased re-export. Guessed yes.
   **medium** [modules] (c03)

7. **Are the methods of a `pub enum` callable from an importer?**
   `interfaces.md` says *"A method of a `pub struct` needs no `pub` of its
   own… The same goes for a `pub interface`"*. Enums are not mentioned.
   Guessed `e.fee?` works from main. **high** [modules] (c01)

8. **Can a struct be built by keyword through a module prefix,
   `items.Item(sku: "P1", …)`?** `app.lume` only shows the positional
   `model.User("Ed", …)`. Guessed yes. **high** [modules] (c02)

9. **If an imported module's name matches a field or local in the same file,
   which one does a bare name mean?** `stock.lume` first had a field called
   `items` and also imported the module `catalog.items`. Nothing says which
   wins inside a method. I **avoided** it by renaming the field to `known`.
   **low** [modules]

10. **What is a root-level module called in error messages?** For
    `c10_fares/fares.lume` next to `main.lume`, I guessed `fares`, based on
    `mods.secret` for `mods/secret.lume` in `errors/private_access.expected`.
    **medium-high** [modules] (c10)

11. **Is calling a private function with arguments,
    `fares.peak_multiplier(true)`, the same error as reading `secret.hidden`?**
    The only example is a bare name with no call. Guessed the same wording:
    `` `peak_multiplier` exists in module `fares` but is not `pub` ``.
    **high** [modules] (c10)

12. **Must `import` lines come first, before constants and definitions?**
    Nothing in docs mentions imports at all. I copied the examples and put
    imports after the header comment and before everything else, without
    knowing whether it is required. **high** [modules]

### Concurrency

13. **What may a `spawn:` body contain: a `for`, a nested `match`, a local
    `var`?** `async.lume` shows one-line and two-line bodies, and `match/`
    shows a `match` with indented arms. I used `var done = 0`, a `for` and
    two nested `match`es in one body (c06). `errors/spawn_changes_copy`
    forbids changing an *outer* `var`. I guessed that a `var` declared inside
    the body is the task's own. **medium** [concurrency]

14. **Are the `shared var` warnings standard error, and are they part of "the
    output"?** Looked in: `async.expected`, `match/main.expected` and
    `port/app_async.expected` all begin with the warning text, mixed in with
    the program's output. `install.md` and the docs say nothing about it.
    (`errors.md` does say the `!` warning goes to stderr.) Guessed stderr,
    and left the warnings out of EXPECTED OUTPUT. **medium** [concurrency]

15. **What does `Time.sleep` take: milliseconds? an `Int`?** `Time` appears
    in no doc page. `async.lume` passes `30 - id * 5`, which looks like ms.
    It does not affect my output. **medium** [concurrency]

16. **Can a `shared var` hold a struct from another module and have its
    `var self` method, which returns `() or Error`, called and matched from
    inside a task?** `errors/shared_readonly` implies that `shared var c` +
    `c.bump` is fine, and `app_async.lume` calls `store.insert` through a
    `shared var` field. No example matches a failing method under the lock.
    **medium-high** [concurrency] (c06)

17. **After the tasks are done, can I walk a map *field* of a `shared var`
    struct (`for name, cents in book.balances`) and call its methods
    (`book.total`)?** `match/FINDINGS.md` says a `shared var` map can be
    walked. `app_async.lume` reads `store.users.len`. A field walked by `for`
    is neither of those. **medium-high** [concurrency] (c06)

18. **Is a top-level constant usable inside `spawn:`?** The rule is that
    "locals it mentions are copied in". Constants are not locals. I used
    `BATCHES.enumerate` outside the spawn and `batch` inside it. **high**
    [concurrency] (c06)

19. **Do `?` and an early `return Error(...)` work inside an `async def …
    -> T or Error`?** No example does either inside an `async def`. **high**
    [concurrency] (c07)

20. **Can `?` be used inside a `spawn:` body?** `functions-and-blocks.md`
    says `?` in a block leaves the *block*, and only when the block's type can
    carry it. Is `spawn:` a block for that rule? **Avoided**: c07 binds
    `r = await run(spec)` and gives `r` back whole. **low** [concurrency]

21. **Is anything locked across an `await`?** In c07 a task awaits and then
    does `finished += 1`. I assumed the lock is taken and released per use,
    as the warning says ("every use of it takes a lock"), so an `await` in
    the same task never holds it. **high** [concurrency]

22. **Can `spawn:`/`await` go inside a `while` body, with
    `var tasks: [Task[T]]` declared fresh each time round?** No example does
    this. **high** [concurrency] (c05)

23. **How is a `shared var` handed to a helper function?** A `shared var` is
    a local of `main`. Can a parameter be `shared var`? `app_async.lume`
    uses a struct field typed `shared var Store`. **Avoided**: every task
    body in my programs is written inline in `main`. **low** [concurrency]

24. **What may cross into or out of a task?** Can a task return a struct that
    holds a list (`Task[Page]`), or be given one? Nothing states a rule (Rust
    would need `Send + 'static`). Guessed anything that can be copied.
    **high** [concurrency] (c05)

25. **Does a `shared var` warning appear even when the program is fine, once
    per declaration?** From the `.expected` files it seems so. That only
    matters if #14 is wrong. **medium** [concurrency]

### Other

26. **Does a function or method declared `-> () or Error` that runs off the
    end return success?** `errors.md` only ever writes `return ()`
    explicitly. `match/main.lume` and `modules/app.lume` have `main`s that
    end in `puts`, but I found none for an ordinary function or method.
    c01 `apply`, c04 `install`, c06 `apply` and c08 `run` all rely on this.
    **medium-high** [other]

27. **How tightly does postfix `?` bind?** It is not in the precedence table
    in `operators.md`, and neither are `!` and `await`. Would
    `whole.to_int? * CENTS` work? **Avoided** by binding first
    (`w = whole.to_int?`). **medium** [other]

28. **How does a method call a sibling method that takes no arguments?** Bare
    `take` could be read as a field. Is it `take()`, `self.take`, or
    `self.take()`? And is `self.take?` the method `take?` or `take` + `?`
    (see Confusing #5)? I wrote `self.take()?`. **medium** [other] (c08)

29. **Is `split` a list or a lazy chain?** `strings.md` prints it as a list,
    but `stdlib.lume` and `match/records.lume` write `.split.to_list`, and
    `modules/model.lume` matches `line.split(",")` directly. I matched on it
    directly in c01, c04 and c07, and used `.to_list` before passing it
    on or matching it in c06, c08 and c09. **medium** [other]

30. **Does `avg` work straight on a lazy chain?** `collections.md` lists
    `join, sum, len, sort, first, max, min, count, any?, all? and fold`, and
    the tour adds "and the rest". `avg` appears only in `stdlib.lume`, on a
    list. c09 does `rows.map(_.points).avg`. **medium** [other]

31. **Can a list pattern contain a string literal:
    `["push", n]`, `[src, "->", dst, amount]`?** `patterns.lume` has list
    patterns (bindings only) and string patterns (not inside a list). I
    guessed the combination works. **medium-high** [other] (c06, c07, c08)

32. **When a map value is overwritten, does the key keep its original
    position?** `collections.md` says "keeps the order you put things in"
    but does not cover rewriting a key. c06 prints balances after many
    rewrites. **high** [other]

33. **Do tuples compare and sort lexicographically?** c09 sorts
    `[(Int, Str)]`. `app_async.lume` sorts `(Int, Str)` without saying how.
    **high** [other]

34. **Does `_` shorthand work on `T?`'s `map` (`known[sku].map(_.cents)`)?**
    `errors.md` shows only `{ |n| … }`. **high** [other] (c02)

35. **Do `min_by`/`max_by` work on a list of interface values
    (`[items.Priced]`)?** **high** [other] (c02)

36. **Can a string literal go inside `#{}` inside a block inside another
    `#{}`, as in `"…#{ranked.map { |t| "#{t.1}=#{t.0}" }.join(", ")}"`?**
    `lib/report.lume` nests a `", "` one level deep, but I found nothing
    deeper. **medium** [other] (c09, c10)

37. **Top-level constants: which values are allowed?** Constants are not
    described anywhere in `docs/`. From `mini/lexer.lume` (sets),
    `lib/report.lume` (`"""` strings) and `match/main.lume` (Int, Str), I
    guessed that a list of lists (c06 `BATCHES`) is fine too. **high**
    [other]

38. **May a match arm be an assignment on one line, `Ok(_) -> done += 1`?**
    `control-flow.md` shows `-> puts …` and `-> ()` inline, and assignments
    only in indented arms. **Avoided** with the indented form. **low**
    [other]

39. **Does `sort` return a real list that has `.at` and `remove_at`?**
    `collections.md` says `sort` "ends the chain" but not what it gives back.
    c03 does `s = xs.sort; s.at(n / 2)`, and c04 does
    `ready = ready.sort; ready.remove_at(0)`. **high** [other]

40. **Which names are reserved?** I used `all`, `it`, `left`, `src`, `dst`
    and `known` as bindings or parameters, and avoided `from`. No page lists
    the keywords. `lib/FINDINGS.md` lists some (`where`, `next`, `match`,
    `in`, `test`, `import`, `pub`, `extend`, `interface`, `assert`) in
    passing. **high** [other]

---

## 3. ANSWERED — expected to guess, found clearly

These were all answered, but **most of the module and concurrency answers
came from `examples/errors/*.expected` help texts or `FINDINGS.md` files,
not from any doc page.**

- **Where module paths start from.** `errors/missing_module.expected`:
  *"module paths are relative to the directory of the file that holds
  `main`"*. `users/store.lume` confirms it by importing `users.model`, not
  `model`. That answered c02's `catalog/stock.lume` importing
  `catalog.items.Item`.
- **`import a.b` binds `b`.** Shown by `import users.model` → `model.User`.
- **Aliasing, and importing a module and one of its items together.**
  `app.lume` has `import users.store.Store` and `import users.store as st`.
- **`pub` fields and variants.** `errors/pub_field` and `errors/pub_variant`:
  *"a field is public with its struct, so it takes no `pub`"*. Fields of a
  `pub struct` are readable from importers.
- **`pub def` on a method.** Accepted and redundant (`interfaces.md`).
- **`pub` constants.** `pub REVIEW = 50` in `match/scoring.lume`, used as
  `scoring.REVIEW`, and bare inside its own file.
- **`pub import` passes a module on**, and the file that declares it can use
  it too (`shipping/parcels.lume`).
- **Private helpers.** Methods of a pub struct can call private free
  functions of their module (`model.lume`'s `describe_role`).
- **The private-access error**, word for word
  (`errors/private_access.expected`).
- **Extends travel with the import** (`travel/`). I did not need this.
- **`async`/`await` rules.** `await` only in `async def` or `spawn:`;
  calling an `async def` without `await` is an error; `await` uses a task
  up; `spawn:` copies locals in; changing an outer `var` inside `spawn:` is
  an error that names `shared var`; `shared` without `var` is read-only;
  `spawn:` inside a method cannot touch `self`. All come from the error
  catalogue.
- **`await` on a list of tasks gives results in spawn order.**
  `async.lume` shows it: `fetch(i)` sleeps `30 - id*5`, so the later ids
  finish first, yet the output is `["item 1", …, "item 4"]`.
  `match/FINDINGS.md` says it outright. That is what makes c07's
  `specs.zip(results)` safe.
- **A spawn body's value** is its last expression, `Task[T]` (`async.lume`).
  `Task[X or Error]` carries a failure home (`match/`).
- **`for a, b in list_of_tuples`.** Only from `port/app_async.lume`
  (`for status, text in (await tasks).sort`). No doc shows it.
- **List methods missing from the reference.** `remove_at`, `insert`,
  `group_by` (in first-appearance order), `flat_map`, `zip`, `avg` (an
  `[Int]` gives a `Float`) and `uniq` all come from `stdlib.lume` and its
  `.expected`, not `collections.md`.
- **Well answered by the reference pages:** `pad`/`pad_right`/`decimals`
  (including "pad fills with spaces, never zeros", which is why c01 has
  `two_digits`); `split(sep)` vs bare `split`; `lines` on `"""`
  (indentation stripped, no trailing newline); `repeat(0)`; map insertion
  order; writing through a missing key starting empty (`users[d].push(…)`);
  `.or([])` on a known map; `[]` fine as a constructor argument; implied
  `Ok`; `Error(e) -> e`; `.or_error(msg)?`; `return Error(...)` vs a stray
  `Error`; multi-line `if` as a value; `elif`; trailing `if` on an
  assignment and on `next`; `()` as a do-nothing arm; `..` inclusive; code
  point ordering; enums as map keys (`Hashable`); qualified vs bare variant
  names in one file; `self.x` accepted beside the bare field; `var self`.
  These pages are good: I did not have to guess at a single printing or
  string detail.

---

## 4. CONFUSING OR WRONG

1. **`st.empty` vs "a function call always carries `()`."**
   `examples/modules/app.lume` calls a no-argument module function with no
   parens, twice. `functions-and-blocks.md` says in bold that this is an
   error (*"`greet` is a function; call it with `greet()`"*). Either
   module-prefixed calls are an undocumented exception, or `st.empty` is a
   value, and the reader cannot tell which.
2. **One file, three names in `interfaces.md`'s `pub` section.** The code
   block is headed `# users/model.lume`, the prose says an importer calls
   `shapes.Sq(...)`, and the quoted error says *"module `lib.shapes`"*.
3. **`docs/README.md` promises "every method" for collections**, but
   `collections.md` has no complete list. `remove_at`, `insert`, `group_by`,
   `partition`, `flat_map`, `zip`, `uniq`, `avg`, `find`, `sort_by`,
   `enumerate`, `each`, `to_set`, `last`, `take`/`skip` are all used in
   examples or other pages. `strings.md` has the compiler's own "`Str` has:"
   list; lists, maps and sets have nothing like it.
4. **The tour never mentions `import`, `pub`, top-level constants or
   `async`.** It ends in "Going further" as if the language stopped at
   interfaces. The only doc sentence about modules is the `pub` section in
   `interfaces.md`.
5. **`?` after a method name is ambiguous to the eye.** `errors.md` writes
   `first = xs.first?` (`first` + try), while `empty?`, `some?`, `ok?` and
   `admin?` are method names that end in `?`. `_.admin?` and `xs.first?`
   look the same. No page says how the parser tells them apart, or what
   `x.empty??` would mean.
6. **The precedence table leaves out postfix `?`, `!` and prefix `await`.**
   `await (1..5).map { … }.to_list` in `async.lume` only works if `await`
   binds looser than `.`. `match/FINDINGS.md` notes `puts (await t)?` is
   refused. None of this is in `operators.md`.
7. **`.expected` files mix stderr warnings into the output** (`async`,
   `match`, `port/app_async`) with no note saying which stream they are on.
   A reader copying the format would put the warnings in their own expected
   output.
8. **`Time`, `File`, `Env` and `Dir` are undocumented.** They appear only in
   examples and in one error help text (*"Dir has exists?, make, list, walk
   and remove"*).
9. **List patterns (`[x, ..]`, `[first, ..rest]`, no `[.., last]`) and match
   guards (`Wrap(A(n)) if n > 5`) have no reference section.**
   `control-flow.md`'s `match` section and `structs-and-enums.md` cover
   neither. They are only in `examples/patterns.lume` and FINDINGS files.
10. **`errors.md`'s own caveat undercuts its rule.** A stray `Error(...)` in
    a `-> () or Error` function *"is currently accepted and silently
    discarded"*. That is exactly the kind of function I wrote four times. It
    is honest, but it means the compiler will not catch that mistake in the
    most common case.
11. **User variants named `Ok` in `port/app_async.lume`.** `enum Response`
    has an `Ok(body:)` variant, built as `Response.Ok(...)` but matched bare
    as `Ok(_)` in `status`. It works by the scrutinee-type rule, but it is a
    confusing thing to put in an example without a comment, given that
    `Ok`/`Error` are otherwise the built-in result.

---

## 5. Verdict

**For single-file programs, yes.** The reference pages answer the question
a reviewer actually has, often word for word, and I made no guesses about
printing, strings, padding, `split`, ranges, `if`-as-value or error
handling. Of my 15 [other] guesses, most are small (precedence of `?`,
whether `avg` ends a chain, lexicographic tuples) and would be caught at
compile time rather than giving a wrong answer.

**For modules and concurrency, only by reverse-engineering the examples, and
it cost 25 of my 40 guesses** (12 modules, 13 concurrency), although those
areas were perhaps a third of my code. What I learned came mostly from the
*error catalogue* (the help text in `missing_module.expected` is the only
statement of how import paths work) and from `FINDINGS.md` files (the only
statement that `await` keeps spawn order). A new user would not know to look
there. The examples also contradict the docs once (#1 above), which makes
everything else in them less trustworthy as a teacher.

The two areas cost differently. **Module guesses are mostly syntax**:
item-only imports, `Enum.Variant` after an import, aliased re-exports, `()`
on module calls. If I guessed wrong, the compiler says so with a good
message. **Concurrency guesses are about semantics**: what a task may
contain, when the lock is held, what crosses into a task, which stream the
warnings are on, whether `?` works in `spawn:`. A wrong guess there can
compile and still behave differently from what the writer expected.

**The single change that would help most: a concurrency reference page**
(`async def`, `spawn:`, `await` on one task and on a list, spawn-order
results, `Task[T]`, `shared` vs `shared var`, what a lock covers and for
how long, what may be captured, the warning on stderr), written in the same
question-first style as `errors.md`. A modules page is a close second and
could be half a page: path rules, the four `import` forms, `pub` on
everything, `pub import [as]`, and how variants of an imported enum are
written.

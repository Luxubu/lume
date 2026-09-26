# Reviewer C — round nine notes

Focus: whole programs, most across several modules, and concurrency (`?`
inside `spawn:`, `await t?`, `async def main -> () or Error`, async and
block-taking interface methods across files, kept blocks crossing modules
and going into tasks, a generic interface method called through a bound in
another file, a program that stops on a failed task).

## What I read

- `corpus/m45/BRIEF.md`.
- `docs/tour.md`; `docs/reference/concurrency.md`, `modules.md`,
  `interfaces.md`, `errors.md`, `functions-and-blocks.md`, `generics.md`,
  `names.md`, `collections.md`, `printing.md` (most of it); parts of
  `methods.md` (lists, Int, Float, tuples), `structs-and-enums.md` (`def
  self`, fields vs interfaces, tuple destructuring), `control-flow.md`
  (`if`, `while`, `next`/`break`), `patterns.md` (arm bodies),
  `operators.md` (unary minus), `io.md` and `numbers.md` (how a failing
  program reports).
- `examples/async.lume`, `examples/interface_methods.lume`,
  `examples/kept_blocks.lume`, `examples/errors_and_results.lume` with their
  `.expected`; `examples/match/` (all three files, `.expected`,
  `FINDINGS.md`); `examples/travel/`; `examples/modules/` (app, blocks,
  users/*); `examples/lib/` (README, seq, table, report);
  `examples/port/app_async.lume`; from `examples/errors/`: `iface_async_default`,
  `iface_generic_through_pointer`, `iface_pass_held_value`, `task_try_mixed`,
  `spawn_uses_self`, `spawn_changes_copy`, `block_kept_self`,
  `try_in_plain_fn`, `private_access`.
- Not read: the generated `examples/*/.lume/` build folders (compiler output),
  anything in compiler/, tests/, README.md, HISTORY.md, docs/QUESTIONS.md, or
  the other reviewers' files in corpus/m45/.

## The programs

| path | what it is | RUN/FAIL | leans on |
|---|---|---|---|
| `c01_feeds/` (main + feeds/source, kinds, collect) | a `pub interface Source` with an `async def fetch -> Str or Error`; structs in another file; a generic `pull[S: Source]` in a third file awaits it; four tasks awaited as a list, matched; tasks using `?` twice (succeeds) and once (fails, printed as `Error(...)`); the interface held in a list | RUN | async interface methods through a bound; `?` on `await s.fetch()`; `Task[Str or Error]`; start-order results |
| `c02_pipeline/` (main + flow/steps, stage) | kept blocks made in one module, kept in a struct field of a type from another, in a map returned from a module, in a tuple; a struct full of blocks copied into three tasks; a block taken out of a map inside a task | RUN | blocks as values; blocks crossing into tasks; calling a returned block at once; printing a map of blocks |
| `c03_ledger/` (main + bank/account, rules) | `async def main -> () or Error`; `shared var` of a struct from another module whose `var self` method returns `() or Error`; tasks using `?` on a module function and on that method; `await check?`; `puts r?` over awaited results; a failed task matched; one printed | RUN | `?` inside `spawn:`, `await t?`, tuple destructuring from `?` in a task, `shared var` struct methods |
| `c04_failstop/` (main + jobs/parse, report) | three tasks started together; the first two call an `async def` that uses `?`, the third uses `?` itself; the second batch has a bad line, and `await second?` stops the program | RUN, exit 1 | stderr `error: \`cold\` is not an integer`; failure waits in the task until awaited |
| `c05_walkers/` (main + walk/api, impls, tools) | interfaces with a block-taking method (+ defaults) and a generic method in one file, implemented in another (structs and `extend`s of `[Int]`/`[T]`), generic helpers with bounds in a third; extends travelling to main | RUN | block-taking interface methods across files; generic method through a bound on a struct and on an extended list; held interfaces |
| `c06_held_generic/` (main + shelf/api, store) | the generic interface method is fine through a bound, then called on an item of a struct's `[Mappable[Int]]` field | FAIL | "type parameters of its own … held in a list, a field or a binding", across modules |
| `c07_optional_tasks.lume` | `?` on optionals in `spawn:` (task gives `T?`), `return None` in a task, a task that spawns and awaits its own task, fresh task lists in a `while` with a `shared var` counter | RUN | `Task[Int?]`; implied `Some` in a task; nested spawn |
| `c08_workers/` (main + pool/source, jobs, queue) | a worker pool: jobs gathered through a block-taking interface method on held values; three workers pull from one `shared var` queue through helpers in another module with `shared var` parameters; an interface value copied into a task | RUN | `shared var` params across modules; `break` from a `match` arm in `while true` inside a task; nondeterminism kept out of the output |
| `c09_await_try_plain_main.lume` | `await t?` in an `async def main` with no error return type | FAIL | the `?`-has-nowhere-to-go error in the async shape |
| `c10_scores/` (main + stats/top, fetch, checks) | main imports only `stats.top`, which `pub import`s `stats.fetch` and `stats.checks as ck`; `pub async def` awaited through the passed-on prefix inside tasks started from a block; `?` on each result; kept blocks in tuples in a list walked with `for label, ok in …`; a kept block copied into tasks; a task using `?` on two async calls, the second failing | RUN | `pub import` + async across modules; blocks in tuples; `Some((n, s))` |

How the concurrent output stays deterministic: every result is read by
`await` on a list (start order, documented) or on one named task; shared
counters are only changed by `+=` (one lock use each) so the total cannot
depend on interleaving; in c08 which worker gets which job is never
printed, only sums and lengths; in c03 each move touches a different
name, and map keys are sorted before printing.

Stderr warnings (`shared var` declarations, "holds values of different
types behind …") are not part of any EXPECTED OUTPUT. c04 lists its runtime
stderr separately.

## GUESSES — 35 in total

### Modules (7)

1. **Can a bound use an interface imported by name from another module**
   (`import feeds.source.Source` then `[S: Source]`)? Looked: modules.md
   ("An interface works the same way: `[items.Priced]` or `[Priced]`"),
   generics.md (bounds, all single-file). Guess: yes. Confidence: high.
2. **Can a type conform to an interface from a module it never imports**
   (c01 `kinds.lume`, c08 `jobs.lume`)? Looked: interfaces.md ("nothing to
   declare"), no multi-file example of it (travel/lib use `extend`, which
   imports). Guess: yes; conformance is checked where the value is used.
   Confidence: medium-high.
3. **Is a prefixed interface with type arguments a valid element type**,
   `[api.Mappable[Int]]`? Looked: modules.md shows `[items.Priced]` only.
   Guess: yes. Confidence: medium-high.
4. **Does a bound check in main see an `extend` that the generic function's
   own file never imports?** (c05: `tools.labels([7, 8])`, where tools.lume
   does not import impls.lume but main does.) Looked: modules.md "`extend`
   travels with the import… up the chain". Guess: the check is made at the
   call in main, which has the extend. Confidence: medium.
5. **`pub import` of a module in a subfolder** (`pub import stats.fetch`
   inside `stats/top.lume`) arrives in main as `fetch`. Looked: modules.md
   (examples are all top-level files). Guess: yes, by its last path part.
   Confidence: medium-high.
6. **A `pub async def` reached through a passed-on prefix**,
   `await fetch.score(n)`. Looked: concurrency.md ("awaited through the
   prefix like any call"), modules.md (`pub import`). Guess: works.
   Confidence: medium-high.
7. **How the c06 error names an imported interface**: `Mappable[Int]` or
   `shelf.api.Mappable[Int]`. Looked: modules.md ("its types
   (`shop.items.Item`)" are named by path in messages), the error example in
   examples/errors (single file). Guess: probably the plain name, since main
   imported it by name, but could be the path. Confidence: low.

### Concurrency (17)

8. **An `async` interface method may return `Str or Error`**, and the
   structs' `async def fetch -> Str or Error` match it. Looked:
   interfaces.md and interface_methods.lume (only `-> Str`). Guess: yes.
   Confidence: medium.
9. **`await s.fetch()?` vs `await s.fetch?`.** Names may end in `?`, so
   `s.fetch?` could read as a method `fetch?`. Looked: names.md,
   concurrency.md (`await fetch(url)?` has brackets; `s.to_int?` elsewhere
   works). Guess: I wrote `()` to be safe; both probably work. Confidence:
   medium.
10. **An async interface method called through a bound inside an `async def`
    which is itself awaited in `spawn:`** (c01 `pull`). Looked:
    interfaces.md ("called through a parameter typed as the interface or a
    bound"). Guess: works. Confidence: medium-high.
11. **`?` on a `() or Error` as a statement inside `spawn:`**
    (`book.apply("zed", 0 - 1)?`, c03), on a `shared var` struct's method.
    Looked: concurrency.md (the ledger example only `match`es it). Guess:
    works; the task becomes `Str or Error`. Confidence: medium.
12. **Tuple destructuring of a `?` result inside a task**,
    `(who, cents) = rules.parse_move(line)?`. Looked: structs-and-enums.md
    ("on anything that gives a tuple"). Guess: works. Confidence:
    medium-high.
13. **Exact stderr text when `await t?` passes a failure out of `main`.**
    Looked: io.md (`error: <message>`, status 1, no second line),
    errors.md (for `!`: an extra `(set LUME_BACKTRACE=1 …)` line and
    "non-zero status"), concurrency.md ("the message on standard error and
    exit status 1"). Guess: exactly `error: \`cold\` is not an integer`, no
    backtrace hint, exit 1. Confidence: medium.
14. **In a task that uses `?` on an optional, is a bare last value the
    implied `Some`?** (c07 `n * 10`, `a + b`, `halve_even(...)? + 1`.)
    Looked: concurrency.md ("gives back a `T?` the same way"),
    functions-and-blocks.md ("A block whose result is a `T?` has no matching
    implied `Some` — write it out"). Guess: yes for `spawn:` (it is like an
    async block / function, not a block). Confidence: medium.
15. **`return None` inside such a task.** Looked: concurrency.md (`return 5`
    only). Guess: fine; ends the task with `None`. Confidence: medium.
16. **A `spawn:` inside a `spawn:`, awaited inside it.** Looked:
    examples/match/FINDINGS.md ("a task spawned inside a task works"), no
    code shown. Guess: works. Confidence: medium-high.
17. **A struct whose fields hold kept blocks can be copied into tasks**
    (c02 `spawn: plan.apply(n)`), as can a bare kept block and a map of
    them. Looked: functions-and-blocks.md ("it may go to another task"),
    concurrency.md ("no special rule for what may cross"). Guess: works.
    Confidence: medium-high.
18. **An interface value taken out of a `[Source]` can be copied into a
    task** (c08 `s0 = sources.at(0)`, `spawn: gather(s0).len`). Looked:
    concurrency.md ("anything can be copied"), interfaces.md (held values
    are behind a pointer); nothing mentions a held interface crossing into
    a task. Guess: works (cloned box). Confidence: low-medium.
19. **`shared var` parameters in a function of another module**
    (`pub def take(q: shared var [Int]) -> Int? = q.pop`), called with a
    `shared var` from a task in main. Looked: concurrency.md (same-file
    `bump`; "works the same when the struct comes from another module").
    Guess: works. Confidence: medium-high.
20. **`q.pop` on a `shared var [Int]` parameter** is one use of the lock and
    gives `Int?`. Looked: concurrency.md ("A `shared var` list … is changed
    the same way, with methods"). Guess: yes. Confidence: medium-high.
21. **`None -> break` in a `match` inside `while true` inside a task.**
    Looked: patterns.md (arm may be `break`), control-flow.md. Guess: breaks
    the `while`. Confidence: medium-high.
22. **`spawn:` in a two-name block**, `scored.map { |n, s| spawn: best(s) }`,
    capturing a kept block. Looked: concurrency.md (one-name form only).
    Guess: works; `await` on the `.to_list` gives `[Bool]`. Confidence:
    medium.
23. **`Task[T]` of a task whose body is only `await f()` where `f` returns
    `T or Error`** (c04 `first`, c10 `tasks`) is `Task[T or Error]` without
    any `?` in it, and `await first?` unwraps it. Looked: concurrency.md
    (`spawn: half(10)` then `await t?`). Guess: yes. Confidence: high.
24. **The error for `await t?` in an `async def main` with no error type**
    (c09) is the plain-main text, pointing at the `?` (21:14). Looked:
    errors.md, examples/errors/try_in_plain_fn. Guess: same wording, maybe a
    help naming `async def main`. Confidence: medium-low.

### Interfaces (5)

25. **An interface mixing a plain required method, an `async` required
    method, and a default that calls the plain one.** Looked: interfaces.md
    (async methods have no default; defaults may call required ones).
    Guess: allowed. Confidence: medium-high.
26. **A one-line `async def fetch -> Str or Error = text`** (bare `Str`, the
    implied `Ok`). Looked: interface_methods.lume (`async def fetch -> Str =
    text`), errors.md (implied `Ok`). Guess: works. Confidence: medium-high.
27. **A non-generic struct conforms to `Mappable[Int]` with
    `def map_all[V](f: (Int) -> V) -> [V]`** — `Int` where the interface
    says `T`, own name `V` for `U`. Looked: interfaces.md (`Box[T]` uses
    `T`; parameters "matched by position"), generics.md (`Version` works out
    `Comparable[Version]`). Guess: yes. Confidence: medium.
28. **`m.map_all { |n| n * 2 }.sum`** — `U` worked out from the block, then
    a list method on the result. Looked: brief (round-eight fix: "generic
    calls that work `T` out through blocks"). Guess: works. Confidence:
    medium-high.
29. **Where the c06 error points**: the `.` before `map_all` (24:21), as in
    examples/errors/iface_generic_through_pointer; and that the chain
    `s.slots.at(0)` (field then list item) is refused the same way.
    Confidence: medium.

### Other (6)

30. **Printing a map whose values are blocks**:
    `{"double": <block>, "inc": <block>}`. Looked: functions-and-blocks.md
    (`puts add10 #=> <block>`, a struct field prints `<block>`). Guess: same
    inside a map. Confidence: low-medium.
31. **A block in a tuple**: `pair: (Str, (Int) -> Int) = ("half", { |x| x /
    2 })`, `(label, h) = pair`, `h` then callable and sendable to a task.
    Looked: functions-and-blocks.md ("a map or a tuple"), no example.
    Guess: works. Confidence: medium.
32. **A multi-line list literal of `(Str, block)` tuples as a whole
    function body**, block types taken from the declared return type
    `[(Str, (Int) -> Bool)]`. Looked: kept_blocks.lume (multi-line list with
    a typed binding). Guess: works. Confidence: medium.
33. **`for label, ok in ck.rules(50)`** — two names taking apart a
    `(Str, block)` tuple, then `ok(s)` inside another block. Looked:
    methods.md (two names take a pair apart), concurrency.md
    (`for who, cents in moves`). Guess: works. Confidence: medium.
34. **`Some((n, s))` as a pattern** on `max_by` over a list of tuples.
    Looked: patterns.md headers (tuples, nested enums), methods.md
    (`Some(("ann", 3))` prints). Guess: works. Confidence: medium-high.
35. **Multi-line constructor argument** `Plan(stages: [` … `])` with a
    trailing comma, and **unused bindings** (`who` in c03's `bad` task) give
    no error or warning. Looked: lib/report.lume (multi-line call),
    nothing on unused locals. Guess: both fine. Confidence: medium.

## Doc contradictions (or near ones)

1. **Implied `Some` in a task.** functions-and-blocks.md: a block giving
   `T?` has no implied `Some` ("a gap rather than a rule"). concurrency.md:
   `?` on an optional makes a task give `T?` "the same way" as the
   `T or Error` case, where the last line is the implied `Ok`. A `spawn:`
   body is called "a block of statements", so it is unclear which rule it
   follows (guess 14).
2. **How a failing program ends.** errors.md says a `main -> () or Error`
   failure ends with "a non-zero exit status" and shows the `!` failure with
   a `(set LUME_BACKTRACE=1 …)` line; io.md and concurrency.md say exit
   status 1, and io.md shows no backtrace line. Probably two different
   paths (`!` panics, `?` returns), but no page says so.
3. **Quoted error texts are abbreviated.** concurrency.md gives the
   copy-in-`spawn:` error as "`count` inside `spawn:` is a copy" / "declare
   it `shared var count`"; examples/errors/spawn_changes_copy.expected has
   the longer "…is a copy, so changing it here would not be seen outside"
   and a longer help. Not wrong, but a reader predicting exact text has to
   know the doc trims.
4. **`(await t)?` next to `puts`.** examples/match/FINDINGS.md: "`puts (await
   t)?` is rejected for the space before `(`"; concurrency.md: "`(await t)?`
   means the same and is still accepted". Consistent once you know the
   `puts (` rule from printing.md, but read on their own they seem to
   disagree.

## Verdict

The docs got me through ten whole programs with no guess about spelling a
concurrent program: `spawn:`, `await` on a list, `?` inside a task,
`await t?`, `shared var` and its parameters are all stated plainly with
examples, and so are `pub import`, prefixes and where paths start. The
guesses cluster where features combine and no page shows the combination:
an async interface method returning `T or Error`; `?` on optionals in a
task (the implied-`Some` question, where two pages pull opposite ways);
what may be copied into a task when it is a block, a struct of blocks, or
an interface value; how kept blocks print inside a map; and the exact
stderr and error texts once modules put paths into type names. None of
these look like bad design, but each one needs a sentence or an example:
"`spawn:` bodies get the implied `Some`/`Ok` like a function", "a held
interface value can be copied into a task", and a multi-file example with
an interface in one file and its implementations in another.

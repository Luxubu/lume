# Reviewer A — round eight notes

## What I read

In full: `docs/tour.md`, `docs/reference/functions-and-blocks.md`,
`structs-and-enums.md`, `operators.md`, `concurrency.md`, `errors.md`,
`collections.md`, `printing.md`, `methods.md`, `strings.md`, `numbers.md`,
`control-flow.md`, and the first 260 lines of `patterns.md`; the keyword table
in `names.md`; the constants section of `modules.md`; the headings of
`generics.md`, `interfaces.md`, `io.md` and `modules.md`; `docs/examples.md`.

Examples: `kept_blocks.lume`, `type_functions.lume`, `ordering.lume`,
`async.lume`, `blocks_of_your_own.lume` with their `.expected` files, and in
`examples/errors/` every `block_*`, `call_*`, `static_*` program plus
`tuple_order_parts`, `type_call_method`, `value_call` and `x`, with their
expected messages.

I did not read `docs/QUESTIONS.md`, the compiler, `tests/`, other corpus files,
`README.md` or `HISTORY.md`. I ran nothing.

## The programs

| file | what it is | RUN/FAIL | what it leans on |
|---|---|---|---|
| `a01_temperature_log.lume` | reads mixed °F/°C readings, reports | RUN | `def self.from_f/from_c/parse`, failing type function with `?`, `fs.map(Temp.from_f)`, `count(_.hot?)`, `max_by`, `avg` on a chain, `main -> () or Error` |
| `a02_text_tools.lume` | named text-cleaning steps and a command table | RUN | blocks in a binding, a struct field, a list of structs and a **map**; `Some(f) -> f(...)`; `compose`/`wrap` returning blocks; `compose(...)("hi")`; `steps.at(2).run("x")`; printing `<block>` |
| `a03_block_factories.lume` | number transforms built at run time | RUN | `adder(2)(3)`, `scaler(3)(adder(1)(4))`, `twice(adder(10))(1)`, a block keeping a list of blocks, `return { |x| x }`, blocks in tuples `[(Str, (Int) -> Int)]` |
| `a04_signup_rules.lume` | username validation | RUN | type functions building structs with kept blocks (`Rule.min_len(3)`), zero-arg `Rule.no_spaces`, field block called bare in a method, `?` in a loop, `Ok([...])`/`Error(...)` printing |
| `a05_leaderboard.lume` | race ranking | RUN | tuple `<`, `<=`, `>`; a method returning a 3-tuple used as a `sort_by` key; `min` on tuples; `Entry.parse`; trailing `if` on assignment; `pad` columns |
| `a06_async_orders.lume` | pricing orders from a slow source | RUN (ends with `main` failing) | `await load(id)?` in an `async def`, `await price(0)?` in `async def main -> () or Error`, spawn per item and `await` the list, constant map, `.or_error(...)?` |
| `a07_event_bus.lume` | register handlers and fire events | RUN | `var self` method storing a block param in a field, `bus.on("x") { ... }`, `() `-result field block `h.run(p)`, kept blocks changing a `shared var` in plain `main`, a `do` block with no args reading it |
| `a08_inventory.lume` | parse shop inventory lines | RUN | enum type functions (`Category.parse`, `Category.all`), one type's function calling another's, `lines.map(Item.parse)` (a failing type function as a block), `["card","bow"].map(Item.free)`, tuple keys |
| `a09_wallet_formatter.lume` | a method that hands back a formatter | FAIL | kept block in a method reading a field → `a block that is kept cannot use \`self\` or its fields` |
| `a10_calculator_table.lume` | calculator from a map of 2-arg blocks | FAIL | `ops[op](a, b)` on a `T?` → `... is a \`((Int, Int) -> Int)?\`, not a block, so it cannot be called` |

## GUESSES

**Total: 21.**

1. **Does `<` work between two tuples?** Looked: `operators.md` ("Tuples
   compare part by part"), `examples/ordering.lume` (+ `.expected`),
   `examples/errors/tuple_order_parts.lume`, and `methods.md` § Tuples, which
   says the opposite: "`<` and `>` between two tuples are refused, so compare
   them by sorting or with `max`". Guessed: it works (the example is run by
   the test suite, `methods.md` is stale). Confidence: 85%. (a05)
2. **Is a tuple with a `Str` part usable as a `sort_by` key and with `min`?**
   Looked: `methods.md` (`sort_by` shown only with scalar keys), `numbers.md`
   ("a tuple that holds a `Float` cannot be sorted with `sort` yet" — while
   `ordering.lume` runs `[(2.5, "x"), (0.5, "y")].min`). Guessed: an
   `(Int, Int, Str)` key sorts and `min`s fine; I kept Floats out of tuples to
   dodge the contradiction. Confidence: 85%. (a05, a08)
3. **Can a map hold blocks (`{Str: (Str) -> Str}`)?** Looked:
   functions-and-blocks § Blocks as values lists "a binding, a field or a
   list, or handed back from a function" — maps are never mentioned.
   Guessed: yes, a typed map works like a typed list. Confidence: 75%. (a02, a10)
4. **May a map literal span lines with a trailing comma?** Looked: only a
   multi-line *list* literal exists (`kept_blocks.lume`). Guessed: yes.
   Confidence: 80%. (a02, a10)
5. **Can a block bound by `Some(f)` in a `match` arm be called, `f("Lume")`?**
   Looked: functions-and-blocks (calls shown on bindings, fields, `at(0)`),
   `call_optional_block` error. Guessed: yes. Confidence: 85%. (a02)
6. **Can a tuple hold a block — `[(Str, (Int) -> Int)]` — and be taken apart
   by `for name, f in named`?** Looked: nothing shows a function type inside a
   tuple type. Guessed: yes, and the type parses. Confidence: 60%. (a03)
7. **Can `return` hand back a block literal early — `return { |x| x }` — in a
   function declared `-> (Int) -> Int`?** Looked: only `= { ... }` bodies are
   shown. Guessed: yes, the declared result type types the block.
   Confidence: 70%. (a03)
8. **Can a kept block use a captured list of blocks inside an ordinary
   (non-kept) block — `{ |x| fs.fold(x) { |acc, f| f(acc) } }`?** Looked:
   `kept_blocks.lume` has `Pipeline.run` doing this as a *method*, not inside a
   returned block. Guessed: yes. Confidence: 80%. (a03)
9. **May `def self.name -> T =` break onto the next line?** Looked: the
   `= expr` line break is shown for plain `def` only. Guessed: same for
   `def self`. Confidence: 90%. (a04, a09)
10. **May a type function build a *kept* block that captures its parameter
    (`def self.min_len(n) -> Rule = Rule(..., check: { |s| s.len >= n })`)?**
    Looked: the rule "a kept block inside a method cannot use `self`" and the
    `block_kept_self` error; nothing about type functions. Guessed: allowed, as
    there is no `self` to capture. Confidence: 90%. (a04)
11. **Does `.to_list` copy a list of structs that hold blocks, so pushing to
    the copy leaves the original at length 4?** Looked: collections.md
    ("On something that is already a list it is a copy"). Guessed: yes, blocks
    are shared `Arc`s and copy fine. Confidence: 85%. (a04)
12. **Does the `_` shorthand take a method whose name ends in `?` —
    `good.count(_.hot?)`, `s[0].map(_.alpha?)`?** Looked: `_` examples use
    `_.upcase`, `_.len == 2`; `c.map(_.alnum?)` appears in methods.md on a
    list. Guessed: yes, also on a `Char?`. Confidence: 85%. (a01, a04)
13. **Can a `var self` method keep the block it is given by storing it in a
    field (`handlers.push(Handler(event: event, run: f))`), with the caller's
    block using a `shared var`?** Looked: "A function may keep the blocks it is
    given … Lume sees it" — shown only for `compose`/`twice`. Guessed: yes.
    Confidence: 70%. (a07)
14. **Does a kept block that *reads* a `shared var` see changes made after the
    block was made?** Looked: "a `shared var` is the one thing it changes
    where it lives". Guessed: yes, it reads the live value (2, then 3).
    Confidence: 75%. (a07)
15. **Does a trailing `{ ... }` block work after a *method* call that already
    has arguments — `bus.on("login") { |who| ... }`?** Looked: shown for a
    function (`time_it("a") { ... }`) and for a method with only a block
    (`Bag(...).each { ... }`). Guessed: yes. Confidence: 90%. (a07)
16. **Can a type function that returns `T or Error` be handed to `map` —
    `lines.map(Item.parse)` giving a list of results?** Looked: only
    `map(Temp.from_f)` (a plain result) is shown. Guessed: yes.
    Confidence: 80%. (a08)
17. **Does `await f()?` work inside an `async def` other than `main`, whose
    result is `Int or Error`?** Looked: concurrency.md says `await fetch(url)?`
    "waits and then passes a failure on" and shows `await t?` in `main`.
    Guessed: yes. Confidence: 90%. (a06)
18. **When `async def main -> () or Error` fails, does anything more reach
    standard output?** Looked: errors.md and concurrency.md ("stops with the
    message on standard error and exit status 1"). Guessed: stdout just stops;
    stderr shows the message, probably as ``error: `x` is not an integer``
    (format guessed from the `!` example). Confidence: 85% on stdout, 50% on
    the stderr format. (a06)
19. **a09's exact error and position.** Looked:
    `examples/errors/block_kept_self.{lume,expected}` — but that uses a
    zero-argument `do` block; mine is `{ |c| ... }` reading a field in
    interpolation. Guessed: the same message, pointing at the `{`.
    Confidence: 85%. (a09)
20. **a10's exact first line.** Looked: `call_optional_block.expected` names
    `steps.first()` as `` `steps.first` ``. Guessed: an index expression is
    named something like `` `ops[...]` `` — unknown. Confidence: 40% on the
    spelling, 85% on the `((Int, Int) -> Int)?` / "not a block" / "unwrap it
    first" parts. (a10)
21. **Does a10 fail *there* and not earlier, and does `ops.contains?(op)`
    fail to narrow the type?** Looked: nothing on flow typing; function name
    as a map value (`"pow": power`) and the nested `Ok((op, a, b))` pattern
    are only shown for lists/bindings and in patterns.md respectively.
    Guessed: no narrowing, and nothing earlier is refused. Confidence: 70%. (a10)

Questions I steered around rather than guessed (not counted): whether a
local inside a `def self` may share a field's name (I renamed to `pts`,
`secs`, `cat`); tuples holding a `Float` in `sort`/`min`; zero-padding (used
`decimals`).

### Doc inconsistencies found on the way

- `methods.md` § Tuples says `<`/`>` between tuples are refused, while
  `operators.md` and `examples/ordering.lume` show them working.
- `numbers.md` and `methods.md` say a tuple holding a `Float` cannot be
  sorted; `ordering.lume` calls `.min` on `[(2.5, "x"), (0.5, "y")]`, and
  `operators.md` lists what may be in an ordered tuple without Float caveats
  — yet also says every part must be "a number, text, a `Char`, a `Bool`",
  while `ordering.lume` puts a struct with `def <` inside one.
- functions-and-blocks lists where a block can be kept (binding, field, list,
  result) but never a map, a tuple or a `T?`.
- `blocks_of_your_own.lume` uses `xs.sum(weight)` with a block; the methods
  table lists `sum` with no argument.

## Verdict

Yes, mostly. Functions of a type are covered well: the structs-and-enums
section, `type_functions.lume` and the seven `static_*` error programs answer
every question I had about calling, bare-name calls inside, generic
inference and handing one over as a block. Blocks as values are well
explained for the cases the page names (binding, field, list, returned,
`adder(1)(5)`, `compose`, the copy rule and the `self` rule), and the error
catalogue is the best teacher for what is refused. The gaps are at the
edges: containers other than lists (maps, tuples, options) are never shown
holding blocks, and storing a given block from inside a `var self` method is
implied rather than shown. `await f()?` is one sentence in concurrency.md,
enough but easy to miss. Tuple ordering is the weak spot. `methods.md` and
`numbers.md` still say the opposite of `operators.md`, so a new user who
reads the method table first would think `<` on tuples is refused.

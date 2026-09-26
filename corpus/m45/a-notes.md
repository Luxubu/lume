# Reviewer A — round nine notes

## What I read

In full: `corpus/m45/BRIEF.md`; `docs/tour.md`, `docs/examples.md`;
`docs/reference/functions-and-blocks.md`, `concurrency.md`, `interfaces.md`,
`errors.md`, `collections.md`, `printing.md`, `structs-and-enums.md`,
`methods.md`, `strings.md`, `control-flow.md`, `generics.md`, `patterns.md`,
`names.md`, `numbers.md`, `operators.md`; the "Constants" section of
`modules.md`. Examples: `kept_blocks.lume`, `interface_methods.lume`,
`async.lume` (with their `.expected` files), `generic_extend.lume`,
`blocks_of_your_own.lume`. I also grepped `docs/` and `examples/` for a
block kept in a map or a tuple and for `extend Name[Int]` — neither has a
single example anywhere.

Not read: `docs/QUESTIONS.md`, `compiler/`, `tests/`, other `corpus/`
files, `README.md`, `HISTORY.md`. Nothing was run; outputs are worked by
hand (python3 used only to check padding widths and two averages).

## The programs

| file | what it is | RUN/FAIL | leans on |
|---|---|---|---|
| a01_command_table.lume | a calculator that dispatches `add 2 3` lines through a map of blocks | RUN | block in a map, `ops[name].or_error(..)?` on an optional block, block giving `Int or Error` via inline `if`, `(Int, Int) or Error` + `(a, b) = f()?` |
| a02_password_rules.lume | password checker: named rules kept as `(Str, block)` tuples in a struct | RUN | block in a tuple, `def self.min_len` returning a block, `var self` method keeping a block, `pick.1(7)`, printing a tuple with a block |
| a03_config_tasks.lume | reads five config texts concurrently, reports what failed | RUN | `?` inside `spawn:` → `Task[Config or Error]`, constant read in a task, `?` on an optional in a task → `Str?` |
| a04_folder_walker.lume | walks a folder tree, sizes and largest file, via a visitor interface | RUN | interface method taking a 2-arg block, defaults calling it by bare name, recursive struct, `extend {Str: Int}`, `[Walkable]` list |
| a05_event_bus.lume | pub/sub bus: topics → list of handler blocks | RUN | `{Str: [(Str) -> ()]}` field, push through a missing key, `Bus.new`, kept blocks changing a `shared var` list |
| a06_retry_jobs.lume | retrying flaky jobs, one task each | RUN | `async def` interface method with an argument, called through an interface-typed param; `await f(..)?` in `spawn:`; `await [a, b, c]` |
| a07_box_summaries.lume | one generic `Box[T]` summarised differently for ints and words | RUN | `extend Box[Int]` / `extend Box[Str]`, generic method whose `U` comes from a block, `Box.single(7)` |
| a08_grade_report.lume | parse `name: scores` lines, grade under a chosen policy | RUN | failing type function passed to `map`, `?` in a type function, map of `(Float) -> Str` policies, calling a kept block inside another block |
| a09_chat_bot_counter.lume | chat-bot command table where one command counts its calls with a plain `var` | FAIL: `` `calls` was copied into this block when the block was made `` | kept block changing a copied `var` |
| a10_sheet_columns.lume | spreadsheet report looping over `[Sheet]` calling a generic interface method | FAIL: `` `map_rows` has type parameters of its own (`U`), so it cannot be called on a `Sheet` held in a list, a field or a binding `` | generic interface method on a held value |

## GUESSES

**Total: 44.** Confidence is H (would bet on it), M (more likely than not),
L (coin toss or worse).

1. **Can a block be a map value, and how is the map typed?** `{Str: (Int, Int) -> Int or Error}` (a01), `{Str: (Float) -> Str}` (a08), `{Str: (Str) -> Str}` (a09). Looked: functions-and-blocks.md "Blocks as values" says "a map or a tuple" in one sentence; no example anywhere in docs/ or examples/. Guessed it works with the obvious spelling. **M**
2. **Does a multi-line map literal with a trailing comma parse?** (a01, a08, a09). Looked: only a multi-line *list* literal with a trailing comma exists (kept_blocks.lume). Guessed yes. **H**
3. **Is `(Int, Int) -> Int or Error` read as a block giving `Int or Error`** (not `((Int,Int) -> Int) or Error`), also inside `{Str: ...}`? Looked: functions-and-blocks.md `f: (Str) -> Int or Error`. Guessed yes. **H**
4. **Does a two-name block literal assigned to a `(Int, Int) -> ...` type take two arguments** (rather than taking a pair apart)? Looked: `fold`, `fold_all` take `{ |acc, n| }` as a handed block; no kept two-argument block anywhere. Guessed yes. **M**
5. **Can a kept block's literal be an inline `if` whose branches are `Error("...")` and a bare `Int`** (the implied `Ok`)? (a01 `div`). Looked: errors.md shows this for a function's `if/else`; functions-and-blocks.md says a block's bare last value is the implied `Ok`. Guessed yes. **M**
6. **Does `.or_error(msg)?` work on an optional whose value is a block, giving a callable block?** (a01). Looked: methods.md `T?` table (generic `T`). Guessed yes. **M**
7. **Is `(Int, Int) or Error` a valid return type, and does `(a, b) = f()?` destructure after `?`?** (a01). Looked: structs-and-enums.md tuple destructuring (`(x, y) = split(...)`), errors.md. Neither shows a tuple inside `or Error`. Guessed yes. **M**
8. **Can a tuple hold a block, typed `(Str, (Int) -> Int)`, and a list of such tuples be a field?** (a02). Looked: same one sentence as 1; nothing else. Guessed yes. **M**
9. **Is `pick.1(7)` a call of the block in the tuple's second place?** (a02). Looked: functions-and-blocks.md "called straight away by writing the arguments right after it" with `adder(1)(5)`, `steps.at(0)(3)`; nothing with `.0`/`.1`. Guessed yes; the lexer might read `1(` oddly. **L**
10. **Does a tuple holding a block print as `("square", <block>)`?** (a02). Looked: a struct holding one prints `<block>` (functions-and-blocks.md); tuples print like `(1, "two", true)` (printing.md). Guessed by combining. **M**
11. **Is the block half of a tuple, taken apart by a two-name block (`{ \|label, check\| not check(s) }`), callable?** (a02). Looked: methods.md "A block with two names takes a pair apart". Guessed yes. **M**
12. **Can a `var self` method keep a block parameter by pushing it into a field** (a02 `add`, a05 `on`)? Looked: "A function may keep the blocks it is given" — shown only for a function returning a block (`compose`). Guessed methods and storing into a field are covered too. **M**
13. **Can a function of a type (`def self.min_len`) hand back a block**, `-> (Str) -> Bool = { \|s\| ... }`? (a02). Looked: structs-and-enums.md (`def self`) + `adder` in functions-and-blocks.md. Guessed yes. **H**
14. **Does a trailing `{ }` block follow a *method* call that already has an argument**, `pw.add("has a digit") { \|s\| ... }`, `bus.on("order") { ... }`? Looked: functions-and-blocks.md shows it for functions (`time_it("a") { ... }`) and says "Methods take blocks the same way". Guessed yes. **H**
15. **In a `spawn:` body that uses `?` on an optional, is a bare last line the implied `Some`?** (a03 `found`/`missing`). Looked: concurrency.md says `?` on an optional makes the task give back `T?` "the same way"; functions-and-blocks.md says a *block* whose result is a `T?` has **no** implied `Some` and fails only at build time. Guessed the task behaves like a function (bare value → `Some`). **L**
16. **Is `var tasks: [Task[Config or Error]] = []` the spelling for a list of such tasks?** (a03). Looked: concurrency.md "is a `Task[T or Error]`". Guessed yes. **H**
17. **A list pattern with a guard and a one-line `return v` arm**, `[k, v] if k == key -> return v`, in a `Str?` function (implied `Some` on `return`)? (a03). Looked: patterns.md (guards, `return` in an arm), errors.md (`return x` in `first_even`). Guessed yes. **H**
18. **Does `for name, text in SOURCES` walk a constant list of tuples, and are the loop names copied into each `spawn:`?** (a03). Looked: concurrency.md `for who, cents in moves` + "constants can be used inside `spawn:`". Guessed yes. **H**
19. **Can a struct hold a list of itself** (`Folder.subs: [Folder]`)? (a04). Looked: structs-and-enums.md shows recursive *enums* only. Guessed yes (it is a `Vec`). **M**
20. **Can an interface method's block parameter (`visit`) be called from inside another block handed on to a recursive call** (`sub.walk { \|p, s\| visit(...) }`)? (a04). Looked: interfaces.md says the block is lent as `&mut dyn FnMut`; nothing about re-lending it from a closure. Guessed yes. **M**
21. **May a non-kept block inside a method use a field bare** (`name` in `Folder.walk`'s inner block)? (a04). Looked: "A kept block inside a method cannot use `self` or its fields" implies a handed block can. Guessed yes. **M**
22. **Can an interface default call the required block-taking method by bare name with a two-parameter block, both `{ }` and `do \|path, size\|`?** (a04). Looked: interfaces.md `each_item { \|x\| t += x }` (one parameter). Guessed yes. **H**
23. **Are unused block parameters (`path` in `count`) accepted without an error?** (a04). Looked: nothing says either way; no `_` placeholder for block params is documented. Guessed yes (at most a stderr warning). **M**
24. **Is `extend {Str: Int} with Walkable` (a map at concrete types) allowed?** (a04). Looked: generics.md shows `extend {K: [Int]}`, `extend {K: V}`, `extend [Int]`. Guessed yes. **H**
25. **Can a map literal be an item of a `[Walkable]` list literal?** (a04). Looked: interfaces.md puts `[10, 20]` in `[Walk]`. Guessed maps work the same. **M**
26. **Does the "holds values of different types behind `Walkable`" warning go to stderr, not stdout?** (a04). Looked: seen only in `interface_methods.expected`, which (concurrency.md says) records both streams. Guessed stderr. **H**
27. **Does pushing through a missing key work when the values are lists of blocks, inside a `var self` method** (`handlers[topic].push(h)`)? (a05). Looked: collections.md `index[w].push(i)` in `main`. Guessed yes. **M**
28. **Does `handlers[topic].or([])` give a list of blocks I can `for` over and call?** (a05). Looked: collections.md (`.or([])` on a known map). Guessed yes. **H**
29. **Can a kept block stored in a struct field call a method (`push`) on a `shared var` list from a plain `def main`?** (a05). Looked: functions-and-blocks.md shows `total += x` on a `shared var` Int in a plain `main`, in a binding. Guessed a list method through a field-held block works too. **M**
30. **Is `new` free to use as a type-function name** (`def self.new`)? (a05). Looked: names.md keyword list (not there). Guessed yes. **H**
31. **Can an interface `async` method take an argument and give `Int or Error`?** (a06). Looked: interfaces.md shows only `async def fetch -> Str`. Guessed yes. **M**
32. **Is `Error("...")` alone a valid body for an `async def` that returns `Int or Error`, with an unused parameter?** (a06 `Broken.run`). Looked: errors.md (Error as last expression). Guessed yes. **H**
33. **Does `await` on a list *literal* of tasks, `await [a, b, c]`, work?** (a06). Looked: concurrency.md awaits a bound list or a `.map { spawn: ... }.to_list` chain. Guessed yes. **M**
34. **Does `await Flaky(...).run(1)` await the method call on a freshly built value and print `Ok(100)`?** (a06). Looked: concurrency.md `await Worker(3).slow`. Guessed yes. **H**
35. **Can `extend Box[Int] with Summary` and `extend Box[Str] with Summary` both be written, and each call pick the right one?** (a07). Looked: the brief says round eight fixed it; generics.md only shows `extend Stack[T]` and says two extends are fine when no type fits both. No example at chosen arguments anywhere. Guessed yes. **M**
36. **Inside `extend Box[Int]`, is the field `items` bare and typed `[Int]`**, so `items.sum` is allowed? (a07). Looked: interfaces.md "its fields are also there bare". Guessed yes. **M**
37. **Does a struct method with its own type parameter (`map_items[U]`) returning `Box[U]` work out `U` from the block, and does the result then take the `Box[Str]` summary?** (a07). Looked: blocks_of_your_own.lume `map_to[U]` returns `[U]`; nothing returns the generic struct at a new argument. Guessed yes. **M**
38. **Does `Box.single(7)` (a generic type's `def self`) work out `T` from its argument?** (a07). Looked: structs-and-enums.md sentence "a generic type's function works out its T ... from its arguments". Guessed yes. **H**
39. **May `nums` and `words` be used after being put into the `board: [Summary]` list?** (a07). Looked: examples.md mentions "copy or move by analysis" (ownership.lume, not read). Guessed yes. **M**
40. **Can a failing type function be passed to `map` by its qualified name, `lines.map(Student.parse)`, and the chain walked with `for`?** (a08). Looked: structs-and-enums.md `[50.0].map(Temp.from_f)`, methods.md "`for` walks a chain". Guessed yes. **H**
41. **In a `-> Student or Error` type function, can a `match` be the last expression with one arm giving a `Student` (implied `Ok`) and the other `Error(...)`, and can a `?` sit inside a `for` inside the first arm?** (a08). Looked: errors.md (if/else version), patterns.md (arm blocks). Guessed yes. **H**
42. **Does a one-line `if … elif … elif … else` fit inside `{ }` as a kept block's body?** (a08). Looked: control-flow.md one-line `if/elif/else` as a value; functions-and-blocks.md "including an `if`". Guessed yes. **H**
43. **For a09: is the copied-`var` error the first (and only) one reported, located at `calls += 1` inside a `do |arg|` kept block that is later put in a map?** Looked: functions-and-blocks.md lume-bad example (a no-argument `do` block). Guessed the same message for a one-argument `do |arg|`. **H**
44. **For a10: is the loop variable over a `[Sheet]` "held" in the sense of the rule, and does the message name the interface as `` `Sheet` ``** (no type arguments)? Looked: interfaces.md lume-bad (`held.at(0).map_all`, `Mappable[Int]`). Guessed yes to both; the exact type-name wording is the uncertain part. **M**

## Docs that contradict each other or the examples

- **`?` in a task vs `?` in a block that gives `T?`.** concurrency.md: a task
  that uses `?` on an optional gives back `T?` "the same way" (the same as
  `T or Error`, where the bare last line is the implied `Ok`). But
  functions-and-blocks.md: a block whose result is `T?` has *no* implied
  `Some` and "passes `lume check` and then fails to build". A `spawn:` body
  is described as both "like a function body" and "as an `async` block does
  in Rust". Which rule a task follows is not said (guess 15).
- **A space before `{` after `puts`.** printing.md's heading is "A space
  before `(` or `{`", and it says a space before an opening bracket is
  ambiguous. The same page prints `puts {"x": 1}` and `puts {"one", "two"}`
  with a space and no complaint, and generics.md writes
  `puts({"pear", "fig"}.in_order)` with brackets. So either `{` is fine or it
  is only a problem with a method after it. The text never says which.
- **"Put in a map or a tuple".** functions-and-blocks.md lists a map and a
  tuple as places a block can be kept, and kept_blocks.lume's header lists
  "a binding, a field, a list, a result" — no map, no tuple. No example in
  docs/ or examples/ keeps a block in either one, though the brief calls
  this a round-eight fix.
- **`extend` at chosen arguments.** The brief names `extend Box[Int]` /
  `extend Box[Str]` as fixed. generics.md says only that a name in the target
  that is not a type is a parameter, which implies `Box[Int]` is concrete, and
  gives no example. It is not a contradiction, but the feature is shown
  nowhere in the docs.
- Small one: a function with no parameters is declared `def greet -> Str`
  in functions-and-blocks.md but `def may_fail() -> Int or Error` in
  errors.md. Both forms seem to be accepted, but neither page says so.

## Verdict

For the everyday things (strings, collections, errors as values, `match`,
`def self`, blocks handed to calls) the docs held up. I wrote eight
realistic programs and almost every line came straight from a documented
example. The guesses cluster on this round's own features: a block kept in a
map or a tuple has one sentence of documentation and no example, so the
typing, the lookup (`ops[name]` is an optional block), calling a tuple's
block (`pick.1(7)`) and how the tuple prints are all inferred. The same goes
for `extend Box[Int]`/`extend Box[Str]`. Blocks-that-take-blocks in
interfaces are well covered for one-argument blocks, and less so for
two-argument ones or for re-lending the block from inside another. The
riskiest single point is whether a `spawn:` body that uses `?` on an
optional gets an implied `Some`: two pages point different ways, and the
failure mode that the docs themselves warn about ("passes `lume check`, then
fails to build") is the one that would hit. One more example in
functions-and-blocks.md, a `{Str: (Int) -> Int}` table plus a
`(Str, (Int) -> Int)` pair, would remove about eight of these 44 guesses.

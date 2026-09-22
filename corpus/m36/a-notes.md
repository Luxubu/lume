# Round m36, reviewer A — everyday programs

Read: `docs/README.md`, `docs/tour.md`, `docs/examples.md`, `docs/install.md`,
every page in `docs/reference/`, and in `examples/`: `stdlib`, `printing`,
`collections`, `sets`, `chars`, `loops`, `blocks`, `pipes`, `enums`, `shadow`,
`lib/*`, `port/wordcount`, `shipping/parcels`, and about a dozen files in
`examples/errors/`. I did not open `docs/QUESTIONS.md`, the compiler, `tests/`,
or anything else in `corpus/`. I did not run anything. I used a Python
scratch script only as a calculator to count padding widths and do the
float arithmetic. Every language rule I applied came from the docs.

## 1. The programs

| file | what it is | runs? | what it leans on |
|---|---|---|---|
| `a01_wordfreq.lume` | word-frequency report | RUN | `"""` + `lines`, bare `split`, `chars.filter(_.alpha?).join("")`, map tally, key order, `to_list` + `sort_by` on tuples with a two-name block, stable sort for ties, `for a, b in xs.take(5)`, `pad`/`pad_right`, `repeat`, `max_by` on `.keys`, an interpolated `Some` and map |
| `a02_csv_sales.lume` | CSV sales summariser | RUN | `split(",").map(_.trim)`, `.at`, `to_int?`/`to_float?` in a `T or Error` fn, implied `Ok`, `lines.skip(1)`, `enumerate`, `match` Ok/Error + `e.message`, `{Str: Float}` totals, `sum(_.total)`, `decimals(2).pad(10)`, `max_by` |
| `a03_gradebook.lume` | grade book | RUN | struct methods, Float averages, multi-line `if`/`elif` as a value, `sort_by` with a Float key + `.reverse` (ties reversed), `decimals(1)`, `flat_map`, `avg`, a `max_by` tie, `Option.map(_.name).or`, map/list in interpolation |
| `a04_invoice.lume` | invoice formatter | RUN | `Int.pad`, `decimals(2)` on inexact floats and on a negative, `"$" + Str`, one-line `if` value, trailing `if` on `puts`, an `= expr` body broken after `=`, `sum` over Floats |
| `a05_logparse.lume` | log-line parser | RUN | bare `split`, `skip(4).join(" ")`, `replace`, string `match` with `\|`, trailing `if`/`unless` on `return`, implied `Some`, `s[0..1]`, a blank line inside `"""`, `for … where`, `to_set`, printing list/map/set/`Some((Str, Int))` |
| `a06_textwrap.lume` | greedy text wrapper in a box | RUN | building `[Str]`, `if`/`elif`/`else`, trailing `unless` on a call, `pad_right` not cutting an over-wide word, centring by integer division, `(i + 1).pad(2)`, `map(_.len).max.or(0)`, `for` over `take(3)` |
| `a07_shopping.lume` | shopping list and pantry check | RUN | `uniq`, code-point `sort`, `sort_by(_.downcase)` stability, case-folded tally, map `filter{\|k, v\|}.keys`, `to_set`, set `diff`/`intersect` order, case-sensitive `contains?`, `m[k].push` from nothing, printing `{Str: [Str]}`, set `filter` |
| `a08_ciphers.lume` | Caesar, palindromes, initials, tally | RUN | `Char.code`, `Int.to_char`, `%` with a negative operand, `chars.map{}.join("")`, `Char` predicates, `s[0]` as `Char?`, `capitalize`, `match` on `Char` with guard arms, `{Char: Int}`, a `Char` printed inside a tuple inside `Some`, a `def name?` function |
| `a09_temperatures.lume` | weekly temperature summary + bar chart | RUN | list pattern `[a, b]` on `split`, `to_float` on `"15"`/`"-2.5"`, printing `[Float]`, `sum`/`max`/`min`/`sort` on Floats, `round` on halves, `Float.to_int`, `Int.max`, `pad` on a Float, `repeat(0)`, `count(_…)` |
| `a10_basket_average.lume` | basket summary with Float / Int | **FAIL**: `cannot combine` | the documented rule that `Int` and `Float` never mix, hit in the most common place: `total / xs.len` |

Nine are meant to run and one is meant to fail. In the headers, a line that
is just `#` stands for a blank line of output.

## 2. GUESSES

**22 guesses**

1. **Can `for` loop over a lazy chain, e.g. `for w, n in ranked.take(5)`?**
   (a01, a06)
   Looked in: strings.md, which says "`for` takes a list, a set, a map or a
   range" and does not list a chain; control-flow.md; collections.md. The
   comment in `examples/blocks.lume` lists `for` among the consumers, and
   `pipes.lume` loops over `text.lines |> .reject(...)`.
   Guessed: yes. **Medium-high.**

2. **Does a block with two names take a tuple item apart**, as in
   `pairs.sort_by { |w, n| 0 - n }`, `.filter { |w, n| … }`, `.map { |w, n| w }`
   and `.max_by { |c, n| n }` over a `[(Str, Int)]`? (a01, a05, a08, a10)
   Looked in: functions-and-blocks.md, which shows two names only for `fold`
   and `enumerate`; collections.md, which shows them only for a map. The
   examples show it: `stdlib.lume` has `zip(...).map { |n, i| … }` and
   `port/wordcount.lume` has `sort_by { |w, n| -n }`.
   Guessed: yes, on every list method. **Medium-high.**

3. **Does `max_by` work straight on a lazy chain** (`counts.keys.max_by { … }`)?
   (a01)
   Looked in: collections.md. Its list of methods that end a chain is "`join`,
   `sum`, `len`, `sort`, `first`, `max`, `min`, `count`, `any?`, `all?` and
   `fold`", and `max_by` is not on it. The tour adds "and the rest".
   Guessed: yes. **Medium.**

4. **When two items tie, which one does `max_by` / `min_by` return: the first
   or the last?** (a03: Cy and Eve both average 88.0)
   Looked in: collections.md, which says only that `max_by` gives a `T?`;
   stdlib.lume.
   Guessed: the first (Cy). **Low.** Rust's `Iterator::max_by` returns the
   *last*, so if Lume passes it through, a03's line reads
   `top B student: Eve`.

5. **Is `sort_by` stable?** (a01, a07, a08 all break ties with it)
   Looked in: collections.md, which says "**`sort` is ascending and
   stable**" and says nothing about `sort_by`. The output of
   `port/wordcount` (`fox` before `dog` on equal counts) is consistent with
   stable.
   Guessed: stable. **Medium-high.**

6. **Is `Float` ordered?** That is: do `[Float].sort`, `.max`, `.min`, and
   `sort_by`/`max_by` with a Float key work? (a03, a09)
   Looked in: generics.md (`Ordered`, and "`Float` is not Hashable"),
   operators.md, collections.md. Only `1.5.max(0.5)` in `stdlib.lume` touches
   it. Rust's `f64` is not `Ord`, which is why I doubted it.
   Guessed: yes, all of them work. **Medium.**

7. **How does `Float.round` round an exact half?** (a09: 7.5 → 8.0, and
   -1.25 → -1.0 feeds `max(0)`)
   Looked in: operators.md, which shows only `3.7.round #=> 4.0`, and
   printing.md, which gives `decimals` a rounding rule but not `round`.
   Guessed: half away from zero, like `decimals`. **Medium.**

8. **Does `Float.to_int` give a plain `Int`, or an `Int or Error` like
   `Str.to_int`? Does it truncate?** (a09)
   Looked in: operators.md: "`.to_int` is how you cross over", and nothing
   more. `json.lume` and `mini/eval.lume` use `n.to_int.to_s` on a Float,
   which suggests a plain `Int`.
   Guessed: a plain `Int` that truncates. I round first, so the truncation
   does not matter here. **Medium-high.**

9. **Does `"15".to_float` (no decimal point) succeed? Does `"-2.5"`?** (a09)
   Looked in: errors.md and strings.md. Both show only `"x".to_float` failing.
   Guessed: `Ok(15.0)` and `Ok(-2.5)`. **Medium-high.**

10. **What does `%` give when the left side is negative?** (a08:
    `(0 - 1) % 26`)
    Looked in: operators.md. It shows `-7 / 2 = -3`, truncating toward zero,
    and only `7 % 2` for `%`.
    Guessed: `-1`, the remainder that matches truncating division, so
    `caesar("abc", -1)` gives `` `ab ``. **Medium-high.**

11. **Is the last expression of a `-> T?` function wrapped in `Some`
    automatically**, the way `T or Error` gets its implied `Ok`? (a05's
    `parse` ends in a bare `Entry(...)`)
    Looked in: errors.md. Its only example is `return first + 0` in
    `head -> Int?`, which prints `Some(5)`. functions-and-blocks.md says a
    *block* returning `T?` has "no matching implied `Some`", which implies
    functions do have one.
    Guessed: yes, for the last expression as well as for `return`.
    **Medium-high.**

12. **Does a trailing `if`/`unless` work on `return`?** (a05:
    `return None if words.len < 5`, `return None unless known`)
    Looked in: control-flow.md: "It works on an assignment, and on `next`
    and `break`". `return` is not mentioned.
    Guessed: yes. **Medium-high.**

13. **List patterns in `match`**, such as `[day, value] -> …`: do they exist,
    do they match only that exact length, and do they work on the result of
    bare `split` as well as `split(",")`? (a09)
    Looked in: control-flow.md, whose `match` section has no list patterns
    and no guards, and structs-and-enums.md. They appear only in
    `examples/pipes.lume` (`match line.split(","):` / `[name, age] ->`).
    Guessed: yes, exact length only, and bare `split` behaves the same.
    **Medium.**

14. **Does an empty line inside a `"""` block count when the common
    indentation is worked out?** (a05's log has a blank line in it)
    Looked in: strings.md, "drops … the common indentation of the lines
    inside". Blank lines are not mentioned.
    Guessed: blank lines are ignored. I wrote a05 so that its output is the
    same either way, because every line is `split` or `trim`med. **Low.**

15. **Does assigning to a key that already exists keep its place in the
    map?** (a01, a02, a05, a07, a08: every tally)
    Looked in: collections.md: "A map keeps the order you put things in"
    and "A tally prints in the order it was built". "The order you put
    things in" could mean the *last* put. The stdlib example
    `m.merge({"c": 30, "d": 4})` printing `{"a": 1, "b": 2, "c": 30, "d": 4}`
    suggests the first insertion wins.
    Guessed: it keeps its first position. **Medium-high.**

16. **How does a computed `Float` print when it is not exact?** For example,
    `50.23` summed in binary is `50.230000000000004`. How many digits
    appear? (a02, a04, a09)
    Looked in: printing.md: "A `Float` keeps its point: `2.0` prints as
    `2.0`". Nothing covers inexact values.
    Guessed: the shortest form that round-trips, as in Rust. I **routed
    around it** by using `decimals` wherever a sum was inexact, and printed
    bare only values that are exact (12.5, 24.0, 21.5). That is exactly the
    routing-around printing.md says it exists to prevent. **Low** (on what
    is printed).

17. **Must a file have `def main`?** Nearly every snippet in the docs is
    bare top-level statements (`xs = [3, 1, 2]` / `puts …`), but the tour
    says "`def main:` is where a program starts".
    Guessed: a real file needs `def main`, and the snippets are wrapped by
    the doc tester. All ten programs have one. **Medium.**

18. **Is a statement that starts with `(` read as an expression**, as in
    `(" ".repeat(left) + s).pad_right(width)` as the last line of a
    function? (a06) `(a, b) = …` destructuring also starts with `(`, and
    printing.md is wary of `(` after `puts`.
    Looked in: structs-and-enums.md (destructuring) and printing.md.
    Guessed: yes, it is an expression. **Medium-high.**

19. **May the same name be bound in two sibling scopes?** For example,
    `Some(e) -> …` in one loop and then `for e in entries` in a later loop,
    or `Ok(s)` in a `match` and later `for s in sales`. (a02, a05, a09)
    Looked in: tour.md, "A name cannot be re-bound in a nested block", and
    `shadow.lume`, which turned out to be about something else (see §4).
    Guessed: siblings are fine. Only nesting inside an existing binding is
    refused. **Medium-high.**

20. **Which words are reserved?** I renamed `all` to `every_score` in a03
    out of caution (because of `all?`), and kept `log`, `input`, `text`,
    `lines`, `line`, `other` and `digits` as names.
    Looked in: operators.md and control-flow.md. There is no keyword list.
    `lib/FINDINGS.md` names some keywords (`where`, `next`, `match`, `in`,
    `test`, `import`, `pub`, `extend`, `interface`, `assert`), and
    control-flow.md says `loop`/`until` are *not* keywords.
    Guessed: none of my names are reserved. **Medium-high.**

21. **Does `.to_set` work straight on a lazy chain**
    (`wanted.map(_.downcase).to_set`)? (a07)
    Looked in: collections.md, which does not mention `to_set` at all, and
    `sets.lume`, which writes `.map { … }.to_list.to_set`. That `to_list`
    made me doubt it.
    Guessed: yes. **Medium.**

22. **What is the full text of the Int/Float error for `Float / Int`?** (a10)
    Looked in: operators.md, whose snippet has only `#! cannot combine`, and
    `examples/errors/`, whose 106 cases include none with `cannot combine`
    in the expected text. That is surprising, given how commonly first-week
    code hits this error.
    Guessed: `error: cannot combine `Float` and `Int` with `/`` at the `/`,
    with help that mentions `.to_float`. I claim only "cannot combine".
    **High** on that fragment, **low** on the rest.

## 3. ANSWERED

These are things I expected to have to guess and found answered clearly.
Many of them were the "guessed most" items from earlier rounds, and they
are now nailed down.

- **How every value prints.** Text inside a container is quoted, text on its
  own or in `#{}` is bare, a Float keeps its `.0`, and a `{}` is empty for
  both sets and maps. Structs print with named fields, and `Some`/`None`/
  `Ok`/`Error` print as written. printing.md's one rule ("a value on its own
  prints as itself; inside another, as you would write it") let me predict
  `Some("frog")`, `Some(("web", 3))`, `{"produce": ["apples"], …}` and
  `skipped: ["garbage line"]` with confidence. `port/app_async.expected`
  even shows how an embedded `"` is escaped inside a list.
- **`pad` vs `pad_right`**: left vs right, spaces only, never truncates,
  works on Int and Float. a06's over-long word depends on "a column can
  overflow but never lie".
- **`decimals(n)`**: text result, and the rounding rule including negatives.
- **`join`** works on non-text lists and directly on a lazy chain, with no
  quotes. **`.to_list`** is needed only to store, return or index.
- **Map order**: insertion order for `keys`, `values`, `to_list` and `for`,
  plus `m[k].or(d)` and write-through auto-creation (`counts[k] = …`,
  `index[k].push(x)`).
- **`sort`**: ascending and stable, `.reverse` for descending, code-point
  order with capitals first. `sort_by { |w| w.downcase }` is the way to
  sort as a reader expects (a07 relies on all of this).
- **Strings**: `len` counts characters; `slice(offset, count)` vs `s[a..b]`
  (inclusive) vs `...`; clamping; the two `split`s, including `""` cases;
  `lines`; `capitalize` lowering the rest; `repeat(0)`; `#` inside strings;
  `"""` stripping and having no trailing newline.
- **`Char`**: no literal, a one-character string works wherever a `Char` is
  wanted, `s[i]` is `Char?`, `Int.to_char` is `Char?`, `Char` works as a
  map key, and the full `Char` method list.
- **Operators**: the precedence table (`not a and b`), integer division
  truncating toward zero, no mixing of Int and Float, and `round` returning
  a Float.
- **Control flow**: trailing `if` binds the whole statement, `unless` exists
  only as a trailing form, `elif`, a multi-line `if` works as a value if it
  starts on the `=` line, `next`/`break` behave in `for`, `while` and nested
  loops, `for … where`, `()` as the empty statement, `match` arms can be
  blocks.
- **Blocks**: how far `_` reaches (`_.len == 2`, `_.upcase.reverse`), `_`
  used once, a function name used as a block, `{}` holding a single
  expression.
- **Failure**: the implied `Ok`, `?`, `e.message`, the exact `to_int` and
  `to_float` error texts (a02 depends on "`` `oops` is not an integer ``"),
  and `first`/`max`/`max_by` giving `T?`.
- **Structs**: positional and keyword constructors, fields used bare inside
  methods, and no `()` needed on a method call.
- **Empty literals** need a type.
- Methods not in the reference that I found in `examples/stdlib.lume` with
  exact output: `uniq` (keeps the first of each), `avg` (gives a Float),
  `flat_map`, `zip`, `group_by` (first-seen key order), map
  `filter`/`reject`/`map_values`, `count(block)`, `sum(block)`,
  `Int.max/min/clamp`, `to_set`, and `add` on a set returning a Bool.
  `sets.lume` shows that set results keep insertion order (`a.diff(b)` is
  `{1, 2}`) and that `filter` on a set gives a set. Match guards
  (`_ if c.upper? ->`) are in `chars.lume`.

## 4. CONFUSING OR WRONG

1. **The README promises "every method" for collections, and the page does
   not have them.** The reference table says collections.md covers "lazy
   chains, `.to_list`, ordering, every method". collections.md has no method
   list. strings.md ends with the complete `Str` and `Char` lists (the
   compiler's own error text), and that is excellent. Nothing similar exists
   for `[T]`, `{K: V}`, `{T}`, `T?`, `Int` or `Float`. `uniq`, `flat_map`,
   `zip`, `group_by`, `partition`, `avg`, `sort_by`, `take_while`, `skip`,
   `last`, `each`, `to_set`, `insert`, `remove_at`, `flatten`, `merge`,
   `map_values`, `reject`, `count(block)`, `sum(block)` and `sqrt` appear
   only in `examples/stdlib.lume` and elsewhere. Guesses 3, 5, 6, 8 and 21
   all come from this gap.

2. **A stale comment in a linked example contradicts the reference.**
   `examples/lib/table.lume` says "`to_float` says 0.0 for anything it
   cannot read, so the table checks first". errors.md says
   `"x".to_float #=> Error("`x` is not a number")`. Someone reading the
   library example would take away the wrong rule.

3. **`examples/sets.lume` has the wrong type in its header**: "`s[i]` is a
   `Str?` (one character)". strings.md is emphatic that it is a `Char?`.

4. **`shadow.lume` is described as the opposite of what it shows.**
   examples.md lists it as "why rebinding in a nested block is an error".
   The file actually demonstrates *allowed* self-transform rebinding
   (`name = name.trim`), which the tour contradicts: "A name is written once
   and does not change". The self-transform exception is otherwise
   documented only in the help text of `examples/errors/reassign.expected`.
   A new user would not know `x = x.trim` is legal.

5. **Features used in the examples are missing from the reference.**
   control-flow.md's `match` section has no guards (`_ if cond ->`, used in
   `chars.lume`) and no list patterns (`[a, b] ->`, used in `pipes.lume`).
   operators.md's `|>` section has no `x |> .method` form, although
   `pipes.lume` opens with it.

6. **Floats are the least-documented everyday type.** The docs never say
   how an inexact float prints, how `round` handles halves, what
   `Float.to_int` returns or how it rounds, or whether Floats can be sorted
   or given to `max`/`min`. Money, averages and temperatures are the first
   things a person computes, and I routed around all of it with `decimals`.

7. **Snippets without `main`.** Almost every reference snippet is a series
   of bare statements, and a few wrap them in `def main:`. Nothing tells the
   reader which of these is a valid *file*. The harness evidently wraps them,
   but the reader is not told (guess 17).

8. A small point: functions-and-blocks.md says "there is no `return`
   keyword at the end" and then says "`return` exists for leaving early".
   errors.md uses `return n / 2` as the *last* line in several examples. The
   wording reads as if a final `return` were wrong, but the examples show it
   is only unnecessary.

## 5. Verdict

**Yes, a new user could write these first-week programs from these docs**,
and with far less guessing about the things earlier rounds tripped on.
printing.md, the `pad`/`join`/`decimals` sections, map ordering, sort
stability and the whole of strings.md are clear, specific and correct as
far as I can tell by reading. Every one of those previously guessed areas
now has a direct answer. Of my 22 guesses, most are at the *edges* of the
collection API (tuple-destructuring blocks, tie-breaking, what works on a
lazy chain) or concern Floats. None is about the core printing and layout
rules.

**The single change that would help most:** give collections.md a complete
method table like the one strings.md has, for `[T]`, `{K: V}`, `{T}` and
`T?`. For each method it should give the return shape (a list, a lazy
chain, `T?`, a map or a set) and one line of behaviour: stability,
tie-breaking, whether a two-name block takes a tuple apart. Then add a
short "Floats" section covering printing of inexact values, `round`,
`to_int` and ordering. Those two additions would remove roughly
two-thirds of my guesses.

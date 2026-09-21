# m28 review — features meeting each other

Twelve programs written from `README.md` and the top-level `examples/*.lume`
only. The compiler, the tests, the other corpora and the library/port/mini/site
examples were not read, and nothing here was run or compiled. Every expected
output was worked out by hand.

Ten programs are meant to run; `y11` and `y12` are meant to be rejected.

## 1. What each program probes

| # | File | Seam |
| --- | --- | --- |
| y01 | `y01_blocks_and_generics.lume` | A generic function's block parameter handed on to another generic (`apply_twice`), to a built-in (`xs.sum(w)`), and to its own recursive call (`repeat_until`). `_` shorthand and a function name as the block. A block that closes over a `var` and changes it. `puts f(x) do \|y\|` with an indented body. A pipe whose right side is a generic call still needing a `_` block: `items \|> weigh(_.price)`. |
| y02 | `y02_generic_results.lume` | Generics crossed with `T or Error` and `T?`. A block declared `(Row) -> T or Error` whose body itself uses `?`. `?` on the block's result inside the generic. `.or(fallback)` where the fallback is what fixes `T`. `?` on an optional plus an early `return None` in a `T?` function. A tuple pattern nested inside `Some(...)`. |
| y03 | `y03_interfaces_in_collections.lume` | A `[Priced]` list holding two struct types; an interface default reached both through the interface value and straight on the type; the same interface as a *bound* (`[T: Priced]`); `sort_by` over interface values; a struct with `+`, `<`, `==` and whether `<` alone satisfies the built-in `Ordered` bound of a user-written generic; `sort`/`max`/`min`/`fold` over that type. |
| y04 | `y04_generic_tree.lume` | `enum Tree[T]` with `match self` methods; a bounded recursive generic that rebuilds the enum from an arm's bindings; a generic fold taking a block over a generic enum; a two-parameter generic deciding on a tuple of `Bool`s; one deliberate probe of a nested pattern through a recursive field (`Node(_, Leaf, Leaf)`) — see ambiguity 1. |
| y05 | `y05_keys_and_nesting.lume` | `{Cell}` and `{Cell: Str}` (struct as set item and map key); `{Suit: Int}` (field-less enum as map key); `{Str: {(Int, Int)}}` changed in place on a missing key; `{Int: {Int: Str}}` written `grid[i][j] = v`; `group_by` over structs; set algebra over tuple sets; set `filter` giving a set. |
| y06 | `y06_ownership_seams.lume` | A struct stored in a list then used again; `var u = t` as a copy; `xs[i].method(...)` where `xs` is a *field* of another struct; `for var t in items` inside a `var self` method; a `for` loop pushing to the list it walks; a `var xs: [Int]` parameter filled by the callee; last-use move. |
| y07 | `y07_async_generics.lume` | An `async def` that is also generic *and* takes a block, called with `_`, with a named function, and with a trailing `{...}` block after `await`. `spawn:` around a generic call, awaited as a list of `Int?`. `shared var` Int and list written from tasks built inside a `map {...}` block. A read-only `shared` and a plain local captured by spawned blocks. |
| y08 | `y08_report_text.lume` | An interface default that takes an argument and loops (`line(widths)`), reached through the bound `[R: Row]`; `at` inside generic code; `enumerate` as `i, c`; `xs[i] = v`; `pad_right` + `trim_right` to build a table; `decimals` on Int, on exact binary fractions and on a negative; `trim_left`/`trim_right`; character-counted `s[i]`/`s[a..b]` against `s.len` on a non-ASCII string. |
| y09 | `y09_tests_over_generics.lume` | `test`/`assert` over generic and interface code: comparing a generic's `[Str]` to a literal; `== Some(x)` and `.none?` on a generic block function's `T?`; operators a type never wrote (`>=`, `!=`, `sort`, `max`, `min` from `<` and `==`); `!` inside a test; statements before asserts. |
| y10 | `y10_pipes_meet_generics.lume` | `\|>` into plain and generic functions; a piped call whose remaining argument is a `_` block; multi-line pipes alternating user functions and built-in methods; a pipe producing `T or Error` handled with `.or`/`.error?`; the same generic block function piped and called directly. |
| y11 | `y11_err_unordered_bound.lume` | **Deliberate error.** A generic bound `[T: Ordered]` meeting a type that has `==` and `+` but not `<`. Checks that the error is reported at the call site in `main`, names the bound, and names the missing `<` — not reported inside the (correct) body of `best`. |
| y12 | `y12_err_question_in_block.lume` | **Deliberate error.** `?` inside a block whose declared type is `(Str) -> Str`, so it cannot carry a failure — while the enclosing `main` returns `() or Error` and could. Checks that the error blames the *block*, names the declared block type, and does not silently let the `?` escape into `main`. |

## 2. Ambiguous, surprising or under-specified in the README

Numbered roughly by how much it cost me.

1. **The README says two opposite things about nested patterns on recursive
   enum fields.** The feature list says Lume has "nested patterns through a
   recursive enum's fields (`Binary("+", Num(0.0), r, _)` reaches inside the
   pointer a recursive field lives behind)". The Ownership section says the
   opposite: "A recursive `enum Tree: Leaf / Node(left: Tree, ...)` works as
   written; the `Box` Rust needs is emitted for you (**a nested pattern on such
   a field is an error that says to `match` the field in the arm**)." These
   cannot both hold. `y04` writes `smallest` the defensive way (a `match` on the
   field) and `lone_left?` the optimistic way (`Node(_, Leaf, Leaf)`), and the
   header flags it. Whichever is true, one of these two sentences should go.

2. **`pad` versus `pad_right`: which side is padded is never stated.**
   `examples/stdlib.lume` prints both `"ab".pad_right(5)` and `"ab".pad(5)` but
   I cannot see the expected output. `examples/collections.lume` uses
   `w.pad_right(6)` to make a left-aligned word column, which suggests
   `pad_right` = "pad on the right". I therefore guessed `pad(4)` on `"7"` is
   `"   7"` and `pad_right(4)` is `"7   "`, and `y08`'s whole table depends on
   it. Two methods whose names differ by one word and whose behaviour differs by
   which side deserves one clause in the README.

3. **Is `Str.len` characters or bytes?** The README is careful to say `s[i]` is
   "counting characters not bytes" and that `s.chars` is a `[Char]`, but `len`
   is listed flatly among the Str methods. For `"héllo wörld"` that is 11 versus
   13. `y08` asserts 11 and also prints `s.chars.len` beside it so the two can
   be compared.

4. **Does `join` work on a list that is not `[Str]`?** `join` is listed under
   "on lists" with no type restriction, but `examples/generic_extend.lume`
   writes `items.map { |x| "#{x}" }.to_list.join(", ")` — converting to text
   first, inside a generic, which is exactly where you would want
   `[Int].join(", ")` to just work. I assumed it does not and mapped to `Str`
   everywhere. If it does work, the example is misleading; if it does not, the
   list-method list should say `join` is `[Str]`-only.

5. **Is `.to_list` required before a consumer like `join` or `sort`?**
   `examples/blocks.lume` line 34 binds `labels = users.map { ... }` and then
   calls `labels.join(", ")` with no `.to_list`. `examples/generic_extend.lume`
   inserts `.to_list` in the same position. The README says chains are lazy and
   names `to_list`, `sum`, `each`, `for`, `puts` as consumers, but does not say
   whether `join`/`sort`/`len` are consumers too. I wrote `.to_list` everywhere,
   which may be redundant noise in the generated code.

6. **Does a field satisfy an interface's method requirement?** "a type conforms
   by having the methods, with nothing to declare". A struct with a field
   `cents: Int` and an interface asking `def cents -> Int` is the very first
   thing a new user writes. Nothing says whether a field counts. I renamed the
   field to `price` in `y03` to avoid finding out, but this will bite people.

7. **Does defining `<` make a type satisfy a user-written `[T: Ordered]`
   bound?** The README says "`[T: Ordered]` for `<`" (from the generics
   paragraph) and, separately, "`<`; ... `<=`, `>`, `>=`, `sort`, `max`, `min`
   follow from `<`" (from the operator-methods paragraph). It never joins the
   two: is `Ordered` a built-in interface that a user's `def <(o: Money)`
   structurally conforms to? `y03` (`biggest(ms)`), `y09` (`ranked(ss)`) and
   `y11` (the negative case) all rest on the answer being yes.

8. **Implicit `Ok` across the arms of an `if`/`match` used as a value.**
   `examples/errors_and_results.lume` has, in an `Int or Error` function,
   `if n < 0: Error(...) elif n > 150: Error(...) else: n` — one arm an `Error`,
   one a bare `Int`. But the error list promises an error for "branches of an
   `if`/`match` used as a value that give different types". So the implicit-`Ok`
   rule must run per arm, before the branch-type check — which is worth stating,
   because it is the difference between a working and a rejected program. I
   dodged it in `y02` with a `return Error(...)` guard.

9. **Returning a `T or E` value from a `T or E` function: pass-through or double
   wrap?** `parse_age` writes `n = s.to_int?` and then returns `n`, never
   `s.to_int` directly, even though the types line up. If a bare value is
   implicitly `Ok`, then `def f -> Int or Error: s.to_int` would be
   `Ok(Ok(...))`, which cannot be what is meant. The rule must be "an expression
   that is already `T or E` passes through", but it is not written down. I wrote
   `?` everywhere to avoid it.

10. **`?` collides with method names ending in `?`.** `?` is a postfix operator
    (`s.to_int?`) and also the last character of a great many method names
    (`adult?`, `ok?`, `error?`, `some?`, `none?`, `digit?`, `empty?`). So
    `puts total?` is genuinely ambiguous: apply `?` to `total`, or call a method
    named `total?`. `y10` was going to write exactly that; I bound the value
    first instead. Nothing in the README says how this is resolved, and
    `doubled?.sum` (postfix `?` followed by a method call) is likewise not shown
    anywhere — `y02` avoids it with an extra binding.

11. **`xs[i] = v` on a plain list is never shown.** The README's sentence on this
    covers `idx[w].add(x)`, `grid[i].push(x)`, `grid[i][j] = v`, `xs[i].field = v`
    and `xs[i].method(...)`, plus "a bare `xs[i]` as a value is a `T?`". A plain
    `xs[i] = v` on a `[Int]` appears nowhere. `y08`'s `widths_of` uses it.
    (`grid[i][j] = v` reads as a map-of-map form in context, not a list form.)

12. **A no-argument mutating method.** Every `var self` method in every example
    takes at least one other argument (`o.submit("Luan")`, `g.visit(1)`,
    `s.push(1)`, `m.move(2.0, 3.0)`). A `def bump(var self)` called as `t.bump`
    — no parentheses, as the README encourages for no-argument methods — is not
    shown, and reads like a field access. I gave every mutating method an
    argument to sidestep it.

13. **`await` meeting a trailing block.** `y07` writes
    `nine = await widths([3]) { |n| n * n }`. `await` takes an expression; a call
    with a trailing block is an expression; but nothing shows the two together,
    and `await` is spelled as a prefix keyword rather than a method. Likewise
    `await` in the middle of an arithmetic expression (`five + await f(x)`),
    which I split into two bindings rather than risk.

14. **The `_` shorthand meeting the pipe.** `y01` writes
    `items |> weigh(_.price)` and `y10` writes `words |> only(_.len == 3)`. Both
    `|>` and `_` are implicit-argument features: `|>` says "the left side goes in
    the first hole", `_` says "the block's one argument goes here". The README
    defines each alone (`x |> f(y)` is `f(x, y)`; `_` is "one argument, used
    once") but never together. This is the single most likely thing in my corpus
    to behave differently from how I expect.

15. **`spawn:` as a sub-expression in a list literal.** `examples/async.lume`
    nests `spawn:` inside `push(...)` and inside a `map { ... }` block, so it is
    an expression. But `await [spawn: a, spawn: b]` — two `spawn:` blocks inside
    a list literal, where the `:` might be read as something else — is not shown.
    `y07` binds them first.

16. **Three features at once, async included.** `async def widths[T](xs: [T],
    size: (T) -> Int)` in `y07` is async + generic + block-taking. Each is
    documented alone; nothing says they compose, and milestone 12 predates
    generics (22), blocks (23) and parameterised interfaces (25). Relatedly: the
    error list has "a `spawn:` inside a method that uses `self`" but says nothing
    about `spawn:` inside a generic function, inside a block parameter's body, or
    inside an interface default.

17. **`shared var` versus the "`var` changed inside a `spawn:` block" error.**
    The error list says a "`var` changed inside a `spawn:` block (it is a copy)"
    is rejected. `examples/async.lume` lines 44–48 do exactly that with
    `shared var hits = 0` and `spawn: hits += i`. The rule is evidently "a plain
    `var`, not a `shared var`", but the error-list wording does not carry the
    exception, so a reader meets a documented error that the flagship example
    appears to commit.

18. **Interface values outside a list.** "`[Shape]` holds mixed types behind a
    pointer and says so once" is stated only for lists. `{Str: Shape}`,
    `{Shape}`, `(Shape, Int)` and a `Shape?` are unspecified — and `{Shape}`
    would additionally need `Hashable`, which an interface cannot promise. I
    restricted `y03` to `[Priced]`.

19. **Interface defaults with an ordinary argument.** Every default in the
    README and examples is either no-argument (`shout`, `describe`, `size`,
    `listed`, `pair`) or takes the interface's own type parameter
    (`before?(other: T)`). `y08` writes `def line(widths: [Int]) -> Str` inside
    `interface Row` — an argument unrelated to the interface. It should be fine,
    but it is untested territory in the documentation.

20. **`m.get(k)` versus `m[k]`.** `get` is listed among map methods; `m[k].or(0)`
    is the form every example uses. The README never says what `get` returns or
    why both exist. I used `m[k]` throughout.

21. **`.or({})` is exercised, `.or([])` is not.** `examples/collections.lume`
    and `examples/sets.lume` recover missing map values with `.or({})`. The
    equivalent `.or([])` for a `[T]?` runs straight into "an empty `[]` binding
    with no type" — which is about bindings, not arguments, but a reader cannot
    tell. `y05` uses `contains?` instead of `groups[3].or([])`.

22. **`filter` is asymmetric between lists and sets.** On a set, "the block
    methods with `filter`/`reject` giving a set back" — so `s.filter {...}.len`
    works. On a list, `filter` builds a lazy chain, so `xs.filter {...}.len` may
    need `.to_list` first. Same method name, different result kind, and the
    difference is only visible in the sets clause.

23. **`decimals` on negatives, and on a parenthesised negative literal.**
    "halves away from zero, for money and reports" implies
    `(-0.125).decimals(2)` is `"-0.13"`, but no example shows a negative, and
    `(-0.125).decimals(2)` also probes whether a negative float literal can take
    a method at all. `y08` does both. I deliberately used only exactly
    representable binary fractions (`0.125`, `0.375`) so that only the rounding
    rule — not float representation — is under test; `2.345.decimals(2)` would
    tell you nothing, since `2.345` is not a half in binary.

24. **`decimals` on an `Int`.** `examples/stdlib.lume` has `7.decimals(1)`. The
    README's method lists give `decimals` its own prose sentence and put it in
    neither the `Str` list nor the `Int`/`Float` list. I assumed `"7.0"`.

25. **The type of `.or("?")` on a `Char?`.** Both `examples/sets.lume`
    (`s2[0].or("?")`) and `examples/chars.lume` (`word[3].or("?")`) write it, so
    a `Str` default on a `Char?` is accepted. But is the result a `Char` or a
    `Str`? It matters the moment you do anything with it — `+=` into a Str,
    `.upcase`, `.to_s`, `.code`. `y08` avoids depending on it by using
    `n.slice(0, 1)` in `initials`, and only prints the known-good
    `"lume"[0].or("?")` form.

26. **Chaining after a trailing block.** `examples/blocks_of_your_own.lume` line
    162 has `puts b.keep { |n| n > 2 }.items`, so `{...}.method` chains. But
    `f(x) { ... }!`, `f(x) { ... } == y` and `await f(x) { ... }` are not shown,
    and a `do |x|` block obviously cannot be chained at all since its body runs
    to the end of the indented region. I split all three out into bindings
    (`y07`, `y09`).

27. **`next` and `break`.** `next` appears once, in `examples/chars.lume` line
    29, and is in neither the feature list nor the "spellings other languages
    use" list — yet `continue` is explicitly rejected there "with the Lume form",
    which must mean `next`. `break` appears nowhere at all;
    `examples/generic_interfaces.lume`'s `ordered` simulates it with
    `i = out.len`, which strongly suggests there is no `break`. Both facts belong
    in the README. I used neither.

28. **Things I reached for that do not appear to exist:**
    - `xs.sum` over a user type with `+`. `sort`, `max` and `min` "follow from
      `<`", but nothing says `sum` follows from `+`. `y03` uses
      `fold(Money(0)) { |acc, m| acc + m }` instead.
    - Storing a block in a field or returning one — explicitly "not yet", which
      is fine, but it means the natural "pass a comparator around" design is
      unavailable and every such helper must take the block afresh.
    - A descending `sort` / `sort_by` — only `.reverse` afterwards.
    - `Time.now_ms` is used in `examples/blocks_of_your_own.lume` and mentioned
      in the milestone-19 row, but is absent from "what works today", which
      mentions only `await Time.sleep(ms)`.
    - `.to_s` on anything but a `Char`.
    - Any statement of how `puts` renders a `Bool`. I assumed `true`/`false`
      throughout and kept every other rendering out of my expected output by
      building strings explicitly.

29. **A field-less enum as a set item or map key.** "items and map keys may be
    `Int`, `Str`, `Bool`, tuples of those, or a struct/enum made of them —
    never a `Float`". An enum whose variants carry nothing (`enum Suit: Clubs /
    Hearts / Spades`) is not "made of" anything, so the sentence does not quite
    cover it, and neither does a struct like `Card(suit: Suit, rank: Int)` that
    holds such an enum. `y05` uses both as keys.

30. **Two `main` shapes cannot obviously be combined.** `def main`,
    `def main -> () or Error` and `async def main` each appear; `async def main
    -> () or Error` does not. Nor does any file combine `test` blocks with an
    `async def main`. `y07` therefore uses no `?` at all, and `y09` has a plain
    `def main`.

31. **`#` inside a string literal.** `#` starts a comment and `#{` starts
    interpolation; a lone `#` inside a string should be literal, but nothing says
    so, and `r"#{not interpolated}"` is the only nearby fact. I replaced a `"#"`
    with `"*"` in `y08` rather than make a table depend on it.

32. **Trailing `if`/`unless` on statements other than assignment.**
    `examples/loops.lume` shows `biggest = n if n > biggest` and
    `puts "..." if ...`. `y01`/`y05`/`y06`/`y10` use it on `+=`, on
    `out.push(x)` and on `board.add(...)`. Presumably any statement takes the
    modifier, but the README only says "trailing `if`/`unless`".

33. **`x.max(y)` and `xs.max` share a name with different shapes.** On `Int` it
    is a two-argument pick returning `Int`; on a list or set it is
    no-argument returning `T?`. `examples/generics.lume` uses both within four
    lines (`l.depth.max(r.depth)` and `largest`). Documented, but a generic
    function over `T: Ordered` that wants "the larger of two" has to know which
    one it is getting.

34. **Empty-collection behaviour is inconsistent and only partly stated.**
    `avg` on an empty list is "a `Float`; `0.0` for an empty list", while
    `first`, `last`, `max`, `min`, `pop` are all `T?`. Nothing says what `sum`
    of an empty list is (presumably `0`, but for a `[Float]`? and for a generic
    `T`?), which matters directly for `y01`'s `weigh` and `y03`'s `total`.

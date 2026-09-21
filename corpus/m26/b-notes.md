# m26 — review notes: functions that take behaviour, and closures

Twelve programs, written from `README.md`, `examples/blocks_of_your_own.lume`,
`examples/blocks.lume` and `examples/pipes.lume` only. No compiler source was
read and the compiler was not run; every expected output in a header was
worked out by hand.

## What each program probes

- **b01_block_forms** — all four ways to pass behaviour to a parameter of my
  own: inline `{ |x| ... }`, `do |x|` with an indented body, the `_`
  shorthand, and a function's name; plus `(A, B) -> C` passed after a call
  that already has two arguments, and `() -> ()` with the parentheses gone.
- **b02_closures** — a block reading an immutable binding, changing a `var`
  outside it, capturing a `var` that is *declared inside a loop* (each turn
  must get a fresh, empty one), assigning a struct field and pushing to a
  struct's list field from inside a `do` block, and `_` closing over a `var`
  whose value changes between two calls.
- **b03_nested** — a `do` block that hands its own block to another
  block-taking function; two `{ ... }` blocks nested on one line; an inner
  block reading the outer block's parameter; three levels of `do` blocks all
  writing one outer `var`, then one dedent back to the top.
- **b04_methods** — a generic struct with `each` / `keep` / `change` /
  `fold_in` methods, chained so a `}` is immediately followed by `.`; a
  method that changes the element type; `_` as a method argument; and my
  `each` winning over the list's built-in `each`.
- **b05_generic_blocks** — a bound (`Ordered`) on a type parameter that only
  ever appears as the block's *result*; a block parameter in a non-final
  position, reachable only via `_`; a block called twice per iteration;
  three type parameters; `xs.at(i)` in generic code; an empty input so the
  block never runs; and `f(x) { ... }.field` chaining.
- **b06_block_errors** — `(T) -> U or Error` parameters, `?` applied to the
  block call, `?` written *inside* the block body, a bare value auto-`Ok`,
  `match Ok/Error`, `.or` / `.error?`, and `if: ... else: Error(...)` as the
  whole body of a `T or Error` function.
- **b07_match_in_block** — a multi-line `match` as the right-hand side of a
  binding inside a `do` block, with a statement after it at the block's own
  indentation; a guard arm; `Word(w) | Punct(w)` binding the same name from
  two variants; list patterns and `Some`/`None` inside blocks.
- **b08_builtin_blocks** — `map`, `filter`, `reject`, `fold`, `sort_by`
  (both forms), `group_by` (both forms), `partition`, `flat_map`,
  `max_by`/`min_by`/`find`, `take_while`, `count`, `any?`, `all?`, `avg`, a
  lazy `filter.map.take.to_list` chain, and `enumerate`.
- **b09_maps_sets_strings** — `m[k] = m[k].or(0) + 1` written *inside a
  block*; a map's `filter`/`reject`/`each` with `|k, v|`; `map_values(_ * 2)`;
  a set's `filter`/`reject` giving a set back and `each` in insertion order;
  block methods on `s.chars` and on the lazy `s.split`.
- **b10_block_shapes** — a block whose body is one `if ...: a else: b` used
  as a value; a block whose body is one `if` with no `else`; two such
  block-`if`s in a row (the milestone 18r regression); a block returning a
  list; a block returning a tuple, then `for a, b in` and `(a, b) = `
  destructuring; a multi-statement `do` block whose last expression is its
  value.
- **b11_pass_on** — the same block parameter handed on twice in one
  expression, to two separate calls, down two levels of functions, and from
  a method to a free function twice; with a counter in the block so the
  number of inlined runs is observable (must be 4, not 2).
- **b12_block_scope_errors** — the only deliberate-error program. A block
  parameter shadowing an outer binding, and an assignment from inside a
  block to an immutable outer binding. Both are expected to be rejected;
  the header says what the only acceptable non-error behaviour would be.

## Ambiguous, surprising or under-specified in the documents I was allowed

1. **A zero-argument block in brace form is never shown.** `() -> ()` is
   documented only as `repeat do` with an indented body. Whether
   `twice { puts "tick" }` or `twice { || puts "tick" }` parses is unstated,
   so b01 uses only the `do` form. This is a real everyday shape (a `retry`
   or `time_it` call on one line) and it has no documented spelling.
2. **`enum` declarations are never spelled out.** The README shows uses
   (`Shape.Circle(1.0)`) and one piece of prose (`enum Tree: Leaf / Node(left:
   Tree, ...)`) that may or may not be syntax. b07 guesses indented variants.
   If that guess is wrong b07 fails for a reason unrelated to blocks.
3. **`var self` is named but never written.** Because of that I could not
   write a method that changes its own fields from inside a block — arguably
   the single most likely closure corner in a real program. b02 mutates a
   struct from *outside* instead, via `t.field = v` and `t.list.push(x)`.
4. **How values print is mostly undocumented.** `puts` of a `T or E`, a
   `T?`, a tuple, a nested list, a map, a set, a `Float`, or a `[Str]`
   (quoted or not?) has no example with expected output next to it —
   `blocks_of_your_own.lume` prints a `[Int] or Error` twice with no
   expectation recorded. I routed almost everything through `.join`,
   `.len`, `match` or `.decimals(n)` so my expected output is pinned; the
   one deliberate exception is the nested-list print in b10.
5. **`join` on a non-`Str` list** is listed without an element-type
   restriction but only ever demonstrated on `[Str]`. I mapped to `Str`
   first rather than depend on it.
6. **Map block arity.** Does `m.each`/`filter`/`reject` yield `|k, v|` or a
   single tuple? Not stated. b09 assumes two parameters; `map_values`
   assumes one.
7. **`partition`'s result shape.** b08 reads it as a tuple (`.0`/`.1`); it
   could equally be a two-element list.
8. **`fold`'s built-in call shape.** `xs.fold(0) { |acc, x| ... }` is assumed
   by analogy with the user-written `fold_all(xs, start) { ... }`.
9. **`min_by`/`max_by` return type.** Assumed `T?`, like `min`/`max`, so
   `.or(...)` is applied. If they return `T` the `.or` is an error.
10. **How many statements fit in an inline `{ }` block?** The feature list
    says "one statement in an inline block"; milestone 17 says "statements in
    inline blocks" (plural). I kept inline blocks to one statement
    throughout and used `do` for more.
11. **Does a `do` block have a value?** Every documented `do` block is run
    for its effect. b10 and b11 assign the result of a call whose `do`
    block's last expression is meant to be the block's value; nothing in the
    documents confirms that works.
12. **Chaining onto a free call that ends in a block.** `b.keep { ... }.items`
    is shown for a *method*. `f(x) { ... }.field` (b05) is not shown anywhere.
13. **A block parameter that is not last.** Only `_` or a function name can
    reach it; the `{ }` and `do` forms presumably cannot. This is never said,
    and it quietly constrains API design.
14. **What `?` inside a block returns from.** `blocks_of_your_own.lume` says
    "`?` inside it passes the error up, as anywhere", but the block is
    *inlined into the caller*, so "up" could mean out of the block or out of
    the enclosing function. b06 is written so both readings give the same
    output, but the two differ as soon as a block-taking function wants to
    keep going after a failing item.
15. **An untyped `[]` as an `.or` default.** An empty `[]` *binding* with no
    type is an error; `m[k].or([])` (b08, b09) is not a binding, so I assumed
    the element type is inferred from the map.
16. **`Char` versus `Str`.** `s[i]` is a `Char?`, and there is no documented
    literal for a `Char` default, so `s[i].or("?")` looks like a type error.
    b09 uses the slice `w[0..0]` instead. A `Char` literal spelling is missing.
17. **Multi-line list literals** are never shown, so b08 keeps a five-element
    struct literal on one long line.
18. **Shadowing is stated only in passing** — "nested rebinding is an error"
    and a parenthetical "no shadowing" in the corpus summary. Nothing says
    whether a *block parameter* may reuse a name from the enclosing scope,
    which is the single most common accident in closure-heavy code. b12
    exists to force an answer.
19. **A type parameter that appears only in a block's result** (b05's
    `K: Ordered`) has to be inferred from the block body, not "read off the
    call" as the README describes generics working. That is a different
    inference problem and the documents do not distinguish them.
20. **"Behaviour cannot be stored in a field or returned yet"** is the one
    clearly stated limit, so none of these programs try it — but it also
    means `map_of(xs, f)` inside `apply_twice` (b11) is the only way to reuse
    a block, and the header of b11 says what the observable call count must be.

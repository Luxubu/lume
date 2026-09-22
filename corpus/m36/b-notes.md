# Reviewer B, round seven: types and abstractions

I read `docs/README.md`, `docs/tour.md`, `docs/examples.md`, all ten
`docs/reference/*.md` pages, and in `examples/`: `structs`, `enums`,
`interfaces`, `generics`, `generic_interfaces`, `generic_extend`,
`blocks_of_your_own`, `errors_and_results`, `patterns`, `ownership`,
`stdlib`, `collections`, part of `port/graph.lume` and `mini/`
(`parser.lume`, `README.md`, `FINDINGS.md`), and about 20 files in
`examples/errors/`. I did not open `docs/QUESTIONS.md` and did not run
anything. Every expected output was worked out by hand.

## 1. The programs

| file | what it is | what it leans on |
|---|---|---|
| `b01_bank_account.lume` | accounts with validated deposit, withdraw, fee, transfer | `var self` methods returning `() or Error` / `Int or Error`; enum `Txn` with a method; `var` parameters on a free `transfer`; `?` on method calls; `main -> () or Error` |
| `b02_shapes.lume` | a shape library behind an interface | two required methods plus two defaults; structural conformance; `extend` that only overrides a default; bare method calls inside methods; `[Shape]`; `max_by`; a `-> Shape` function returning different structs |
| `b03_stack_queue.lume` | generic `Stack[T]` and a two-list `Queue[T]` | generic structs; `var self` on generic types; `T?` results; `field = []`; a `var self` method that also takes a block; a generic builder |
| `b04_expr_eval.lume` | an evaluator, printer and simplifier for arithmetic | recursive enum; `Int or Error` evaluation with `?` inside arithmetic; `.or_error`; nested literal patterns through recursive fields; a catch-all arm that binds a name |
| `b05_player_states.lume` | a media player state machine | two enums with data; `match (state, event)` with payloads in both parts; a `State or Error` transition; a `var self` method that replaces an enum field; interpolating enum values |
| `b06_inventory.lume` | a stock room with operations that can be refused | a struct holding `{Str: Item}`; `match` on a lookup then writing the same key; early `return Error` inside an arm; `() or Error` as a parameter type; the key order after an overwrite |
| `b07_retry_blocks.lume` | `retry`, `logged`, `count_where` | a generic function taking `(Int) -> T or Error`; `return` from inside a `for` and a `match`; `?` inside a `do` block; a `() -> ()` block; `_` shorthand as a block argument |
| `b08_extend_builtins.lume` | built-in types learn to describe themselves | `extend Int/Str/Bool/[T] with I`; overriding a default in one extend; two interfaces on `Int`; `[T: Describe]`; a `[Describe]` list mixing four kinds of value |
| `b09_generic_bounds.lume` | a sorted list, `min_max`, counting, `best_by` | `Ordered` met by `<` alone; `SortedList[T: Ordered]`; `(T, T)?`; a struct as a `Hashable` map key; `[T, K: Ordered]` with a block `(T) -> K` |
| `b10_interest_loop.lume` | **meant to FAIL**: a `var self` method called on a `for` loop name | expected: `` `a` is immutable, but `add_interest` changes it `` |

## 2. GUESSES

**36 guesses**

Confidence is how sure I am that my guess matches what the compiler does.

1. **Can a free function take a `var` parameter, and does the call site need to mark it?**
   (`def transfer(var src: Account, var dst: Account, ...)`, called as
   `transfer(ada, bob, 50)`.) I looked in functions-and-blocks.md,
   structs-and-enums.md and the tour, and none of them mention it. The
   syntax appears only in `examples/port/graph.lume`,
   `examples/site/markdown.lume` and two `examples/errors/var_param_*` files.
   Guess: yes, with no marker at the call, and the caller's binding must be
   `var`. **Medium.** (b01)

2. **Can a call ending in `?` stand alone as a statement when its success value is thrown away?**
   (`src.withdraw(amount)?` where the success is an `Int`.) errors.md
   covers a failure nobody looks at. It does not cover a *success* value that
   nobody uses. Guess: allowed. **Medium-high.** (b01)

3. **In a `-> () or Error` function whose last line is a statement (`history.push(...)`, `dst.deposit(amount)?`), is the implied `Ok(())` supplied?**
   errors.md says "a bare `T` returned ... is wrapped in `Ok`", but the
   `() or Error` example in the tour and in errors.md ends in an explicit
   `return ()`. `examples/errors_and_results.lume` and `port/graph.lume`
   end without one, which suggests yes. Guess: yes. **Medium-high.** (b01, b06)

4. **Does `Ok(_)` match a `() or Error`?** No example matches on a unit
   result. Guess: yes. **Medium-high.** (b01, b06)

5. **Can a method call another method of the same type by its bare name?**
   (`perimeter` inside `Triangle.area`; `square?` inside `extend Rect with Shape`.)
   structs-and-enums.md says "a *field* is written bare". Bare *method*
   calls appear only in interface defaults (`compare(other) < 0`,
   `"#{name}: #{area}"`) and in `examples/mini/parser.lume`. No page states
   the rule for methods. Guess: yes. **Medium-high.** (b02, b03)

6. **Can a no-argument `var self` method be called bare, with no `()` and no `self.`, from inside another `var self` method?**
   (`match pop:` inside `drain`.) The mini parser writes `advance()` *with*
   parentheses in this spot, and functions-and-blocks.md says "a bare name
   is a value, not a call" for functions. Guess: a bare `pop` works because
   it is a method. **Medium-low.** (b03)

7. **What may go inside `#{...}`, in particular a `{ |s| ... }` block with its own braces?**
   (`"big ones: #{shapes.filter { |s| s.area > 5.0 }.map(_.name).join(",")}"`.)
   printing.md and strings.md describe `#` and `#{` but not what the
   interpolation may contain. Guess: any expression, braces included.
   **Medium.** (b02)

8. **When two items tie, does `max_by` / `min_by` return the first or the last?**
   collections.md gives the result type (`T?`) but not the tie rule. I
   changed the data to avoid a tie. **Low** (I had no basis for either
   answer). (b02)

9. **Can a function declared `-> Shape` return two different struct types from its `if`/`else` branches?**
   interfaces.md says the return type is one of "all four places", but its
   only example returns an element of an existing `[Shape]`, and it
   stresses that a list literal of two structs needs an annotation. Guess:
   the declared return type is enough. **Medium.** (b02)

10. **Which uses of an interface print the "go through a pointer" warning, and does it go only to stderr?**
    No docs page mentions this warning. I found it only in
    `examples/interfaces.expected`, where it opens the expected output.
    Guess: it prints on stderr for an annotated mixed `[I]` binding, and
    stdout is unaffected. **Medium.** (b02, b08)

11. **Can `for` walk `list.reverse`?** control-flow.md shows
    `(1..5).reverse` in a `for`, and collections.md shows `.sort.reverse.join`.
    Neither shows `for x in xs.reverse`. Guess: yes. **Medium-high.** (b03)

12. **Inside a `var self` method, does `back = []` take its type from the field?**
    collections.md lists where an empty literal is fine: "a typed binding, a
    struct field, an argument...". "A struct field" most likely means a
    constructor argument, not assignment to a field. Guess: works.
    **Medium-high.** (b03)

13. **Which list methods exist?** The README says collections.md
    covers "every method", but that page has no full list. strings.md does
    have one for `Str`. I took `last`, `pop`, `insert(at, x)`, `enumerate`
    and `sqrt` on Float from `examples/stdlib.lume`, `structs.lume` and
    `generics.lume`. Guess: they exist with the signatures the examples
    imply (`pop` and `last` give `T?`; `insert(i, x)` inserts before `i`).
    **Medium-high.** (b03, b09, b02)

14. **Can a tail `match` in an `Int or Error` method mix arm kinds?** The
    arms in question are a bare `Int` (implied Ok), a value that is already
    `Int or Error` (`env[n].or_error(...)`, which should pass through), and
    a block with an early `return Error(...)`. errors.md shows a bare arm
    beside `Error(e) -> e`, but not beside a pass-through `T or Error`
    expression. Guess: each arm is converted separately. **Medium-high.** (b04)

15. **Can `self` be used again inside an arm of `match self` that has already bound its fields?**
    (`self.show` in the `Div(l, r)` arm.) Ownership is described as
    "inferred" (examples/ownership.lume). There is no docs page on it.
    Guess: fine. **Medium.** (b04)

16. **Can a pattern nest through a recursive enum's fields with a literal inside (`Add(Num(0), e)`, `Neg(Neg(e))`)?**
    The docs pages show only one level (`Circle(r)`, `Box(w, h)`).
    `examples/patterns.lume` nests non-recursive enums (`Wrap(A(n))`).
    Recursive nesting shows up only in `examples/mini/README.md` and
    `FINDINGS.md` item 3, which says it used to be refused. Guess: works now.
    **Medium.** (b04)

17. **Can a catch-all arm bind a name (`other -> other`, `(state, event) -> ...`)?**
    control-flow.md shows `_ ->` only. `mini/parser.lume` uses
    `other -> Error(...)`. Guess: yes. **Medium-high.** (b04, b05)

18. **When arms overlap, does the first matching arm win?** No page says so.
    The guard examples in `enums.lume` rely on it. Guess: yes, top to
    bottom. **High.** (b04, b05)

19. **Which words are reserved?** No page has a keyword list. I renamed
    `next` (a keyword, so it cannot be a binding). I avoided `from` (it might
    be used by imports) and `open`. I kept `Var` as a variant name and `at`
    as a local and a field name. Guess: only lowercase keywords are
    reserved. **Medium.** (b01, b04, b05, b09)

20. **Can a `match` on a tuple bind payloads in both parts?** (`(Playing(t, p), Tick(n))`.)
    patterns.lume matches a tuple of payload-free variants, and
    structs-and-enums.md matches `(n, s)` over plain values. Guess: yes, and
    the exhaustiveness check accepts my final binding arm. **Medium-high.** (b05)

21. **Inside a `var self` method, can the field `state` be passed by value to a function and then reassigned from the result?**
    (`match step(state, e): Ok(ns) -> state = ns`.) Guess: the compiler
    copies or moves as needed. **Medium-high.** (b05)

22. **When a map key is overwritten, does it keep its original position or move to the end?**
    collections.md: "A map keeps the order you put things in", with no
    word on overwriting. Guess: it keeps its original position, so the
    keys print `bolt,nut,gear`. **Medium.** (b06)

23. **Can `items[name] = ...` be written inside an arm of `match items[name]:`?**
    Guess: yes, because the matched value is a copy. **Medium-high.** (b06)

24. **Can a `match` whose arms are assignments (one arm also has an early `return Error`) be the last statement of a `() or Error` method?**
    Guess: yes, it is a `()` and is wrapped in `Ok`. **Medium.** (b06)

25. **Can `() or Error` be a parameter type, and does passing a result to a function count as looking at it?**
    (`def report(r: () or Error)`, `report(inv.add(...))`.) errors.md's
    "nothing looks at" rule talks about statements, `?`, `match` and `_ =`.
    Guess: an argument counts as use. **Medium.** (b06)

26. **Is a type parameter inferred from a block's result?** (`T` in
    `retry[T](..., work: (Int) -> T or Error)`, `K` in
    `best_by[T, K: Ordered](xs, key: (T) -> K)`.) generics.md: "read off the
    arguments, or off the type the result goes into". It does not say
    whether a block counts as an argument for this. `blocks_of_your_own.lume`'s
    `map_all[T, U]` suggests yes. **Medium-high.** (b07, b09)

27. **What does `return` do inside a block: leave the block or leave the enclosing function?**
    functions-and-blocks.md settles this for `?` ("leaves the block, never
    the function") but says nothing about `return`. I avoided it. **Low.** (b07)

28. **Can `extend [T] with I` be used when `I` itself takes no type parameter?**
    generics.md shows generic extends only with generic interfaces
    (`Bag[T]`, `Ranked[T]`). Guess: yes, because a name in the target that
    is not a type becomes a parameter. **Medium.** (b08)

29. **Can a list value (`[1, 2]`) sit in a `[Describe]` list literal next to an Int, a Str and a Bool, conforming through a generic `extend [T]`?**
    Guess: yes. **Medium-low.** (b08)

30. **Can one built-in (`Int`) be extended with two interfaces in two separate `extend` blocks?**
    Every docs example extends a type once. Guess: yes. **Medium-high.** (b08)

31. **Can `Bool` be extended?** interfaces.md shows `Int`, `Str` and
    `[Int]`, and says "It works on the built-ins too". Guess: `Bool`
    included. **High.** (b08)

32. **How is an optional tuple type spelled?** Guess: `(T, T)?`. There is
    no example of a `?` after a parenthesised type. **Medium.** (b09)

33. **Can a bounded generic struct (`SortedList[T: Ordered]`) be built inside a generic function whose own `T` carries the same bound?**
    generics.md: "the bound is checked where the value is built". Guess:
    the function's bound satisfies it. **Medium-high.** (b09)

34. **Is the name a plain `for` binds immutable as far as `var self` methods go, does `for var` exist, and what is the error text?**
    structs-and-enums.md says a `var self` method needs a `var` binding.
    It says nothing about loop names. `for var` appears only in
    `examples/ownership.lume` and `examples/errors/for_var_*`. Guess: the
    same `` `a` is immutable, but `add_interest` changes it `` message as for
    a `let`-style binding. **Medium.** (b10)

35. **Is there any way to write a constructor-like function on a type (`Account.empty()`, a static or associated function)?**
    structs-and-enums.md, functions-and-blocks.md and the tour say nothing
    either way. I wrote a free `open_account` function. **Low** (on whether
    one exists).

36. **Can a block with one argument follow a call with no parentheses (`find_first { |i| ... }`)?**
    functions-and-blocks.md says only the no-argument form needs `()` or
    `do`, and its bad example is `twice { puts "tick" }`. The comment in
    `examples/blocks_of_your_own.lume` says "When the block is the only
    argument, the parentheses can go: write `find_first { |i| ... }`". I
    did not rely on it, and always wrote the `()`. **Low.**

## 3. ANSWERED

These are things I expected to guess and found stated clearly:

- **`var self`**: the spelling (`def bump(var self, by: Int)`), that it is
  never passed at the call, the error without it, and that the receiver
  must be a `var` binding. The error messages are quoted.
- **Enum shape**: one variant per line, every payload field named, no
  commas or `|`. Methods sit under the variants and `match self`.
  Positional variant construction works (`Rect(2.0, 3.0)`).
- **Bare and qualified variant names**, and the ambiguity error.
- **Recursive enums need no `Box`**, and they print in full.
- **Implied `Ok`** for a bare value, pass-through with no `Ok(Ok(...))`,
  and `Error(e) -> e` rather than `Error(e) -> Error(e)`. These were exactly
  the questions I had. So was "`Error` has exactly one field, `message`".
- **`?`**: where it is allowed, that a `T?` and a `T or Error` do not mix,
  and `.or_error(msg)` as the bridge. `?` in a block leaves the block, and
  only in a block typed `-> T or Error`. There is no implied `Some` in a
  block, which the page calls a gap.
- **Interface defaults** may call required methods; an `extend` may
  override a default but not a method the type already wrote, and may not
  add extra methods. A field does not satisfy an interface method, and the
  fix is spelled out.
- **Interfaces in all four positions** (parameter, list element, field,
  return), and the need to annotate a mixed list.
- **`<` alone gives `Ordered`**, `sort`, `max`, `min`, `<=`, `>`, `>=`.
  `Hashable` is automatic, and there is one bound per parameter (the
  `+` and comma traps are explained).
- **What a bare `T` can do**: `==` and interpolation, but not `<`.
- **Blocks of your own**: the `(A) -> B` type and the `() -> ()` call
  forms. Blocks cannot be stored, which answered the "store a callback in a
  field" question before I asked it, and the enum alternative is shown.
- **Printing**: a Str inside a value prints quoted, interpolation is
  value-as-itself, and a Float keeps `.0`.
- **Map order is insertion order.** `for k, v in map` is shown.
- **`if` as a one-line value** with `elif`, and `-x` on any expression.

## 4. CONFUSING OR WRONG

1. **An example comment contradicts the reference on block call syntax.**
   `examples/blocks_of_your_own.lume` says: "When the block is the only
   argument, the parentheses can go: write `find_first { |i| ... }` or
   `repeat do`". functions-and-blocks.md says: "a bare `name { ... }` with
   nothing before the brace is not a call the parser will take". The
   reference's bad example is the no-argument case. The example's
   `find_first` takes an argument and does not exist anywhere in that
   file. One of the two is stale, or the rule depends on whether the block
   has arguments and nothing says so.
2. **The reference pages leave out whole features that the examples use.**
   None of these appear in `docs/reference`, and a second-week user will
   need several of them:
   - `var` parameters on free functions
   - `for var x in xs`
   - match guards (`Circle(r) if r > 10.0 ->`)
   - list patterns (`[first, ..]`)
   - range patterns (`1..9 ->`)
   - `Variant(..)`
   - nested patterns through recursive enums
   - top-level constants (mini FINDINGS item 1: "top-level `NAME = value` constants")
   - generic methods (`def map_to[U]`)
   - a method's own name winning over a free function

   control-flow.md's `match` section covers only literal arms, `|` and
   `_`.
3. **"every method" is promised for collections and not delivered.** The
   README table says collections.md has "every method", but the page
   covers about fifteen. `last`, `pop`, `insert`, `remove_at`, `uniq`,
   `zip`, `flat_map`, `group_by`, `partition`, `find`, `take_while`,
   `sort_by`, `sum(block)` and the numeric `sqrt`, `pow`, `clamp` exist
   only in `examples/stdlib.lume` and `enums.lume`. strings.md does print
   the full list for `Str`. The list and number pages need the same.
4. **Implied `Some` in a `-> T?` function is shown but never stated.**
   errors.md's `head` returns `first + 0` and prints `Some(5)`, so a bare
   value is wrapped in `Some`. The tour's `half` writes `Some(n / 2)` by
   hand, and functions-and-blocks.md says a *block* has "no matching
   implied `Some`". A reader cannot tell whether functions have one without
   noticing the `+ 0` example. (Why `+ 0`?)
5. **Top-level statements versus `def main`.** Many snippets are bare
   top-level statements (`xs = [3, 1, 2]` / `puts ...`). Others wrap the
   same thing in `def main:`. errors.md's bare `n = "x".to_int?` example
   fails with "`main` does not return `T or E`". No page says that a file
   of top-level statements is an implicit `main`, or when that is allowed
   (for example, alongside `struct` definitions).
6. **Mixed interface lists print a warning, and the docs never say so.**
   `examples/interfaces.expected` begins with a warning ("calls on its
   items go through a pointer"). interfaces.md recommends exactly that
   pattern ("say `[Shape]` when you mean the mixture") with no mention of
   the warning.
7. **Methods called bare.** The docs stress that *fields* are bare inside
   methods, and use bare *method* calls in interface defaults, without
   ever stating that methods are bare too, or whether a zero-argument one
   then needs `()`. The mini example writes `advance()` in one place and
   `peek` in another.
8. **`() or Error` functions.** The tour and errors.md both end their
   `main -> () or Error` with `return ()`, which suggests it is required.
   The examples show it is not. errors.md also admits that a stray
   `Error(...)` statement in such a function "is currently accepted and
   silently discarded". That is honest, but it means the rule this page
   calls the most important is weakest in exactly the kind of function a
   `var self` mutator tends to be.

## 5. Verdict

Yes, mostly. For the second-week work of designing your own types, the
reference pages answer the questions that used to be guessed:

- the enum spelling
- `var self`
- implied `Ok` and `Error(e) -> e`
- `?` in blocks
- interfaces versus fields
- bounds
- blocks that cannot be stored

The page-per-question style works, and the quoted error messages are the
most useful part. What a new user will hit is the gap between the
reference and the examples. Many features the examples rely on are on no
reference page: `var` parameters, `for var`, match guards, list and range
patterns, nested patterns, `Variant(..)`, constants, bare method calls, and
most list methods. A user who reads only `docs/` will not know these exist;
a user who reads the examples has to reverse-engineer the rules.

**The single change that would help most** is a "patterns and mutation"
section in structs-and-enums.md (or a patterns page), plus a full method
list for lists, maps, sets, numbers, `T?` and `T or Error`, printed the way
strings.md already prints one for `Str`. Between them they would remove
about a third of my guesses (1, 6, 11, 13, 16, 17, 18, 32, 34).

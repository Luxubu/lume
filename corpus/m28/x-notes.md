# m28 review — generic `extend`, parameterised interfaces, conforming containers

Twelve programs, written from `README.md` and the four top-level examples
(`generic_extend.lume`, `generic_interfaces.lume`, `interfaces.lume`,
`generics.lume`) only. The compiler, the test suite and the other corpora were
not read, and nothing here has been run. Every expected output was worked out
by hand and checked twice.

Ten programs are meant to run; **x11 and x12 are deliberate-error programs**
and say so in their headers.

Method names in the running programs were chosen to *avoid* colliding with the
extended container's built-ins (`tally_count` rather than `count`, `first_row`
rather than `first`), so that each program tests one thing. **x09 is the one
place where the collisions are deliberate** — see finding 3.

---

## 1. What each program probes

| # | File | What it probes |
| --- | --- | --- |
| x01 | `x01_two_bags_one_list.lume` | Two *different* parameterised interfaces generically extended onto the **same** target (`[T]` gets both `Bag[T]` and `Joined[T]`), and one of them reaching `[T]`, `{T}`, `{K: V}` at `V` and a two-field user generic. A default that takes an argument (`glued(sep)`) and a default written on top of it (`width`). One `Bag[Str]`-bounded helper over all four shapes, with `pad_right` alignment so the output is character-exact. |
| x02 | `x02_layered_defaults.lume` | Defaults stacked **four deep** — `count` → `empty_ledger?` → `summary` → `banner(title)` → `shout(title)` — reached through a generic `extend` where the container only ever supplied `entries`. Includes a default that calls a *free generic function* rather than a built-in, a default that takes an argument and one that calls that one, and the empty-container branch on a `[Int]`. |
| x03 | `x03_argument_only_param.lume` | An interface parameter that appears **only in an argument**, never a result (`def accepts?(x: T) -> Bool`), so conformance must be read backwards. `extend [T: Hashable]`, `extend {T}` (implied bound) and `extend {K: V} with Accepts[K]` — conformance at the **key**, which the README's examples never take. Plus a plain struct inferred as `Accepts[Int]` with `Int` written only once, and a bare `contains?(x)` on self in one extend against an explicit `self.` in another. |
| x04 | `x04_nested_containers.lume` | Generic `extend` on **nested** targets: `extend [[T]] with Flat[T]` (parameter two levels down, interface takes the *inner* type) and `extend {Str: [T]} with Flat[T]` (a target mixing a concrete type with a parameter, parameter again nested). Plus `Grid[T]` and a **non-generic** `Wordbook` whose interface argument (`Str`) has to be worked out from a method's result type and appears nowhere as a type argument. Two helpers bounded at different arguments. |
| x05 | `x05_bounds_on_extend.lume` | Generic `extend` against all three kinds of bound at once, three of them live on `[T]` in one file: `Ordered` (`interface Ranked[T: Ordered]`, bound repeated on the list extend, omitted on the set extend), `Hashable` (a default building a `{T: Int}`), and a **bound that is itself an interface** (`Roster[T: Titled]`) whose default calls `Titled`'s *own default*. `ladder_of[T: Ordered, R: Ranked[T]]` puts `T` only inside another parameter's bound. |
| x06 | `x06_tuple_and_optional.lume` | Generic `extend` on shapes the README does not enumerate: a **tuple** (`extend (A, B) with Pairish[A, B]`, two parameters from one target, `self.0`/`self.1` in the body, a default returning `(B, A)`) and an **optional** (`extend T? with Holder[T]`, `match self` over `Some`/`None` inside the extend). A helper bounded on an interface with both arguments pinned. The program I am least sure about, and it says so. |
| x07 | `x07_two_params_one_conform.lume` | A user generic with **two** parameters conforming at one of them — and to a second interface at the *other* one, both live at once (`extend Pair[A, B] with Bag[B]` beside `extend Pair[A, B] with Labelled[A]`), so each extend has a parameter the interface never mentions. `Tagged[L, R]` whose conforming field is already an `[R]`. `describe[T, B: Bag[T]]` leaves the argument open rather than fixing it to `Bag[Str]`. A `Pair` inside a `Pair`, so `T` is itself a user generic. |
| x08 | `x08_interface_values.lume` | A **built-in container held as an interface value**: `[Bag[Int]]` holding a `[Int]`, a `{Int}`, a `{Str: Int}` and a `Crate[Int]` together, iterated with `for`. A parameterised interface as a **struct field type** (`cargo: Bag[Int]`), filled once with a user type and once with a bare list literal, with a method of the holding struct calling the interface's defaults. A set literal going straight into an interface-typed binding. |
| x09 | `x09_shadowing_builtins.lume` | Interface **defaults whose names collide with the extended container's built-ins**: `count` (built-in takes a block, this one does not), `first` (built-in is a `T?`, this one a `Str`), `sum` (built-in numeric, this one a `Str`) and `contains?` (built-in takes a `T`, this one a `Str`, so on a `[Int]` they differ in argument type). `Chain[T]` then writes `count` itself, so a type's own method, an interface default and a built-in all collide on one name. Deliberately avoids overriding `len` — see finding 3. |
| x10 | `x10_tests_blocks_results.lume` | Generic `extend` meeting the rest of the language: `pub interface`/`pub struct`/`pub def`, `test` blocks with `== Some(...)` and `== Ok(...)`, a default returning a `T?`, a default returning a `T or Error` with the implied `Ok` and a `match` at the call site, a default that takes a **block** and hands it to a built-in (`rows.find(ok)`), and a default carrying **its own extra type parameter** (`mapped[U]`) called with a block and with the `_` shorthand. |
| x11 | `x11_error_missing_method.lume` | **Deliberate error.** A generic `extend [T] with Shelf[T]` that supplies `stock` and never supplies the required `label`. Asks whether "an `extend` that leaves a method out" still fires when the target is `[T]` rather than a named type, and whether the message points at the extend, names `label`, and is not confused by the three defaults that *are* present. A correct `{K: V}` extend sits beside it for contrast. |
| x12 | `x12_error_argument_mismatch.lume` | **Deliberate error.** `extend [T] with Bag[Str]` whose `items` gives back `[T]` — a fixed interface argument against a parameterised target, the mistake you make copying `extend Int with Measures[Str]`. Asks for a signature-comparison error at the extend, not "unknown type `T`" and not a silent acceptance that fails later at a call site. A second mistake (`extend Poimt with Named`, the README's own example of an unknown type rather than a silent parameter) is at the bottom, commented out because the compiler will stop at the first. |

---

## 2. Ambiguous, surprising or under-specified in the README

Ordered roughly by how much they cost me while writing. Quotes are from
`README.md` unless marked.

1. **How does an `extend` cross a module boundary? There is no answer at all.**
   The README says *"only `pub` items cross a file boundary"* and *"`pub
   interface` crosses modules"*, but an `extend` has no name, so there is
   nothing to mark `pub`. With milestone 27 this stops being a detail:
   `extend [T] with Bag[T]` is a statement about **every list in the
   program**. Does it apply in a module that imports the file, in a module the
   file imports, or only in the file it is written in? May two modules each
   write `extend [T] with Bag[T]` (Rust would call this a coherence
   violation)? May a module extend `[T]` with an interface it did not
   declare? I wrote no `pub extend` in x10 and left a comment saying why —
   this is the single biggest hole I found.

2. **Two generic extends on the same target are never shown, and the clash
   rule is unstated.** x01 puts `extend [T] with Bag[T]` and `extend [T] with
   Joined[T]` in one file. Nothing says this is allowed. Nothing says what
   happens if both interfaces declare a default called `size` — first wins,
   last wins, error at the extend, or error only at the ambiguous call. Since
   defaults become *"the type's own methods now, with no helper in between"*
   (`generic_extend.lume`), two interfaces over one shape is the normal case,
   not an exotic one.

3. **Defaults landing on a container shadow its built-ins — or do they? — and
   the recursion hazard is unmentioned.** The only sentence on shadowing is in
   the *blocks* paragraph: *"a method of your own wins over the built-in of the
   same name"*, and its example is `bag.each` on a struct you wrote. x09 asks
   whether an interface **default** reaching `[T]` shadows the list's `count`,
   `first`, `sum` and `contains?`. The sharp corner: if it does, then

   ```ruby
   interface Sized[T]:
     def raw -> [T]
     def len -> Int = raw.len     # `raw` is a [T], which now has this `len`
   ```

   is infinite recursion with nothing in the source that looks recursive. I
   left `len` alone in x09 rather than hang the suite on it, but the language
   should either forbid shadowing a built-in from a default or say that
   built-ins win inside default bodies.

4. **Overlapping generic extends: `[[Int]]` matches two patterns.** With both
   `extend [T] with Bag[T]` and `extend [[T]] with Flat[T]` in scope, a
   `[[Int]]` is simultaneously a `Bag[[Int]]` (T = `[Int]`) and a `Flat[Int]`
   (T = `Int`). No specificity rule is stated. I kept those two extends in
   different files (x04 vs x07/x09) to avoid finding out, but x07 and x09 both
   do rely on `[[Int]]` matching a plain `extend [T]`.

5. **The enumeration and the general rule disagree about which shapes may be
   extended.** The rule is *"A name inside the target that is no type of yours
   is a parameter"*, which is fully general. The enumeration right before it is
   *"`extend [T] with Bag[T]:`, `extend {T} with Bag[T]:`, `extend {K: V} with
   Bag[V]:` and `extend Stack[T] with Bag[T]:`"* — four shapes, all one level
   deep, none of them a tuple or an optional. Lume has `(A, B)` and `T?` as
   first-class shapes, and they are the next two things anyone will try. x06
   writes both. x04 writes `[[T]]` and `{Str: [T]}`. If any of these is not
   supported the README should list what is; if they all are, the four
   examples read as exhaustive when they are not.

6. **May a target introduce a parameter the interface never uses?** x07 has
   `extend Pair[A, B] with Bag[B]` (`A` unused) and `extend Pair[A, B] with
   Labelled[A]` (`B` unused). This is exactly the shape Rust restricts with its
   coverage rule, and the README says nothing either way. Related: nothing says
   the interface's arguments must be a *function* of the target's parameters,
   which is what makes x12's error an error.

7. **The bound goes in two places, and which one is authoritative is unclear.**
   `generic_extend.lume` writes **both**

   ```ruby
   interface Ranked[T: Ordered]:
   extend [T: Ordered] with Ranked[T]:    # bound repeated
   extend {T} with Ranked[T]:             # bound omitted
   ```

   with the explanation *"inside `{...}` a `:` already means a map, so a set's
   item takes the bound its interface declared"*. So the set form inherits and
   the list form repeats. Is `extend [T] with Ranked[T]` (list, no bound) an
   error, or does it inherit too? Must the extend's bound *match* the
   interface's, or may it be stronger? x05 repeats the bound everywhere a list
   is the target and omits it on sets, mirroring the example, but I was
   guessing.

8. **There is no equality bound.** *"`[T: Ordered]` for `<`, `[T: Hashable]` to
   be a map key or set item, and any interface name for its methods"*. Nothing
   gives `==`, yet the list's own `contains?` and `index_of` clearly need it.
   In x03 I wrote `extend [T: Hashable] with Accepts[T]` purely so `contains?`
   would be legal — a strictly stronger requirement than I wanted, and possibly
   still not the right one.

9. **`"#{x}"` on an unbounded `T` is assumed to always work, and is never
   stated.** Every `listed`/`shown`/`ladder` default in these programs — and
   `generic_extend.lume`'s own `def listed -> Str = items.map { |x| "#{x}"
   }.to_list.join(", ")` — interpolates a value of an unbounded parameter.
   There is no `Display`/`Show` bound in the bound list, so this must be
   universal, but the README never says so. It is the one capability every
   generic-extend example depends on.

10. **Does `join` accept an `[Int]`?** The README lists `join` under *"on
    lists"* with no item-type restriction, but every example maps to
    `"#{x}"` first. I worked around it in x03 and x05 (with a comment saying
    why), which made two lines uglier than a user would write them.

11. **Map `contains?`: key or value?** *"on maps `len`, `keys`, `values`,
    `get`, `remove`, `contains?`, `merge`..."*. Sets have `contains?` for an
    item; for a map either reading is plausible. x03 uses
    `self.keys.contains?(x)` to dodge it.

12. **`keys` and `values` are not described consistently.**
    `generic_extend.lume` writes `self.values.to_list`, implying `values` is
    lazy; `collections.lume` writes `idx.keys.sort` with no `to_list`. Either
    `keys` and `values` differ, or `to_list` is a no-op on a list and the two
    examples are just inconsistent. In generic code — where *"the read generic
    code needs"* `at` exists precisely because there is no fallback — that
    difference matters.

13. **Conformance inferred from a *result* type, for a non-generic type.** The
    README's worked example is an argument: *"`Version` with `def
    compare(other: Version)` is a `Comparable[Version]` without naming it"*.
    x04's `Wordbook` has only `def cells -> [Str]` and must therefore be a
    `Flat[Str]`, with `Str` written nowhere as a type argument. That direction
    is implied by `Bag` but never stated, and it raises a question the README
    does not: if two interfaces in scope both require `cells -> [Str]`, does
    the type conform to both, and is a call to a default they share ambiguous?

14. **A parameter that is *only* an argument.** `interface Accepts[T]: def
    accepts?(x: T) -> Bool` is the same shape as `Renders[T]`, but
    `generic_interfaces.lume` only ever uses `Renders` with the argument
    written out (`Renders[Str]` as a value type, `extend Int with
    Measures[Str]`). Nobody ever *infers* `T` from an argument-only position.
    x03 makes `Span` with `def accepts?(x: Int)` do it. Unclear what happens if
    a type has two methods that could each fit the interface at different
    arguments.

15. **Interface methods with their own extra type parameters.** x10 writes `def
    mapped[U](f: (T) -> U) -> [U]` as an interface **default**.
    `blocks_of_your_own.lume` has `def map_to[U]` on a struct, so per-method
    parameters exist; nothing says an interface method may add one on top of
    the interface's own, or how that composes with a bound like `[S:
    Source[Str]]`.

16. **Blocks inside interface defaults, reached through a bound.** The blocks
    paragraph promises *"the block is compiled into the call, so there is no
    boxing and no lookup at run time"* and, separately, *"behaviour cannot be
    stored in a field or returned yet"*. x10's `def pick(ok: (T) -> Bool) -> T?
    = rows.find(ok)` is a default taking behaviour, on a built-in container,
    reached through a bound. That is three features at once and no paragraph
    covers the combination.

17. **A bound interface's *defaults* inside another interface's default.**
    x05's `interface Roster[T: Titled]` has `def marquee = members.map { |m|
    m.shout }...`, and `shout` is `Titled`'s default, not a required method.
    Whether a bound brings a type's *defaults* into scope or only its required
    methods is not stated.

18. **Several extends on the same target shape with different bounds.** x05 has
    `extend [T: Ordered]`, `extend [T: Hashable]` and `extend [T: Titled]` all
    on `[T]`, expecting `[Int]` to pick up two and `[Track]` the third. That
    bound-directed selection is the obvious reading but is never described, and
    it interacts badly with finding 2 if two of them share a default name.

19. **Can a built-in container sit behind an interface pointer?** *"`[Shape]`
    holds mixed types behind a pointer and says so once"*. Every value that
    goes behind an interface in the examples is a struct. x08's `[Bag[Int]]`
    holds a `[Int]`, a `{Int}`, a `{Str: Int}` and a `Crate[Int]` — a list of
    pointers, one of which points at a list. Also, *"says so once"* is never
    explained: is that a note the compiler prints? Would x08 print it four
    times, once, or not at all — and would that show up in the expected stdout
    I wrote?

20. **A parameterised interface as a struct field type.** The README gives
    *"`Renders[Str]` as a value type"* and a `[Shape]` field is implied by the
    `Shape` examples, but a field declared `cargo: Bag[Int]` (x08) combines
    them, and the ownership paragraph's *"a borrowing crate type in a field"*
    error shows fields are where representation choices bite.

21. **`puts` of a `T?` has no documented rendering.** `generic_extend.lume`
    itself writes `puts [3, 1, 2].best` and `puts Stack(vals: [7, 8]).items`,
    so those forms are fine, but the README never fixes the printed text of
    `Some(9)` versus `9`. I used `.or(...)` everywhere rather than guess, which
    is why x05 prints `9` and not `Some(9)`.

22. **One-liner method bodies: `:` or `=`?** *"functions (`def` block and
    one-liner forms...)"* is about functions. Inside interfaces and `extend`
    blocks, `generic_extend.lume` uses `def size -> Int = items.len`. Inside
    `struct` bodies, every example uses the indented block form. So I could not
    tell whether `struct Track: def title -> Str = name` is legal, and used the
    block form for struct methods throughout — see x05, x09, x11.

23. **`== Ok(...)` in an assert.** `generic_interfaces.lume` asserts `... ==
    Some(Weight(grams: 9))`, so `T?` has `==`. x10 asserts `scan(["p", "q"]) ==
    Ok("2:p")`. Nothing says a `T or Error` compares, and nothing says whether
    two `Error` values compare at all (`Error` has `.message` and little else).

24. **How deep does the `_` shorthand go?** The documented forms are
    `xs.map(_.name)` (a field), `each(xs, _ * 2)` and `map_all([1,2,3], _ *
    10)`. x10 uses `_.upcase` — a no-argument *method*, not a field — on the
    strength of `_.name`. I did not dare write `_.len == 2` (shorthand under a
    comparison) and spelled it out as a block instead; the README should say
    where the shorthand stops.

25. **`describe[B: Bag[Str]]` versus `describe[T, B: Bag[T]]`.** The
    milestone-27 example fixes the argument. x07 leaves it open, on the
    strength of `keys_of[K, V, T: Keyed[K, V]]` from milestone 25 — but that
    one has the parameters in *result* position. Whether the open form works
    for generic-extend conformance is a guess.

26. **`lume fmt` on the new targets.** The suite *"checks that formatting every
    example is idempotent and leaves the generated Rust unchanged"*, but the
    README says nothing about the canonical layout of `extend {Str: [T]} with
    Flat[T]:`, `extend [[T]] with Flat[T]:` or `extend (A, B) with Pairish[A,
    B]:`. Worth checking that x04 and x06 survive a format round-trip.

27. **Things I reached for that do not appear to exist at all.**
    - No way to *ask* whether a type conforms — no `is Bag[Str]` test, no
      conformance pattern in a `match`, no `where` clause on an `extend`.
    - No associated type: an interface cannot say "the thing I hold" without
      the caller naming it, which is why x07 needs the two-parameter
      `describe[T, B: Bag[T]]` dance to write something as ordinary as "print
      any bag".
    - No private helper inside an `extend` — every method in an `extend` block
      is presumably part of the conformance, so a shared piece of workings has
      to become a free function (x02's `count_of`).
    - No equality bound (finding 8) and no display bound (finding 9).
    - No way to give a default *per target* — e.g. to say "sets get this
      cheaper `size`" — without writing the method out in each `extend`.

28. **The error list does not obviously cover x12's mistake.** The nearest
    entries are *"a value used as an interface it does not satisfy (naming the
    missing method or the signature that differs)"* and *"an `extend` that
    leaves a method out"*. An extend whose *arguments* cannot be satisfied for
    every `T` is neither. If milestone 27 added a rule here, the error list
    should gain a line for it.

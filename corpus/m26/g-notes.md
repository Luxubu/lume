# m26 review notes — user-defined generics and interfaces with type parameters

Written from `README.md`, `examples/generics.lume`, `examples/generic_interfaces.lume`
and `examples/interfaces.lume` only. The compiler was never read and never run;
every expected output in the program headers was worked out by hand.

## What each program probes

- **g01_generic_fns** — inference for one and two type parameters: from an
  argument, from a nested generic call, from two arguments that must agree,
  from a tuple's parts, and (the interesting one) `blank[T](n: Int) -> [T]`
  where `T` appears *only* in the return type and has to come off the typed
  binding. Also a generic function taking a `(T) -> T` block, called in the
  trailing-block form.
- **g02_generic_structs** — generic structs with one and two parameters,
  `Box[Box[Int]]` / `Box[Box[Str]]` (a generic holding a generic, once
  inferred and once annotated), `swapped -> Pair[B, A]`, a generic *method*
  `with_left[C]` that adds a third parameter to a two-parameter struct, and
  `Counter[Pair[Int, Str]]` started from an empty list.
- **g03_generic_enums** — a recursive generic enum `Chain[T]` (the boxed
  field's type mentions `T`), methods that `match self` and recurse,
  `Chain[Chain[Int]]`, a bare `Empty` typed only by its context, a
  two-parameter `Either[L, R]`, and `fold_chain[T, A]` which passes its block
  parameter on to its own recursive call.
- **g04_ordered_hashable** — `Ordered` at Int, Str and Float through one
  insertion sort; `Ordered` returning `(T, T)?` with `xs[0]?` as the empty
  guard; `Hashable` as a set item and a map key, satisfied by a user struct
  and by a tuple as well as by scalars; and `invert[K: Hashable, V: Hashable]`,
  which uses `V` as a key in its result.
- **g05_interface_bounds** — plain interfaces: defaults, one type overriding
  a default and one not, `[T: Named]` at two structs and at `Int` via
  `extend`, two parameters with two different interface bounds, a bound on a
  generic *struct*'s parameter, and an interface as a struct *field* type.
- **g06_comparable** — `interface Comparable[T]`, the `[T: Comparable[T]]`
  bound on three functions and on a generic struct, a `compare` that reverses
  the order (so the defaults must really route through the type's method), and
  `Ruler`, whose `compare(other: Int)` makes it a `Comparable[Int]` and
  nothing else — used both for its inherited `before?(Int)` and for a
  `[T: Comparable[Int]]` bound.
- **g07_interface_values** — `Renders[Str]` and `Renders[Int]` live in one
  program: interface-as-parameter, `[Renders[Str]]` holding two unrelated
  structs, `[Renders[Int]]` holding a struct and an extended `Str`, a default
  method reached on a built-in that conforms only through `extend`, and
  `render_list[T, R: Renders[T]]` where `T` is pinned from both the bound and
  an argument.
- **g08_generic_methods** — generic methods on a generic type whose extra
  parameters come only from a block: `map_to[U]` (U from the block's result),
  `fold_into[A]`, `zip_with[U, V]` (three parameters at once), plus `_`
  shorthand blocks on a method and chained trailing blocks, and `Seq[Seq[Int]]`.
- **g09_generic_results** — generics crossed with `T?` and `T or Error`:
  `.or_error` in a generic body, `?` inside a `for` loop in a function
  returning `Int or Error`, `?` in a function returning `(T, T) or Error`
  with the bare tuple becoming `Ok`, a `Hashable`-keyed generic map lookup,
  and a generic `first_where` with `return Some(x)`.
- **g10_generic_collections** — a generic struct whose field is `{K: [V]}`,
  started from `{}`; `V` instantiated at a list type; `K` instantiated at a
  tuple; generic set algebra; and a generic `grouped` over `[(K, V)]` at two
  different key/value pairings.
- **g11_missing_bounds** — *deliberate error*: `{T: Int}` and `<` on a bare
  `T` with no bound, beside correctly-bounded twins. Should be rejected at the
  definition, not at the instantiation, and must not leak a rustc `Hash`/
  `PartialOrd` message.
- **g12_conformance_mismatch** — *deliberate error*: `Cell` has
  `compare(other: Int)`, so it is a `Comparable[Int]`; passing `[Cell]` to
  `[T: Comparable[T]]` needs `Comparable[Cell]`, which it is not. The same
  type is used correctly elsewhere in the file, so a checker that just looks
  for "a method named `compare`", or that solves `Comparable[?]` before
  pinning `T`, will wrongly accept it.

## Ambiguous, surprising or under-specified in the README

1. **How values print.** `puts` on an optional, a list, a map, a set, a tuple
   or a struct has no documented spelling. `examples/generics.lume` prints
   `first([3, 1, 2])` (a `T?`) and `tally(...)` (a map) with no expected output
   beside them. I avoided depending on this almost everywhere — the programs
   unwrap with `.or`, `match`, or build strings — but it is a real gap for a
   reviewer writing expected output by hand.
2. **`join` on a non-`Str` list.** The README lists `join` under list methods
   with no element-type restriction, and lists it for sets next to `sum`/`max`.
   g04 and g10 join `[Int]` and `[Float]` lists (and g04 also depends on how a
   `Float` renders: I assumed `2.5`, `3.25`, `1.5`). If `join` is `Str`-only,
   say so where the method is listed.
3. **Bounds on a generic type's parameters.** Every documented bound is on a
   function (`def largest[T: Ordered]`). Whether `struct Stack[T: Ordered]:` /
   `enum E[T: Hashable]:` is legal is never said, and it is the first thing
   anyone writing a `SortedList` needs. g05, g06 and g10 all assume yes.
4. **Does a `{K: V}` parameter imply `K: Hashable`?** `def f[K, V](m: {K: V})`
   ought to be rejected (or the bound inferred) but the README does not say
   which. g10 writes the bound explicitly; g11 tests the unbounded body.
5. **Two bounds on one parameter.** There is no syntax shown for
   `[T: Ordered + Hashable]` or `[T: Named, Sized]`. This is common enough
   (sort-then-dedup) that its absence is surprising; I worked around it.
6. **`extend` and interface defaults.** "an `extend` that leaves a method out"
   is listed as an error, but a conforming type is also said to get the
   defaults as its own methods. `examples/interfaces.lume` supplies `name`
   even though it has a default, and `generic_interfaces.lume`'s extends have
   no defaults to skip — so the example set never settles it. g05's
   `extend Int with Named:` supplies only the required method.
7. **An interface as a field type.** Interfaces are documented as parameter
   types and as list element types (`[Shape]`, "behind a pointer and says so
   once"), never as a struct field type. g05's `Desk.owner: Named` assumes it
   works; if it does not, the error should say so rather than "unknown type".
8. **Zero-parameter generic functions.** `def main:` and `def title -> Str:`
   drop the parentheses, so a generic function with no value parameters would
   presumably be `def none_of[T] -> T?:`. That reads oddly and is never shown.
   I avoided it by giving `blank` a dummy `Int` parameter, but the form should
   be settled.
9. **Generic methods that add their own parameters.** `def map_to[U](...)`
   inside `struct Seq[T]:` is the natural way to write `map`, and nothing in
   the README forbids it, but no example has a method with its own `[U]`.
   g02 and g08 lean on it heavily.
10. **Blocks handed to recursive generic calls.** "a block is compiled into
    the call, so there is no boxing" plus "a block parameter can be handed on
    to another function" are hard to reconcile for a *recursive* handoff
    (`fold_chain` in g03). Monomorphising that needs an infinitely deep closure
    type unless something special happens. If it is not supported, it should be
    a named error, not a rustc recursion-limit message.
11. **Blocks as method parameters.** All the block examples are free
    functions. `s.map_to { |x| ... }` and `s.fold_into(0) { |a, x| ... }`
    (g08) assume methods take blocks the same way, including the trailing-block
    form and the `_` shorthand.
12. **`{}` and `[]` as constructor arguments.** `var seen: {Str} = {}` is
    documented, and "`{}` outside a typed binding" was an m18r defect. g10
    writes `Index(slots: {})` where only the *binding's* type says what `{}`
    is — one level further out than the documented case. Same question for
    `.or([])` in g10's `Index.get`.
13. **Parameterised interfaces at two arguments in one program.** Nothing says
    a type may conform to `Renders[Str]` and another to `Renders[Int]` at the
    same time, or that `[Renders[Int]]` and `[Renders[Str]]` are distinct value
    types. g07 assumes both.
14. **Conformance at an argument other than `Self`.** `extend Int with
    Measures[Str]` shows it for `extend`, but not for a plain struct: g06's
    `Ruler` conforms to `Comparable[Int]` by having `compare(other: Int)`, and
    is expected to inherit `before?(other: Int)` directly. The README's phrase
    "a parameter that appears only inside another's bound comes from that
    conformance" suggests this, but it is the same shape that must be
    *rejected* in g12, so the rule is load-bearing and stated only in passing.
15. **`xs[0]?` inside a function returning something other than `T?`.**
    `generics.lume` uses it where the return type is `T?`. g04's `min_max`
    returns `(T, T)?` and g06's `closest` returns `Int?` while `?`-ing a `T?`
    from a differently-typed list; I read "`?` early return in a function
    returning `T?`" as "returning any optional", which may be narrower than
    intended.
16. **Method names that shadow built-ins on a generic type.** g03's
    `Chain.values` and g08's `Seq.len`/`Seq.text` collide with list/`Str`
    vocabulary. The README promises "a method of your own wins over the
    built-in of the same name" for blocks; it should say the same for methods
    on your own generic types.

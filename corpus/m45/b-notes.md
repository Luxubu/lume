# Reviewer B — round nine notes

## What I read

- `corpus/m45/BRIEF.md`
- `docs/reference/interfaces.md`, `generics.md`, `functions-and-blocks.md`,
  `structs-and-enums.md` (in full)
- `docs/reference/collections.md`, `methods.md`, `printing.md` (in full);
  `concurrency.md` (in full); `names.md` (keywords, reserved type names);
  `numbers.md` (section headings, division, rounding); `tour.md` (Interfaces)
- `docs/README.md` / `docs/examples.md` (only via grep for `extend`)
- Examples, with their `.expected`: `interface_methods`, `extend_coherence`,
  `generic_extend`, `type_functions`, `kept_blocks`, `generic_interfaces`,
  `interfaces`, `generics`, `blocks_of_your_own`, head of `ownership`
- `examples/errors/`: every `iface_*`, `extend_*`, `block_kept_*`,
  `block_equality`, `block_returned`, `generic_uninferable`,
  `generic_unused`, `generic_wrong_arity`, `static_*`, `type_call_method`,
  `value_call`, `call_not_a_block`, `call_optional_block`, `block_not_a_block`

Nothing under `compiler/`, `tests/`, other `corpus/` files, `README.md`,
`HISTORY.md` or `docs/QUESTIONS.md`. Nothing was run.

## The programs

| file | probes | meant to | expected |
|---|---|---|---|
| `b01_walk_defaults.lume` | block-taking interface method; defaults calling it (one default calls another); held in list, field, binding; param; bound; `extend {Int}` | RUN | 11 lines |
| `b02_mappable_known_type.lume` | `map_all[U]` through param, param handed on, bound, `[T, M: Mappable[T]]`, concrete; held list/field use only `size`/default | RUN | 12 lines |
| `b03_generic_method_on_field.lume` | `map_all` called on an interface-typed field, by bare name inside the struct's method | FAIL | `` `map_all` has type parameters of its own (`U`), so it cannot be called on a `Mappable[Int]` held in a list, a field or a binding `` |
| `b04_async_source.lume` | `async` interface method from structs and `extend Int`; awaited through param, bound over list items, `spawn:`; held list uses plain method + default | RUN | 8 lines |
| `b05_async_on_held_item.lume` | `await s.fetch` on an item of `[Source]` | FAIL | message not documented for `async`; guessed by analogy |
| `b06_extend_chosen_args.lume` | `extend Box[Int]`, `Box[Str]`, `Box[[Int]]`, `Pair[Int, Str]`, `Pair[Str, Int]` with one interface; `Box[T]`/`Pair[A, B]` with another | RUN | 11 lines |
| `b07_overlap_own_generic.lume` | `extend Box[Int] with Tagged` then `extend Box[T] with Tagged` | FAIL | `` `extend Box[T] with Tagged` overlaps `extend Box[Int] with Tagged` `` |
| `b08_blocks_in_generic_types.lume` | blocks in `Box[(Int) -> Int]`, `Stack[...]` via `Stack.empty`, map, tuple; `b.item(6)`, `b.get()(5)`, `s.top!(10)`, `table["inc"]!(9)`, `pair.0(1)`; generic fn with a list of blocks | RUN | 13 lines |
| `b09_each_generic_defaults.lume` | `Each[T]` with a block method and defaults incl. `mapped[U]` (a default with its own type parameter); `Ring[T]`, `[T]`, `{T}` conform; `extend Ring[Int]` with a second interface | RUN | 10 lines |
| `b10_pass_held_binding.lume` | a binding `m: Mappable[Int]` passed to a param typed `Mappable[Int]` whose body only calls `size` | FAIL | `` this `Mappable[Int]` is held through a pointer, and `Mappable[Int]` has methods only a known type can call (`map_all`) `` at `count(m)` |

## GUESSES — 36

1. **Can a `[Walk]` be used again after being stored in a struct field (so it
   must be copied)?** Looked: `examples/ownership.lume` header ("a value used
   again later is copied"), interfaces.md `Scene` example (which never uses
   `mixed` afterwards). Guess: yes, it is copied. Confidence: medium. (b01)
2. **Does a set literal (`{5, 1}`, `{2, 2, 3}`) take `Int` items and convert
   to `Walk` inside a `[Walk]` literal and at a `Walk` parameter?** Looked:
   interfaces.md (only `Int`, `Str`, `[Int]` extends), generics.md
   `describe({"red"})`. Guess: yes. Confidence: medium-high. (b01)
3. **Where does the "holds values of different types behind `X`" warning go,
   and when does it fire?** Looked: concurrency.md "Where the warnings go"
   (only covers `shared var`), `interface_methods.expected` (warns for
   `walks` but not for `held`, which is also mixed). Guess: stderr, not part of
   stdout; it may or may not fire. Confidence: medium. (b01, b04, b06, b09)
4. **May a default change a local (`best = match best: ...`) inside a `do`
   block handed to the interface's own block method?** Looked:
   functions-and-blocks.md "A block closes over what is around it, and may
   change it"; interfaces.md `t += x`. Guess: yes. Confidence: high. (b01)
5. **Can a parameter typed `Mappable[Int]` be handed on to another parameter
   typed `Mappable[Int]` (`doubled_again`)?** Looked: interfaces.md known-type
   paragraph; `iface_pass_held_value`. Guess: yes, a parameter is a known
   type. Confidence: medium. (b02)
6. **Is `T` in `[T, M: Mappable[T]]` worked out from how `Box[Str]`
   conforms, with a block parameter `(T) -> Str` given after the call?**
   Looked: `generic_interfaces.lume` `keys_of[K, V, T: Keyed[K, V]]`. Guess:
   yes. Confidence: medium-high. (b02)
7. **Are the interface's *defaults* (not just required methods) callable on
   a held value when the interface has a generic method?** Looked:
   interfaces.md "its other methods called". Guess: yes. Confidence: high.
   (b02)
8. **Can an interface-typed field be given a list literal (`Holder(m: [9])`)?**
   Looked: interfaces.md `Scene`; `interface_methods.lume` `held`. Guess: yes.
   Confidence: medium-high. (b02)
9. **Is the b03 error reported inside `lines` (bare field name in the
   struct's own method), with the same wording as for a list item?** Looked:
   interfaces.md, `iface_generic_through_pointer`. Guess: yes, same
   message. Confidence: medium. (b03)
10. **May an `extend` supply an `async def`?** Looked: interfaces.md ("each
    type gives its own"), `interface_methods.lume` (only structs do). Guess:
    yes. Confidence: medium. (b04)
11. **Is `await x.fetch` allowed on an item of `xs: [S]` when `S` is bound
    by `Source`?** Looked: interfaces.md ("a bound" is allowed; "a list item"
    refused — of the interface's type). Guess: allowed, since the type is
    known. Confidence: medium. (b04)
12. **Does `spawn: await Fixed(text: "task").fetch` await the method call
    (not something shorter)?** Looked: concurrency.md "`await` covers
    everything to its right". Guess: yes. Confidence: high. (b04)
13. **The exact message for an `async` method on a held value.** Looked:
    interfaces.md (message quoted only for type parameters), examples/errors
    (no `async` counterpart). Guess: `` `fetch` is `async`, so it cannot be
    called on a `Source` held in a list, a field or a binding ``.
    Confidence: high that it is refused, low on the wording. (b05)
14. **Does the `for s in sources` loop name count as a held value?** Looked:
    interfaces.md. Guess: yes, it is an item of a `[Source]`. Confidence:
    high. (b05)
15. **Is `extend Box[Int] with I` (your own generic type at a chosen
    argument) allowed at all?** Looked: generics.md "Generic `extend`" (only
    `Stack[T]`), grep of all docs/examples for `Box[Int]` / `extend X[`
    (nothing). Only the brief mentions it. Guess: yes, since `Int` is "a
    type of this program's" it is not a parameter. Confidence: medium. (b06)
16. **Inside `extend Box[Str]`, is the bare field `value` a `Str`?** Looked:
    interfaces.md "its fields are also there bare". Guess: yes. Confidence:
    medium. (b06)
17. **Is a nested chosen argument (`Box[[Int]]`) accepted?** Looked:
    generics.md (`{K: [Int]}` works for maps). Guess: yes. Confidence:
    medium. (b06)
18. **Does `extend Pair[A, B]` introduce two parameters at once?** Looked:
    generics.md `extend {K: V}`. Guess: yes. Confidence: medium-high. (b06)
19. **Are `Box[Int]`, `Box[Str]`, `Box[[Int]]` (and `Pair[Int, Str]` vs
    `Pair[Str, Int]`) non-overlapping under the coherence rule?** Looked:
    generics.md `[Int]`/`[Str]`/`{K: [Int]}` example. Guess: yes.
    Confidence: high. (b06)
20. **Does `Box.of(7)` inside a `[Tagged]` literal get `T = Int` from its
    argument and then convert?** Looked: structs-and-enums.md (a type's
    function works `T` out from its arguments). Guess: yes. Confidence:
    medium-high. (b06)
21. **The b07 message: which extend is named first, and the help text.**
    Looked: generics.md, `extend_concrete_overlap.expected` (the later one is
    named first). Guess: `` `extend Box[T] with Tagged` overlaps `extend
    Box[Int] with Tagged` ``, help "both would apply to `Box[Int]`".
    Confidence: medium. (b07)
22. **Is a block in a *generic* field (`item: T`, `T = (Int) -> Int`)
    called like a method, `b.item(6)`?** Looked: functions-and-blocks.md
    ("One in a field is called like a method") — shown only for a field
    declared with a block type. Guess: yes. Confidence: low-medium. (b08)
23. **`b.get()(5)`: a method with explicit `()` whose result is then
    called.** Looked: functions-and-blocks.md (`steps.at(0)(3)`; `()` on a
    method is allowed). Guess: works. Confidence: medium-high. (b08)
24. **Can a function's name (`double`) be passed to a generic `T` parameter
    (`Box.of(double)`) when the binding says the block type?** Looked:
    functions-and-blocks.md ("where a block is expected"), kept_blocks
    (`f: (Int) -> Int = double`). Guess: yes. Confidence: medium. (b08)
25. **`Stack.empty` with `T` a block type from the binding, then
    `s.push({ |x| x * x })` — a literal block to a `T` parameter.** Looked:
    type_functions.lume, functions-and-blocks.md ("where it goes has to say
    it"). Guess: works. Confidence: medium. (b08)
26. **Does `puts` of a `T?` holding a block print `Some(<block>)`, and a
    generic struct holding one `Box(item: <block>)`?** Looked:
    functions-and-blocks.md (`Rule(... check: <block>)`), kept_blocks
    (`Some(Rule(... <block>))`). Guess: yes. Confidence: medium. (b08)
27. **`table["inc"]!(9)`: index a map of blocks, unwrap, call.** Looked:
    functions-and-blocks.md (`steps.first!(3)`). Guess: works. Confidence:
    medium. (b08)
28. **`pair.0(1)`: a tuple part that is a block, called straight away.**
    Looked: functions-and-blocks.md (map or tuple can hold a block; call
    syntax shown only after `)`). Guess: works. Confidence: low-medium. (b08)
29. **`apply_all([{ |w| w + "!" }], "hi")`: a block literal in a list whose
    element type is known only once `T` is read off the *later* argument.**
    Looked: functions-and-blocks.md, generics.md, the brief ("generic calls
    that work `T` out through blocks"). Guess: works. Confidence: low-medium.
    (b08)
30. **`apply_all([double, adder(1)], 5)`: a list literal of a function name
    and a returned block, typed only by the parameter.** Looked: kept_blocks
    (such a list always had its type said on a binding). Guess: works.
    Confidence: medium. (b08)
31. **May an interface *default* have a type parameter of its own
    (`def mapped[U](f: (T) -> U) -> [U]` with a body)?** Looked:
    interfaces.md (only required generic methods shown); async defaults are
    refused, generic ones not mentioned. Guess: allowed. Confidence:
    medium-low. (b09)
32. **Can `[Each[Int]]` still be held, and its non-generic defaults called,
    when one default is generic?** Looked: interfaces.md. Guess: yes.
    Confidence: medium. (b09)
33. **Can `Ring[T]` conform to `Each[T]` by its own method while
    `extend Ring[Int] with Summed` adds another interface only at `Int`,
    and `slots.sum` works there?** Looked: generics.md, interfaces.md. Guess:
    yes. Confidence: medium. (b09)
34. **Does `mapped` pick the right `U` from a block that gives `Bool`
    (`[5, 6].mapped { |n| n > 5 }`), and does it clash with the built-in
    `map` family?** Looked: methods.md list of list methods (no `mapped`,
    `items`, `count_where`), `extend_default_recurses`. Guess: no clash,
    `[false, true]`. Confidence: medium-high. (b09)
35. **Is the refusal at `count(m)` even though `count` calls only `size`?**
    Looked: interfaces.md ("the function could call `map_all` on it").
    Guess: yes, refused at the first such call. Confidence: medium-high.
    (b10)
36. **Does a typed binding `m: Mappable[Int] = [3, 4]` count as "held
    through a pointer", even with one concrete type behind it, while
    `m.size` on it is fine?** Looked: interfaces.md ("a binding of the
    interface's type"). Guess: yes. Confidence: high. (b10)

## Contradictions and gaps

- **`extend` at chosen arguments of your own generic type (`Box[Int]`) is
  documented nowhere.** The brief says round eight fixed it; generics.md
  shows only `extend Stack[T]`, and no example uses it. The rule "a name that
  is no type of this program's is a parameter" implies it, but nothing says
  what the fields are inside such an extend or how coherence treats it.
- **A parameter `m: Mappable[Int]` and a binding `m: Mappable[Int]` are
  spelled identically and follow opposite rules** (one may call `map_all`,
  the other may not). The docs state it but never say *why* a parameter is a
  "known type" (presumably it becomes a Rust generic). A sentence would help.
- **The mixed-list warning is inconsistent in the examples.**
  `interface_methods.expected` warns for `walks` (line 82) but not for
  `held: [Mappable[Int]] = [Box(items: [3]), [4, 5]]` (line 92), which also
  mixes two types. No doc page describes this warning or says where it goes;
  concurrency.md's "Where the warnings go" covers only `shared var`.
- **No message is documented for an `async` method on a held value.**
  interfaces.md puts `async` under the same rule as type parameters but only
  quotes the type-parameter message.
- interfaces.md shows the refused call as `held.at(0).map_all`, the error
  example as `held[0]!.map_all`; same message, fine, but the two spellings
  of "item" invite the question whether `for m in held` is also covered
  (I assumed yes).
- Interface defaults with type parameters of their own are neither shown
  nor refused anywhere, although `async` defaults are explicitly refused.

## Verdict

The block-taking interface method is well covered: the page, the example and
its `.expected` agree, and I had to guess little beyond ownership of a
`[Walk]` and warning plumbing. The "known type" rule for generic and `async`
methods is stated clearly enough to predict refusals, and the two quoted
messages (list item, passing a held value) cover most cases, but `async` has
no message of its own, and the parameter-versus-binding distinction reads as
arbitrary without the one-line reason. The weakest area is the one this round
set out to test: `extend Box[Int]` / `Pair[Int, Str]` at chosen arguments is
absent from the docs entirely, so b06, b07 and part of b09 rest on inference
from the `[Int]`/`[T]` rule. Blocks in generic containers are workable, but
the call forms on anything other than a named binding or a block-typed field
(`b.item(6)`, `pair.0(1)`, `table["inc"]!(9)`) are guesses.

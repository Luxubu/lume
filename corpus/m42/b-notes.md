# Reviewer B: notes (round eight, milestone 42)

Focus: types and abstractions (structs, enums, interfaces, `extend` and the
coherence rule, generics with bounds, `def self.` on generic types, and kept
blocks in fields and generic containers), and where the new features meet
them.

## What I read

- `corpus/m42/BRIEF.md`
- `docs/tour.md`
- `docs/reference/`: `generics.md`, `interfaces.md`, `structs-and-enums.md`,
  `functions-and-blocks.md`, `methods.md`, `collections.md`, `printing.md`,
  `operators.md`, `numbers.md`, `names.md`, `errors.md` (the sections on
  implied `Some`/`Ok`, `.or`, `!` and the other methods), `patterns.md` (the
  table, what an arm can do, covering everything)
- `examples/`: `kept_blocks`, `type_functions`, `extend_coherence`,
  `generic_extend`, `generic_interfaces`, `generics`, `interfaces`,
  `structs`, `enums`, `blocks_of_your_own` (each `.lume` with its
  `.expected`)
- `examples/errors/`: the directory listing, plus `block_equality`,
  `extend_bounds_overlap`, `extend_concrete_overlap`, `extend_nested_overlap`,
  `extend_tuple_overlap`, `extend_generic_missing`, `extend_default_recurses`,
  `block_kept_self`, `block_returned`, `block_value_untyped`,
  `block_kept_result`, `static_generic_unresolved`, `static_in_interface`,
  `static_variant_clash`, `static_calls_method`, `static_self`,
  `static_top_level`, `static_typo`, `type_call_method`, `value_call`,
  `call_optional_block`, `iface_mismatch`, `iface_missing`,
  `iface_generic_unmet`, `generic_bound_unmet`, `generic_uninferable`,
  `struct_as_pattern`, `wrong_enum`

I did not open the compiler, tests, README, HISTORY, QUESTIONS.md, or the
other reviewer's files. I did not run anything. The only tool I used for
arithmetic was a calculator, to check how three float results print (1.7,
200.0, 0.25).

## The programs

| file | what it is | RUN/FAIL | what it leans on |
|---|---|---|---|
| `b01_units.lume` | lengths with units: `Unit` enum and `Length` struct, both with `def self.` | RUN | `def self.` on an enum and a struct; one type's function calling another's; `?` inside a `def self.` that can fail; an interface default met by an enum; a type function handed over as a block; `+` on a struct |
| `b02_stacks.lume` | generic `Stack[T]` with `empty`/`of`, and a bounded `Ranked[T: Ordered]` | RUN | `T` read off a binding, an argument and a method parameter; `empty` and `empty?` side by side; a `var` generic parameter; a bounded generic struct with a `def self.`; `Ordered` met by writing `<` |
| `b03_rules.lume` | password rules: a `Rule` struct holding a `(Str) -> Bool` | RUN | kept block in a field; `def self.` constructors that capture a parameter; a method that builds a new kept block from the field (binding it to a name first); a `?`-named function as a block value; printing `<block>` |
| `b04_pricing.lume` | discount strategy as an interface and as a block | RUN | an interface met by a struct that only holds a block; defaults reaching it; a mixed `[Pricing]` list; `[P: Pricing]`; an interface value copied into a kept block; `f(x)(y)` |
| `b05_pipeline.lume` | generic `Pipeline[T]` holding `[(T) -> T]` | RUN | a generic container of kept blocks; a `var self` method that keeps the block it is handed; `fold` over the blocks; a generic function that returns a kept block; printing a list of blocks |
| `b06_disjoint_extends.lume` | one interface extended onto `Box[Int]`, `Box[Str]`, `Pair[Int, Str]`, `Pair[Str, Int]` and `[Box[Int]]` | RUN | the permitted side of the coherence rule, on my own generic types; a second interface extended generically next to them; fields bare inside an extend; a mixed `[Describe]` list |
| `b07_specific_extend.lume` | `extend Cell[T]` plus a "better" `extend Cell[Str]` | FAIL | "the more specific does not win": the overlap error, as it would read for a user's own type |
| `b08_button_equality.lume` | a menu of buttons, each holding a block, then `chosen == greet` | FAIL | `==` refused on a struct that holds a block |
| `b09_bounded_extends.lume` | `extend Bag[T: Ordered]` and `extend Bag[T: Hashable]` of one interface | FAIL | "bounds do not separate them", and a bound written inside a user generic target |
| `b10_event_log.lume` | log handlers: a `Level` enum with `<`, a generic `Handler[T]` interface, a `FnHandler[T]` that holds a block, and a map of formatter blocks | RUN | an enum with `def self.` and an operator; a generic interface met by a plain struct and by a generic block-holding struct; a mixed list as a field argument; a block that gives `Str?`; a map whose values are blocks |

For the three FAIL programs, EXPECTED OUTPUT is empty and the predicted error
is quoted in the header. Each FAIL program also says what it would have
printed with the fault removed. For b04 and b06 I put the predicted
mixed-list warning (standard error) in the header, not in EXPECTED OUTPUT.

## GUESSES

**Total: 33.**

Confidence is how likely I think it is that the program behaves as I wrote.

1. **Does an enum conform to an interface structurally and get its
   defaults?** (b01: `Unit` has `label`, so it gets `tagged`, and
   `tag_all[T: Labeled](Unit.all)` works.) Looked: interfaces.md,
   generics.md "Interfaces as bounds"; every example uses a struct or an
   extended built-in. Guessed yes. Confidence 80%.
2. **Is another type's function called as `Unit.parse(...)` from inside
   `Length`'s `def self.`?** Looked: structs-and-enums.md "Functions of a
   type" (bare name inside the same type; `model.User.guest` from another
   file). Guessed yes, qualified by the type name. 90%.
3. **Does `?` work on `opt.or_error("...")` inside a `def self.` that
   returns `T or Error`?** Looked: methods.md `or_error`, errors.md `?`, and
   `type_functions.lume` (`to_float?` inside `def self.parse`). Guessed yes.
   90%.
4. **Does a struct inside `Ok` print as `Ok(Length(value: 250.0, unit:
   Millimeters))`, with the enum field printed bare?** Looked: printing.md,
   enums.expected (`Order(... status: Approved(...))`). Guessed yes. 90%.
5. **Can `T` of a type function be read off a method parameter
   (`nested.push(Stack.empty)` where `push` wants `Stack[Int]`)?** Looked:
   structs-and-enums.md ("from its arguments or from where the value goes",
   shown only for a typed binding); `static_calls_method` error. Guessed
   yes. 65%.
6. **May a type function `empty` sit next to a method `empty?`?** Looked:
   structs-and-enums.md ("A function may not share its name with a field, a
   method"); names.md on `?` names. Guessed that these are two different
   names. 80%.
7. **Does a bound on a generic struct (`Ranked[T: Ordered]`) hold inside its
   `def self.from`, so `xs.sort` is allowed there?** Looked: generics.md
   "Bounds on a generic type's parameters" (only methods shown). Guessed
   yes. 80%.
8. **Can a `var` parameter be a generic struct (`drain[T](var s:
   Stack[T])`), with its `var self` methods called on it and the change
   seen by the caller?** Looked: functions-and-blocks.md "`var`
   parameters" (only `[Int]` shown). Guessed yes. 85%.
9. **Does a nested generic struct print as `Stack(items: [Stack(items: [1,
   2]), Stack(items: [])])`?** Looked: printing.md, generics.md. Guessed
   yes. 90%.
10. **Inside a method, is `inner = check` (a block field, bare, no
    arguments, no annotation) the field's value, ready to be captured by a
    new kept block?** Looked: functions-and-blocks.md ("bind what it needs
    to a name first"; a bare function name is a value; `add10 = adder(10)`
    needs no annotation). Guessed yes. 65%.
11. **Does a block built in a `def self.` capture that function's parameter
    (`{ |s| s.len >= n }`)?** Looked: `kept_blocks.lume` (captures a local
    in `main`), `adder`. Guessed yes. 90%.
12. **Can a `?`-named function (`no_spaces?`) be handed over as a kept block
    value in a struct field?** Looked: functions-and-blocks.md "A
    function's name as a block", names.md. Guessed yes. 85%.
13. **Is a multi-line list literal with a trailing comma accepted as a
    constructor argument?** Looked: `kept_blocks.lume` (as a binding's
    value only). Guessed yes. 85%.
14. **Does a struct whose only field is a block conform to an interface
    through a method that calls the field, and pick up the interface's
    defaults (`name` becomes "custom")?** Looked: interfaces.md,
    structs-and-enums.md. Guessed yes. 85%.
15. **Can a kept block copy in a value whose type is an interface
    (`as_block(p: Pricing) -> (Int) -> Int = { |b| p.price(b) }`)?**
    Looked: functions-and-blocks.md "It copies in what it uses when it is
    made"; nothing on interface values. Guessed yes. 50%.
16. **Is the mixed-list warning printed for `deals: [Pricing]` and
    `things: [Describe]`, worded like the one in `interfaces.expected`, on
    standard error?** Looked: interfaces.expected, printing.md `warn`.
    Guessed yes, with the binding's and interface's names filled in. 70%.
17. **Does a `var self` method keep the block it is handed by pushing it
    into a `[(T) -> T]` field?** Looked: functions-and-blocks.md ("A
    function may keep the blocks it is given … Lume sees it", shown only by
    returning a composed block). Guessed yes. 70%.
18. **Can a `_.trim` shorthand, or an inline `{ |x| ... }` after a bare
    method name, go to a parameter whose block is then kept?** Looked:
    functions-and-blocks.md, `blocks_of_your_own.lume`. Guessed yes. 75%.
19. **Does a list of blocks inside a struct print as `[<block>, <block>,
    <block>]`?** Looked: functions-and-blocks.md (a single field prints
    `<block>`), `kept_blocks.expected`. Guessed yes. 80%.
20. **Can a generic function return a kept block (`then[T](f, g) -> (T) ->
    T`), with an inline block as the second argument typed from the
    first?** Looked: `compose` in functions-and-blocks.md (only `Int`).
    Guessed yes. 75%.
21. **Can `extend` target a user generic struct at concrete arguments
    (`Box[Int]`, `Pair[Int, Str]`) and give different answers to
    `Box[Int]` and `Box[Str]`?** Looked: generics.md "Generic `extend`"
    and "the Rust rule" (concrete targets shown only for built-ins, and
    user types only as `Stack[T]`). Guessed yes. 80%.
22. **Are a struct's fields bare inside an extend of a generic struct at
    concrete arguments?** Looked: interfaces.md ("its fields are also there
    bare"). Guessed yes. 85%.
23. **Do two instantiations of one generic struct (`Box[Int]` and
    `Box[Str]`) sit in one `[Describe]` list as different types behind
    the interface?** Looked: interfaces.md "Where an interface can be
    used". Guessed yes. 75%.
24. **What is the overlap error's wording for a user's own generic type,
    and which path form does it name?** (b07) Looked: generics.md,
    `extend_concrete_overlap` and `extend_nested_overlap` expected output.
    Guessed that the error is reported at the second extend with
    `` `extend Cell[Str] with Show` overlaps `extend Cell[T] with Show`
    from <path>:41 `` and help "both would apply to `Cell[Str]`". The path
    is the file as given on the command line. 75%.
25. **Is nothing in b07 refused before the overlap (for example the extend
    of `Cell[T]` itself, or `Cell.of`)?** Looked: generics.md. Guessed
    no. 85%.
26. **What does `==` on a struct that holds a block say?** (b08) Looked:
    functions-and-blocks.md (says it is refused, no text shown);
    `examples/errors/block_equality` (text only for block `==` block).
    Guessed the same message, `` `==` cannot compare blocks ``, at the
    `==`. 55%.
27. **Do `buttons.find { ... }` and `.first.or(quit)` on a `[Button]` work
    without needing `==` on `Button`, so the first error is the `==` I
    wrote?** Looked: methods.md. Guessed yes. 85%.
28. **Is a bound accepted inside a user generic target (`extend Bag[T:
    Ordered]`)?** Looked: generics.md (a bound inside `[...]` is shown only
    for the list form `[T: Ordered]`, and inside `{...}` it is not
    allowed). Guessed yes. If not, b09 fails with a parse error instead.
    70%.
29. **Does the bounds clash name the target with the bound removed,
    `` `Bag[T]` is already a `Summary` ``, with a help line pointing at the
    first extend?** Looked: `extend_bounds_overlap.expected`. Guessed yes.
    65%.
30. **May an enum define `<`, and do `>=`, `filter { |l| l >= Info }` and
    `.max` then follow?** Looked: interfaces.md "Operator methods" and
    generics.md (shown only on structs). Guessed yes. 75%.
31. **Does a generic struct holding a block (`FnHandler[T]`) meet a generic
    interface `Handler[T]` at `T = Event`, next to a plain struct
    (`Threshold`) whose argument is worked out as `Event`?** Looked:
    generics.md "Interfaces with type parameters",
    `generic_interfaces.lume` (`Renders[Str]`). Guessed yes. 70%.
32. **Is a mixed list accepted as a struct field argument whose field is
    typed `[Handler[Event]]`, with no annotated binding?** Looked:
    interfaces.md ("the annotation on `mixed` is doing work";
    structs-and-enums.md builds `Scene(rest: [Sq(2.0), Circle(1.0)])`).
    Guessed yes. I could not tell whether it warns here, so I predicted no
    warning. 70%.
33. **Can a map's values be kept blocks (`{Str: (Event) -> Str}`), written
    as inline literals and called after `for name, fmt in formats`, and can
    the block in `FnHandler(f: { |e| ... })` take its parameter type from
    the binding's `FnHandler[Event]` through the generic constructor?**
    Looked: functions-and-blocks.md "Blocks as values" (a binding, a field,
    a list, a result; maps are not mentioned). Guessed yes to both. 65%.

## Verdict

The type layer is well documented where these features meet it.
`def self.` on generic types, blocks in struct fields, and the coherence
rule each have a page section, a runnable example and error examples with
exact text. Most programs could be written with no guessing in their core.
The guesses come from combinations the docs never show together:

- a bound or concrete arguments inside a *user* generic `extend` target (the
  docs use built-in lists for nearly every coherence example);
- `==` on a struct holding a block (said to be refused, with no message
  shown);
- blocks kept by being pushed into a field, or kept inside a map;
- a kept block that copies in an interface value;
- operators on an enum.

Adding one example of the coherence rule on a user generic struct
(`Box[Int]` next to `Box[T]`) and the `==`-on-a-struct-with-a-block message
would remove the least certain guesses.

A doc inconsistency outside my focus: methods.md "Tuples" says `<` and `>`
between tuples are refused. operators.md "Comparison" shows `(1, "b") < (1,
"c")` printing `true`. Milestone 42's tuple ordering probably made
methods.md stale.

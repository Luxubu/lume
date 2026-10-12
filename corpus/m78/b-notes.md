# Reviewer B — notes

## What I read

- `corpus/m78/BRIEF.md`.
- `docs/reference/collections.md` in full, especially "Generators: `yield`",
  "Taking a sequence: `xs: Iterator[T]`" and "Types of your own: `next`".
- `docs/reference/interfaces.md` in full, especially "An interface's own
  methods may use interfaces" and the `async` / type-parameter section.
- `docs/reference/methods.md` in full, especially the tuple-block paragraphs
  under `[T]` and "Lazy chains".
- `docs/reference/structs-and-enums.md` in full (`def self.`, `var self`,
  calling a method by bare name, `match self`).
- `docs/reference/printing.md` in full; the relevant parts of
  `generics.md` (generic structs and enums, interfaces as bounds, interfaces
  with type parameters), `control-flow.md` (`if` as a value, `match` arms,
  `for` with two names), `patterns.md` (the forms, multi-line arms),
  `functions-and-blocks.md` (block forms; kept blocks and what they copy),
  `names.md` (keywords), `concurrency.md` ("Where the warnings go").
- Examples: `generators`, `interface_signatures`, `enumerate_blocks`,
  `iterators`, with their `.expected`; the warning lines of
  `interfaces.expected`, `interface_methods.expected`,
  `generic_interfaces.expected`.
- `examples/errors/`: every `generator_*`, `iface_*`, `interface_*` and
  `yield_*` pair, plus `block_tuple_arity`, `block_arity`,
  `for_three_names`, `for_two_vars_not_pairs`, `destructure_arity`,
  `tuple_method`, `iterator_bound_var_self`, `iterator_param_var`,
  `immutable_receiver`, `readonly_self`, `extend_var_self`, `static_self`,
  `block_kept_changes_var`, and `UNREACHED.md`.
- I did not open `packages/`: nothing in it concerns milestones 75–77.

Aside, not a guess: `names.md` still lists `yield` among the Rust words that
"work as fields and function names too", but `yield` is now Lume's own
generator statement. Either the list or the parser is out of date.

## The programs

| file | probes |
|---|---|
| `b01_gen_method_copy.lume` (RUN) | generator method's copy taken at the call (a `var` struct's fields reassigned after the call and before the walk); a generator method calling a plain method by bare name; a recursive generator method on the values of a field, with the tree changed after the call; the result handed to an `Iterator[Str]` parameter |
| `b02_gen_method_shapes.lume` (RUN) | generator method on a generic struct at `Int` and `Str`, with a parameter; on an enum with `match self` (one-line `yield` arm, indented arm, `for` over tuples), giving pairs to `for x, y` and a two-name `count`; recursive on an enum's own fields; a static `def self.` generator, called on the type and by bare name from another `def self.` |
| `b03_chains_into_gens.lume` (RUN) | endless chains into generators whose blocks read an `Int` `var` (changed after the call), a `Str` local and a list local; chain over a list into a generator worked out at the call (side effects in `map` print before "called"); chain over a range stays lazy (side effects interleaved with the walk); chain over a list into a plain function stays lazy; a struct `next` sequence starting an endless chain, still at its start afterwards |
| `b04_tuple_blocks.lume` (RUN) | triples on map, filter, sort_by, max_by, min_by (tie), any?, count, find; a block building triples then one taking them apart; quadruples on map and `sum` with a block; `fold` over quadruples with two names; one name for a whole tuple; pairs; `enumerate` over triples giving `(Int, tuple)`; three names over a generator's triples |
| `b05_iface_in_signatures.lume` (RUN) | interface method taking an interface value (a value of another type; held `[Shape]` items as receiver and argument), giving one back (of a different type from the receiver), a default using both; generic `Source[T]` with a required `-> Iterator[T]` written as a generator in one type and as a chain over a list field in another; a default handing back a chain; a generator default; bound `[S: Source[Int]]`; a `[Source[Int]]` list with its generator default called; the two mixed-list warnings |
| `b06_iterator_fns.lume` (RUN) | plain `-> Iterator[Int]` functions giving a list literal, a chain over a list, a range, a struct with `next`, another generator's result, and a generic `-> Iterator[T]` giving a list; each result moved on with `next` on a `var`, then walked by a chain or `for`, which carries on; one handed to an `Iterator[Int]` parameter |
| `b07_gen_method_twice.lume` (FAIL) | a generator method's result kept in a local and walked twice |
| `b08_enum_gen_var_self.lume` (FAIL) | `var self` on an enum's generator method using `match self` |
| `b09_quad_three_names.lume` (FAIL) | three names over quadruples on `sort_by`: the message and the four-name help |
| `b10_enumerate_flatten.lume` (FAIL) | four names over `enumerate` of triples (no flattening of `(i, (a, b, c))`) |
| `b11_iface_default_async_param.lume` (FAIL) | an interface *default* whose second parameter is an interface with an `async` method |
| `b12_gen_in_extend_struct.lume` (FAIL) | a generator in an `extend` of the program's own struct |
| `b13_gen_method_iface_param.lume` (FAIL) | a generator *method* taking an interface-typed parameter |
| `b14_iter_fn_twice.lume` (FAIL) | the result of a plain (no-`yield`) `-> Iterator[Int]` function, kept in a local, walked twice |

## Guesses

1. **b01** — that `s.books = [...]` and `tree.subs = []` (whole-field
   assignment of a list, the second an empty literal) work on a `var` struct.
   "Changing a field" in `structs-and-enums.md` shows only an `Int` field, and
   "Where `[]` and `{}` need a type" in `collections.md` lists a struct field
   for construction, not for assignment.
2. **b01** — that a generator method may call another (plain) method of its
   type by bare name, `tag(b)`, and that it runs on the copy. The docs' `Node`
   example in `collections.md` uses only a field and a recursive call on a
   child; `generators.lume`'s `roll_call` uses only fields.
3. **b02** — that a generator *method* works on a generic struct (`Ring[T]`).
   `generators.lume` has a generic free generator (`repeat[T]`) and a
   non-generic generator method, never both at once.
4. **b02** — that `yield` may be a one-line `match` arm (`Dot -> yield (0, 0)`)
   inside a generator method on an enum. `collections.md` says `yield` may sit
   in the generator's own `match`es, but every example puts it on an indented
   line.
5. **b02** — that a static `def self.` function may be a generator and be
   called by bare name from another `def self.` (`counting(0, 1).take(3)`).
   Neither `structs-and-enums.md` ("Functions of a type") nor `collections.md`
   says anything about it.
6. **b03** — that a `var` read by a chain's block may be reassigned after the
   chain has gone into a generator (and that the block keeps the old value, 3,
   not 100). `collections.md` says the blocks "take copies of what they read",
   but its examples use immutable locals (`k`, `step`); the only nearby rule,
   `functions-and-blocks.md` on kept blocks, covers a `var` changed outside a
   *kept* block, which this block is not said to be.
7. **b04** — that `fold` with a two-name block over quadruple items gives the
   total and the whole quadruple, and is not refused by the "names 2
   arguments, but the items have 4 parts" rule (`block_tuple_arity`).
   `methods.md` says tuple blocks work "on every method that takes a block",
   and separately that `fold`'s block takes the total and an item; how the
   two rules combine is not shown.
8. **b04** — that a block with three names takes apart the triples of a
   *generator* (`triples(3).filter { |a, b, c| .. }`). `methods.md` shows
   triples only on a list; `collections.md` mentions pairs only, for
   generators.
9. **b05** — that a required `-> Iterator[Int]` method may be written as a
   chain over the struct's own list field (`xs.map { |x| x * 10 }`).
   `interfaces.md` and `interface_signatures.lume` show only a chain over a
   range (`Squares`), which borrows nothing from the value.
10. **b05** — that a type's `-> Shape` method may give back a value of
    another conforming type (`Bar.grown` gives a `Sq`). Every example gives
    back its own type.
11. **b06** — that a plain `-> Iterator[Int]` function may give back a range
    (`1..n`) and a struct with `next` (`Down(n: n)`). `collections.md` lists
    "a generator, a chain or a list" for results, and the range and the
    struct only for an `Iterator[T]` *parameter*.
12. **b06** — that a generic `def repeat3[T](x: T) -> Iterator[T] = [x, x, x]`
    is accepted (`T` known from the argument; `x` named three times).
    No example gives a generic list back as a sequence.
13. **b08** — that the generator `var self` check fires first on an enum, with
    the same wording as `generator_var_self` (a struct), rather than some
    other complaint about the enum, `match self`, or the call on `w`.
14. **b09** — the help line for four parts:
    ``name one for each part, `{ |a, b, c, d| ... }`, or one for the whole tuple``.
    `block_tuple_arity.expected` shows only the three-part form, so I assumed
    the letters grow with the number of parts. Also that the check is the same
    on `sort_by` as on `map`.
15. **b10** — that `enumerate` over triples is refused with "names 4
    arguments, but the items have 2 parts", with the help
    ``{ |a, b| ... }``. No example shows a pair's version of this message.
16. **b11** — that the refusal in `interface_param_async` applies to a default
    with a body as well as to a required signature, points at the second
    parameter (`f`, column 25), and keeps the same wording with these names.
17. **b12** — that `generator_in_extend`'s message is the same for an
    `extend` of a struct as of `[T]`, and that nothing is reported first
    (such as the interface or the struct lacking `spread`).
18. **b13** — that a generator *method* with an interface parameter is refused
    with `generator_block_param`'s message (shown there only for a block
    parameter of a free function), with the caret at `who`. `collections.md`
    says only "Not yet: ... a generator that takes a block or an interface
    value".
19. **b14** — that the result of a plain (no-`yield`) `-> Iterator[Int]`
    function, bound to a local, is used up like a generator rather than
    becoming a list ("Binding a chain to a name gives a list", `methods.md`),
    and that the message still calls it "a generator". `collections.md` says
    such a function "gives a sequence the same way" as a generator, but no
    example walks one twice.

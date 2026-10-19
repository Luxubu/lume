# Reviewer B — round nineteen notes

## What I read

- `corpus/m90/BRIEF.md`.
- `docs/reference/collections.md` (all of it, especially "Generators",
  "Async generators" and "Chains over an async sequence").
- `docs/reference/concurrency.md` (all of it, especially "Whichever finishes
  first: `next_done`", what a task copies, `shared var` warnings).
- `docs/reference/interfaces.md` (all of it: `extend`, generators in an
  `extend`, interface values, the `[Shape]` binding warning).
- `docs/reference/functions-and-blocks.md` (all of it: kept blocks, what they
  copy, `shared var` in a block).
- `docs/reference/printing.md`; parts of `structs-and-enums.md` (enum
  methods, `match self`), `generics.md` (generic structs and enums),
  `patterns.md` (indented match arms, `break` in an arm); a grep of
  `strings.md` and `methods.md` for `reverse` and `split`.
- `examples/`: `async_generators`, `generators`, `generic_extend`,
  `kept_blocks`, `async`, `enumerate_blocks`, `iterators`, `tasks_blocking`
  (`.lume` and `.expected`).
- `examples/errors/`: `async_chain_self`, `async_gen_chain`,
  `async_gen_method`, `async_gen_method_outside_async`,
  `async_gen_outside_async`, `async_gen_print`, `next_done_not_awaited`,
  `next_done_not_var`, `filter_not_bool`, `block_kept_changes_var`,
  `block_kept_self`, `generator_in_extend`, `generator_block_param`,
  `task_takes_generator`, `generator_used_twice`, `chain_no_items`,
  `count_without_block`, `spawn_uses_self`, `spawn_changes_copy`, `typo`,
  `no_field`, `option_method`, `tuple_method`, `method_no_block`,
  `block_on_int`, `for_two_vars_not_pairs`, `generator_var_self`,
  `generator_as_interface`, `generator_in_list`, `own_task_args`,
  `yield_in_spawn`, `immutable`, `push_immutable`, `block_wrong_result`,
  `block_type_mismatch`, `builtin_arg_type`, `compare_not_bool`,
  `extend_generic_unnamed`, and `UNREACHED.md`. I used these for the layout
  of an error (gutter width, caret column) and for the wording of messages.

I did not read the compiler, `tests/`, `tools/`, other corpus folders or any
`.lume` folder, and ran nothing.

## Programs

| File | Run/Fail | What it probes |
|---|---|---|
| `b01_chain_methods.lume` | RUN | every async chain method; chains of chains; `take(0)` (also on an endless one); `skip` to and past the end; `take` more than there are; a two-name block after `enumerate` |
| `b02_chain_walk.lume` | RUN | `for i, w` and `for (i, w)` over an async `enumerate`; `.next` on a `var` chain, then `for`/`.to_list` carrying on; `.next` after a `take(1)` chain ended |
| `b03_chain_captures.lume` | RUN | a chain's block copying text and a list; a `var` changed after the chain is made; a `do` block changing a `shared var`; a block reading a `shared var` changed later; a chain moved into a `spawn:` |
| `b04_async_methods.lume` | RUN | async generator methods on a struct (copy taken at the call), an enum (`match self`, `Off -> return`), a generic struct; called in a one-line and an indented `spawn:` |
| `b05_next_done.lume` | RUN | `next_done` on an empty `var` list; picking the 1 ms task of three; `await tasks` after; draining with `while`/`match`/`break`; pushing to the drained list and awaiting it |
| `b06_extend_gens.lume` | RUN | generators in `extend` of a struct, of `Str` (bare `split`), of `[T]` (for a plain interface), of `Crate[T]`; a struct's extend generator walks the copy from the call |
| `b07_extend_iface.lume` | RUN | extend generators through an interface parameter, through a `[Walk]` of a struct, a `Str` and an `[Int]`; an interface default built on the generator, also on an empty `[Int]` |
| `b08_gen_blocks.lume` | RUN | generators keeping a block: a `var` changed after the call; one handed back by a function whose block uses that function's parameter; the same inside an async generator; plain and async generator methods taking a block |
| `b09_gen_iface.lume` | RUN | generators keeping an interface value: struct, enum (both variants), `Str` via `extend`, items of a `[Sized]`, an async generator, a `var` replaced after the call |
| `b10_chain_copy_change.lume` | FAIL | an async chain's `do` block changing a copied `var` |
| `b11_async_filter_int.lume` | FAIL | an async `filter` block giving an `Int` |
| `b12_source_after_chain.lume` | FAIL | the async generator used after a chain took it |
| `b13_async_join.lume` | FAIL | `join` on an async chain (`map` then `join`) |
| `b14_method_plain_def.lume` | FAIL | an async generator method walked in a plain `def` called from async `main` |
| `b15_self_in_gen_chain.lume` | FAIL | a chain block using a field inside an async generator method, whose body already owns a copy of `self` |

## Guesses

1. **Two-name block after an async `enumerate`** (`.map { |i, n| .. }`)
   works as it does on a list or a plain sequence. The docs show two-name
   blocks only on lists and plain sequences; the async examples use `|p| p.0`.
   *b01.* Looked in: collections.md "Pairs", enumerate_blocks.lume,
   async_generators.lume.
2. **`take(0)` gives `[]` and stops at once**, endless source included.
   *b01.* Looked in: collections.md "Chains over an async sequence" ("one
   that has its last item lets go of its source").
3. **`skip` past the end gives `[]`** and does not stop the program. *b01.*
   Looked in: collections.md.
4. **`for (i, w) in` (with brackets) works over an async chain.** It is
   shown for lists only. *b02.* Looked in: enumerate_blocks.lume,
   collections.md.
5. **`.next` on a `var` holding an async chain** (not a bare generator)
   works, and a later `for`/`.to_list` carries on from there. The docs say
   so for a `var` holding an async generator, and that the chains "give an
   async sequence too". *b02.* Looked in: collections.md "Async generators".
6. **Printing `Some((0, "ash"))`**: a tuple inside `Some` keeps its own
   brackets. *b02.* Looked in: printing.md (no tuple-in-`Some` example).
7. **`.next` on an ended `take(1)` chain gives `None` each time** after the
   first item. *b02.* Looked in: collections.md ("Once it has ended, `next`
   gives `None`", said of generators).
8. **A chain (not a bare generator) moved into `spawn:` and walked there**
   works; the task takes it. *b03.* Looked in: collections.md ("a task that
   uses one takes it"), async_generators.lume (a bare generator only).
9. **A multi-line `do |n|` block on an async chain method**, bound to a name,
   compiles. *b03, b10.* Looked in: functions-and-blocks.md (the `do` form),
   collections.md.
10. **A chain block reading a `shared var` sees its value when the item
    arrives**, not when the chain was made, so `[11, 12]`. *b03.* Looked in:
    functions-and-blocks.md (kept blocks and `shared var`), collections.md.
11. **The `shared var` warnings**: one per declaration, in source order, on
    stderr, path printed as `corpus/m90/<file>`. *b03.* Looked in:
    concurrency.md "Where the warnings go", kept_blocks.expected.
12. **`Off -> return` as one arm and an indented `for`/`yield` as the other**
    in an async generator's `match self`. *b04.* Looked in: patterns.md,
    collections.md ("`return` alone ends it").
13. **A one-line `spawn: Ring(..).twice.to_list` with no `await` written**
    works, though a body with no `await` is said to run as blocking work.
    *b04.* Looked in: concurrency.md "A task that waits without `await`",
    collections.md ("`.to_list` wait[s], so [it goes] in ... a `spawn:`").
14. **An async generator method walks the copy from the call**: changing
    `s.label` afterwards does not show. *b04.* Looked in: collections.md ("Its
    task walks a copy of the value, as a generator method does").
15. **`await idle.next_done` on an empty `var` list gives `None` at once.**
    *b05.* Looked in: concurrency.md ("`None` once the list is empty").
16. **`break` in a `match` arm leaves the `while`; a list drained by
    `next_done` can be pushed to and awaited again.** *b05.* Looked in:
    patterns.md, concurrency.md.
17. **`extend [T] with Walk` for an interface with no type parameter**: the
    `T` comes only from the target. *b06, b07.* Looked in: generic_extend.lume
    (only with `Bag[T]`), interfaces.md, extend_generic_unnamed.
18. **`extend Str with Walk` and `extend [T] with Walk` side by side**, with
    no overlap error. *b06, b07.* Looked in: interfaces.md, the
    `extend_*_overlap` error names (not read).
19. **A struct's `extend` generator walks the copy from the call**, so
    `p.stops = [..]` afterwards does not show: `["to x"]`. *b06.* Looked in:
    interfaces.md ("A generator default walks a copy"), collections.md.
20. **An interface default (`first_step`) built on a required generator
    method, with `.first` on the sequence**, works on a value through a
    pointer and on an `[Int]` directly. *b07.* Looked in: interfaces.md "An
    interface's own methods may use interfaces".
21. **`[7]` as an item of a `[Walk]` literal** is taken as an `[Int]` that
    fits through `extend [T]`. *b07.* Looked in: interfaces.md (`walks:
    [Walk] = [Bag(..), [10, 20]]`, with a concrete `extend [Int]`).
22. **The binding warning for `ws` and `held`**, in the wording interfaces.md
    shows, on stderr. *b07, b09.* Looked in: interfaces.md.
23. **A function whose body is `= keep([..], { |x| x > limit })`** hands back
    a generator whose block keeps `limit`. *b08.* Looked in:
    functions-and-blocks.md ("A function may keep the blocks it is given"),
    collections.md.
24. **A trailing block on a generator method call, then `.to_list`.** *b08.*
    Looked in: functions-and-blocks.md, generators.lume.
25. **Passing a held interface item (`held.at(0)`) to a parameter typed
    `Sized`** is accepted, since `Sized` has no generic or async method.
    *b09.* Looked in: interfaces.md (it refuses only the generic/async case).
26. **A bare variant `Small` passed to an interface parameter.** *b09.*
    Looked in: structs-and-enums.md.
27. **b10's message is the kept-block one**, with the caret at `total`.
    Looked in: block_kept_changes_var, generator_block_param, collections.md
    ("changing one inside is refused").
28. **b11's message is the list one** (`the block of `filter` gives back an
    `Int`...`), with the same help. Looked in: filter_not_bool.
29. **b12 is caught at the chain line** (`c = g.map`, caret at `g`) with
    generator_used_twice's wording ("is a generator"), not at the later
    `g.to_list`, and not worded "async generator". Looked in:
    generator_used_twice, collections.md ("A chain moves the sequence").
30. **b13: the type is written `async Iterator[Str]` after a `map`**, the
    caret is at `.join`, and the help is async_gen_chain's. Looked in:
    async_gen_chain.
31. **b14: the error is at `s.titles` inside the plain `def`**, named
    `` `s.titles` ``, caret at the `.`. Looked in:
    async_gen_method_outside_async (which names `Feed(...).items`),
    async_gen_outside_async.
32. **b15 is refused even inside an async generator method**, whose own body
    walks a copy of `self`, with async_chain_self's wording for `take_while`.
    Looked in: async_chain_self, collections.md.
33. **`lume check` prints only the error block for a FAIL**: no summary
    line, no warnings (none of the FAIL files have a `shared var` or an
    interface-list binding). Looked in: examples/errors/*.expected.

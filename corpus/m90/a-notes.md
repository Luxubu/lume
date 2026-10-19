# Reviewer A, round nineteen (m90)

## What I read

Only `docs/` (not `QUESTIONS.md`), `examples/` and the brief. I did not run the
compiler or any program; every expected output was worked out by hand.

- `docs/README.md`
- `docs/reference/collections.md`, all of it (Generators, Async generators,
  Chains over an async sequence, Taking a sequence)
- `docs/reference/concurrency.md`, all of it (`next_done`, the worker-pool
  generator, copies into tasks, warnings)
- `docs/reference/interfaces.md`, all of it (generator defaults, generators
  in `extend`, interface values, the binding warning)
- `docs/reference/functions-and-blocks.md`, all of it (kept blocks, copying)
- `docs/reference/printing.md`, `structs-and-enums.md`, `control-flow.md`
- `docs/reference/operators.md` (the precedence table), `errors.md` (first
  half), `generics.md` ("Generic `extend`" onwards), `methods.md` (up to the
  list table), `names.md` (keywords, refused type names)
- `examples/async_generators.lume` + `.expected`, `examples/generators.lume`
  + `.expected`, `examples/kept_blocks.lume` + `.expected`, the start of
  `examples/async.lume`, and the warning headers of
  `examples/interfaces.expected` and `examples/interface_methods.expected`
- `examples/errors/`: `async_chain_self`, `async_gen_chain`,
  `async_gen_method`, `async_gen_method_outside_async`,
  `async_gen_outside_async`, `async_gen_print`, `next_done_not_awaited`,
  `next_done_not_var`, `generator_in_extend`, `generator_block_param`,
  `generator_used_twice`, `generator_var_self`, `task_takes_generator`,
  `block_kept_changes_var`, `block_kept_self`, `yield_in_block`,
  `iface_unused_param`, `reserved_type` (each `.lume` and `.expected`). These
  gave the exact layout of an error: the `  --> file:line:col` line, a gutter
  as wide as the line number plus one, and a caret at the column.

## The programs

| file | meant to | what it does |
|---|---|---|
| `a01_paged_fetch.lume` | RUN | Paginated fetcher: an async generator walks pages until an empty one (`return`), chained with filter/map, reject/take, skip/enumerate, take_while/map; `next` on a `var` then a `for` that carries on; an empty result. |
| `a02_worker_pool.lume` | RUN | `await tasks.next_done` one at a time down to `None`, `len` after each; the rest awaited in start order; a drain loop; a bounded pool as an async generator with chains over it (results sorted before printing). Sleeps 1 / 200 / 400 ms. |
| `a03_feed_reader.lume` | RUN | Async generator methods on a struct and on an enum (`match self` arms that yield); another method walking one with `for`, and chaining over one with the field bound to a name first; the copy of the value taken at the call. |
| `a04_report_extend.lume` | RUN | Generators in `extend`s of `{Str: Int}`, `[Int]`, `Str` (with a parameter) and a struct; an interface default that is a generator built on the required one; chains over what they give. |
| `a05_retry_block.lume` | RUN | A retry policy as a block kept by an async generator (a `var` it reads changed after the call); a kept block from a binding; an endless plain generator taking a block or a function's name; chains over each. |
| `a06_policy_interface.lume` | RUN | Generators that keep an interface value: items of a `[Policy]` list, a concrete value changed after the call, an interface default (`name`); an async generator of pairs walked with two names and chained. Predicts the `[Policy]` binding warning on stderr. |
| `a07_sensor_chains.lume` | RUN | An endless async generator of readings ended by take, take_while, skip after map, reject then enumerate, a `break`; an async chain bound to a name whose block copied a `var` changed afterwards; an async generator that walks a chain over another in its body. |
| `a08_digest_field_in_chain.lume` | FAIL | A struct method chains over an async generator and the `reject` block reads a field. Expected: the "runs as items arrive" error at the block's `{`. |
| `a09_orders_fold.lume` | FAIL | `fold` on an async generator. Expected: "an `async Iterator[Order]` has no `fold` yet" with the help naming the chains. |

## Guesses

1. **An async chain can be bound to a name and walked later, and its block
   keeps what it copied when the chain was made.** `high =
   readings(1).filter { |r| r > threshold }`, then `threshold = 100`, then
   `high.take(3).to_list` gives `[23, 14, 23]`. The docs show async chains
   only used on the line that makes them. Files: `a07`. Looked: collections.md
   "Chains over an async sequence" ("copied in when the chain is made", "A
   chain moves the sequence it is made from"), and "It moves like a
   generator" (a generator may be kept in a local).
2. **An item of a `[Policy]` list can be handed to a generator's `p: Policy`
   parameter**, and the `for p in policies` loop variable gives no warning of
   its own. The only stderr is the one warning for the `policies` binding.
   Files: `a06`. Looked: interfaces.md (a held value is refused at an
   interface parameter only when the interface has generic or `async`
   methods; "only a binding" warns), collections.md ("An interface value is
   copied through `lume_box`, as a list of interface values copies its
   items").
3. **An async generator method can be called by its bare name inside another
   method of the same type and chained** (`posts.filter { .. }` in
   `Feed.long_ones`; `for p in posts` in `total_words`). The docs give the
   bare-name rule for methods in general but never show it for a generator
   method. Files: `a03`. Looked: structs-and-enums.md "Calling another
   method of the same type", collections.md "A method may be one".
4. **`fold` gets the same message as `count`, and the type in it is spelled
   `async Iterator[Order]`** for a struct item type. The caret goes on the
   `.` before `fold`, as in `async_gen_chain`. Files: `a09`. Looked:
   `examples/errors/async_gen_chain.expected`, collections.md ("the methods
   that wait for every item, like `count` or `fold`").
5. **In `a08` the only error is the one for `reject`**, with the caret on
   its `{`, as `async_chain_self` puts it on `filter`'s. `take(most)` reads a
   field on the same line, but not in a block, so I take it as allowed (the
   same line also appears in the valid `Weekly.picks`). Files: `a08`.
   Looked: collections.md (the `lume-bad` block and its rule names only
   blocks), `examples/errors/async_chain_self.expected`.
6. **An unused name in a two-name block (`{ |k, v| k.upcase }`) gives no
   warning**, and a fully concrete map target `extend {Str: Int} with ..`
   works like the documented `extend {K: [Int]}`. Files: `a04`. Looked:
   methods.md (`{ |name, n| name }` in its examples, with no stderr shown),
   generics.md "Two `extend`s of one interface".

Things I checked rather than guessed, so they are not listed: `next_done`
giving `None` on an empty list; `return` ending an async generator; `next` on
a `var` async generator followed by a `for` that carries on; a block given
to a generator being copied at the call; a generator method walking a copy
taken at the call; `not` binding tighter than `and`; `1..n` including `n`;
`pad` and `pad_right` never cutting text short; nested quotes inside `#{}`;
the warning's layout.

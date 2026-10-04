# Reviewer B — round fifteen (milestones 70–72): the edges

## What I read

- `corpus/m73/BRIEF.md`.
- `docs/reference/collections.md`, all of it, and closely: "Types of your
  own: `next`", "Generators: `yield`", "Taking a sequence: `xs: Iterator[T]`",
  "Lists" (literals across lines) and "When a list is copied, and when it
  moves" (the `split` slices paragraph).
- `docs/reference/control-flow.md` (trailing `if`, `match` arms, loops),
  `docs/reference/functions-and-blocks.md` (block forms, `do`, function
  names as blocks, `var` parameters), `docs/reference/printing.md`,
  `docs/reference/strings.md` (`split`, `lines`), `docs/tour.md`; greps of
  `docs/reference/concurrency.md` (`async def`, `spawn:`) and
  `docs/reference/interfaces.md` for `Iterator`.
- `examples/generators.lume` + `.expected`, `examples/iterators.lume` +
  `.expected`, `examples/split_slices.lume` + `.expected`,
  `examples/sets.lume` (set order), `examples/logq/main.lume`,
  `examples/logq/FINDINGS.md`, multi-line literals in
  `examples/collections.lume`, `examples/kept_blocks.lume`,
  `examples/shipping/main.lume`, `examples/fmt/messy.lume`.
- `examples/errors/`: every `generator_*`, `yield_*`, `iterator_*`,
  `items_and_next`, `next_wrong_shape`, `for_var_next`, `for_var_items`,
  `loop_no_items`, `task_used_after_move`, `tasks_used_after_await`,
  `var_param_*`, `return_*`, `dropped_result`, `UNREACHED.md`, and the list
  of every `^error` line across the `.expected` files (for the wording of
  argument and type messages, and the gutter width at two-digit lines).
- Not read: compiler source, `tests/`, `bench/`, other `corpus/` folders,
  `README.md`, `HISTORY.md`, `CHANGELOG.md`, `docs/QUESTIONS.md`, and the
  generated Rust under `examples/*/.lume/` (it turned up in a grep; I did not
  open it).

## The programs

| file | probes |
|---|---|
| `b01_gen_shapes.lume` (RUN) | `yield` in `if`/`elif`/`else`; in `match` arms, one with a trailing `yield x if c`; bare `return` mid-loop; a generator that reaches no `yield` (printed, `len`); `next` by hand past the end, twice |
| `b02_gen_generic_zip.lume` (RUN) | generic generator of pairs (`for a, b`, two-name block); an empty one; a generator that calls itself; `zip` with a generator on the left, on the right of a list, and an endless one zipped with a list |
| `b03_seq_params.lume` (RUN) | one `xs: Iterator[Int]` given a generator, a chain, a range, a list, a set with a duplicate, a struct with `next` moved by hand first; `xs.next` then the rest handed on to another `Iterator[Int]` function; caller's list and struct unchanged |
| `b04_chain_into_gen.lume` (RUN) | when a chain is worked out: into a plain function (lazy, interleaved) vs into a generator (worked out first); side effects in the `map` show the order |
| `b05_split_slices.lume` (RUN) | `split` from a `var` that grows after; list returned from a function; items kept in struct fields; items compared and interpolated; list given to a block; nested splits read by position; list walked with `for` after its source was replaced, then printed |
| `b06_multiline.lume` (RUN) | nested lists broken mid-list with trailing commas at each level; a map; a call with one argument per line and a trailing comma; a list argument opened on the call's line; a set; a chain after a multi-line list's `]` |
| `b07_gen_as_list.lume` (RUN) | a generator where a list is wanted: a `[Int]` field and a `[Int]` parameter; a generator moved by `next`, then printed (what is left) |
| `b08_gen_after_for.lume` (FAIL) | generator walked by `for`, then `.len`: error at the `for` |
| `b09_gen_after_call.lume` (FAIL) | generator moved into an `Iterator[Int]` parameter, then used again by the caller |
| `b10_gen_in_tuple.lume` (FAIL) | generator in a tuple |
| `b11_gen_in_field.lume` (FAIL) | generator in a struct field typed `Iterator[Int]` |
| `b12_yield_in_main.lume` (FAIL) | `yield` in `main` |
| `b13_yield_in_do_block.lume` (FAIL) | `yield` on its own line in a `do \|x\|` block inside a generator |
| `b14_async_generator.lume` (FAIL) | `async def` with `yield` ("Not yet") |
| `b15_param_used_twice.lume` (FAIL) | an `xs: Iterator[Int]` parameter walked twice inside the function |

## Guesses

1. **b01** — `next` on a generator that has already ended gives `None` again
   (and again). The docs say what happens after `None` only for a type of
   your own ("gives whatever your code gives"); nothing for a generator.
   Looked: collections.md "Types of your own: `next`" and "Generators",
   `examples/generators.lume` (stops at the first `None`).
2. **b01** — a trailing `if` on a `yield` that is a one-line `match` arm
   (`Some(v) -> yield v * 10 if v > 1`) guards the `yield`, not the arm.
   Trailing `if` on a statement and `yield` in an arm are each shown
   (control-flow.md; `examples/logq/main.lume` `Ok(e) -> yield e`), never
   together.
3. **b03** — an `xs: Iterator[T]` parameter, after `xs.next`, can be handed
   on whole to another `Iterator[T]` parameter, and the second function sees
   only the rest. `examples/generators.lume` does `xs.next` then
   `xs.to_list`, and the docs say a generator may be "handed on", but no
   example hands a *parameter* on.
4. **b04** — a chain given to a generator's parameter is worked out at the
   call, so its side effects come before the line after the call ("made"),
   not when the first item is asked for. collections.md says only "it is
   worked out first"; `examples/logq/FINDINGS.md` says the same.
5. **b06** — a comma after the last *argument of a call* is accepted. The
   docs' sentence follows "lists, maps and argument lists", but the only
   example of a trailing comma is a list. Looked: collections.md "Lists",
   every multi-line literal in `examples/`.
6. **b06** — continuation lines inside brackets may be indented however you
   like (`      3],`), and a line may start with a closing bracket followed by
   more arguments (`  ], {"k": 1})`). Not shown anywhere;
   `examples/fmt/messy.lume` comes closest.
7. **b06** — a chain may follow a multi-line list's `]` on its own line
   (`].map { .. }.join(",")`). No example; looked as for 5.
8. **b07** — "Given where a list is wanted, it gives its items" covers a
   `[Int]` struct field and a `[Int]` function parameter, not only a typed
   binding (the only form `examples/generators.lume` shows).
9. **b09** — the wording when a generator is moved into an `Iterator[Int]`
   parameter and then used again: I assumed the "used up here" message of
   `generator_used_twice`, pointing at the argument. The task message says
   "given away here" for a value pushed into a list, so this may say that
   instead.
10. **b10** — a tuple gets the same message as a list ("kept in a list or a
    tuple"), with the caret on the tuple's `(` as the list one is on the `[`.
    Only the list case has an example (`generator_in_list`).
11. **b11** — the whole message for a generator put in a field, and that a
    field may be typed `Iterator[Int]` at all (the docs say only that "a
    value can be held as an `Iterator[Int]`"). It could instead give the
    `generator_as_interface` message ("cannot be held as an `Iterator[Int]`
    value"). Caret on the generator call guessed.
12. **b12** — `yield` in `main`: assumed the `yield_outside_generator`
    message at the `def`, with the template help naming `main`. It could
    instead be the `yield_in_test` shape ("... and `main` is not one") at the
    `yield`.
13. **b14** — the message, help and position for an `async` generator are
    all guessed (modelled on `generator_method`); collections.md lists it
    under "Not yet" and no example shows it.
14. **b15** — that walking an `xs: Iterator[T]` parameter twice is refused
    at compile time at all (collections.md: "walked like a generator, once"),
    and that the message calls `xs` "a generator" with the
    `generator_used_twice` wording, at the first walk.

# Reviewer B — notes, round eighteen (m86)

## What I read

- `docs/reference/collections.md` (generators, "Async generators", taking a
  sequence), `docs/reference/concurrency.md` (whole page),
  `docs/reference/io.md` (`Process`, `Dir`, `File`, `Path`),
  `docs/reference/printing.md`, `docs/reference/control-flow.md` (ranges,
  `match`, `()`), `docs/reference/structs-and-enums.md` (`def self.`),
  `docs/reference/errors.md` (`main -> () or Error`).
- `examples/async_generators.lume`, `examples/generators.lume`,
  `examples/async.lume`, `examples/tasks_blocking.lume`, and the `.expected`
  of each; `examples/logq/main.lume` (a `match` arm that yields).
- `examples/errors/`: `async_gen_*`, `generator_*`, `yield_*`, `task_*`,
  `tasks_used_after_await`, `spawn_*`, `process_*`, `await_*`,
  `async_not_awaited`, `own_task_*`, and `UNREACHED.md`.
- `packages/http/.lume/lib-test.rs`, lines 460-480 only: the generated
  runtime's `lume_process_run_in`. This is a build artifact that sits under
  `packages/` (so the brief allows it), and it is the only place I found the
  text of the "folder that does not exist" error:
  ``cannot run `PROGRAM` in `DIR`: no such folder``. The docs do not give it.
  Worth knowing: `packages/*/.lume/` holds generated Rust, which shows the
  runtime to any reviewer.

## The programs

| file | probes |
|---|---|
| b01_gen_shapes.lume | RUN: `yield` in `for`/`if`/`match` (with a `()` arm), `yield "..#{await f()}"`, `yield await f()`, bare `return` mid-loop, a generator that yields nothing (`[]`, empty `for`) |
| b02_gen_generic_params.lume | RUN: generic async generator at `Str`, `Int`, `Bool`; one taking a list (caller's list unchanged); one taking `xs: Iterator[Int]` given a plain generator, a list, a range |
| b03_gen_walks.lume | RUN: `break` out of an endless one; `next` to and past the end (None twice); two made, then walked one after the other; `.to_list`; `next` then `for` carrying on; pairs with `for a, b in` and `next` on pairs |
| b04_spawn_captures.lume | RUN: a `spawn:` copies a list, a struct, text and a `var Int` changed afterwards; one-line `spawn:` with and without `await`; a task that takes another task; a task that takes and walks an async generator; async generators started in tasks made in a block |
| b05_process_run_in.lume | RUN: `Process.run_in` exit code and both outputs; `sh -c` with `>&2` and `exit 5`; a relative folder; a program that fails (not an `Error`); a missing folder (`Error` and its message) |
| b06_process_in_tasks.lume | RUN: `Process.run` in one-line tasks, results in start order; `?` inside a task, `await t?`; `run_in` on a missing folder inside a task |
| b07_static_async_gen.lume | FAIL: `async def self.upto` that yields, in a struct |
| b08_gen_used_after_for.lume | FAIL: async generator walked by `for`, then `.to_list` |
| b09_gen_in_plain_def.lume | FAIL: async generator with an argument called in a plain `def` helper |
| b10_gen_printed.lume | FAIL: `puts g` on a local holding an async generator |
| b11_gen_filter.lume | FAIL: `g.filter { .. }.len` on a `var` async generator |
| b12_gen_method_params.lume | FAIL: async generator method with a parameter that reads a field |
| b13_task_takes_async_gen.lume | FAIL: a one-line `spawn:` takes an async generator; `for` over it afterwards |
| b14_task_takes_task.lume | FAIL: a `spawn:` awaits another task; that task is awaited again after |
| b15_yield_in_spawn_async.lume | FAIL: `yield` inside `spawn:` inside an async generator (its help line still says a generator cannot be `async`) |

## Guesses

1. **b01** — `await` inside a string interpolation inside a `yield`
   (`yield "doubled #{await double(x)}"`) is accepted. Looked in
   concurrency.md ("`await` covers everything to its right") and
   collections.md; no example puts `await` inside `#{}`.
2. **b01** — bare `return` ends an async generator as it ends a plain one.
   collections.md says so for generators; the async section only says "it
   moves like a generator".
3. **b02** — a generic async generator (`async def echo_n[T](..)`) is
   allowed. Generic plain generators are shown (`repeat[T]` in
   examples/generators.lume); nothing says either way for async ones.
4. **b02** — an async generator may take `xs: Iterator[Int]` and be handed a
   plain generator, a list or a range. Documented for plain generators only
   ("Taking a sequence").
5. **b03** — `next` on an ended async generator gives `None` again on every
   later call. Stated for plain generators ("Once it has ended, `next`
   gives `None` every time"); assumed to hold for async ones.
6. **b03** — `Some` of a tuple prints as `Some(("cyd", 3))`. printing.md
   shows `Some("pear")` and tuples separately, never nested.
7. **b04** — calling an async generator and `.to_list` on it inside a
   one-line `spawn:` made inside a `map` block, with no `await` keyword in
   the task body. examples/async_generators.lume walks one in a `spawn:`
   without `await`, but not inside a block.
8. **b05** — the program runs with the repository root as its working
   folder, so `Process.run_in("corpus/m86", ..)` finds `BRIEF.md`. The
   brief says `lume check` is run from the root; I assumed `lume run` is too.
9. **b07** — a static `async def self.` async generator counts as an async
   generator *method* and is refused with the method message, caret at
   `async`. Nothing in the docs mentions `async def self.`; this could as
   well compile and print `[1, 2, 3]`.
10. **b08** — an async generator used up by `for` gives the plain
    generator's message ("`g` is a generator, and it is used up here..."),
    caret at `g` in `for x in g`. Looked at generator_used_twice; the docs
    say only "it moves like a generator".
11. **b09** — the message spells the call `nums()` even though it was
    written `nums(3)`. async_gen_outside_async only shows a call with no
    arguments.
12. **b10** — printing a *local* holding the async generator gives the
    same message as `puts nums()`, caret at the local's name.
13. **b11** — `filter` on a `var` gives the same message as `map` on a call,
    the type written `async Iterator[Int]`, caret at the `.`.
14. **b13** — a task taking an *async* generator gives the plain
    generator's message from task_takes_generator, caret at `spawn`.
15. **b14** — a task taking another task: the message is the
    "cannot be copied, and the task takes it" one, caret at `spawn`.
    No example shows this case; it might instead be the "`inner` is a task,
    and it is given away here" message with the caret at `inner` on the
    `await inner` line.
16. **b15** — `yield` in a `spawn:` inside an *async* generator gives the
    same error, help line unchanged (it still says "a generator cannot be
    `async`", now untrue).

# Reviewer A, round eighteen

## What I read

- `corpus/m86/BRIEF.md`.
- `docs/tour.md`, and in `docs/reference/`: `concurrency.md`,
  `collections.md` (with "Generators" and "Async generators"), `io.md`
  ("Process", "File", "Dir", "Path"), `errors.md`, `printing.md`,
  `structs-and-enums.md`, `control-flow.md`, `functions-and-blocks.md`,
  `names.md`, `methods.md`, `strings.md`, the head of `patterns.md`, and
  the constants section of `modules.md`.
- `examples/async_generators.lume` and `.expected`, `examples/async.lume`
  and `.expected`, `examples/tasks_blocking.lume` and `.expected`,
  `examples/generators.lume` and `.expected`, `examples/jobr/jobr/main.lume`,
  the head of `examples/healthcheck/main.lume`.
- `examples/errors/`: `async_gen_chain`, `async_gen_method`,
  `async_gen_outside_async`, `async_gen_print`, `async_not_awaited`,
  `generator_used_twice`, `generator_in_list`, `process_args_list`,
  `process_no_try`, `task_takes_generator`, `task_used_after_move`,
  `tasks_used_after_await`, `yield_in_spawn`, `task_or`/`ok`/`error`/
  `or_error`, `own_task_args`, `spawn_uses_self` (sources and `.expected`).
  These gave me the exact layout of an error and of a warning.
- `packages/http/.lume/lib-test.rs`, the runtime for `Process.run_in`, for
  the error text of a folder that does not exist
  (`cannot run `ls` in `<dir>`: no such folder`). It is generated code
  under `packages/`, so it was within the brief; no page states this text.

Nothing outside those folders; no compiler run, no program run.

## The programs

| file | what it is | leans on |
|---|---|---|
| `a01_paged_api.lume` | an async generator that pages through a fake API (`async def fetch_page -> Page or Error`), following each page's link; walked with `for`, `break`, `.to_list`, into a map, and from an `async def` that returns early | async generators, `match await ..`, `yield` in `match` arms, `T?` fields, map constants, `index[k].push`, `return Error(..)` |
| `a02_poller.lume` | a poller that yields enum readings until one is final, then `return`s; walked with `for`/`match`, `.next` on a `var` past the end, `.to_list`, and into a map | async generators, bare `return` in one, `.next` after the end, enums with methods, `if` as a value, multi-line `match` as a value |
| `a03_job_runner.lume` | a sequential job runner: structs for jobs, an enum for outcomes; `sh -c` with `printf`, `sort`, `ls`, `echo`, a non-zero exit, a missing program, a missing folder; files under `/tmp/lume-review-a/jobs` | `Process.run`, `Process.run_in`, `Ok((code, out, err))` patterns, raw strings, `pad_right`, `Dir`, `File`, `Path`, map `filter`/`keys` |
| `a04_retry.lume` | a retry loop around a command that keeps a counter in a file and passes on its third run; a command that always fails; a program that cannot start; two retry runs as tasks with `?` inside | `Process.run_in` with `?` in an `async def -> [T] or Error`, `await Time.sleep` back-off, `await f()?`, `?` inside `spawn:`, tuple destructuring with `_` |
| `a05_pipeline.lume` | a pipeline: an async generator parses lines into events; one task takes a generator made outside it, one makes its own, one keeps `.to_list`; a second async generator walks the first and yields only sales | tasks taking a generator, structs and maps returned from tasks, `match` on `Str` literals, `Error` messages from `to_int`, an async generator inside an async generator |
| `a06_fanout.lume` | a fan-out: one task per feed, each walking its own async generator (made in a loop, taken by its task); different waits, results in start order; a `shared var` counter; a second fan-out with one-line `spawn:` in a `.map` | `await` on a list of tasks, `Task[[Str]]`, `zip`, `shared var` and its warning on standard error, constants holding structs |
| `a07_note_reader.lume` | an async generator that yields `Note or Error` for each file in a folder, reading each with `cat` through `Process.run_in`; a sub-folder gives an error item, a missing folder an error from `Dir.list` | `Iterator[T or Error]`, `yield Ok(..)`/`yield Error(..)`, `Dir.list`, `Path.ext`, `ls`, printing `Some(Error(..))` |
| `a08_shared_feed.lume` | FAIL: an async generator handed to a task, then walked again by `main` | "a task that uses one takes it" |
| `a09_filter_readings.lume` | FAIL: `.filter { .. }` straight on an async generator | "Not yet: chain methods over it" |

## Guesses

1. **a01**: `return Error(..)` from inside a `for` over an async generator
   (in `count_from`) compiles and lets go of the generator. The docs say
   letting go ends it "after a `break`, or once nothing holds it"
   (collections.md, "Async generators"); a `return` out of the loop is not
   named. I assumed it behaves like `break`.
2. **a02**: a multi-line `match` can be the value of an assignment to a map
   key, `report[job] = match s:` with the arms indented below. control-flow.md
   shows a multi-line `if`/`match` only after `name =`; concurrency.md shows
   `counts[key] += 1` but not a multi-line value.
3. **a02**: `.next` on a `var` async generator after it has ended (by a bare
   `return`) gives `None`. collections.md says this for plain generators
   ("Once it has ended, `next` gives `None` every time") and that `s.next`
   works on an async one; it does not say the "after the end" part for the
   async kind.
4. **a05**: an async generator may walk another async generator with `for`
   in its own body (`sales_only` over `events`). collections.md says `for`
   over one works "in async code", and an async generator's body may
   `await`, but no example or page shows one inside another.
5. **a05**: `yield` as a one-line `match` arm (`Sale(item, c) -> yield
   (item, c)`), with the other arm `_ -> ()`. collections.md says `yield`
   works in the generator's `match`es; every example puts it on its own line.
6. **a06**: `FEEDS.map { |f| spawn: headlines(f, 1).to_list }.to_list`, an
   async generator called and walked in a one-line `spawn:` that sits inside
   a `.map` block. concurrency.md shows `spawn:` in a `.map` block, and
   collections.md says calling one in a `spawn:` body is fine; not both
   together.
7. **a07**: an async generator may give `Iterator[Note or Error]` and
   `yield Ok(note)` / `yield Error("..")`. Nothing shows a result as the item
   type, and errors.md says "You never write `Ok(...)` yourself" (for
   returns), though `Ok(5)` is shown as a value in printing.md. I wrote `Ok`
   out because I could not tell whether a bare `yield note` would be wrapped.
8. **a07**: the text of a failed `Dir.list`:
   ``cannot list `/tmp/lume-review-a/none`: No such file or directory (os error 2)``.
   io.md gives the `File.read` text and says `Dir.list` uses "cannot list";
   I put the two together.
9. **a08**: the error for an async generator taken by a task and used after
   is the same as for a plain generator, word for word, and points at the
   `spawn:` (examples/errors/task_takes_generator.expected). collections.md
   only says "a task that uses one takes it" for the async kind.
10. **a09**: `.filter` on an async generator gives the same error as `.map`
    in examples/errors/async_gen_chain.expected, names the type
    ``async Iterator[Int]``, and points at the `.` before `filter`.

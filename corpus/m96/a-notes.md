# Reviewer A — notes, round twenty (m96)

## What I read

- `corpus/m96/BRIEF.md`.
- `docs/reference/collections.md`: lazy chains, lists, `next`, generators,
  "Async generators", "Chains over an async sequence", "Methods that wait
  for the items", "Taking a sequence".
- `docs/reference/io.md`: all of it, mainly `Process` and "A program running
  alongside: `Process.spawn`".
- `docs/reference/http.md`: all of it.
- `docs/reference/concurrency.md`: all of it (spawn, await on a list, `?` in
  a task, tasks that await nothing).
- `docs/reference/printing.md`, `docs/reference/strings.md`, and parts of
  `errors.md`, `modules.md` (types through a prefix, fields public with a
  `pub` struct) and `packages.md` (`lume.toml`, path dependencies).
- `packages/http/lume.toml`, `packages/http/lib.lume` (to see the shape of
  an Error's message, `"{method} {url}: {reason}"`, and the `Request` and
  `Response` fields), `packages/http/test.expected`.
- `examples/async_generators.lume` + `.expected`,
  `examples/process_spawn.lume` + `.expected`, `examples/http_demo/`,
  the top of `examples/healthcheck/main.lume` and its `lume.toml`.
- `examples/errors/`: `child_lines_outside_async`, `child_wait_not_awaited`,
  `async_gen_print`, `async_gen_outside_async`, `async_gen_chain`,
  `async_not_iterator`, `async_not_awaited`, `next_done_not_awaited`,
  `process_no_try`, `process_args_list`, `spawn_changes_copy`,
  `async_chain_self` (each `.lume` and `.expected`), and `UNREACHED.md`.

Nothing else: no compiler source, `tests/`, `tools/`, other corpus files
or `.lume` folders. I did not run the compiler or any of the programs.

## The programs

| file | meant to | what it does |
|---|---|---|
| `a01_waiting_methods.lume` | RUN | `first`, `find`, `any?`, `all?`, `count`, `sum`, `fold`, `each` on async generators: one that awaits each item, an endless one, a method that is one, an empty one, and after `filter`/`map`/`take`/`enumerate` chains. `each` blocks change the names around them |
| `a02_pipes.lume` | RUN | text piped through `sort` (plain, `-r`, `-n`, `-u`), `tr` (map, squeeze), `wc -l` and `head` by a helper that writes, closes the input, reads the lines and turns a non-zero exit into an `Error`. Two programs in a row, `lines.enumerate`, and a write after `close_input` |
| `a03_conversation.lume` | RUN | a `sh` adding loop, a line out and a line back. A counter that reports its total once its input closes, `read_line` giving `None` at the end, `read(3)` on 8 bytes, and `kill` giving `-1` |
| `a04_errors_and_codes.lume` | RUN | `lines` and `error_lines` with exit codes 0, 1, 3 and 64, a shell that kills itself (`-1`), 3000 stderr lines while stdout is read first, `error_lines.first`, the spawn Errors for a missing program and a missing folder, and `spawn_in` a folder it makes |
| `a05_programs_in_tasks.lume` | RUN | scripts run in tasks with results in the order the tasks started, `Child`ren kept in a list and finished in tasks, `?` inside `spawn:` meeting a program that cannot start, and slow programs started first whose answers still come first |
| `a06_stream_functions.lume` | RUN | plain `def`s that take or give `async Iterator[T]` as chains (`map`, `reject`, `take`, a block that keeps a parameter), an async generator that walks one it is given, and async defs that walk one to an answer. They are fed by generators, a kept chain and a `Child`'s lines |
| `a07_http_requests/` (`lume.toml`, `main.lume`) | RUN | the `http` request builder, with each step giving a new `Request`, headers kept in order and the last timeout winning. Then requests that cannot be made (port 9, a bad URL) by `send`, `fetch`, `open`, `get`, `delete`, `http.send`, `post` and from tasks, and `Response`s made by hand |
| `a08_wait_unawaited.lume` | FAIL | `code = c.wait` with no `await` in an async helper |
| `a09_plain_walk.lume` | FAIL | a plain `def` that takes `xs: async Iterator[Str]` and walks it with `for` |

## Guesses

1. **`await c.read(n)` after the output has ended gives `None`**, not
   `Some("")`. File: `a03_conversation.lume` (the fourth `read(3)`). Where I
   looked: io.md's `Child` table, which says "the next `n` bytes of its
   output, or what is left when it ends first" and gives the type `Str?`;
   `examples/process_spawn.lume` uses `read(n)` only while bytes are left.
   I took `None` from the type and from how `read_line` behaves.
2. **An async generator may take a parameter `xs: async Iterator[Str]` and
   `for` over it** (`numbered` in a06, fed a chain over a `Child`'s lines and
   an async generator). File: `a06_stream_functions.lume`. Where I looked:
   collections.md, "Async generators": "A parameter `xs: async Iterator[T]`
   takes one by move ... a generator's, a chain's, or a `Child`'s `lines`".
   The docs show that parameter only in general terms and never on an async
   generator.
3. **`all?` on an empty async sequence is `true`, and `any?` is `false`.**
   File: `a01_waiting_methods.lume`. Where I looked: collections.md says the
   waiting methods "give what a list's would", but no page shows a list's
   `all?` on an empty list. I assumed it works as Rust's `all` does.
4. **The whole error text of a09.** File: `a09_plain_walk.lume`. Where I
   looked: `examples/errors/UNREACHED.md` lists the message "`{}` hands out
   items that arrive later, so a `for` over it waits for each: it goes in
   async code" as unreachable, because "no parameter type carries one into a
   plain `def`". That note looks older than `xs: async Iterator[T]`
   parameters. I guessed four things:
   - that this message is the one now reached, with `xs` filling the gap;
   - that the caret is under `xs` (column 12);
   - that the help line is `walk it in an `async def` or a `spawn:` body`,
     copied from `child_lines_outside_async.expected` and
     `async_gen_outside_async.expected`;
   - that the refusal comes at the `for` and not at the parameter or the call.

   The whole block is a guess. It may be one of the most likely things in
   this round to differ.

Things I did not count as guesses, because a page or an example shows them:
- a08's text, taken from `child_wait_not_awaited.expected`, with the line
  and column counted by hand.
- In a07, the start of every http Error (`"{method} {url}: "`), from
  `packages/http/lib.lume`. a07 prints only the part before the first `": "`,
  so ureq's own wording does not matter.
- The `spawn_in` message for a missing folder. io.md gives it for `run_in`
  and says `spawn_in` has "the same messages".
- `-1` for a program ended by a signal (io.md).
- Results in the order tasks were started (concurrency.md).
- `Ok(12)` for a task whose body uses `?` (concurrency.md).

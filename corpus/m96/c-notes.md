# Reviewer C — notes

## What I read

- `corpus/m96/BRIEF.md`.
- `docs/reference/`: `io.md` (all of it, "A program running alongside" closely),
  `http.md`, `collections.md` (async generators, chains, "Methods that wait for
  the items", taking a sequence), `concurrency.md` (`next_done`, tasks, `?` in
  tasks, what a task copies), `errors.md`, `printing.md`, `strings.md`,
  `methods.md`, `control-flow.md`, `structs-and-enums.md`, `patterns.md`,
  `functions-and-blocks.md`, parts of `interfaces.md` (conformance, where an
  interface can be used, the warning on interface bindings, async methods),
  parts of `names.md` (keywords, reserved type names, binding again), parts of
  `numbers.md` and `operators.md`, the start of `packages.md` (`lume.toml`),
  the `pub`/field lines of `modules.md`; `docs/tour.md`.
- `examples/`: `process_spawn.lume` and `.expected`, `async_generators.lume`
  and `.expected`, `http_demo/`, `healthcheck/`, `jobr/jobr/main.lume`,
  `pipes.lume`.
- `packages/http/`: `lib.lume`, `lume.toml`, `test.expected` — for the fields of
  `Request` and `Response` and the exact form of a request's error message
  (`"{method} {url}: {ureq's error}"`).

I did not run the compiler or any program.

## Programs

| file | what it is | leans on |
|---|---|---|
| `c01_build_steps.lume` | build tool: steps run in a build folder, output streamed with a prefix, a failure stops the build, the rest skipped | `spawn_in`, `write`, `close_input`, `lines` into an `async Iterator[Str]` parameter, `error_lines`, `wait`, write after `close_input`, `spawn_in` in a missing folder, enums with payloads, map order, `File`/`Dir`/`Path` |
| `c02_sh_repl.lume` | REPL client talking to a `sh` `while read` server, one-line and `.`-ended multi-line answers | `write` + `await read_line` in turns, `read_line` → `None` after exit, `error_lines` after exit, `lines.each` changing a `var`, match on `Str?` with literal and guards, tally map |
| `c03_word_pipeline.lume` | text pipeline: `tr` → `tr` → `sort`, each stage pumped into the next, then word runs counted | `async def` taking `async Iterator[Str]` and a `Child`, an async generator taking a child's `lines`, `count`, `first`, `map`+`sum`, `all?`, `fold`, `any?`, `find` on an endless program then `kill`, sets, `sort_by` on a tuple key, a file in `/tmp/lume-review-c/` |
| `c04_job_pool.lume` | job runner: `sh -c` jobs in tasks, at most N at once, results from an async generator around `next_done` | `[Task[Done]]`, `next_done`, `yield` from a pool, `count`/`map.to_list`/`find`/`fold` on the generator, results printed in job order (not finish order), an interface parameter, `subset?` |
| `c05_http_planner/` (`lume.toml`, `main.lume`) | request planner: plan lines checked and built with the request builder; bad lines warned about; the good ones sent where nothing listens | `http.request`, `.header`, `.with_body`, `.timeout`, `Request` fields, `await fetch` from tasks, `open`, `http.send`, `send` to a bad URL, a hand-built `http.Response`; `EXPECTED STDERR` for the `warn` lines |
| `c06_frames.lume` | client for a `sh` server that frames answers as a language server does (`Content-Length`, blank line, body) | `read_line` for the header, `read(n)` for bodies (one with a newline in it), pipelined requests, a broken header, `read(n)` when output ends early and after the end, an async generator taking a `Child`, `.map.to_list` and `count` on it |

All six are `Meant to RUN.` None should make the compiler warn: no `shared var`,
no `!`, no binding of an interface type, no unreachable arm.

## Guesses

1. **A `Child` handed to an async generator is the same running program.**
   `c06` passes `server` to `frames(server)`, walks it, then does `await
   server.wait`; `batch` the same with `count`. I assume the generator's copy of
   its argument reads the same output and the original can still be waited for.
   Looked: `collections.md` ("It has its own copy of its arguments"), `io.md`
   ("A `Child` can be kept in a list or a field and handed to tasks"),
   `examples/process_spawn.lume` (a `Child` passed to `ask` and used after, a
   `Child` copied into `spawn:` from a list). Files: `c06`.
2. **`read_line` and then `lines` on one child carry on from the same place.**
   `c02` reads two `.`-ended answers with `read_line`, then walks the rest with
   `batch.lines.each`, expecting only the lines after. Looked: `io.md` ("Each
   output is read by a thread of its own and kept until it is asked for"); the
   example uses one or the other per child, never both. Files: `c02`.
3. **`await c.read(n)` after the output has ended gives `None`.** `read(10)` with
   five bytes left giving `Some("short")` is documented; asking again after that
   is not. Looked: `io.md` table ("`Str?` ... or what is left when it ends
   first"). Files: `c06`.
4. **`Process.spawn_in` in a folder that is not there gives the `run_in`
   message**, ``cannot run `ls` in `/tmp/lume-review-c/c01/no-such-dir`: no such
   folder``. `io.md` gives that message for `run_in` and says `spawn`/`spawn_in`
   have "the same messages as `Process.run`". Files: `c01`.
5. **A struct from a package prints with its bare type name**:
   `Request(method: "GET", url: ..., headers: [("X-Trace", "on")], body: "",
   timeout_ms: 250)`, not `http.Request(...)`. Looked: `printing.md` (only
   types of the same file are shown), `packages/http/lib.lume` for field order.
   Files: `c05`.
6. **An async generator may walk an `async Iterator[T]` parameter with `for`.**
   `c03`'s `runs(words: async Iterator[Str]) -> Iterator[(Str, Int)]`. The docs
   say such a parameter takes the sequence by move, and that plain generators
   chain into each other through `Iterator[T]` parameters, but no example has an
   async generator taking one. Looked: `collections.md` "Async generators" and
   "Taking a sequence". Files: `c03`.
7. **An escape inside a string literal inside an interpolation** —
   `"#{body.replace("\n", " / ")}"` — is read as a newline. Quotes inside
   `#{...}` are shown in `http_demo` and `healthcheck`; an escape inside such a
   nested literal is not shown anywhere I read. Files: `c06`.
8. **What the outside programs do on this Mac** — not Lume, but I could not check
   any of it: `tr -cs a-z '\n'` (the `\n` given as two characters from a raw
   string) splits on every non-letter and squeezes runs into one newline;
   `wc -l < f | tr -d ' '` gives a bare `4`; `sleep 0.2` takes a fraction;
   `sh -c no-such-command-c04` writes exactly one line to stderr and exits 127;
   a `while read` loop's `sh` exits 0 at end of input; the endless `sh` loop in
   `c03` ends in a way that `wait` reports as `-1` (SIGKILL, or SIGPIPE once
   `find` lets go of its output — `io.md` says both are `-1`); `sort` puts
   `A… < On… < The…` and the lower-case words in plain order whatever the
   locale. Files: `c01`, `c02`, `c03`, `c04`, `c06`.

Checked rather than guessed, and so not counted: the waiting methods take no
`await` (`collections.md` and `examples/async_generators.lume`), `first`/`find`
give `T?` and print as `Some(...)` (`process_spawn.expected`), the
"cannot write to `cat`: its input is closed" message (`io.md`), `-1` after
`kill` (`io.md`), a request's error message starting `"GET <url>: "`
(`packages/http/lib.lume`: `format!("{} {}: {}", m, u, e)`), "not a url" giving
an `Error` rather than a crash (`packages/http/lib.lume`'s own test), the header
order from `.header` then `.with_body` (`lib.lume` appends), the one-line
`Some(r) -> yield r` arm (`concurrency.md`), `next_done` giving `None` when the
list is empty (`concurrency.md`), and `Response.header` reading a name in any
case (`lib.lume`).

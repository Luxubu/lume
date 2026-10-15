# Reviewer C, round eighteen

## What I read

- `corpus/m86/BRIEF.md`.
- `docs/tour.md`, `docs/examples.md`, and in `docs/reference/`: `concurrency.md`,
  `collections.md` (all of it, "Async generators" most closely), `io.md`
  (the "Process", "File", "Dir", "Path" sections), `errors.md`,
  `structs-and-enums.md`, `interfaces.md`, `patterns.md`, `control-flow.md`,
  `functions-and-blocks.md`, `methods.md`, `numbers.md`, `names.md`,
  `operators.md`, `strings.md`, `printing.md`, and the "Constants" section of
  `modules.md`. I grepped `generics.md`, `packages.md` and `http.md` for async.
- `examples/async_generators.lume` and `.expected`, `examples/async.lume` and
  `.expected` (to see how the `shared var` warning looks with a path and a
  two-digit line), `examples/tasks_blocking.lume`, `examples/generators.lume`,
  `examples/iterators.lume`, `examples/jobr/jobr/main.lume` (for
  `Process.run` inside an async function), and `examples/match/main.lume`.

I did not read the compiler, `tests/`, `tools/`, `bench/`, other corpus
folders, `docs/QUESTIONS.md`, the README, HISTORY or CHANGELOG, and I ran
nothing.

## The programs

| file | what it is | leans on |
|---|---|---|
| `c01_build_steps.lume` | a build orchestrator: `sh -c` steps run in folders under `/tmp/lume-review-c/build`, results streamed through an async generator that stops after the first failure | async generator yielding enum values, `return` in one, `Process.run_in` (missing folder as `Error`, non-zero exit as a value), `Ok((0, out, _))` patterns, Dir/File/Path, map, set `diff` |
| `c02_crawler.lume` | a breadth-first crawler over a fake site in a constant map, with a sleeping async `fetch` | async generator with a queue and a seen-set, a plain generator called inside it, `await` in a `match` subject, `next` by hand and then `for`, `break`, `.to_list`, a task that takes one |
| `c03_event_bank.lume` | an event-sourced bank whose events come from an async generator parsing a `"""` log | list patterns over `split`, `?` in the parser, unparseable lines turned into a `Bad` event, a `var self` method giving `() or Error`, a task that takes a second stream and returns a tuple |
| `c04_chat_rooms.lume` | a chat-room simulation: a task per room walks that room's message stream | async generator called inside `spawn:`, an interface list (`[Rule]`) copied into tasks, `Str?` rule results, `shared var` list and counter (with the stderr warnings), results in start order |
| `c05_log_tail.lume` | a log tailer: writer tasks append lines with `sh -c`, reader tasks walk async generators that poll the files and end at `END` | file polling in an async generator, `return` from inside nested loops, a raw `"""` shell script passed to `Process.run`, `await t?` in `main -> () or Error`, `grep -c` exit 1 as a value, `Process.run_in` with `ls` |
| `c06_health_rounds.lume` | a health monitor: rounds of probes, a state machine per service | `spawn:` and `await` on a list of tasks inside an async generator, `g.next` in a `while` until `None`, `match` on a `(State?, Health)` tuple, enum `!=`, `zip`, a set, the documented "cannot run" message, `.to_list` on an async generator inside a task |

## Guesses

1. **`return` ends an async generator** (c01, c05). c01's generator `return`s
   after yielding its first failure. c05's `tail` `return`s from a `for`
   inside a `while` when it reaches `END`. `collections.md` says "`return`
   alone ends it" only for plain generators. Its "Async generators" section
   says only that an async generator "moves like a generator".
2. **`yield` inside a `match` arm's indented block works in an async
   generator** (c01, c02, c03). For plain generators, `collections.md` says
   `yield` may go in "its loops, `if`s and `match`es". The async section and
   `examples/async_generators.lume` only show `yield` directly in a `for` or
   `while`, so I assumed the plain-generator rule carries over.
3. **`spawn:` and `await tasks` inside an async generator's body** (c06).
   Each round starts one task per probe and awaits the list before it yields.
   `collections.md` only says that `yield` may not go inside a `spawn:`.
   `concurrency.md` does not mention generators. I assumed nothing else
   forbids it.
4. **Interface values can be copied into a `spawn:`** (c04). Each room's task
   gets its own copy of `rules: [Rule]`, a list of three different structs
   held as an interface. `concurrency.md` says "Anything can be copied in",
   but its list (numbers, text, lists, maps, structs) leaves out interface
   values. `interfaces.md` says nothing about tasks.
5. **The warning's `-->` line shows the path as given to `lume`** (c04). I
   wrote `corpus/m86/c04_chat_rooms.lume:98:3`, following
   `examples/async.expected`, which shows `examples/async.lume:48:3`. I also
   assumed one warning per `shared var` in declaration order, and the
   two-digit gutter (`   |`) as in that file. I moved the declarations to
   lines 98 and 99 so I would not have to guess the three-digit gutter.
6. **A method call is allowed as a one-line `match` arm.** Examples:
   `Page(d, p, _) -> by_depth[d].push(p)` (c02), `Error(err) ->
   refused.push(err.message)` (c03), `None -> kept.push(...)` (c04), and
   `Failing(..) -> _ = ever_failed.add(name)` (c06). The list in
   `patterns.md`, "What an arm can do", is: a value, a `puts`, an assignment,
   `next`, `break`, `return` or `()`. I assumed a call that gives `()` counts
   as a value. In c06 I wrote `_ =` so the `Bool` from `add` does not clash
   with the `()` arm.
7. **A `-> () or Error` body may end in a statement `if` or `match`** (c03).
   `Bank.check` ends in an `if` with no `else`. `Bank.apply` ends in a `match`
   whose arms end in assignments, `_ = ...` or `return Error(...)`.
   `errors.md` says such a function "that reaches its end without failing has
   succeeded; nothing needs writing at the end". I assumed that still holds
   when the last line is an `if`/`match` statement and not an expression the
   compiler might take as the result.
8. **Unused names give no warning** (all programs). Examples: `v` in c02's
   `for v in crawl("/", 3)`, and pattern names a program never reads. The
   docs name only three warnings: `shared var`, `!`, and an unreachable
   `match` arm. So I predicted standard error only for c04.

Things I checked rather than guessed: `next` by hand on an async generator
(the example's `puts t.next`), a task taking a generator, a non-zero exit
coming back as a value, the "cannot run `...`" message, and `Process.run_in`
on a missing folder giving an `Error`. I don't count the shell side as a Lume
guess: `wc -l` pads its count on macOS, and the program trims it. For c05's
order to hold, each `printf` append has to land in a single write. The tailer
drops any line that does not end in a newline yet.

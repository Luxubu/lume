# Reviewer A — notes (round twenty-two, everyday code)

## What I read

- `corpus/m110/BRIEF.md`
- `docs/stability.md`
- `docs/reference/names.md` (all, especially "A name that is never used")
- `docs/reference/testing.md` (all)
- `docs/reference/io.md` (all, especially "A program running alongside")
- `docs/reference/interfaces.md`, "Methods that take a block, have type
  parameters, or are `async`"
- `docs/reference/collections.md`: lazy chains, lists, maps, sets,
  generators, async generators, their chains and waiting methods
- `docs/reference/errors.md`: `!`, "Where the program stopped", other methods
- `docs/reference/concurrency.md`: "A failure inside a task", "Where the
  warnings go"
- `docs/reference/printing.md`, `docs/reference/methods.md` (Str/Char/Int
  tables, tuple blocks)
- `docs/tutorial/18-programs-alongside.md`; `docs/tutorial/12-...` (tests part);
  skimmed `docs/tutorial/17-async-sequences.md`
- `examples/unused_names.lume` + `.expected`, `examples/process_spawn.lume` +
  `.expected`, `examples/async_interfaces.lume` + `.expected`,
  `examples/async_generators.lume`
- `examples/tests/async_tests`, `parse`, `assert_once` (`.lume` + `.expected`)
- `examples/errors/`: the file list, and `await_in_test`, `async_not_iterator`,
  `arity`, `async_gen_outside_async`, `child_lines_outside_async`,
  `child_wait_not_awaited`, `iface_missing`, `iface_mismatch`,
  `iface_unused_param`, `async_gen_method`, `generator_as_interface`,
  `process_no_try`, `dropped_result`, `immutable`, `unknown_function`, `typo`,
  `async_not_awaited`, `loop_var_twice`, `block_param_twice`,
  `for_three_names`, `for_two_vars_not_pairs`, `destructure_same_name`,
  `task_or_error`, `task_ok`, `task_or`, `task_error`, `task_try_mixed`,
  `own_task_args`

Nothing was compiled or run. The line and column numbers in the headers
were counted with `awk` over my own files (text only).

## My programs

| file | probes | meant to | stderr / exit |
|---|---|---|---|
| `a01_word_tally.lume` | `tr` and `sort` as `Child`ren, write/close_input/lines; a map tally; an unused parameter and an unused local | RUN | 2 warnings |
| `a02_sources.lume` | generic interface requiring an async generator (`def items -> async Iterator[T]`), one type in its body, one via `extend`; functions taking the interface: `sum`, chained `map`, `count`, `skip(1).first`; `enumerate` over the sequence | RUN | none |
| `a03_line_counts.lume` | `Process.run("sh", ["-c", "wc -l < path"])`, files under `/tmp/lume-review-a/`; unused name from a tuple taken apart; unused block parameter in a map's `filter` | RUN | 2 warnings |
| `a04_shell_calculator.lume` | `sh` loop kept in a struct field, asked in turns by an async method; `read_line` after the end, `wait` twice, write after `wait`, `kill`, `kill` after the end | RUN | none |
| `a05_sort_tests.lume` | `async test` with `Process.spawn`; `lume test FILE sort` filter; one failing assert on a list | RUN (via `lume test ... sort`) | exit 1 |
| `a06_podium.lume` | `Process.run("printf", ...)` with `_` in the tuple; `sort_by` + `reverse`; `.at(3)` past the end | STOP | stop message, exit 101 |
| `a07_plain_for_async.lume` | a plain generator where the interface requires an async one | FAIL | — |
| `a08_default_feed.lume` | an interface giving a default async generator | FAIL | — |
| `a09_byte_counts.lume` | three `wc -c` `Child`ren in tasks, results in list order; `_` twice in one block over triples; one unused block parameter | RUN | 1 warning |

## Guesses

1. **Warnings come in source order, top of the file to the bottom.**
   Files: a01, a03. Looked: brief ("in the order the compiler finds them"),
   `examples/unused_names.expected` (source order there, struct methods
   first because they come first). Nothing says it outright.
2. **An unused parameter of a plain top-level function gets the same help
   as a method's**, "name it `_width` if it is there only to fit a
   signature, as an interface's method may be". Files: a01. Looked:
   `examples/unused_names.expected` (only shows a method's parameter),
   names.md.
3. **A name left unread from a tuple taken apart**, `(code, out, err) =
   ...?`, is warned as a local ("`err` is given a value but never used",
   help "use it, or name it `_err`...") rather than as a pattern binding,
   with the column at the name itself. Files: a03. Looked: names.md lists
   "a local ... a name a pattern binds" without saying which a tuple
   binding is; `destructure_same_name.expected` puts its caret at `(`.
4. **`await sorted(await split_words(text)?)?`** — an `await` with `?`
   inside the argument of another awaited call — compiles. Files: a01.
   Looked: `examples/process_spawn.lume` (`puts await ask(shout, w)?`),
   concurrency.md (`await t?`); no nested example.
5. **A type fits a generic interface spelled `def items -> async
   Iterator[T]` by writing `async def items -> Iterator[Int]` in its own
   body, with no `extend`**, and an `extend` may write the other spelling.
   Files: a02. Looked: interfaces.md and `examples/async_interfaces.lume`
   (`Shelf` does this for a non-generic interface; `Feed[T]` uses the other
   spelling).
6. **Through a field of a plain (non-`var`) binding and inside a non-`var`
   method, a `Child` can be written to and closed** (`child.write`,
   `calc.child.close_input`). Files: a04. Looked: io.md ("A `Child` can be
   kept in a list or a field"), tutorial 18 ("cheap to copy and share; every
   copy is the same running program"); no example writes through a field.
7. **The write-after-`wait` error names the program as it was given**:
   ``cannot write to `sh`: its input is closed``. Files: a04. Looked: io.md
   shows it only for `sort`; `examples/process_spawn.lume` prints only
   `.error?`.
8. **The filtered summary with more than one left out reads `(2 not
   matching `sort`)`**, same words as for one. Files: a05. Looked:
   testing.md (only `(1 not matching `rounded`)`, with one failing).
9. **`left:`/`right:` show a list of `Str` with its items quoted**,
   `["10", "2", "33"]`. Files: a05. Looked: testing.md (`Some("starter")`),
   printing.md (a value inside another prints as written).
10. **`(await through(...)).or([])`** — an empty list as `.or`'s default,
    typed from the `[Str] or Error` it is called on. Files: a05. Looked:
    collections.md ("`.or([])` on a known map" is fine); not shown for a
    `T or Error`.
11. **The whole a07 message**: wording modelled on
    `examples/errors/iface_mismatch.expected`, with the two types written
    `-> Iterator[Str]` and `-> async Iterator[Str]` (collections.md: "Its
    type is written `async Iterator[T]`, as messages name it"), placed at
    the argument where `Fixed` is used as a `Feed`. It may instead be
    reported at the method, or in other words. Files: a07. Looked:
    interfaces.md, collections.md, `iface_mismatch`, `iface_missing`.
12. **a08's help keeps `(...)` and `T` literally** — `async def served(...)
    -> Iterator[T]` — for a method with no parameters giving `Str`, as
    `examples/errors/async_gen_method.expected` does for `items`; and this
    error is the first one reported. Files: a08. Looked: that example,
    interfaces.md.

Where the docs disagree (observations, not guesses):

- collections.md, in the list under "Generators: `yield`", still says
  "**Not yet:** an `async` generator in an interface", while the async
  section below it, interfaces.md, and stability.md ("an interface requiring
  one — settled in 0.7") say an interface may require one. Only a
  *default* one is refused.
- stability.md opens "Lume is at 0.6" but lists parts "settled in 0.7".
- stability.md settles "`lume test` and its name filter", but testing.md
  does not say what "A name that matches no test says so" prints.

**Total: 12 guesses.**

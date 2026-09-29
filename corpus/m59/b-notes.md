# Reviewer B — the edges of the new features, and what the editor shows

## What I read

- `corpus/m59/BRIEF.md`.
- `docs/install.md` (the commands, "Editors", the `--json` format).
- `docs/reference/io.md` in full (`Env.exit`, `Process`, "When I/O fails").
- `docs/reference/errors.md` in full (`map_error`, the dropped-failure rules, `?`).
- `docs/reference/names.md` in full (a type of your own named `Task`).
- `docs/reference/concurrency.md` in full.
- `docs/reference/control-flow.md` in full (`for` over pairs, `if` as a value).
- Parts of `functions-and-blocks.md` (return type mismatch, `var` parameters,
  `?` in a block), `generics.md` (generic structs), `methods.md` (pairs,
  `enumerate`, `zip`), `collections.md` (maps), `patterns.md`,
  `structs-and-enums.md` (tuple destructuring), `printing.md`, `modules.md`
  (missing argument through a prefix).
- Every `#!` line in `docs/` (grepped), to collect the known message shapes.
- `examples/errors/`: the first line and the `help:` of every `.expected`, and
  in full: `own_task_args`, `shared_param_order`, `destructure_arity`,
  `loop_paren_one`, `pattern_arity`, `builtin_arg_type`, `dropped_result`,
  `dropped_failure`, `missing_arg`, `block_wrong_result`, `noelse`,
  `missing_arm`, `arity`, `result_method`, `option_method`, `typo`,
  `unknown_type`, `generic_wrong_arity`, `iface_mismatch`,
  `for_var_immutable`, `spawn_without_async_main`, `task_try_mixed`,
  `shared_readonly`, `block_not_a_block`, `reserved_type`,
  `no_such_namespace_item`, `block_call_arity`, `var_param_copy`,
  `tasks_used_after_await`, `async_not_awaited`, `spawn_changes_copy`,
  `missing_error_arm`, `call_not_a_block`, `try_in_plain_fn`, `block_returned`.
- `examples/jobr/` in full (FINDINGS, both packages, `queries.expected`,
  `check.sh`, `jobs.txt`); the `Env.exit` lines of `examples/csvq/csvq/main.lume`
  and `examples/site/main.lume`; one line of `examples/mini/FINDINGS.md`
  (a parameter mismatch message).
- The `lume_process_run` function in `examples/match/.lume/main.rs`, generated
  Rust that sits under `examples/`. It gives the failed-to-start message:
  ``cannot run `<program>`: <system reason>``. It is build output and may be
  stale, but it is the only place that says what the message is.

Not read: `docs/QUESTIONS.md`, the compiler, `tests/`, the rest of `corpus/`,
`README.md`, `HISTORY.md`. Nothing was run.

## The programs

| file | what it probes | RUN/FAIL |
|---|---|---|
| `b01_process_start_vs_exit.lume` | failing to start (the only `Error`) versus exiting non-zero (`false`, `exit 7`, `test -f`), no shell (`&&` is text), an argument with spaces, empty stdin for the child, the output's trailing newline in interpolation, `?.0` | RUN |
| `b02_process_no_try.lume` | `(code, out, err) = Process.run(...)` with no `?` or `match` | FAIL |
| `b03_process_wrong_args.lume` | `Process.run("sh", "-c 'echo hi'")`: a `Str` where `[Str]` goes (+ `--json`) | FAIL |
| `b04_map_error_paths.lume` | `map_error` on success (block not run), on failure, stacked twice, followed by `?`, on Process.run's start failure; the replaced message ending `main` | RUN |
| `b05_map_error_gives_str.lume` | a `map_error` block that gives a `Str` instead of an `Error` | FAIL |
| `b06_map_error_on_option.lume` | `map_error` on a `T?` (`.first`) (+ `--json`) | FAIL |
| `b07_exit_branches.lume` | `Env.exit` ending a function's match arm, the `else` of a multi-line `if` value, an `if` with no `else` (statement), and a match whose only arm is `Env.exit` as the body of `-> Int`; exit status 5 | RUN |
| `b08_own_generic_task.lume` | a generic `struct Task[T]` of your own beside `spawn:`, with `Task[Str]` in a binding's type | RUN |
| `b09_own_generic_task_list.lume` | own generic `Task[T]`, then `[Task[Int]]` given what `spawn:` gives: two types spelled the same (+ `--json`) | FAIL |
| `b10_for_bracket_pairs.lume` | `for (a, b) in` over `enumerate`, a map, `zip`, with `where`, with `next`/`break`, hiding an outer `i` | RUN |
| `b11_for_three_names.lume` | `for (i, w, extra) in xs.enumerate` | FAIL |
| `b12_shared_var_reversed.lume` | a parameter typed `var shared [Int]`, the words the wrong way round | FAIL |

5 RUN, 7 FAIL. Not written as programs of their own, because 12 slots did not
reach: `for (x) in` with one name in brackets (it is
`examples/errors/loop_paren_one` exactly, whatever the iterable), and
`x = if bad: Env.exit(2)` with no `else` (control-flow.md already gives the
first line of that message, ``an `if` used as a value needs an `else` ``).
b07 covers the statement form of an `if` without `else` that exits.

## Guesses

Things I checked, and so did not count: the error layout, including the wider
gutter for two-digit lines (`block_wrong_result.expected`); the path after
`-->` is the one given to `lume check`; the JSON keys and their order
(install.md); how `Ok("0:")` and `Error("...")` print (printing.md,
errors.md); how a `?` in `main` ends the program (io.md); `for` names hiding
an outer name (names.md); map order (collections.md); `zip` order
(methods.md); `Ok((code, out, err))` as a pattern (jobr); the start-failure
message (generated runtime in `examples/match/.lume/`, see above).

1. **b01, a child's standard input.** Looked in io.md: nothing on stdin. The
   generated runtime calls Rust's `Command::output()`, which does not pass
   stdin on, so `cat` with no file sees end of input at once and exits 0.
   Assumed that, from Rust, not from the docs.
2. **b01, `(Int, Str, Str) or Error` as a parameter type.** io.md gives it
   only in a table. Tuple types exist (`-> (Int, Int)` in structs-and-enums.md)
   and `T or Error` takes any `T`; assumed the two compose in a signature.
3. **b01, `Process.run(...)?.0`.** `.0` on a tuple is in `tuple_method`'s help,
   and `Env.stdin?.lines` shows a method after `?`. Assumed `.0` works there
   too.
4. **b02, the message and help.** Nothing shows a `T or Error` destructured
   as a tuple. Assumed the shape of `result_method.expected` with "cannot be
   taken apart directly" in place of "cannot be used with `+` directly".
5. **b02, the column.** Assumed col 3, the `(`, as in `destructure_arity`,
   rather than the value (as `result_method` points at `n`).
6. **b03, the message and help.** No example of a built-in *function* given a
   wrong argument type. Took `builtin_arg_type` (`` `.add` on a `{Int}` takes an
   `Int`, but this is a `Str` ``) and wrote `` `Process.run` takes a `[Str]` ``;
   the help (a list, one item each) is invented. `examples/mini/FINDINGS.md`
   suggests a second shape for user functions, `` `last` is a `Value`, but
   this is a `[Str]` ``, which would make it `` `args` is a `[Str]` ... ``.
7. **b03, the column.** Assumed the argument's first character, as
   `builtin_arg_type` and `block_not_a_block` do.
8. **All three `--json` outputs, the `file` field.** install.md shows
   `"main.lume"` for `lume check --json main.lume`. Assumed it is the path as
   typed, so `corpus/m59/...`, matching the text form.
9. **b06 `--json`, the em dash.** Assumed it is written as the character
   itself, not escaped as `—` (the character is serde_json's default,
   if that is what writes it). b03's help also assumes `"` is escaped `\"`,
   which JSON requires, so that part is not a guess.
10. **b05, the message and help.** Took `block_wrong_result` (a block given to
    a parameter that asks for a `Bool`) and put `Str`/`Error` in, with "an"
    before `Error` as it writes "an `Int`". A kinder message would say
    ``wrap it: `Error("...")` ``; I did not assume one.
11. **b05, the column.** The `{`, as `block_wrong_result`.
12. **b06, the message and help.** Assumed `map_error` on `Int?` takes the
    general "may be absent" path of `option_method.expected`. The compiler may
    instead point at `.or_error`, which would be better.
13. **b06, the column.** The `.` before `map_error`, as in `option_method` and
    `no_such_namespace_item`.
14. **b07, `Env.exit` ending an arm of a function's result.** io.md shows it
    only for a `match` bound to a name. Assumed the result of a function
    `-> Int` is the same case.
15. **b07, the `else` of a multi-line `if` value ending in `Env.exit`.** io.md
    says "a branch whose other branches give a value" and shows `match`.
    Assumed `if`/`else` too.
16. **b07, a match whose only arm is `Env.exit`, as the body of `-> Int`.**
    No branch gives an `Int` at all. Rust accepts it; assumed Lume does and
    does not ask for a value from somewhere.
17. **b08, a generic `struct Task[T]` of your own is allowed.** names.md says
    "a `struct Task` or `enum Task`" and "`Task[..]` means yours too", but
    shows no generic one. Assumed it is allowed and that `spawn:` still works.
18. **b09, the message.** Entirely invented, from `builtin_arg_type`: the list
    element type and what `spawn:` gives both print as `Task[Int]`. If that
    is what the compiler says, it is the worst message in this round; if it
    has a special case like `own_task_args`, the text is unknown to me.
19. **b09, the column and no help.** The argument, as `builtin_arg_type`; no
    help line, since `builtin_arg_type`'s help is about parsing.
20. **b10, brackets over a map.** control-flow.md says the bracket spelling
    "means the same" and shows it on `enumerate` only. Assumed `for (k, v) in m`.
21. **b10, brackets with `where`.** Assumed `for (i, w) in xs.enumerate where
    i != 1` works as the unbracketed form does.
22. **b11, the message and help.** Took `destructure_arity` word for word with
    the numbers swapped (the tuple has 2, 3 are named).
23. **b11, the column.** The `(`, col 7. `loop_paren_one` points at col 9,
    the `)`, for `for (w)`; so it may be the `)` or the third name instead.
24. **b12, the message.** No example of `var shared`. Assumed a parse error in
    the shape of ``expected a variable name, found `(` ``:
    ``expected a type, found `var` ``, with no help. Milestone 58 fixed only the
    other order (FINDINGS item 5), so I assumed this one has no special message.
25. **b12, the column, and no warning first.** The `var`. Assumed a parse error
    stops the check before the `shared var seen` warning is produced, so
    only the error is printed.

**25 guesses.** 16 of them (4–13, 18, 19, 22–25) are about the text or
position of an error message, which is what the brief says matters most now.

## Verdict

The docs are good on what the new features *do* and thin on what the editor
*says* when they are misused. What a program does I could almost always
check: Process.run's split between failing to start and exiting non-zero is
stated plainly, jobr shows the `Ok((code, out, err))` pattern, `Env.exit` as a
branch that gives nothing is shown, and the start-failure message could be
found (only in generated Rust, which a user would not look in; io.md should
give it, since it prints `.error?` where `e.message` would teach more).

The messages are the gap. Of the seven failing programs, only one message
shape was close to certain; for the new features there is no documented
error at all for: `Process.run` with a wrong argument type, a result used as
the tuple it holds, `map_error` with a block that gives a `Str`, `map_error`
on a `T?`, three names over pairs, and `var shared`. Each was guessed from the
nearest older message.

Two edges deserve a look whatever the messages turn out to be:

- **b09.** With a *generic* `struct Task[T]` of your own, `[Task[Int]]` is a
  valid type, so the `own_task_args` check cannot fire, and the push fails
  with two types that print alike. names.md mentions only the non-generic case.
- **b06.** `map_error` on a `T?` is the natural next slip after learning
  `map_error`; the fix (`.or_error`) is one word, and the message should name it.

Smaller doc points: io.md does not say the child's standard input is empty,
nor that `wc`'s padding differs between systems (b01 trims it); names.md does
not say whether a generic own `Task` is allowed; control-flow.md shows the
bracketed `for` only with `enumerate`.

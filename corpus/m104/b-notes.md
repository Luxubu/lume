# Reviewer B — round twenty-one (m104): the edges

Nothing was compiled or run. Every expected output below was worked out by
hand from the docs and examples listed here.

## What I read

- `corpus/m104/BRIEF.md`
- `docs/reference/collections.md` (all, especially "A chain too long for one
  line", generators, `next`), `docs/reference/operators.md` (precedence, `|>`),
  `docs/reference/structs-and-enums.md` ("A struct with no fields", `==`
  and printing, enums), `docs/reference/errors.md` (`!`, "Where the program
  stopped"), `docs/reference/numbers.md` ("What stops the program"),
  `docs/reference/names.md` (keywords, rebinding), `docs/reference/methods.md`
  (list methods, `take`/`skip`, lazy chains), `docs/reference/strings.md`
  (`Char` and `Str`), `docs/reference/printing.md`,
  `docs/reference/control-flow.md` (`if` as a value), `docs/reference/interfaces.md`
  (`extend ... with`), `docs/reference/generics.md` (generic structs),
  `docs/reference/concurrency.md` (tasks, "A failure inside a task"),
  `docs/reference/modules.md` and `docs/reference/packages.md`, the module
  examples under `docs/reference/modules/basics/`, `docs/stability.md`
- `docs/tutorial/02-names-and-values.md`, `03-numbers-and-text.md` (division by
  zero stop), `07-blocks-and-chains.md` (§7.5), `08-structs.md` (§8.5,
  `price = price`), `10-absence-and-failure.md`, `12-generics-modules-and-tests.md`
  (tests, the `skip` hint in "Try it")
- `examples/chains_over_lines.lume`, `examples/unit_structs.lume`,
  `examples/pipes.lume` (with their `.expected`), `examples/fmt/messy.lume`
  and `blocks.lume`, `examples/tests/*.lume` and `.expected`,
  `examples/packages/` (`tally`), `examples/lib/FINDINGS.md` (the `at` message),
  `examples/errors/UNREACHED.md`, and these refused programs with their
  `.expected`: `leading_dot_block`, `keyword_as_name`, `keyword_as_value`,
  `self_assign_field`, `struct_empty_block`, `unit_struct_bare`,
  `overflow_constant`, `assert_with_message`, `assert_empty`, `bad_keyword`,
  `pipe_no_stage`, `import_keyword`, `pattern_keyword`, `variant_empty_parens`,
  `pattern_empty_variant_parens`, `typo`, `char_or_str`, `char_position_str`,
  `char_list_type`

## My programs

| id | file | probes | meant to |
|---|---|---|---|
| b01 | `b01_chain_comments.lume` | comment lines (indented and at the left edge) between chain steps; a comment at the end of a step | RUN |
| b02 | `b02_chain_blank_line.lume` | a blank line between chain steps | RUN |
| b03 | `b03_chain_in_bodies.lume` | chains after `return`, in an `if` in a `for`, in a `match` arm body, in a `do` body; `take(-1)`, `skip(-2)`; a `Char` key on a `{Str: Int}` | RUN |
| b04 | `b04_chain_if_pipe.lume` | `.name` after an inline `if`/`else`; `.name` and `\|>` lines mixed; after a line ending in `{ }`; a list literal chain ending in `\|> puts` | RUN |
| b05 | `b05_unit_struct_places.lume` | field-less struct in a list, `==`/`!=`, set, map key (literal and written through), enum payload, generic argument, field, `extend ... with`, interface parameter, `match`; methods-only struct | RUN |
| b06 | `b06_generator_stop.lume` | `xs.at(5)` past the end inside a generator | STOP |
| b07 | `b07_module_stop/` (package: `lume.toml`, `main.lume`, `split.lume`) | division by zero in another module of the package | STOP |
| b08 | `b08_task_overflow.lume` | `Int` overflow in `*` inside a spawned task, told when awaited | STOP |
| b09 | `b09_test_stop.lume` | a test that stops, under `lume test` (exit 1), tests after it still run | STOP (under `lume test`) |
| b10 | `b10_assert_outside.lume` | `assert` outside a test, in a `var self` method | STOP |
| b11 | `b11_dot_starts_do_block.lume` | `.upcase` as the first line of a `do` block body | FAIL |
| b12 | `b12_struct_colon_empty.lume` | `struct Marker:` with nothing under it | FAIL |
| b13 | `b13_marker_no_parens.lume` | `Marker` without `()` inside a list literal | FAIL |
| b14 | `b14_label_self_assign.lume` | `label = label` in a `var self` setter (the `price = price` fix, another name) | FAIL |
| b15 | `b15_keyword_name.lume` | `next` as a binding (fine as a field) | FAIL |

10 meant to run or stop, 5 meant to fail.

## Guesses

1. **Comment lines inside a chain are skipped.** A `.name` line after one or
   more comment lines continues the last code line, whether the comment is
   indented like the steps or at the left edge; a comment at the end of a step
   line is fine. — `b01`. Looked: collections.md "A chain too long for one
   line" and tutorial 07 §7.5 ("A line that begins with `.` always continues
   the line above"); `examples/chains_over_lines.lume` only has a comment
   before a statement, never between steps; `examples/fmt/messy.lume` shows
   comments at column 0 inside a body survive.
2. **A blank line inside a chain is skipped the same way**, so the `.name`
   line after it still continues the line above. — `b02`. Same places as 1;
   nothing shows a blank line between steps.
3. **`take` with a negative count gives no items** (`[]`). — `b03`. Looked:
   methods.md (`take`/`skip` "never" stop), tutorial 12 "Try it" 1 (says a
   negative `n` to `skip` drops none, which I took as documented); nothing
   says what `take(-1)` gives.
4. **A `Char` works as the key of a `{Str: Int}`**, both to read
   (`counts[c].or(0)`) and to write (`counts[c] = ...`), and the map prints
   with quoted keys. — `b03`. Looked: strings.md ("a `Char` is accepted
   wherever a `Str` is wanted", a `{Char: Int}` tally); the brief lists this as
   a fix, but no doc shows a `Char` indexing a `{Str: ...}` map.
5. **A `.name` line after an inline `if ... else: v` belongs to `v` only**
   (the line is continued as text, and `.` binds tightest), so `n = 5` gives
   `big` and `m = 1` gives `SMALL`. — `b04`. Looked: control-flow.md ("`if` as
   a value"), operators.md precedence; nothing covers a continuation after an
   inline `if`. (If it applies to the whole `if`, the first line is `BIG`.)
6. **A `.name` line may follow a `|>` line, and a `|>` line may follow a
   `.name` line**, in one statement. Either reading of `xs.filter{..} |>
   .map(double) .sum` gives 12, so only acceptance is guessed. — `b04`.
   Looked: operators.md `|>` ("the one operator that may continue an
   expression onto the next line", written before milestone 99),
   `examples/pipes.lume`, `examples/fmt/messy.lume`.
7. **A statement may start with a list literal** continued by `.name` lines
   and ending in `|> puts`. — `b04`. Looked: `examples/pipes.lume` starts one
   with `(1..10)` and with `"hello"`, never with `[...]`.
8. **A field-less struct works everywhere a struct with fields does**: as a
   map key (in a literal and written through `m[Marker()] = ...`), as an
   enum payload, as a generic type argument (`Box[Marker]`), as a field, in
   `extend Marker with Named`, and prints as `Marker()` inside each of them
   and in interpolation. — `b05`. Looked: structs-and-enums.md ("A struct
   with no fields", "`==`, printing and hashing come free"),
   `examples/unit_structs.lume` (set, `==`, printing, interface only),
   interfaces.md (`extend` on a struct), generics.md.
9. **The shape of a stop below the message**: `  --> path:LINE` with no
   column, then a gutter line and the source line, no caret, and the gutter
   widens for a two-digit line number (`   |`, `17 |`) as it does for
   compile errors. — `b06`, `b07`, `b08`, `b10`. Looked: errors.md "Where the
   program stopped" (one-digit line only), tutorial 03 (one-digit),
   packages.md `import lib` error (two-digit compile error gutter).
10. **A task that stops prints nothing until it is awaited**, so `started`
    and the first task's result reach standard output, and the stop names
    the line inside `grow` with the same shape as a stop in `main`. — `b08`.
    Looked: concurrency.md "A failure inside a task" (a crash in a task "stops
    the whole program when the task is awaited"), errors.md ("says so once,
    when it is awaited"); numbers.md for the wording `Int overflow in `*``.
11. **The exact `lume test` lines for a stopped test**: `test NAME ...
    FAILED`, then the stop indented four spaces as
    `corpus/m104/b09_test_stop.lume:16: stopped: no item at 5 — the list has 2`
    (path as given on the command line, the line of the `at` in `seat`, not
    the test's line), no source line under it, the remaining tests still run,
    the summary counts it as failed, everything on standard output, exit 1.
    — `b09`. Looked: errors.md ("as `bill.lume:12: stopped: ...`"), tutorial 12
    §12.5, `examples/tests/parse.lume` and its `.expected`.
12. **The wording of a failed `assert` outside a test**:
    `error: assert failed: not overdrawn?` (a condition that is not a
    comparison, so no `left:`/`right:` lines). — `b10`. Looked: errors.md and
    numbers.md list it as a stop but show no message; tutorial 12 shows only
    the in-test form.
13. **A `do` block's first line starting with `.` gives the same message and
    help as a `for` body's.** — `b11`. Looked: collections.md and
    `examples/errors/leading_dot_block.lume` (a `for` body only).
14. **`Marker` without `()` is reported before anything about the list's item
    types**, with the message of `unit_struct_bare` and no help line, at the
    bare name's column. — `b13`. Looked: `examples/errors/unit_struct_bare.lume`,
    interfaces.md ("the items of a list must all be the same type").
15. **The `price = price` message is built from the names**, so `label = label`
    gives the same two lines with `label`, and the earlier `uses += 1` line does
    not change it. — `b14`. Looked: tutorial 08 §8.5,
    `examples/errors/self_assign_field.lume`, stability.md.
16. **`next = ...` is refused like `for = 3`**, with the same message and the
    help built from the word (`next_count`, `my_next`), at the keyword's
    column. — `b15`. Looked: names.md ("Any keyword given a value with `=` ...
    is named in the message"; `next` allowed as a field),
    `examples/errors/keyword_as_name.lume`, tutorial 02.

16 guesses.

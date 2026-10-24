# Reviewer A — notes (round twenty-one, set: everyday code)

## What I read

- `corpus/m103/BRIEF.md`.
- `docs/reference/`: `errors.md` (all, especially "`!` — unwrap or stop" and
  "Where the program stopped"), `collections.md` (all, especially "A chain
  too long for one line"), `structs-and-enums.md` (all, especially "A struct
  with no fields"), `operators.md` (all, for `|>` and precedence),
  `numbers.md` (all, "What stops the program"), `concurrency.md` (all, "A
  failure inside a task"), `interfaces.md` (all, for the binding warning),
  `printing.md`, `methods.md`, `functions-and-blocks.md` (first half),
  `control-flow.md` (ranges), `names.md` (keyword list), `strings.md`
  (`Char`/`Str`).
- `docs/tutorial/`: `03-numbers-and-text.md` (division-by-zero stop),
  `06-lists-maps-and-sets.md` (`.at`), `10-absence-and-failure.md` (`!`,
  `main -> () or Error`), `12-generics-modules-and-tests.md` ("Try it").
- `examples/`: `chains_over_lines.lume`/`.expected`,
  `unit_structs.lume`/`.expected`, `pipes.lume`/`.expected`,
  `interfaces.lume`/`.expected` (warning placement, loop variables),
  `interface_methods.expected`, `interface_signatures.expected`,
  `kept_blocks.expected`, and from `examples/errors/`: `leading_dot_block`,
  `unit_struct_bare`, `keyword_as_name`, `self_assign_field`, `task_error`,
  `chain_no_items`, `struct_empty_block`, `iface_missing`,
  `iface_mismatch`, `typo`, `pipe_no_stage`, and `UNREACHED.md` (first
  part).
- Nothing else. I did not run the compiler or any program.

## Programs

| id | file | meant to | probes |
|---|---|---|---|
| a01 | `a01_order_report.lume` | RUN | chains one step a line: method body after `=`, binding, `for` header, inside call parentheses, followed by `\|>` lines (function stage and `.method` stage) |
| a02 | `a02_pricing.lume` | RUN (+1 warning) | field-less strategy structs behind an interface: as a field, parameter, typed `[Pricing]` binding (warns), `==`; a marker struct in an enum payload, a set, a map key, printed nested |
| a03 | `a03_podium.lume` | STOP | `.at` past the end in `main`, inside an interpolation, after a chained binding |
| a04 | `a04_team_hours.lume` | STOP | `Int` division by zero inside a method; stop names the method's line; chained `for` header |
| a05 | `a05_stock_count.lume` | STOP (+1 warning) | `!` on an `Error` inside a block of a multi-line chain; warning column, stop on the step's line |
| a06 | `a06_shelf_task.lume` | STOP | `.at` past the end inside a `spawn:` body, reported when awaited; chained `puts` argument |
| a07 | `a07_factorials.lume` | STOP | `Int` overflow in `*` inside a generator, walked lazily by a chained `for` header (`.enumerate` / `.skip`) |
| a08 | `a08_bare_strategy.lume` | FAIL | a field-less strategy passed by its bare name (`Formal` for `Formal()`) as an argument |
| a09 | `a09_chain_first_line.lume` | FAIL | a method body whose first line starts with `.` |

For the FAIL programs the whole `lume check` error is under `EXPECTED
OUTPUT:` (the brief names no separate block for it).

## Guesses

1. **The shape of a runtime stop with a two-digit line number.** The docs
   show stops only at single-digit lines (`errors.md` "Where the program
   stopped": `--> seats.lume:5`, then `  |`, then `5 |   puts ...`; tutorial
   03 the same), with no column and no caret. I assumed the gutter widens as
   it does for compile errors (`   |` / `29 | ...`, from
   `examples/errors/iface_missing.expected`) while `  -->` keeps two spaces,
   and that nothing follows the source line. Files: a03, a04, a05, a06, a07.
2. **A stop inside a block of a chain broken over lines names the step's
   own line** (the `.map { |n| n.to_int! }` line), printed as it is in the
   file, leading spaces included. `errors.md` says only that a block "stops
   on its own line"; no example has a chain over lines. File: a05.
3. **Order on standard error for a program that warns and then stops**: the
   compile warning (`!`), then straight on, with no blank line, the stop.
   The warning text and caret column come from `errors.md`; the
   no-blank-line join from the `.expected` files, where warnings run
   straight into output (`examples/interfaces.expected`). File: a05.
4. **A task that stops is told in exactly the same form as any other stop**:
   the message, `-->` with the line inside the `spawn:` body, the source
   line, and no extra wording such as "task stopped", printed once, when
   awaited. `errors.md` says "A task that stopped says so once, when it is
   awaited" but shows no output; `concurrency.md` says only that it "stops
   the whole program when the task is awaited". File: a06.
5. **An overflow inside a generator** gives `error: Int overflow in `*``
   (message from `numbers.md` "What stops the program") on the generator's
   own `f = f * n` line (from `errors.md`), and the lazy walk prints 18!–20!
   first, because the generator only runs when an item is asked for
   (`collections.md` "Generators"). The pieces are documented; putting them
   together is not shown anywhere. File: a07.
6. **A field-less struct printed inside another value** prints as
   `Urgent()`: in an enum payload `High(why: Urgent())`, in a set
   `{Urgent()}`, as a map key `{Urgent(): 3}`. The docs show only
   `puts Marker()` on its own (`structs-and-enums.md`,
   `examples/unit_structs.expected`). File: a02.
7. **A chain of `.step` lines followed by `|>` lines in one expression**
   (`orders` / `.map {..}` / `|> .reverse` / `|> tag_ids`, and
   `.to_list` / `|> count_qty`), where `|>` takes the whole chain above it.
   The precedence table puts `|>` lowest, and `collections.md` says a line
   may begin with `|>`, but no example mixes the two kinds of line. File:
   a01.
8. **A chain broken over lines inside a call's parentheses**
   (`money(orders` / `.map {..}` / `.sum)`). `collections.md` says a line
   break inside call parentheses does not end the statement and that a
   `.name` line continues the line above; the two together are not shown.
   File: a01.
9. **A bare field-less struct as an argument** gives the same message as in
   a binding, `` `Formal` is a type; construct one with `Formal()` ``, with
   the caret on the name and no help line, as
   `examples/errors/unit_struct_bare.expected` shows for `c = Closed`. File:
   a08.
10. **The leading-dot error in a method body** has the same message, caret
    and help as in a `for` body (`examples/errors/leading_dot_block.expected`,
    where the help names `xs` and `.map ...` even though that program had
    neither, so I took the help to be fixed text). File: a09.
11. **`|> .reverse` on a lazy chain** works and gives a list a `[Int]`
    parameter takes. `methods.md` says every list method that does not
    change the list works on a chain, and `examples/pipes.lume` shows
    `|> .method` stages, but `reverse` on a chain through `|>` is not shown.
    File: a01.

Number of guesses: **11**.

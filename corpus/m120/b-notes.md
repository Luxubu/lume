# Reviewer B — round twenty-four: each settled part

## What I read

- `corpus/m120/BRIEF.md`.
- `docs/stability.md` (the Settled table: 15 rows, so one program a row),
  `docs/README.md`, `docs/install.md` (commands, editors, `check --json`),
  `docs/tour.md`.
- Reference pages, whole: `collections.md`, `methods.md`, `io.md`,
  `concurrency.md`, `testing.md`, `packages.md`, `json.md`, `http.md`,
  `crates.md`, `errors.md`, `printing.md`, `names.md`, `patterns.md`,
  `control-flow.md`.
- Tutorial: lunch 15 (first half) and lunch 18 (whole).
- `packages/json/lib.lume`, `packages/http/lib.lume`, their `lume.toml` and
  `test.expected`.
- `examples/`: `check_json.expected`, `crate.lume`, `crates.lume`,
  `crate_builder.lume`, `process_spawn.lume`, `unused_names.lume`,
  `json_config/`, `http_demo/main.lume`, `packages/` (layout, `shelf`),
  `tests/assert_once`, `tests/async_tests`, `tests/parse`, `fmt/messy`,
  `fmt/chain_comments`, `fmt/blocks`, `fmt/generic*` (inputs), and these
  in `examples/errors/` with their `.expected`: `arity`,
  `assert_with_message`, `await_outside_async`, `generator_used_twice`,
  `generator_in_list`, `async_chain_self`, `spawn_uses_self`,
  `items_and_next`, `child_lines_outside_async`, `child_wait_not_awaited`,
  `count_without_block`, `task_takes_generator`, `next_done_not_var`,
  `next_done_not_awaited`, `async_gen_chain`, `async_gen_outside_async`,
  `block_kept_changes_var`, `yield_in_spawn`, `generator_var_param`,
  `shared_lock_in_block`, `task_try_mixed`, `own_task_args`,
  `spawn_changes_copy`, `spawn_copy_compound`, `process_no_try`,
  `time_format_directive`, `time_pattern_ends_percent`,
  `time_date_month_text`, `async_gen_print`, `chain_no_items`,
  `temp_mutation`, `typo`, `option_method`, `method_on_result`,
  `set_index`, `for_var_items`, `untyped_empty`, `leading_dot_block`,
  `task_awaited_twice`, `tasks_used_after_await`, `task_used_after_move`,
  `shared_readonly`, `import_twice`, `private_access`, `iface_pass_held_value`,
  `tuple_method`; and the head of `UNREACHED.md`.

Not read: the other reference pages (`functions-and-blocks`,
`structs-and-enums`, `interfaces`, `generics`, `modules`, `numbers`,
`operators`, `strings`) beyond what the tour says, and the other lunches.

## The programs

| program | stability row | meant to | what it leans on |
|---|---|---|---|
| `b01_nested_patterns.lume` | The language (`match`, enums) | RUN | `\|` nested inside a list pattern, inside a variant, inside a tuple; `Some(1..9)`; `-1` inside a variant on both sides of a `\|`; `[]` as a variant's argument |
| `b02_chain_has_list.lume` | Collections; lazy chains | FAIL | an unknown method on a chain: named `[Int]`, its "has:" list without `push`, `pop`, `insert`, `remove_at` |
| `b03_generator_printed.lume` | Generators | FAIL | `puts g` uses a generator up, so using it again is refused |
| `b04_io_edges.lume` | Files, folders, paths, environment, time | RUN | `File.append` creating, `Dir.make` twice, `Dir.walk` order, `File.exists?` on a folder, every "cannot …" message, `Path` on text only, `Time.parse` before 1970 / partial / capitals / ignored weekday / `%s` negative / its errors, `%j %%`, `Env.args` |
| `b05_run_edges.lume` | `Process.run`, `Process.run_in` | RUN | `run_in` listing a folder and refusing a missing one, `-1` for a signal, arguments with no shell, output with no newline |
| `b06_child_edges.lume` | `Process.spawn`, `spawn_in`, `Child` | RUN | `spawn_in` missing folder, `kill` after the end keeps the code, `write` after `close_input`, `read_line` after `lines`, `count` with no block on `error_lines` |
| `b07_task_pushes_copy.lume` | Tasks | FAIL | a `spawn:` body changing an outside `var` list with `push` |
| `b08_async_uniq.lume` | Async sequences | FAIL | `uniq` on an async sequence, which the page's list leaves out |
| `b09_test_failures.lume` | Tests (`lume test`) | RUN with `lume test` (3 tests fail on purpose; exit 1) | non-comparison `assert` failure, `>` failure, `!` stopping a test, `async test` |
| `b10_import_lib/` | Packages | FAIL (`lume check …/main.lume`) | `import lib` in a package with `lib.lume` |
| `b11_json_walk/` | The `json` package | RUN | `for x in doc`, `doc.filter`, `doc.len`, `opt_int`, a list of own values `to_json`, `pretty`, four `parse` errors, `need_int` too large / fraction, `\u00XX`, `path` past the end |
| `b12_http_steps/` | The `http` package | RUN | a body step replacing a `header` step's Content-Type, a later body replacing `with_file`, the Error naming method and URL from `send`, `send_parts`, `await fetch`, `delete` |
| `b13_fmt_layout.lume` | The commands (`lume fmt --stdout`) | RUN | `for (i, w)` written without brackets, trailing comma written, chain steps re-indented, `puts(x)` |
| `b14_crate_usize.lume` | Rust crates | STOP (exit 101) | `-1` given where regex wants a `usize`; `captures_len` coming back as an `Int` |
| `b15_check_json_warnings.lume` | For editors and tools (`lume check --json`) | RUN with `lume check --json` | warnings only: `ok` true, two warnings in written order, `col` in characters after `ü`/`é` |

Five are meant to FAIL (b02, b03, b07, b08, b10), one to STOP (b14).

## Guesses

1. **Nested `|` is accepted.** `(Say(["hi" | "hello", ..rest]), _)` puts
   `|` inside a list pattern inside a variant inside a tuple. Files: b01.
   Looked: `patterns.md` ("Every pattern inside another can be any of these,
   to any depth"; every `|` shown is at the top of an arm), the rule about
   `Str` literals in alternatives (which I read as not applying, since each
   alternative *is* a literal, not one inside something).
2. **`Move(dx, -1) | Move(-1, dx)` is accepted**: negative literals inside a
   variant, and one name bound at different positions in the two
   alternatives. Files: b01. Looked: `patterns.md` (`-1` only at top level;
   `Two(x, 0) | Two(0, x)` is the same shape, with `0`).
3. **The coverage check takes the whole `match` as complete**, with no
   unreachable-arm warning, though the guarded-looking `Some(1..9)` and the
   list patterns cut the space unevenly. Files: b01. Looked: `patterns.md`
   "Covering everything".
4. **`Say([])` gets its type from the variant's field.** Files: b01. Looked:
   `collections.md` "Where `[]` and `{}` need a type" (a struct field and a
   function argument are listed; a constructor argument only for sets/maps).
5. **The "has:" list is the `help:` line** of the error. Files: b02. Looked:
   `methods.md` (two `#!` lines, layout not shown), `option_method.expected`.
6. **The caret of a method error is on the `.`** before the name. Files: b02,
   b08. Looked: `count_without_block`, `async_gen_chain`, `option_method`.
7. **`puts g` is where the generator is used up**, with
   `generator_used_twice`'s words and the caret on `g`. Files: b03. Looked:
   `collections.md` ("After `for x in g`, `g.sum` or `puts g` …"),
   `generator_used_twice.expected`.
8. **`Path.dir("/top")` is `""`**, not `/` as Rust's `parent` would give.
   Files: b04. Looked: `io.md` table ("everything before the last `/`"; "What
   is missing comes back as `""`").
9. **`Path.base("/tmp/dir/")` is `""`.** Files: b04. Looked: `io.md` table.
10. **`Path.ext("/a.b/c")` is `""`**: the extension of the last part only.
    Files: b04. Looked: `io.md` ("Only the last extension counts" — of the
    name, I take it).
11. **Every `Time.parse` error has the prefix ``` `text` does not match
    `pattern`: ```**, the date-validity ones included. Files: b04. Looked:
    `io.md` ("each after the text and the pattern"; only one whole message
    shown).
12. **A leap February reads "February 2024 has days 1 to 29, not 30".**
    Files: b04. Looked: `io.md` error table (2023 and April shown).
13. **`Time.parse` before 1970 gives a negative time, and `%s` reads
    `-60`.** Files: b04. Looked: `io.md` ("`%s` any number, with a `-` before
    1970").
14. **`File.write`, `File.remove` and `Dir.list` failures read ``cannot
    write `p`: …``, ``cannot remove `p`: …``, ``cannot list `p`: …``** with
    the system's words as for `read`. Files: b04. Looked: `io.md` "When I/O
    fails" (only `read` is shown whole).
15. **`ls` with output to a pipe prints one name a line, sorted.** (The
    system's, not Lume's.) Files: b05.
16. **`read_line` after `lines` has taken everything gives `None`** — the two
    read the same output. Files: b06. Looked: `io.md` Child table, lunch 18.
17. **A method that changes an outside `var` (`push`) in a `spawn:` body is
    refused in `spawn_changes_copy`'s words**, caret on the name. Files: b07.
    Looked: `concurrency.md` "What a task sees: copies",
    `spawn_changes_copy.expected`, `spawn_copy_compound.expected`.
18. **`uniq` is refused on an async sequence**, in `async_gen_chain`'s words
    with `Int` and `uniq`. Files: b08. Looked: `collections.md` (the two
    descriptions disagree, see below), `async_gen_chain.expected`.
19. **An `assert` with `>` shows `left:` and `right:`** as `==` does.
    Files: b09. Looked: `testing.md` ("A failed comparison shows both
    sides"; only `==` shown), `errors.md` ("with both sides when it
    compares").
20. **An `Int? or Error` holding `None` prints `Ok(None)`.** Files: b11.
    Looked: `printing.md`, `methods.md` (no nested case shown).
21. **`send_parts` to a port where nothing listens gives an `Error`** in the
    same `METHOD URL: …` form (read in `packages/http/lib.lume`), without
    first walking the generator to the end in a way that shows. Files: b12.
22. **`lume fmt` keeps one blank line between the header comments and
    `def main`**, and re-indents list items written at six spaces to four.
    Files: b13. Looked: `fmt/messy`, `fmt/chain_comments`.
23. **What "the whole output" is for a `fmt` program**: the output repeats
    the header, so the header cannot hold itself; I wrote "these header
    lines, as they are, then a blank line, then:". Files: b13.
24. **The stop for a `usize` that does not fit is laid out as every stop**
    (`error:`, `  --> file:line`, gutter, the line, no column), on the line
    of `c.get(i)`. Files: b14. Looked: `crates.md`, `errors.md` "Where the
    program stopped".
25. **`date.captures_len` is called without `()`**; the example writes
    `req.method()` but `m.start`. Files: b14. Looked: `crate.lume`,
    `crate_builder.lume`.
26. **`.map { |m| m.as_str }` works on a `regex.Match?`** (the example uses
    `!`). Files: b14. Looked: `crate.lume`, `methods.md` (`T?` `map`).
27. **Two diagnostics are laid out as serde's pretty JSON** (`},` then
    `{` on the next line), keys in the order `check_json.expected` shows.
    Files: b15. Looked: `install.md`, `check_json.expected` (one diagnostic
    only).
28. **`lume check --json` exits 0 with warnings only.** Files: b15. Looked:
    `install.md` ("The exit status is 1 when there is an error").
29. **`lume test`'s exit 1 is written `EXPECTED EXIT: 1`**, the brief giving a
    form only for 101. Files: b09.

**Total: 29 guesses.**

## Where two pages disagree

- `collections.md`, async sequences: "those that need every item before they
  can answer — `join`, `sort`, `max` and the rest" reads as open-ended, but
  the section below lists exactly nine, and `async_gen_chain`'s help repeats
  the nine. `uniq`, `avg`, `to_set`, `group_by`, `partition` need every item
  too (b08).
- `methods.md` says of a tuple "Asking gives an internal error, not a list";
  `examples/errors/tuple_method` shows a proper error, ``a tuple has no
  method `len` ``, with help.
- `install.md` writes the flag both after and before the file:
  `lume check file.lume --json` in the command list, `lume check --json
  main.lume` below it.
- `errors.md` says `?` in `main` ends the program with "a non-zero exit
  status"; `io.md` and `concurrency.md` say exactly 1; the brief says a stop
  exits with 101 — two different endings, which no page sets side by side.
- `testing.md`: an `async test` "runs on its own tokio runtime", while
  `concurrency.md` says "`spawn:` needs `async def main`"; whether `spawn:`
  in an `async test` needs an async `main` is not said (b09 uses one).

## Promises in reference pages that no example shows

- Patterns: `|` nested inside another pattern (b01); a range inside `Some`.
- Methods: the chain's "has:" list (b02); `set.first` "added earliest";
  `superset?`; map `get`; `sum` with a block; `avg` of an empty list `0.0`.
- Generators: `puts g` using a generator up (b03); "Once it has ended, `next`
  gives `None` every time".
- I/O: `File.append` creating (b04); `Dir.make` on an existing folder (b04);
  `File.modified`; every "cannot …" message but `read` (b04); `Path.dir` and
  `Path.base` edge cases (b04); every `Time.parse` error but the first
  (b04); `%s`, `%j` parse; `Env.stdin`; `Env.exit` in a value `match` (in
  docs only).
- Process: the `run_in` missing-folder message (b05); `-1` for a signal from
  `Process.run` (b05); Windows behaviour.
- Child: the `write`-after-close message text (b06); `kill` after the end
  (b06); `spawn_in` failing (b06); `count` with no block (b06).
- Tasks: a `spawn:` that uses `?` on an optional giving `T?`; `next_done` on
  an empty list giving `None`; a method changing a copied `var` (b07); the
  crash of a task never awaited, told at the end.
- Async sequences: `uniq` and the other list methods outside the nine (b08);
  a chain block changing a name it copied ("changing one inside is
  refused") — only the kept-block form has an error example.
- Tests: a non-comparison `assert` failure (b09); a test name filter that
  matches nothing (``none of the 2 tests has `zzz` in its name``); module
  tests named `menu: …` and run first.
- Packages: `import lib` refused (b10); a dependency named unlike its package;
  two packages depending on each other; one name for two folders; the orphan
  rule's message; a dependency of a dependency; git dependencies, `lume.lock`
  and `lume update`.
- JSON: `for x in doc` / `doc.filter` (b11); `pretty` (b11); `opt_…` (b11);
  `need_items`'s error; a number too large (b11); every `parse` error but
  three.
- HTTP: Content-Type replaced by a body step (b12); a later body replacing
  an earlier one (b12); the message of a connection that cannot be made
  (b12); `Opened.chunks` only in the server-run demo.
- Crates: the `usize` check (b14); a `u64` past the largest `Int`; what
  `lume crate` prints; the "uses up `x`" refusal (`UNREACHED.md` says no
  crate available offline reaches it).
- Commands: `lume fmt` dropping the brackets of `for (i, w)` (b13); `lume
  build --checked`; `lume clean`; `lume update`; `lume emit`.
- Editors: `lume check --json` with warnings, and with more than one
  diagnostic (b15); what `lume lsp` answers; `LUME_LSP_LOG`.

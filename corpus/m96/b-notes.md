# Reviewer B — notes

Aim: the edges of milestones 92–95: the waiting methods on async sequences,
`Process.spawn` and `Child` past their ends, `async Iterator[T]` written as
a type, and the `http` request builder and its failures.

## What I read

- `corpus/m96/BRIEF.md`.
- `docs/reference/collections.md`, all of it from "Types of your own: `next`"
  to "Taking a sequence", plus "Lazy chains": async generators, chains over
  them, "Methods that wait for the items".
- `docs/reference/io.md`: `Process`, "A program running alongside", `Time`,
  "When I/O fails".
- `docs/reference/concurrency.md`, all of it.
- `docs/reference/http.md`, all of it.
- `docs/reference/methods.md`: the list table and the set table, the error
  sample at the top.
- `docs/reference/errors.md` (printing an `Error`), `printing.md` (the rule
  and each kind of value), `numbers.md` (how floats print),
  `functions-and-blocks.md` (block forms, kept blocks), `control-flow.md`
  (`if` as a value), `modules.md` (what a `pub` type makes public).
- `examples/process_spawn.lume` and `.expected`,
  `examples/async_generators.lume` and `.expected`, `examples/http_demo/`.
- `packages/http/lib.lume`, `lume.toml`, `test.expected`.
- `examples/errors/`: the `.lume`/`.expected` pairs for `async_*`, `await_*`,
  `child_*`, `process_no_try`, `filter_not_bool`, `block_equality`,
  `block_wrong_result`, `block_kept_changes_var`, `generator_block_param`,
  `generator_used_twice`, `generator_in_list`, `task_takes_generator`,
  `task_used_after_move`, `count_without_block`, `sum_of_text`,
  `compare_not_bool`; `examples/errors/UNREACHED.md`. These gave the whole
  shape of an error as `lume check` prints it: the `-->` line with the path
  as given, the gutter width set by the line number's digits, the caret at
  the `.` of a method call or the `{` of a block.

Nothing outside docs/, examples/ and packages/ was read; nothing was run.

## The programs

| file | RUN/FAIL | probes |
|---|---|---|
| `b01_stop_or_not.lume` | RUN | `first`, `find`, `any?`, `all?` on an endless async generator, after `skip`, `map`/`filter`, `take_while`; the same on empty sequences |
| `b02_count_sum_fold.lume` | RUN | `count` with a block; `sum` of Ints, of Floats, with a block (also a Float block over Ints), on empty sequences; after `filter`/`map`, `skip`/`take`, `enumerate` with a two-name block; `fold` to an Int and to a Str |
| `b03_each_changes_names.lume` | RUN | `each` and `fold` blocks changing `var`s around them (a `do` body, after a chain, on an empty sequence); a kept async chain's block keeping the copy of `k` made with the chain; a program's own `struct Child` with `lines` and `wait` |
| `b04_child_lines_wait.lume` | RUN | waiting methods on a `Child`'s `lines`: `count`, `sum` with a block, `fold`, `find`, `any?` on output that never ends (then `kill`, `wait`), `each` changing names, `first` on no output; `error_lines` read after `lines` |
| `b05_count_no_block.lume` | FAIL | `count` with no block on an async sequence |
| `b06_find_not_bool.lume` | FAIL | a `find` block that gives an `Int` |
| `b07_no_max_yet.lume` | FAIL | `max`, a list method an async sequence lacks, after a `map` |
| `b08_lines_count_outside_async.lume` | FAIL | `c.lines.count { .. }` in a plain `def main` |
| `b09_child_after_the_end.lume` | RUN | `read(n)` past the end and again, `read_line` after the end, `write` after `close_input` and after `wait`, `kill` then `wait` then `read_line` and `write` |
| `b10_source_used_after.lume` | FAIL | an async generator handed to `xs: async Iterator[Int]`, then used again |
| `b11_async_iterator_types.lume` | RUN | `def -> async Iterator[Int]` handing on a chain; `def shouted(c: Child) -> async Iterator[Str]` handing on a chain over `c.lines`; `async def` parameters taking a chain, a generator chain kept in a name; a method parameter taking a `Child`'s `lines` |
| `b12_child_equal.lume` | FAIL | two `Child` values compared with `==` |
| `b13_child_kept.lume` | RUN | `[Child]` mapped to tasks, a `Child` in a struct field, a `Child` captured by a `spawn:` body, `spawn_in("/")`, `spawn_in` a missing folder, a missing program |
| `b14_http_builder/` | RUN | `http.request` builder: each step a new `Request`, the old ones unchanged; `header` order and a repeated name; `with_body` twice gives two `Content-Type`s; `timeout` with a negative number |
| `b15_http_unreachable/` | RUN | `send`, `await fetch`, `open` on `"not a url"` and on `http://127.0.0.1:9/` (POST with a body and a timeout); `http.get`, `http.send` with `"::"`, `http.post` |

## Guesses

1. **The path in `-->`** is the path as passed to `lume check` from the
   repository root, `corpus/m96/<file>`. (b05–b08, b10, b12. Looked:
   `examples/errors/*.expected` show `examples/errors/<file>`.)
2. **An empty `[Float]` sum prints `0.0`.** (b02. Looked: methods.md says
   "`0` when empty" for `sum`; printing.md says a Float keeps its point.)
3. **`sum` with a block that gives a `Float`, over an `Int` sequence, gives
   a `Float` (`1.5`).** (b02. Looked: methods.md's table says `sum` gives
   `T`, and "of what the block gives each".)
4. **`fold("") do |acc, w|` with an indented two-line body is accepted on
   an async sequence.** (b03. Looked: functions-and-blocks.md shows `do |n|`
   with `each` only, and `fold` with `{ }` only.)
5. **A program's own `struct Child` may have a field `lines` and a plain
   method `wait`, and `puts` prints it as `Child(name: "ann", lines: ["a",
   "b"])`.** (b03. Looked: io.md "a program's own type named `Child` takes
   the name instead"; printing.md for structs.)
6. **After `any?` lets go of a `Child`'s endless output, `kill` and
   `await wait` give `-1`** whether the shell had already died of a broken
   pipe or is killed then. (b04. Looked: io.md "`-1` when ended by a
   signal"; process_spawn example `killed: -1`. Not stated what letting
   go of `lines` does to the program.)
7. **`printf ""` writes nothing and `lines.first` gives `None`.** (b04.
   Looked: io.md, `read_line` gives `None` once it has ended.)
8. **`count` with no block on an async sequence is refused with the list's
   message and help**, even though the help names `.len`, which an async
   sequence lacks; it could instead be accepted (Rust's `StreamExt::count`
   takes none) and print `3`. (b05. Looked: methods.md "`count` needs a
   block", `count_without_block.expected`, collections.md "give what a
   list's would", `async_gen_chain.expected` help.)
9. **The message for a `find` block that is not a `Bool`**: "the block of
   `find` gives back an `Int`, but it must give a `Bool`", caret at `{`,
   help "it says yes or no to each item, as in `.find { |x| x > 0 }`".
   Wording and help are copied from `filter`'s. (b06. Looked:
   `filter_not_bool.expected`, `block_wrong_result.expected`.)
10. **`max` after a `map` on an async sequence gives the `join` message's
    shape**, "an `async Iterator[Int]` has no `max` yet", caret at the `.`
    before `max`, same help. (b07. Looked: `async_gen_chain.expected`.)
11. **A waiting method on `c.lines` in a plain `def` gives the same message
    as a `for` over it**, caret at the `.` of `.lines`, and not a message
    about `count`. (b08. Looked: `child_lines_outside_async.expected`;
    UNREACHED.md says the ".x waits for items" message is behind other
    checks.)
12. **`read(1)` once the output is used up gives `None`, not
    `Some("")`;** `read(5)` with one byte left gives `Some("c")`. (b09.
    Looked: io.md table, "or what is left when it ends first".)
13. **`write` after `wait` (and after `kill` then `wait`) gives the
    closed-input message with that program's name**, and `read_line` after
    `kill`+`wait` gives `None`. (b09. Looked: io.md table, which names
    `wait` but shows only `sort`.)
14. **`() or Error` is accepted as a parameter type** (`def said(r: () or
    Error)`). (b09. Looked: errors.md, io.md; only seen as a result type.)
15. **An async generator handed to a parameter and used again gives the
    generator's message**, "`s` is a generator, and it is used up here, so
    it cannot be used after", caret at the argument `s`, the generator's
    help. It may say "async sequence" or "async generator" instead. (b10.
    Looked: `generator_used_twice.expected`, collections.md "It moves like
    a generator", "takes one by move".)
16. **A plain `def` taking a `Child` may hand on a chain over `c.lines` as
    `-> async Iterator[Str]`** without the "walked in async code" error,
    since it walks nothing. (b11. Looked: collections.md "a chain over
    another, or one a package's Rust makes … a `Child`'s `lines`";
    `packages/http/lib.lume`'s `lines` is a plain `def` but made in Rust.)
17. **`==` on two `Child` values is refused; the whole message and help are
    guessed** ("`==` cannot compare `Child` values", caret at `==`, a help
    about comparing what it gives). It might instead compile. (b12. Looked:
    io.md says nothing; `block_equality.expected` for the shape.)
18. **A `Child` in a struct field on an immutable binding can be walked
    (`j.child.lines.count`) and waited (`await j.child.wait`)**, and one
    captured by a `spawn:` body can be walked there. (b13. Looked: io.md "A
    `Child` can be kept in a list or a field and handed to tasks".)
19. **`spawn_in` a missing folder gives ``cannot run `ls` in
    `/no/such/folder-b`: no such folder``**, the `run_in` message. (b13.
    Looked: io.md "with the same messages as `Process.run`", and the
    `run_in` row.)
20. **`http.send("PUT", "::", [], "")` is an `Error`, not a crash** in
    ureq's `request()`. For `"not a url"` the package's own test confirms
    it; for `"::"` it is a guess. (b15. Looked: `packages/http/lib.lume`
    and its tests.)
21. **`http.Response or Error` is accepted as a parameter type** with the
    module prefix. (b15. Looked: http.md, modules.md "its types
    (`shop.items.Item`)".)
22. **`await bad.fetch` on a URL that does not parse gives the same
    `Error` as `send`**, carried back through the task. (b15. Looked:
    `lib.lume` `fetch` is `spawn: req.send` then `await t`.)

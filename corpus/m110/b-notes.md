# Reviewer B — the edges

## What I read

- `corpus/m110/BRIEF.md`
- `docs/reference/names.md` (whole; "A name that is never used")
- `docs/reference/io.md` (whole; "A program running alongside")
- `docs/reference/testing.md` (whole)
- `docs/reference/interfaces.md` (sections 1–3, "Methods that take a block,
  have type parameters, or are `async`")
- `docs/reference/collections.md` ("Generators", "Async generators", chains,
  methods that wait, "Taking a sequence")
- `docs/reference/errors.md` (`?`, `!`, "Where the program stopped")
- `docs/reference/concurrency.md` (`await t?`, `shared var` parameters,
  "Where the warnings go")
- `docs/reference/patterns.md` (the table, "`|`: alternatives")
- `docs/stability.md`, `docs/install.md` (the commands, warnings shown once
  per compile)
- `docs/tutorial/18-programs-alongside.md`, `docs/tutorial/09-enums-and-match.md`
  (9.4), `docs/tutorial/12-generics-modules-and-tests.md` (12.5, by grep)
- `examples/unused_names.lume` + `.expected`, `examples/async_interfaces.lume`
  + `.expected`, `examples/process_spawn.lume` + `.expected`,
  `examples/async_generators.lume`, `examples/generic_extend.lume` (start)
- `examples/errors/`: `async_gen_method`, `async_gen_method_outside_async`,
  `async_gen_outside_async`, `async_not_iterator`, `async_gen_chain`,
  `async_gen_print`, `async_not_awaited`, `await_outside_async`,
  `await_in_test`, `child_lines_outside_async`, `child_wait_not_awaited`,
  `generator_as_interface`, `generator_as_bound`, `generator_in_extend`,
  `iface_unused_param`, `iface_mismatch`, `iface_missing`,
  `iface_generic_through_pointer`, `iface_pass_held_value`,
  `iface_default_calls_async`, `iface_async_default_block`,
  `iface_seq_lends_block`, `interface_param_async`, `loop_var_twice`,
  `block_param_twice`, `destructure_same_name`, `or_pattern_bindings`,
  `task_awaited_twice`, `yield_in_test`, `var_param_copy`,
  `var_arg_expression`, `var_param_immutable`, `shared_param_plain`,
  `UNREACHED.md` (start); every `.expected` with a `warning:` line in
  `examples/` (by grep), to learn the warning layout.

Two things in the docs disagree with each other, found on the way:

- `docs/reference/collections.md`, in the list under "Generators", still says
  **"Not yet:** an `async` generator in an interface", a few lines above the
  "Async generators" section that says an interface may require one (as
  `interfaces.md` and `stability.md` do). The "not yet" line should be about
  a *default* async generator only.
- `docs/stability.md` opens with "Lume is at 0.6", while its table says
  three parts are "settled in 0.7".

## The programs

| id | file | probes | meant to |
|---|---|---|---|
| b01 | `b01_read_through_try_bang.lume` | a name read only through `x?` (a `T or Error` and a `T?`) or `x!`; the `!` warning against an unused local on the line before: which order | RUN, 2 warnings |
| b02 | `b02_later_block_and_underscores.lume` | read only in a later `for` body or a block; `var` parameter changed in place; `_` twice in a block and a `for`; `_.upcase` shorthand: no warnings at all | RUN |
| b03 | `b03_var_only_changed.lume` | a `var` only `+=`'d and a `var` list only pushed to, never read | RUN, 2 warnings |
| b04 | `b04_patterns_or_tuple.lume` | unused name in both `\|` alternatives; in a destructured tuple; in a tuple pattern | RUN, 3 warnings |
| b05 | `b05_shared_var_unused.lume` | `shared var` parameter changed in place (used); `shared var` local only added to by tasks: both warnings, in which order | RUN, 3 warnings |
| b06 | `b06_loop_name_twice.lume` | `for _, _` allowed, `for k, k` refused; an unused local in the same file is not reported | FAIL |
| b07 | `b07_or_alternative_underscore_name.lume` | `Circle(r) \| Square(_r)`: a `_name` is still a name for the alternatives rule | FAIL |
| b08 | `b08_async_iface_generic.lume` | generic interface, `def items -> async Iterator[T]` form; one type fits with `async def … -> Iterator[Str]`, one with a `def … -> async Iterator[Int]` with no `yield` | RUN |
| b09 | `b09_async_iface_extend.lume` | `extend Int with Feed[Int]` (async generator with a parameter, over `self`), `extend [Str] with Named`; `first` on an empty one, `count` with no block | RUN |
| b10 | `b10_async_iface_default.lume` | a default async generator in an interface, method named `posts` | FAIL |
| b11 | `b11_async_iface_plain_generator.lume` | a struct's own plain generator where an async one is required | FAIL |
| b12 | `b12_child_edges.lume` | `read(2)` across `é` (printf `\303\251`), `read` near and after the end, `wait` twice, `kill` after the end, `write` after the end and after `close_input`, `close_input` twice | RUN |
| b13 | `b13_test_filter_none.lume` | `lume test FILE zzz`, matching none | RUN (`lume test`) |
| b14 | `b14_child_wait_again.lume` | a second `wait` without `await`, after an awaited one | FAIL |
| b15 | `b15_async_test_task_stops.lume` | `async test` whose awaited task stops in a plain function; the tests after it still run | RUN (`lume test`), exit 1 |

Five are meant to FAIL (b06, b07, b10, b11, b14). For those, the whole of
what `lume check` prints is under `EXPECTED OUTPUT:`.

## Guesses

1. **`name?` on a bare local is `?`, not a name ending in `?`.** b01
   (`parsed?`, `first?`). Names may end in `?` (`names.md`, "A name may end
   in `?`"); the only bare-name `?` in the docs is `await t?`
   (`concurrency.md`). I assumed `parsed?`, with no `parsed?` bound, is `?`
   applied to `parsed`.
2. **The order of different kinds of warning.** b01, b05. I guessed that
   warnings come by kind, not by line: the `!` and `shared var` warnings
   (found as the code is compiled) come first, then the unused-name ones,
   even when the unused name is on an earlier line (b01: `spare` on line 37,
   `!` on line 38, `!` given first). `examples/unused_names.expected` has only
   unused-name warnings, all in line order; no example mixes kinds. The
   brief's "in the order the compiler finds them" was my only hint.
3. **Where the caret goes for an unused `var` or `shared var` local.** b03,
   b05. Guessed column 3, the start of the statement (`var` / `shared`), as
   the `for` warning points at `for` and the `shared var` warning at
   `shared` (`unused_names.expected`, `concurrency.md`). For `tax = 0` the
   name and the statement start are the same column, so nothing shows which.
4. **A `var` only added to (`count += x`) is warned about**, with the
   local's text, "`count` is given a value but never used", and the local's
   help. b03. `names.md` says "given a value and never read"; that a `var`
   *parameter* changed in place counts as used "because the caller sees the
   change" suggests a local changed in place does not.
5. **A `var` list only pushed to is warned about the same way.** b03.
   Same reasoning as 4; Rust itself would not warn here, since `push` borrows.
6. **A name bound in both alternatives of `|` and never read gets one
   warning**, at the first alternative's name. b04. Nothing shows an unused
   name in an `|` pattern.
7. **A name from a destructured tuple `(whole, rest) = (7, 2)` is warned
   about as a local** ("is given a value but never used", local help), with
   the caret on the `(` at column 3. b04. `destructure_same_name.expected`
   puts its caret at the `(`; nothing shows the warning.
8. **A `shared var` local the tasks only add to also gets the unused
   warning**, after its `shared var` warning. b05. As 4, and 2 for the order.
9. **With an error in the file, no warnings are printed.** b06.
   `examples/errors/UNREACHED.md` says so for the `return`-in-a-block warning
   ("only the error is printed"); I assumed it holds for unused names too.
10. **`_r` counts as a name for "every alternative of `|` must bind the same
    names"**, and the help says "this one binds `_r`". b07.
    `or_pattern_bindings.expected` shows the help form for `r` and `side`.
11. **A struct method `def items -> async Iterator[Int] = count_down(from)`,
    with no `yield`, fits an interface requiring an async generator.** b08.
    The docs say the two forms "say the same", and show a `def … -> async
    Iterator` handing one on only as a free function (`loud` in
    `examples/async_interfaces.lume`).
12. **`extend Int with Feed[Int]` can give an async generator with a
    parameter, walking `self`.** b09. `collections.md` says a generator may
    be one in an `extend` "on a built-in type too", and
    `async_interfaces.lume` shows an async generator in an `extend` of a
    struct; not both together, nor a generic interface fixed at `[Int]`.
13. **The help for a default async generator names the method**: "`async
    def posts(...) -> Iterator[T]`". b10. `async_gen_method.expected` has a
    method called `items`, so it does not show whether `items` is fixed text.
14. **A struct's own plain generator, where an async one is required, is
    told where the struct is used**, in the `iface_mismatch` form ("`Shelf`
    is used as a `Named`, but its `names` is `-> Iterator[Str]` and `Named`
    needs `-> async Iterator[Str]`"), at the argument, with no help line. b11.
    `generator_in_extend.expected` gives this text for an `extend`, at the
    `extend` line; `iface_mismatch.expected` gives the use-site form.
15. **`read(n)` once the output is used up gives `None`.** b12. `io.md`
    says "what is left when it ends first" and that `read_line` gives `None`
    once it has ended; it does not say what `read` gives when nothing is left.
16. **`close_input` twice does nothing the second time.** b12. Not stated.
17. **What `lume test FILE zzz` prints when nothing matches**: a blank line,
    then `0 tests: 0 passed, 0 failed (2 not matching `zzz`)`, exit 0. b13.
    `testing.md` says only that it "says so, and is not a failure"; the
    form comes from its `(1 not matching `rounded`)` example. The blank line
    and "0 tests" (plural) are guesses.
18. **A task that stops inside an `async test` is told at the line where it
    stopped, inside the plain function `pick`**, as
    `FILE:16: stopped: no item at 5 — the list has 3`, on standard output,
    with nothing else on standard error. b15. `testing.md` shows this for a
    plain test and a plain function; `errors.md` says a task stops "on its
    own line" and is told where it is awaited. Nothing shows the two together.
19. **The `_.upcase` shorthand gives no block-parameter warning.** b02. It
    has no name to warn about, but no doc says so.
20. **The warnings are printed because the run compiles the file.** b01,
    b03, b04, b05. `install.md` says warnings are not shown again "each time
    an unchanged program runs". I assumed each program is run once, fresh.

**Total: 20 guesses.**

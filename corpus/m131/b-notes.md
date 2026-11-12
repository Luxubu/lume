# Reviewer B — round twenty-six: the edges

## What I read

- `corpus/m131/BRIEF.md`.
- `docs/stability.md`; `docs/reference/collections.md` (all of it, generators
  and sequences above all); `docs/reference/http.md`; `docs/reference/errors.md`;
  `docs/reference/generics.md` (generic `extend`); `docs/reference/printing.md`
  (first half); `docs/reference/names.md` (never-used names);
  `docs/reference/concurrency.md` (tasks that wait without `await`, `spawn:`
  bodies, copies, where warnings go).
- `docs/tutorial/16-generators.md`, `docs/tutorial/27-a-small-web-service.md`;
  `docs/tutorial/answers.md` (the lunch 10 answer, for how a failure out of
  `main` prints); a passage of `docs/tutorial/22-a-csv-query-tool.md`.
- `docs/tutorial/orders-service/` (all four files).
- `examples/generators.lume` and `.expected`, `examples/generators_order.lume`
  and `.expected`, `examples/unused_names.lume` and `.expected`,
  `examples/http_server/` (all), `examples/notes/main.lume`, `routes.lume`,
  `check.sh`, `check.expected`, `FINDINGS.md`, `lume.toml`.
- `examples/errors/`: `extend_bound_unmet`, `generic_bound_unmet`,
  `generator_used_twice`, `dropped_failure`, `dropped_failure_unit`,
  `dropped_result`, `generator_return_value`, `generator_in_extend`,
  `generator_var_param`, `generator_in_list`, `generator_block_param`,
  `generator_var_self`, `generator_as_bound`, `generator_as_interface`,
  `yield_in_block`, `builtin_arg_type`, `method_on_result`, `missing_arg`,
  `return_value_from_unit` (each `.lume` and `.expected`).
- `packages/http/lib.lume` (the whole of it: `value`, `parse_form`,
  `unescape`, `Server`, `Incoming`, `respond`, `Reply`, `listen`, its tests)
  and `packages/http/lume.toml`.

I ran nothing: no compiler, no program.

## My programs

| id | file | probes | meant to |
|---|---|---|---|
| b01 | `b01_after_yield.lume` | a loop around one `yield`, statements after the `yield`; `for`, `break`, `take(2)`, `find` | RUN |
| b02 | `b02_setup_and_empty.lume` | set-up that is a call, put off until the first item; arguments worked out at the call; a `while` that fails at once, under `to_list`, `sum`, `first`, `for` | RUN |
| b03 | `b03_walked_twice.lume` | a generator in a plain name used up by a `for`, then used again | FAIL |
| b04 | `b04_next_after_end.lume` | `next` on a `var` past the end, with a statement after the last `yield`; a generator called again starts afresh; a `for` after a `next` carries on | RUN |
| b05 | `b05_arm_not_last.lume` | a `() or Error` call ending a `match` arm when the `match` is not the function's last line | FAIL |
| b06 | `b06_if_and_two_yields.lume` | a `yield` inside an `if` in a `for`, under `take(2)`; two `yield`s in one loop, walked whole and under `take(3)` | RUN |
| b07 | `b07_handed_on_twice.lume` | a generator handed to an `Iterator[Int]` parameter twice | FAIL |
| b08 | `b08_take_while.lume` | an endless generator cut by `take_while` (true for a while, false at once, under `sum`) and by `skip(2).take(1)` | RUN |
| b09 | `b09_over_for.lume` | a generator over a `for` of its parameter: a chain from a list (worked out at the call), one from a range (lazy), a list `push`ed after the call | RUN |
| b10 | `b10_http_value/` | `http.value` with a name twice, none, another case, an empty list; on `parse_form`'s pairs; a form that will not decode | RUN |
| b11 | `b11_close_from_task/` | `incoming` ended by `close` from the client's task; `accept` after `close`; `http.value` on `req.params` with a name twice and one with no `=` | RUN |
| b12 | `b12_reply_204/` | a `204` with no body; a second `respond`; a status of 600, refused, then a real answer | RUN |
| b13 | `b13_extend_bound.lume` | `extend [T: Shown]` on `[Int]`, on `[[Int]]`, and on `[Float]` | FAIL |
| b14 | `b14_last_call.lume` | `() or Error` calls ending each branch of a closing `if`/`else` and each arm of a closing `match`; failure out of `main` (exit 1) | RUN (exit 1) |
| b15 | `b15_stray_call.lume` | a stray `() or Error` call before a closing `if`/`else` whose branches end in calls | FAIL |

Five meant to FAIL (b03, b05, b07, b13, b15), nine to RUN, and one (b14) to
RUN and end with exit status 1 through `main`'s `?`.

## Guesses

1. **Where the caret goes when a `for` uses a generator up.** b03. I put it
   on `menu` in `for c in menu:` (column 12), with the same message and help
   as `generator_used_twice`. Looked at `examples/errors/generator_used_twice.expected`
   (the caret is on the name at a method call) and lunch 16.6; neither has a
   `for`.
2. **A hand-over to a parameter counts as the "used up here".** b07. The
   error is on the first `total(e)`, caret on the argument `e` (column 14),
   not on `total`. Looked at `generator_used_twice.expected`, and
   `collections.md` ("handed on — to a parameter `xs: Iterator[T]`").
3. **Which call a `match` that is not last gets refused at.** b05. I
   predict the refusal is the "can fail, and nothing here looks at whether
   it did" one, at the first arm's call (`save` at column 16), not a
   message about the `match` as a whole. Looked at `errors.md` (the three
   forms of the rule), `dropped_result.expected`, `notes/FINDINGS.md`.
4. **Only the first error is printed.** b05 (the `None` arm has a second
   stray call) and b15. Looked at every `examples/errors/*.expected` I read:
   each shows one error, but none of those programs has two.
5. **A `() or Error` call ending an `if`/`else` branch is the function's
   value.** b14, b15. `errors.md` says so for an `Error(...)` that ends a
   branch, and `notes/FINDINGS.md` for a call ending a `match` arm; nothing
   says so for a call ending an `if`/`else` branch. `examples/notes/main.lume`
   shows the `match` arm case only.
6. **A nested list meets the bound through the same `extend`.** b13. I
   predict `[[1], [2, 3]].shown` is accepted, since `[Int]` is `Shown` by
   `extend [T: Shown]`, so the only error is the `[Float]` line. Looked at
   `generics.md` and `extend_bound_unmet`; `FINDINGS.md` says the check
   covers "built-in values and ... an interface this file knows", and
   nothing says whether a list of them counts.
7. **The words for an unmet bound on `[Float]`.** b13. I copied the
   `extend_bound_unmet.expected` form, with `Float` in place of `Int` and my
   interface's name. Looked at that file only.
8. **`take_while` asks for exactly one item past the last it keeps, and no
   more.** b08. That is Rust's `TakeWhile`. `collections.md` says only that
   it asks for "only as many items as they need".
9. **The `next` that finds the end runs what comes after the last
   `yield`, once; the `next`s after that run nothing.** b04. Looked at
   `generators_order` (`count_up(5, 5)` has nothing after its `yield`) and
   `collections.md` ("`next` gives `None` every time it is asked again").
10. **The query reaches the server exactly as written**, `x=1&x=2&flag`,
    through ureq on one side and tiny_http on the other. b11. Looked at
    `http.md` ("what follows `?` in the URL, as it was sent") and the
    package source; how ureq writes a URL out is not on any page.
11. **Building with the `http` package writes no warnings to standard
    error.** b10, b11, b12. `examples/http_server/check.sh` filters warning
    lines out, so the examples do not show whether any are written. I give
    no `EXPECTED STDERR:` for these.
12. **`accept` on a closed server returns at once**, with the package's
    message `the server is closed`, rather than waiting. b11. Looked at
    `http.md` (the `accept` and `close` rows) and `Server.accept` /
    `Server.close` in `packages/http/lib.lume`.
13. **After `close` from the task, the `incoming` loop ends with no more
    items and no failure**, even if a spawned `accept` was already waiting.
    b11. Looked at lunch 27.5, `http.md` ("a `for` over it ends when the server
    is closed"), and `Server.incoming` in the package.
14. **A 204 comes back as status 204, body `""`, `ok?` true**, and tiny_http
    sends it without trouble. b12. Looked at `notes/check.expected`
    (`DELETE /notes/2 -> 204 `) and `http.md`.
15. **A failure out of `main` writes only `error: <message>`** to standard
    error, with no `-->` line, and exits with 1. b14. Looked at `answers.md`
    (lunch 10) and `concurrency.md`; neither shows a full stderr capture.

**Total: 15 guesses.**

## Where the pages disagree, or say little

- `docs/stability.md` opens "Lume is at 1.1". The same page says serving
  HTTP is "settled in 1.2" and `http.value` is "(1.2)", and "Previews: There
  are none". `http.md` agrees with the 1.2 story. `packages/http/lume.toml`
  says `version = "1.1.0"`.
- Lunch 16, "Underneath, in Rust", says a loop around one `yield` is a
  closure "as `naturals` is", but lunch 16 has no `naturals`: its endless
  generator is `ticket_numbers`.
- Lunch 16 says *any* generator that is only a loop around one `yield`
  becomes a `from_fn` closure. `collections.md` adds a condition: "after
  names bound to plain values". The pages promise the same order of work
  either way.
- Of the generator-is-used-up error, the pages and examples show only a
  method call (`.to_list`, `.sum`) as the first use. A `for`, `puts g`, and
  a hand-over to a parameter are named as uses in `collections.md`, but no
  example shows what the message says for them.

## Promises in a page that no example shows

- `http.value` giving `None` for a missing name, or for an empty list
  (only the package's own `test` blocks show it); a name with no `=` in a
  query read through `req.params` (`http.md`: "a name with no `=` has the
  value `""`").
- A second `respond` being an `Error` (`… has been answered already`); a
  status outside 100–599 refused, with the request still open to answer
  (`http.md`).
- `accept` giving an `Error` once the server is closed (`http.md`).
- A `() or Error` call ending an `if`/`else` branch at the end of a
  `() or Error` function being its value. `notes/main.lume` shows the
  `match` arm case only, and no page states the rule; only `FINDINGS.md`
  does.
- `take_while` on a plain (not async) generator whose body does visible
  work; how many items it asks for (`collections.md`).
- A chain that starts from a list, given to a generator, being worked out
  at the call, while one from a range stays lazy (`collections.md`,
  "Taking a sequence"). `examples/generators.lume` shows only the lazy
  `naturals().map` side, and nothing in it prints.
- A generator's set-up that is a call, put off until the first item.
  `generators_order` has a `puts` in its set-up, not a call to a function.
- `extend [T: I]` on a list of lists (`[[T]]` through the same `extend`).
- A list given to a generator being its own copy, so a later `push` does
  not show (`collections.md`: "a list ... is copied in").

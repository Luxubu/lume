# Reviewer B — the edges

## What I read

- The brief, `corpus/m126/BRIEF.md`.
- `docs/reference/http.md` (twice: it changed on disk while I worked; the
  second reading adds the status message `600 is not an HTTP status: one
  from 100 to 599` and "a refused answer sends nothing"), `docs/stability.md`.
- `docs/reference/errors.md` whole, `docs/reference/patterns.md` whole,
  `docs/reference/printing.md` whole, `docs/reference/concurrency.md`
  (all but the `shared var` sections), `docs/reference/methods.md` (the
  opening, `T?`, `T or Error`, tuples; and its new paragraph on a method
  neither an optional nor its value has), the async-generator part of
  `docs/reference/collections.md` and its chains-over-lines part, the
  start of `docs/reference/strings.md`, `docs/install.md` (the commands),
  the `lume fmt` lines in `modules.md`, `control-flow.md`, `packages.md`.
- `docs/tutorial/21-http.md` (old and new), `docs/tutorial/10-absence-and-failure.md`
  §10.6, `docs/tutorial/http-requests/`.
- `packages/http/lib.lume`, `lume.toml`, `test.expected` — the package's
  own source, which settled most of the exact messages.
- `examples/http_server/`, `examples/http_demo/`, `examples/healthcheck/`,
  `examples/nested_list_patterns.lume`, `examples/unused_names.lume`,
  `examples/fmt/*` (all seven), and in `examples/errors/`:
  `optional_method_misspelt`, `option_method`, `result_method`,
  `result_method_like_rust`, `method_on_result`, `dropped_failure`,
  `dropped_failure_unit`, `dropped_result`, `alternative_types_differ`,
  `alternative_catch_all`, `alternative_names_in_list`, `or_pattern_bindings`,
  `missing_error_arm`, `typo`, `nonexhaustive_nested`,
  `nonexhaustive_nested_deep`, `nonexhaustive_tuple`, `list_pattern_on_str`,
  `async_gen_outside_async`, `async_gen_method_outside_async`,
  `pattern_tuple_short`, `partial_arm`; the `exit: 1` lines of
  `examples/csvq/queries.expected`.

Nothing was compiled or run. Every expected output was worked out by hand.

## The programs

| id | probes | meant to | how checked |
|---|---|---|---|
| `b01_decode_edges/` | `decode` of a lone `%`, `50%`, `%4`, `%zz`, `a%4g1`, `%FF`, half a character, lowercase hex, `+`; then `!` on a failed decode | STOP (101) | `lume run` |
| `b02_parse_form_edges/` | `parse_form` of `""`, `a&b=`, `&&`, `+`, `%2B`, `=` in a value, an empty name, a name twice, a bad `%` (named by the piece), bad UTF-8 in a name; `form` read back | RUN | `lume run` |
| `b03_query_on_query/` | `.with_query` after a query, after a bare `?`, after a `#fragment`, with no pairs, twice; the `Error` names the whole URL | RUN | `lume run` |
| `b04_form_after_body/` | `.with_form` after `.with_body`: which headers leave; form after file, file after form, no pairs, timeout kept | RUN | `lume run` |
| `b05_reply_type/` | `Reply.with_body` changing the type; `.header("content-type", …)` as one more header; served and read back | RUN | `lume run` |
| `b06_accept_after_close/` | an `accept` waiting when `close` comes, one after; `requests` after; `close` twice | RUN | `lume run` |
| `b07_listen_taken/` | `listen` on its own server's address; four that do not parse | RUN | `lume run` |
| `b08_incoming_tasks/` | `incoming` in async code, each request answered in its own task, a `for` over the tasks | RUN | `lume run` |
| `b09_header_case/` | headers looked up in another case, on server and client | RUN | `lume run` |
| `b10_nested_list_pattern.lume` | a list pattern inside two variants, under `\|`, both naming `d` | FAIL | `lume check` |
| `b11_alternatives_shape.lume` | alternatives with a `Str` literal in different places, both naming `size` | FAIL | `lume check` |
| `b12_unit_result_match.lume` | a `() or Error` function ending in a `match` whose last arms are `Error(...)`; `?` in `main` | RUN, exit 1 | `lume run` |
| `b13_stray_error.lume` | a stray `Error(...)`, the last line written, inside an `if` in a closing `for` | FAIL | `lume check` |
| `b14_optional_misspelt.lume` | `seat.some` for `seat.some?` on a `Str?` | FAIL | `lume check` |
| `b15_fmt_helpers/` | `lume fmt --stdout` on badly spaced code using `form`, `parse_form`, `with_query`, `with_form`, `decode`, `encode` | fmt | `lume fmt --stdout` |

Nine programs use the `http` package and are folders; the other five are
single files. Four are meant to FAIL; with the `fmt` one, five check
something other than a run. The FAIL headers give the error alone; I do
not give `lume check`'s exit code, which no page states.

## Guesses

1. **No warnings from compiling a program that uses `http`.** Files:
   b01–b09. Looked: `examples/http_server/check.sh` filters `^warning`
   lines out of `lume run`'s output, which hints that some are printed;
   `docs/tutorial/http-requests/main.expected` shows none. Neither says.
2. **A program in a package folder is named in messages as
   `corpus/m126/b01_decode_edges/main.lume`**, so the `!` warning and the
   stop say `--> corpus/m126/b01_decode_edges/main.lume:47:24` and `:47`.
   Files: b01. Looked: the brief (its example is a single file),
   `docs/reference/packages.md` ("naming a file still works in a package").
3. **`.or([])` takes its list type from the result it is called on.**
   Files: b02. Looked: `methods.md` (`or` takes a `T`), and
   `examples/errors/untyped_empty` only by name.
4. **tiny_http sends a `Reply`'s `Content-Type` as given and adds none of
   its own; ureq hands it back as `content-type`.** Files: b05
   (`application/json`, `text/html`), b09 (`text/plain; charset=utf-8`).
   Looked: `http.md` (the `Reply` table), `packages/http/lib.lume`
   (`respond` uses `Response::from_data` and `add_header`); the crate itself
   I could not read.
5. **An `accept` waiting in a task when `close` comes gives an `Error`
   rather than waiting on**, through tiny_http's `unblock`. Files: b06.
   Looked: `http.md` ("an `accept` waiting then gives an `Error`"),
   `lib.lume` `close`. Which message (`the server stopped: …` or
   `the server is closed`) depends on timing, so only `error?` is printed.
6. **`spawn: server.accept` is accepted**: a method on a copied `Server`,
   no `await`, so it runs on the blocking pool. Files: b06. Looked:
   `concurrency.md` ("A task that waits without `await`", "What a task
   sees: copies").
7. **`listen` refuses each of `not an address`, `127.0.0.1` (no port),
   `127.0.0.1:99999` and `127.0.0.1:x`** rather than resolving one of them
   somehow. Files: b07. Looked: `http.md` ("one that does not parse"),
   the package test (`listen("not an address").error?`).
8. **The program ends**: `incoming` runs one `accept` ahead (the
   collections page: "never more than two items ahead"), so after
   `take(3)` a fourth `accept` is left waiting; `server.close` frees it, and
   nothing hangs at the end. Files: b08. Looked: `collections.md`,
   `lib.lume` `incoming`, `concurrency.md` (an unawaited task is cut off).
9. **A one-line `match` arm may be a method call statement**,
   `Ok(r) -> seen.push(...)`. Files: b08. Looked: `patterns.md` ("a value,
   a `puts`, an assignment, `next`, `break`, `return` or `()`" — a call
   that gives `()` is a value, I take it).
10. **ureq sends `X-Dish` and `x-table` as written and in that order, and
    neither side adds any other `x-` header.** Files: b09. Looked:
    `http.md` ("names in lower case, in the order sent"), `lib.lume`.
11. **b10's error points at the first alternative, at the start of the
    arm**, as `examples/errors/alternative_names_in_list` does — there both
    alternatives name inside a list, as here. Files: b10.
12. **b11's error points at the second alternative**, as
    `or_pattern_bindings` and `alternative_types_differ` do. Files: b11.
13. **b11's help line is `give each alternative an arm of its own`.**
    `patterns.md` gives only the first line and says "Split such an arm in
    two"; no example in `examples/errors/` has this error. Files: b11.
14. **A `match` whose arms are a `puts` and `Error(...)` is the value of a
    `() or Error` function.** Files: b12. Looked: `errors.md` ("the last
    expression of a branch is fine"; "a `() or Error` function that reaches
    its end … has succeeded").
15. **An `Error(...)` that is the last line written, but inside an `if` in
    a `for` that closes the function, is refused** — it is not the
    function's value. Files: b13. Looked: `errors.md` ("other than the
    last"), `examples/errors/dropped_failure_unit`.
16. **The article is `a` before `` `Str?` ``**: "a `Str?` has no method".
    Files: b14. Looked: `option_method` ("a `User?`") and
    `optional_method_misspelt` ("an `Int?`") — the article follows the
    sound, I take it.
17. **`lume fmt` lines up the `->` of one-line arms, padding each pattern
    to the longest one-line, unguarded pattern plus one**: `Ok(s)    ->`
    beside `Error(e) ->`. Files: b15. Looked: `examples/fmt/generic.expected`
    (`Leaf          -> 0`) and `messy.expected` (`Draft   ->`, where the
    guarded `Done(at) if …` is left out and the block arm too). No page
    states it.
18. **`lume fmt` does not wrap a long line** (the `req = …` line, 80
    characters). Files: b15. Looked: `examples/fmt/*` — none has a long
    line.
19. **`lume check` prints a refused program's error in the form the
    `examples/errors/*.expected` files show**, with the file as named on
    the command line. Files: b10, b11, b13, b14. Looked: those files;
    `docs/stability.md` ("every refused program in `examples/errors/` is
    run on every change").

**Total: 19 guesses.**

Not counted, as a page or the package's source settles them: every
`decode` and `parse_form` message (the source writes them); `parse_form`
naming only the piece that failed (`` `%G0` in `%G0` ``, not the whole
form); lowercase hex being read; `with_query` after a bare `?` giving `?&q=x`
and after a fragment giving `#top?q=x`; every header list in b04 and b05;
`accept` after `close` saying `the server is closed`; the order of results
from a list of tasks; `error: <message>` and exit 1 from a failing `main`
(`tutorial/10`).

## Where pages disagree, or say less than the code

- **`.with_query` and a `#fragment`.** `http.md` says the pairs are "added
  to the URL's query, after a `?` or the query it has"; the package only
  looks for a `?`, so `…/find#top` becomes `…/find#top?q=x` — the query
  inside the fragment, never sent. And `…/find?` becomes `…/find?&q=x`. No
  page or example shows either (b03).
- **What a `T?` says about a method it lacks.** `methods.md` "Asking the
  compiler" says a `T?` "says to unwrap it first"; the new paragraph under
  `T?` and `examples/errors/optional_method_misspelt` say a name neither
  the optional nor its value has is named as missing, with a suggestion.
  The opening should say both (b14).
- **`decode`'s UTF-8 message names the whole text**, not the bad bytes:
  `http.md` shows only ``"`%FF` does not decode to UTF-8 text"``, where the
  two are the same; the source writes `` `caf%C3` `` for `caf%C3` (b01).
- **`.header(name, value)` is "one more header"** in `http.md`, but the
  package hands each to ureq's `set`, which I believe replaces an earlier
  header of the same name (for names not starting `x-`). Not probed: I
  could not check it.
- **The tutorial's new table** says `.with_query` adds "after a `?`"; the
  reference adds "or the query it has". Not wrong, but shorter.

## Promises no example shows

- `http.decode`'s error for bytes that are not UTF-8, beyond `%FF`.
- `http.parse_form`'s errors at all, and that `""` and `&&` give `[]`
  (only the package's own tests).
- `.with_query` on a URL that already has a query (only the package test),
  and `.with_form` after `.with_body` (only the package test).
- `Reply.with_body` taking the place of `http.reply`'s `Content-Type`, as a
  client sees it.
- `accept` giving an `Error` once the server is closed, and an `accept`
  waiting when it closes.
- `http.listen` giving an `Error` for an address already taken.
- Walking `incoming` and giving each request to a task with
  `spawn: answer(req)` — `http.md` recommends it; `examples/http_server/`
  uses `requests`.
- `Incoming.header(name)` "found in any case", and `Incoming.headers` in
  lower case.
- A second `respond` being an `Error` (``GET /menu has been answered
  already``), a status outside 100–599 (``600 is not an HTTP status: one
  from 100 to 599``), and "a refused answer sends nothing, so the request
  can still be answered".
- "A request never answered keeps its connection open until the program
  ends."
- `server.address` keeping the host as it was given (`localhost:0` would
  stay `localhost:<port>`).
- The `patterns.md` refusal ``alternatives with a string or float inside
  must otherwise look the same`` — only its first line is given, and no
  file in `examples/errors/` has it.
- How `lume fmt` lays out `match` arms: no page states the alignment.
- The exit code of `lume check` on a refused program.

# Reviewer A — everyday code

## What I read

- `corpus/m126/BRIEF.md`
- `docs/reference/http.md`, `docs/stability.md`, `docs/install.md` (the commands)
- `docs/tutorial/README.md`, lunches 01 (messages), 12, 13, 14, 18 and 21
- `docs/reference/testing.md`, `concurrency.md`, `errors.md` (`!`, stops, other methods),
  `methods.md` (asking the compiler, `T?`, `T or Error`), `printing.md`, `strings.md` (start), `json.md`
- `packages/http/lib.lume`, `packages/json/lib.lume`
- `examples/http_server/` (all four files), `examples/http_demo/main.lume`,
  `examples/healthcheck/check.sh`, `docs/tutorial/http-requests/`, `examples/packages/`
  (tally, labels, shelf, `new.expected`), `examples/unused_names.*`
- `examples/errors/`: `tasks_walked_then_used`, `tasks_used_after_await`, `option_method`,
  `optional_method_misspelt`, `result_method_like_rust`, `method_on_result`, `result_method`,
  `map_error_on_option`, `tuple_method`, `async_chain_self` (for a two-digit gutter)

## Programs

| id | file | probes | meant to |
|---|---|---|---|
| a01 | `a01_json_api/` | JSON API on 127.0.0.1:0 with its own client task: `reply_json`, `post_json`, `req.json` + `need_…`, a 400, a 405 | RUN |
| a02 | `a02_search_page/` | `req.params` and `req.query`: `.with_query` with `&` and accents, a name twice, a query added to one, `%25`, a bare name, a bad `%`, none | RUN |
| a03 | `a03_signup_form/` | `.with_form` read with `req.form`: accents, `&`, `=`, `%`, an empty value; a form replacing `.with_body`; a query beside a form; a non-UTF-8 body | RUN |
| a04 | `a04_url_text/` | `http.encode`, `decode`, `form`, `parse_form` on awkward text, and their errors | RUN |
| a05 | `a05_tipjar/` | a library package (`lib.lume` + a private module) with tests, `lume test` on it | RUN (`lume test`) |
| a06 | `a06_tasks_for.lume` | tasks in a list awaited with a `for`: `map`-made, `var`+`push`, programs run, `await t?` | RUN |
| a07 | `a07_answered_twice/` | a second `respond` is an `Error`; statuses 600 and 99 refused and the request still answerable; a 418 | RUN |
| a08 | `a08_unwrap_or.lume` | Rust's `unwrap_or` on an `Int?` | FAIL |
| a09 | `a09_tasks_after_for.lume` | a `var` list of tasks used after the `for` that walked it | FAIL |

## Where pages disagree, or promise what no example shows

1. `docs/tutorial/README.md`: the Part 2 and Part 3 tables still number the lunches 13–24, though
   their files (and Part 1's table) make them 15–26 since lunch 12 was split in three.
2. `docs/tutorial/21-http.md`, "Underneath": "runs it in a task of its own (lunch 16)" — tasks are lunch 18 now.
3. `docs/stability.md` lists the URL and form helpers (`encode`, `decode`, `form`, `parse_form`,
   `.with_form`, `.with_query`) as described in "HTTP, lunch 21"; lunch 21 never mentions any of them
   (nor serving). Only `docs/reference/http.md` does.
4. `docs/reference/methods.md` says a `T?` or `T or Error` "says to unwrap it first"; no page says what
   one says for a method neither it nor its value has, nor that Rust's names are mapped
   (`is_err` → `error?`). Only `examples/errors/optional_method_misspelt` and `result_method_like_rust` show it.
5. `docs/reference/concurrency.md` says Lume says so when a walked list of tasks is used after the loop,
   but gives no message; only `examples/errors/tasks_walked_then_used` does.
6. `docs/reference/http.md` says a status outside 100 to 599 is an `Error`, but not its words, nor that the
   request is still unanswered afterwards (the package checks the status before it takes the request,
   so a later `respond` works — a07 leans on that, read from `packages/http/lib.lume`).
7. `docs/reference/http.md` shows the decode error for `100%`, but not which text it quotes: the package
   quotes the `%` and up to two bytes after it, and in `parse_form`/`params`/`form` it quotes the one
   name or value being decoded, not the whole form (a02, a03, a04 lean on the package for this).
8. `examples/http_server/check.sh` filters warning lines out of `lume run`, while `check.expected` shows
   none and no page says compiling the `http` package warns. Either the filter is only defensive or a
   warning is expected; no page says which (see guess 1).

## Guesses

1. **No warnings on standard error when the `http` and `json` packages are compiled.** Touches a01, a02,
   a03, a04, a07 (no `EXPECTED STDERR` block given). Looked: `examples/http_server/check.sh` and
   `check.expected`, `docs/tutorial/http-requests/main.expected`, `docs/install.md` (warnings shown when
   compiled), `examples/unused_names.expected`.
2. **The `Content-Type` of a `reply_json` reaches the client as `application/json`**, the first (or only)
   `content-type` header — tiny_http adds no default of its own in front. Touches a01. Looked:
   `packages/http/lib.lume` (`reply_json`, `respond`), `docs/reference/http.md`; `examples/http_server`
   reads only `location`.
3. **ureq sends a hand-written query as written**: `%zz` (not a valid escape), `%25`, a bare `flag`, and the
   `+`, `%26` and `%C3%A8` that `.with_query` writes, are neither refused nor re-encoded, so `req.query`
   shows them unchanged. Touches a02. Looked: `docs/reference/http.md` ("as it was sent"),
   `packages/http/lib.lume` (`with_query`, `accept`), `examples/http_server/` (only `q=s`).
4. **ureq sends the one `Content-Type` the request set** (no second one, no charset added), so
   `req.header("Content-Type")` is `application/x-www-form-urlencoded`. Touches a03. Looked:
   `packages/http/lib.lume` (`typed`, `with_form`, `open`), `docs/reference/http.md`.
5. **`unwrap_or` on an `Int?` says "an `Int?` has no method `unwrap_or`" with help "did you mean `or`?"**,
   caret on the `.`. Touches a08. Looked: `examples/errors/optional_method_misspelt`,
   `result_method_like_rust`, `option_method`, `docs/reference/methods.md`, `errors.md`.
6. **`lume check` prints the error and nothing after it** (no count or summary line). Touches a08, a09.
   Looked: `examples/errors/*.expected`, `docs/tutorial/01-hello.md` 1.5.
7. **A `var` list built with `push` and walked by `for` gets the same error as the `map`-made list of the
   example, and a use inside interpolation (`"#{jobs.len} …"`) counts as a use.** Touches a09. Looked:
   `examples/errors/tasks_walked_then_used`, `docs/reference/concurrency.md`.
8. **A `for` over a `var` list of tasks walks and awaits them as over a plain one.** Touches a06 (`counts`).
   Looked: `docs/reference/concurrency.md` (the example's list is not `var`), lunch 18.

Total: 8 guesses.

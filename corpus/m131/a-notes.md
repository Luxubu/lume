# Reviewer A, round twenty-six: everyday code

## What I read

- The brief, `corpus/m131/BRIEF.md`.
- Tutorial: lunch 27 (`27-a-small-web-service.md`) and its
  `orders-service/` (`main.lume`, `lume.toml`, `main.expected`,
  `test.expected`); lunch 16 (`16-generators.md`); lunch 21 (`21-http.md`).
- Reference: `http.md`, `json.md`, `collections.md` (from "Types of your
  own: `items`" to the end), `generics.md` (bounds, generic `extend`, the
  overlap rule), `errors.md` (all), `testing.md` (all), `printing.md` (all),
  `concurrency.md` (spawn bodies, copies, failures in tasks, `shared var`,
  where the warnings go), `structs-and-enums.md` (type functions, by grep),
  `patterns.md` (list patterns, by grep), `stability.md`, `install.md` (the
  commands).
- Packages: `packages/http/lib.lume` and `lume.toml`, `packages/json/lib.lume`.
- Examples: `http_server/` (all), `notes/` (all, `FINDINGS.md` included),
  `generators_order.lume` and `.expected`, `async.expected`,
  `match/main.expected`, `port/*.expected` (the warning lines),
  `json_config/main.lume` (by grep, for `Type.from_json`), and from
  `examples/errors/`: `extend_bound_unmet`, `extend_generic_missing`,
  `extend_bound_on_type`, `dropped_failure`, `dropped_failure_unit`,
  `dropped_result`; a survey of every `warning:` line in `examples/` and
  `docs/`.

## Programs

| id | file | probes | meant to |
|---|---|---|---|
| a01 | `a01_pantry/` (`main.lume`, `lume.toml`) | lunch 27's shape on a pantry of my own: route enum with a literal after a name in a list pattern, `shared var Pantry` with two `var self` methods (one giving `Int?`), `incoming` with a task per request, its own client on port 0, a form, a 400 and two 404s; the `shared var` warning | RUN (warning on stderr) |
| a02 | `a02_pantry_tests.lume` | a01's routes and store under `lume test`, with no network; the last test fails on purpose, to show `left:`/`right:` and the line | RUN under `lume test`, exit 1 |
| a03 | `a03_query/` | `req.params` and `http.value`: no query, one name, two, a name twice (first wins), a non-ASCII value, and a query that does not decode (400 with the `parse_form` message); `req.query` as sent | RUN |
| a04 | `a04_json_body/` | `req.json` and `need_str` / `need_int` / `opt_str` through `def self.from_json`; eleven bodies, nine of them bad in different ways, each a 400 with the error's message; `return Error(..) if` after decoding | RUN |
| a05 | `a05_pages.lume` | a loop-around-one-`yield` generator that prints before and after its `yield`, used with `take`, `filter`+`take`, `next` then `for`, and `next` on an empty one | RUN |
| a06 | `a06_till.lume` | a generator whose set-up prints: made but not walked, `take`, `next` twice then `for` with `break`, `first`, `for` with `break` at once, and `_ =` on one never walked | RUN |
| a07 | `a07_unpriced.lume` | `extend [T: Priced] with Billed` on `[Str]`, where only `Int` is `Priced` | FAIL |
| a08 | `a08_steps.lume` | a `() or Error` function ending in a `match` whose three arms each call a `() or Error` function; `main` matches each result, then `?` on a failing one ends the program with exit 1 | RUN, exit 1 |
| a09 | `a09_kept_midway.lume` | the same arm call when the `match` is *not* the function's end: a dropped `() or Error` | FAIL |

## Guesses

1. **Where a package's warning says it is.** For `lume run` of a package
   folder from the repository root I wrote `--> corpus/m131/a01_pantry/main.lume:89:3`.
   No page shows a warning from a package with a `lume.toml` run from
   outside it: `concurrency.md` "Where the warnings go" shows a bare
   `main.lume:2:3`, `examples/match/main.expected` shows
   `examples/match/main.lume:67:3` but `match/` has no `lume.toml`, and
   `http_server/check.sh` and `notes/check.sh` filter the warnings out.
   Touches a01 (and the brief's rule for a03/a04, which have none).
2. **Nothing else on standard error for a01.** That a path dependency
   prints no "fetching"/"compiling" line, and that the `shared var` warning
   is the only one. `install.md` says warnings show "when a program is
   compiled … not again each time an unchanged program runs", so the block
   holds for a first run only. Looked in `install.md`, `packages.md` (by
   grep), `21-http.md` 21.1. Touches a01, a03, a04.
3. **A malformed query reaches the server unchanged.** a03 asks for
   `/find?q=100%` with `http.get`; I assumed ureq sends `100%` as it is (no
   `%25`) and does not refuse the URL, so `req.query` is `q=100%` and
   `req.params` is the documented `parse_form` error naming the value
   (`` `%` in `100%` is not … ``). `http.md` documents the decode error and
   "as it was sent", but no page says what the client does with a `%` that
   is not an escape. Looked in `http.md` (Requests, URLs and forms,
   Serving), `packages/http/lib.lume` (`with_query`, `open`, `accept`).
   Touches a03 (lines 6 and 12 of its output).
4. **Named arguments are worked out left to right.** a04's body 10,
   `{"table": "x"}`, has no `dish` and a bad `table`; I wrote
   `` `dish` is missing `` because `dish:` comes first in
   `Order(dish: j.need_str("dish")?, table: j.need_int("table")?, …)`. No
   page says in which order a call's arguments, or a struct literal's
   fields, are worked out when two of them can `?` out. Looked in
   `json.md` (Decoding), `errors.md` (`?`), `structs-and-enums.md` (by
   grep). Touches a04.
5. **`_ = generator()` compiles, with no warning, and runs nothing.**
   `errors.md` says `_ = ...` takes any value, and `collections.md` says
   nothing in a generator runs until an item is asked for; but no page
   shows a generator dropped unwalked, and a lint for that would be
   natural. Touches a06 (the last line, `nothing opened`, with no
   `(opening the till at 50)` before it).
6. **The wording of a07's error.** No page in `docs/` shows the error for a
   method of an `extend [T: I]` used on a list of items that are not `I`:
   `generics.md` shows bounds on functions and the overlap errors only.
   I took the form, caret on the `.`, and help line from
   `examples/errors/extend_bound_unmet.expected`, and `notes/FINDINGS.md`
   says the check holds "where the items are built-in values and the bound
   is an interface this file knows", which `Str` and `Priced` are. I also
   guessed that a bound naming one interface (`Priced`) on an `extend` of
   another (`Billed`) is allowed — every example's bound is the interface
   it extends, or `Ordered`. Touches a07.
7. **a09's error is the "nothing here looks at whether it did" one, on the
   call inside the arm.** `errors.md` and `examples/errors/dropped_result`
   show the message for a call on its own line; I assumed a call as an arm's
   value, in a `match` that is a statement followed by another line, is
   refused the same way, pointed at the call (22:10), and that the arms
   being of different kinds (`puts` and a `() or Error` call) is not an
   error of its own first. No page shows a `match` used as a statement with
   a failing arm. Looked in `errors.md`,
   `notes/FINDINGS.md` ("Fixed on the way"), `examples/errors/dropped_*`.
   Touches a09.

Places where the pages could say more (not guesses of mine, but where I
went beyond the reference to the examples):

- The `extend [T: I]` error of milestone 130 is in `examples/errors/` and
  `notes/FINDINGS.md` only; `generics.md`'s "Generic `extend`" does not show
  it, and `FINDINGS.md` says the check is left to Rust for a type of a
  module's own — no page says what a user then sees.
- The m130 rule for a `() or Error` call at the end of an arm that ends a
  `() or Error` function is in `notes/FINDINGS.md`; `errors.md` says it of
  `Error(...)` ("the last expression of a branch") but not of a call.
- `http.md` says `json` comes with `.json` on `Incoming`, and lunch 27
  27.4 says "`req.json` reads a body sent as JSON", but neither shows the
  error it gives; I read it from `packages/http/lib.lume`
  (`the body is not JSON: …`).

**Total: 7 guesses.**

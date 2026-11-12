# Reviewer C — round twenty-six: whole programs

## What I read

- The brief, `corpus/m131/BRIEF.md`.
- Tutorial: lunch 27 (`docs/tutorial/27-a-small-web-service.md`) and its
  program `docs/tutorial/orders-service/main.lume`; lunch 21 (HTTP); lunch 16
  (generators).
- Reference: `http.md`, `json.md`, `collections.md`, `concurrency.md`,
  `testing.md`, `packages.md`, `modules.md`, `names.md`, `errors.md`,
  `generics.md`, `methods.md`, `printing.md`, all whole; `interfaces.md`
  (to "Methods that take a block"), `structs-and-enums.md` (to "`==`,
  printing and hashing"), `control-flow.md` (to "`next` and `break`"),
  `functions-and-blocks.md` (to "Blocks of your own", and "What fits inside
  `{ }`"), `io.md` (`File`, `Dir`, `Path`, `Env`), `strings.md` (ordering to
  the end), `numbers.md` (arithmetic, how a `Float` prints), `patterns.md`
  (lists, what an arm can do).
- `docs/stability.md`; `docs/install.md` (the commands, editors).
- Examples: `examples/notes/` (every file), `examples/http_server/` (every
  file), `examples/generators_order.lume` and its `.expected`,
  `examples/generic_extend.lume`, `examples/site/main.lume`,
  `examples/packages/` (every file); a grep that showed the lines of
  `examples/errors/extend_bound_unmet.*`.
- Packages: `packages/http/lib.lume` and `lume.toml`, `packages/json/lib.lume`
  (all but the parser's middle).
- One slip to report: a plain `ls corpus/m131/` showed me the names of other
  reviewers' files (not their contents); I opened none of them.

Nothing was compiled or run. Every expected output is worked out by hand.

## My programs

| id | what it is | form | checked by | stderr |
|---|---|---|---|---|
| `c01_library/` | lunch 27's shape for a library: books, loans, returns; form and query read with `http.value`; 201/400/404/409; `null` in JSON; tests of the routes and the desk | package, `http` + `json`, 179 lines | `lume run corpus/m131/c01_library/main.lume` | one `shared var` warning |
| `c02_feed/` | an event feed a generator makes, served a page at a time with `?after=` and `?limit=`, the client following `next`; the server prints as each event is made | package, `http` + `json`, 146 lines | `lume run corpus/m131/c02_feed/main.lume` | none |
| `c03_stock/` | a stock service saving every change to `/tmp/lume-review-c/stock/stock.json`; the file shown and opened again after the server stops; `settle` ends in a `match` whose arms are `() or Error` calls | package, `http` + `json`, 180 lines | `lume run corpus/m131/c03_stock/main.lume` | one `shared var` warning |
| `c04_site.lume` | a static site from a pipeline of four generators, each a loop around one `yield`, written to `/tmp/lume-review-c/site`; index newest first, a page per tag, one page read back | one file, no HTTP, 134 lines | `lume run corpus/m131/c04_site.lume` | none |
| `c05_limits/` | a rate-limited proxy: front server with a `shared var` limiter (3 a round per `X-Client`, `429` + `Retry-After`), an upstream server of its own, `502` from `127.0.0.1:9`; bursts sent at once, reported as counts | package, `http` only, 151 lines | `lume run corpus/m131/c05_limits/main.lume` | one `shared var` warning |
| `c06_pantry/` | a library package: `qty.lume`, `recipe.lume`, `lib.lume`; `pub import`, `extend [T: Priced] with Totalled`, `extend Part with json.ToJson`, a generator, tests in every module | library package, `json`, 180 lines over three files | `lume test corpus/m131/c06_pantry/lib.lume` | none |

All six are meant to RUN and exit 0. None prints the port, and none prints
anything whose order depends on timing: c01, c03 and c05 collect the
client's lines and print them after the servers stop; c02 answers one
request at a time with `server.requests` and prints only in `main`'s
thread; c05's bursts are reported as how many were answered and how many
told to wait, which the lock decides and the timing does not.

### Where pages disagree, or promise what no example shows

- `json.md` says the `ToJson` types are `Str`, `Int`, `Float`, `Bool` and
  lists; it does not say that a `Json` is one. `packages/json/lib.lume` has
  `extend Json with ToJson`, and `examples/notes/FINDINGS.md` says so
  (milestone 129). c01 and c02 rely on `[Json].to_json`, from the source,
  not from the page.
- `stability.md` begins "Lume is at 1.1" and in the same page says serving
  is settled in 1.2; `packages/http/lume.toml` says `version = "1.1.0"`.
- `testing.md` says modules' tests run before the entry file's, but not in
  what order the modules come; only `examples/notes/check.expected` shows
  one (note, store, routes).
- `http.md`: "A refused answer sends nothing, so the request can still be
  answered" — no example answers a request after a refused `respond`.
- No page shows a warning from a package program run by naming its
  `main.lume` from another folder, so the `-->` path is a guess (1).

## Guesses

1. **The `-->` path of a warning in a package program** run from the
   repository root as `lume run corpus/m131/<pkg>/main.lume` is
   `corpus/m131/<pkg>/main.lume:L:3`. Files: c01, c03, c05. Looked:
   `concurrency.md` "Where the warnings go", `install.md` (`file` "as given,
   or found from it"), `packages.md` "Naming a file still works", the brief.
2. **The gutter for a three-digit line number** is four spaces and `|`
   (`    |`, `128 |`). Files: c01, c03. Looked: `errors.md` ("the gutter is
   as wide as the line number", two digits shown), `interfaces.md` (the
   two-digit warning), `packages.md` (`16 | import lib`).
3. **The `http` and `json` packages' own files give no warning** when built
   as dependencies, so each program's stderr is only its own warnings.
   Files: c01, c02, c03, c05, c06. Looked: `packages/http/lib.lume`,
   `packages/json/lib.lume` (no `shared var`, no `!` outside tests),
   `examples/http_server/check.sh` (filters warning lines anyway).
4. **Module tests run in import-dependency order** (`qty` before `recipe`),
   then `lib.lume`'s own, unprefixed. File: c06. Looked: `testing.md` "Tests
   in a package", `examples/notes/check.expected`,
   `examples/packages/tally/test.expected`.
5. **Bare variants as the value of a `Moved or Error` method are wrapped in
   `Ok`**, their enum taken from the declared result: `None -> Unknown`,
   `return Short(was.qty) if …`, `Now(goods.at(pos))`. File: c03. Looked:
   `errors.md` "A bare value is the implied `Ok`", lunch 27's `route` (bare
   variants from a plain result), `examples/notes/store.lume` (writes
   `Ok(None)`/`Ok(Some(n))` for an optional).
6. **A `()` arm beside arms that are `() or Error` calls**, as the `match`
   that ends a `() or Error` function (`settle`), is accepted as success.
   File: c03. Looked: `errors.md` ("nothing needs writing at the end"),
   `examples/notes/FINDINGS.md` (the milestone 129 fix), the brief.
7. **A guard on a tuple pattern that holds a list pattern**,
   `("POST", ["items", sku, verb]) if verb == "take" or verb == "add" ->`.
   File: c03. Looked: `patterns.md` (guards shown on plain names; lists and
   tuples shown without guards).
8. **A `shared var` parameter handed on to another function's `shared var`
   parameter** (`answer` calls `shift(stock, …)`) works on the same value.
   File: c03. Looked: `concurrency.md` "Handing a `shared var` to a helper"
   (shown only from a local).
9. **A block in the condition of a trailing `if` on `return`** parses:
   `return NoBook if not books.any? { |b| b.id == id }`. Files: c01, c03.
   Looked: `control-flow.md` "Trailing `if`", `examples/notes/` (trailing
   `if` on `return` without a block).
10. **`spawn:` inside a block inside an `async def` that is not `main`**:
    `(1..n).map { |_| spawn: await status_of(base, who) }.to_list` in
    `burst`. File: c05. Looked: `concurrency.md` (the form in `async def
    main`), lunch 21.5.
11. **Two servers in one program** are within the brief ("its own server"),
    and closing each from `main` ends its `incoming` loop running in a task.
    File: c05. Looked: the brief, `http.md` "Serving" (`close`, `accept`),
    `packages/http/lib.lume` (`incoming`, `close`).
12. **Assigning `{}` to a map field** inside a `var self` method
    (`used = {}`) takes its type from the field. File: c05. Looked:
    `collections.md` "Where `[]` and `{}` need a type" (fields named for
    construction, not for assignment).

Total: **12 guesses.**

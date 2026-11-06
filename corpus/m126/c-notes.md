# Reviewer C — notes, round twenty-five (milestone 126)

Set: whole programs. Six small tools someone would ship with Lume 1.1,
four of them HTTP servers that answer their own client task.

## What I read

- `corpus/m126/BRIEF.md`.
- `docs/reference/`: `http.md`, `json.md`, `collections.md`, `methods.md`,
  `strings.md`, `printing.md`, `errors.md`, `concurrency.md`,
  `control-flow.md`, `patterns.md`, `names.md`, `functions-and-blocks.md`,
  `structs-and-enums.md`, `modules.md`, `packages.md`, `testing.md`, the
  first parts of `io.md` (`File`, `Dir`, `Path`, `Env`) and `numbers.md`
  (literals, arithmetic, converting). I grepped `operators.md` for `!=`
  and `generics.md` for the built-in bounds.
- `docs/stability.md`, `docs/install.md`, `docs/tour.md`.
- `docs/tutorial/`: `README.md`, lunches 12, 13, 14, 21 and 23, and the
  package `docs/tutorial/http-requests/`.
- `packages/http/` (`lib.lume`, `lume.toml`, `test.expected`) and
  `packages/json/` (`lib.lume`, `test.expected`), in full.
- `examples/`: `http_server/`, `http_demo/`, `healthcheck/main.lume`,
  `logq/main.lume`, `chains_over_lines`, `packages/` (tally, shelf),
  `lib/report.test.expected` and its imports, `mini/main.test.expected`
  and its imports, `tests/*.expected`, part of `stdlib.lume`.

Two things to own up to. First, a `cat */lume.toml` I ran in
`corpus/m126/` to check my own files printed the other reviewers'
`lume.toml` files too (package names and dependencies only); I read
nothing else of theirs. Second, some docs changed on disk while I worked
(`git diff` on `docs/` showed it): the tutorial README's numbering, lunch
21's step table, `concurrency.md`, `methods.md` and `http.md`. I wrote
against what I read first, and I mark below what has been fixed since.

## The programs

| id | what it is | how it is run | probes |
|---|---|---|---|
| `c01_kvstore/` | a key-value store over HTTP: GET, PUT, DELETE on `/k/<key>`, and its client | `lume run corpus/m126/c01_kvstore/main.lume` | `http.listen`, `requests` with `break`, `http.get/put/delete/post`, `http.encode` for keys in a path, `http.decode` of them (and of a bad `%ZZ`), a 405 with an `Allow` header, a `var` map handed to handlers |
| `c02_shortener/` | a URL shortener: forms in, encoded codes out, a search answering with a form | `lume run corpus/m126/c02_shortener/main.lume` | `.with_form`, a form body by hand, `req.form` failing on `100%`, `http.encode` into a `Location`, `http.decode` of it, `.with_query` and `req.params`, `http.form` of a map's pairs, `http.parse_form` on the client |
| `c03_todo/` | a form-driven to-do service, with a JSON view | `lume run corpus/m126/c03_todo/main.lume` | `req.form` and an `action` field, `req.params` filtering, `http.reply_json` of `[Todo].to_json`, `Response.json` and `need_str`, `for var` and `reject` in `var self` methods |
| `c04_webhook/` | a webhook receiver that checks headers before parsing JSON bodies | `lume run corpus/m126/c04_webhook/main.lume` | `Incoming.header` in any case, `http.send` with headers of its own, `http.post_json`, `req.json` failing with a position, `need_int`/`need_items`/`opt_str`, a set of delivery ids, a log as `Json.Arr` |
| `c05_logreport.lume` | a static report from an access log | `lume run corpus/m126/c05_logreport.lume` | `group_by` (by `Str` and by `Int`), `sort_by` with a negated number and with a tuple key, `first` on a sort, `max_by` on a map, `uniq`, `take`, `pad`/`pad_right`; a section per task, awaited by a `for` over the list of tasks; written to `/tmp/lume-review-c/`, read back, removed |
| `c06_tab/` | a library package: `lib.lume` passing on modules `money` and `share`, each with tests | `lume test corpus/m126/c06_tab/lib.lume` (exit 1) | `pub import` of two modules, module tests named `money: …`, an `async test` with `spawn:` and no `main`, `!` in tests, one failing comparison with `left`/`right` |

Lengths: 129, 141, 180, 175 and 158 lines, and 150 across the three
files of `c06_tab`. Every program is meant to run; nothing prints the
port, and every server answers one request at a time, so its lines come
in the order its one client sends them. None of them should warn: no
`!` outside tests, no `shared var`, and no name left unread.

## Where the pages disagree, or promise what no example shows

- `docs/tutorial/README.md` numbered parts 2 and 3 as lunches 13–19 and
  20–24, against the files' 15–21 and 22–26 (left over from lunch 12
  being split in three). *Fixed on disk during the round.*
- Lunch 21's table of request steps left out `.with_form` and
  `.with_query`, which `http.md` and `stability.md` call settled in 1.1,
  and its "Underneath" sent tasks to lunch 16 (they are lunch 18).
  *Both fixed on disk during the round.*
- `stability.md` opens with "Lume is at 1.0" while its tables list parts
  "(1.1)" and a preview "since 1.1".
- `http.md` never says whether a request follows a redirect, and serving
  makes it matter: a `Reply` can carry `Location`. (c02, c03 rely on a
  201's `Location` not being followed.)
- `http.md` says `path` is "the path without its query" and `query` is
  "as it was sent", but not whether `path`'s `%XX` escapes are decoded.
  Both server examples use plain paths only.
- The only server example ends its `for` with `.take(5)`; none shows a
  server stopped by a request (`break`), which is how a real one is shut
  down.
- `testing.md` says an `async test` "can start tasks with `spawn:` too,
  whatever kind of `main` the file has", but a library has none; and it
  says modules' tests run before the entry file's without saying in
  what order the modules go.
- No page says whether `lume test` writes its report to standard output
  or standard error.

## Guesses

1. **A path's `%XX` escapes go out as written, even a broken one.** c01
   sends `/k/two%20words`, `/k/caf%C3%A9` and `/k/bad%ZZ`; c02 sends
   `/s/caf%C3%A9%20menu`. I took it that ureq (and the `url` crate under
   it) neither re-encodes nor refuses them, so `%ZZ` reaches the server
   and becomes a 400 from `http.decode`. Looked: `http.md`,
   `packages/http/lib.lume` (`ureq::request(&m, &u)` is given the URL as
   it is); the crate's own rule is nowhere in the repository.
   Files: `c01_kvstore/main.lume`, `c02_shortener/main.lume`.
2. **`req.path` is the path as sent, not decoded**, so the server prints
   `GET /k/caf%C3%A9` and decodes the key itself. Looked: `http.md`
   (`Incoming` table), `lib.lume` (`accept` splits `r.url()` at `?` and
   no more); tiny_http's `url()` is not described. Files: c01, c02.
3. **A 201 with a `Location` header is not followed.** Looked: `http.md`,
   lunch 21, `lib.lume` — no word on redirects. Files: c02 (every
   `/shorten`), c03 (every added to-do).
4. **The `Content-Type` a request gives arrives as given**, with nothing
   added by ureq, so c04's second delivery is refused as
   ``send JSON, not text/plain``. Looked: `http.md` ("a body, with its
   `Content-Type`"), `lib.lume` (`req.set` then `send_string`). File:
   `c04_webhook/main.lume`.
5. **A nested `match` as the last line of an arm's block is that arm's
   value** (the `"GET"` and `"DELETE"` arms of c01's `on_key`, the `Ok`
   arm of c01's `answer` and c02's `resolve`). Looked: `patterns.md`
   ("the last line of each arm is that arm's value"), `control-flow.md`;
   no example nests one that way. Files: c01, c02.
6. **Module tests run in the order `money`, then `share`.** No page says
   the order among modules; `examples/mini` and `examples/lib` are
   consistent with both "in import order" and "dependencies first", and
   I arranged `c06_tab` so both readings agree (`lib.lume` imports
   `money` first, and `share` imports `money`). Files: `c06_tab/*`.
7. **An `async test` may `spawn:` in a library that has no `main` at
   all.** Looked: `testing.md` ("whatever kind of `main` the file has"),
   `examples/tests/async_tests.expected`. File: `c06_tab/lib.lume`.
8. **`lume test` writes its report to standard output**, so it is all in
   `EXPECTED OUTPUT`. Looked: `testing.md`, lunch 14, the brief; the
   `.expected` files record both streams together. File: `c06_tab/lib.lume`.

Checked rather than guessed, for the record: the JSON error position in
c04 (worked through `Parser` in `packages/json/lib.lume` against the
`{"a" 1}` row of `json.md`), every `http.encode`/`http.form` string
(from the `escape` rules in `lib.lume` and `http.md`), the decode error
texts (`unescape` in `lib.lume`), `pad`/`pad_right`, `group_by`'s key
order, `sort_by`'s stability, a map keeping a replaced key's place, and
`lines` dropping a trailing newline (`io.md`).

**Total: 8 guesses.**

# Reviewer A — everyday code

## What I read

- The brief, `corpus/m116/BRIEF.md`.
- `docs/reference/http.md`, `crates.md`, `collections.md` (all), `errors.md`
  (all), `printing.md`, `io.md` (files, dirs, failures), `numbers.md` ("What
  stops the program"), `names.md` (unused names), `strings.md` (Char,
  escapes, raw strings), `methods.md` (list methods, did-you-mean),
  `packages.md` (first half); `docs/install.md` (commands, Editors and the
  `--json` fields), `docs/stability.md`; `docs/tutorial/19-http.md` and
  the package `docs/tutorial/http-requests/` (`lume.toml`, `main.lume`,
  `main.expected` — not its `.lume/` folder).
- `packages/http/` (`lume.toml`, `lib.lume`, `test.expected`).
- `examples/http_demo/`, `examples/healthcheck/` (`main.lume`,
  `check.expected`), `examples/crate.lume`, `crates.lume`,
  `crate_builder.lume` and their `.expected`, `examples/check_json.expected`,
  `examples/async_generators.lume` and `.expected`,
  `examples/chains_over_lines.lume`, `examples/fmt/chain_comments.*`,
  `examples/unused_names.*`, and in `examples/errors/`: `arity`,
  `async_chain_self`, `async_gen_chain`, `crate_no_method`,
  `crate_no_function`, `crate_unsupported`, `dropped_result`, `pub_field`,
  `field_assign_unknown`, `typo`.

Things the docs disagree on, or leave open (not counted as guesses):

- `docs/stability.md` opens with "Lume is at 0.7", while its own table says
  `http`, crates and the `--json` fields are "settled in 0.8".
- `stability.md` says to build a `Request` with `http.request` and its
  steps "not by naming its fields" — but nothing in `http.md` lets a
  program look at a request it built, and lunch 19.6 (and the checked
  `docs/tutorial/http-requests/main.lume`) reads `careful.headers` and
  `careful.timeout_ms`. a01 reads the fields, as the tutorial does; the
  promise does not cover that.
- `http.md` says an `Error`'s message "names the method and the URL", but the
  missing-file `Error` it quotes (``cannot read `/no/such/file`: …``) names
  neither (a02 prints it whole).
- Lunch 19.3's table of steps and the comment on `request` in
  `packages/http/lib.lume` leave out `.with_file`; the lunch's `lume.toml`
  still says `tag = "v0.7.0"`.
- `errors.md` fixes a stop's gutter for line 12 (`12 |` under `   |`) but not
  the `-->` line's indent; see guess 11.

## My programs

| file | probes | meant to |
|---|---|---|
| `a01_requests/` (package) | `http.request` and its steps, each giving a new request; reading `headers`, `timeout_ms`, `method`, `body`; a `Response` made by hand (`ok?`, `header`, `json`); `send` to port 9, `http.get` of a bad URL and `http.send` with headers, each an `Error` naming method and URL | RUN |
| `a02_uploads/` (package) | `.with_file` of a file that is there (port 9 refuses it) and of one that is not (the exact `cannot read` message); `send_parts` of a generator; `send_parts` setting aside a `with_file` body; `await req.fetch` in two `spawn:` tasks | RUN |
| `a03_access_log.lume` | `regex` for an access log: captures, `get(i)`, `replace_all` with `$3/$2/$1`, `find_iter(..).len` | RUN |
| `a04_query_and_hex.lume` | `urlencoding` to build and read back a query string, bad UTF-8 as an `Error`; `hex` for a colour's bytes, `encode_upper`, bytes back to text, the crate's own error text | RUN |
| `a05_capture_group.lume` | an `Int` to a crate's `usize`: `Captures.get(3)` is `None`, `get(-1)` does not fit and stops | STOP |
| `a06_async_laps.lume` | async generator with `join`, `max`, `min` (and of nothing), `min_by`/`max_by` on a tie, `sort`, `sort_by` (stable), `last`, `reverse`; a chain over lines with a comment and a blank line between steps; tasks awaited as a list | RUN |
| `a07_alarm_chain.lume` | a chain over an async sequence, over lines with a comment between steps, whose block uses a field | FAIL |
| `a08_check_json.lume` | `lume check --json` on a program with two `!` warnings, one after an `é` (columns in characters) | RUN (`lume check --json`) |
| `a09_hex_dump.lume` | a crate function that does not exist (`hex.dump`) | FAIL |

## Guesses

1. **A package is run by its `main.lume` from the repository root**, named
   as `corpus/m116/<id>/main.lume`, and `http = { path =
   "../../../packages/http" }` is all its `lume.toml` needs (the `json`
   dependency comes with `http`). Files: a01, a02. Looked: `packages.md`,
   `install.md` ("Inside a package the file can be left out"),
   `docs/tutorial/http-requests/lume.toml`.
2. **A program may make an `http.Response` itself**, as
   `http.Response(status: …, headers: …, body: …)`. Files: a01. Looked:
   `packages/http/lib.lume` (its test does it inside the package),
   `packages.md` ("a package prefix goes wherever a name goes"),
   `examples/errors/pub_field.*` (fields are public with the struct).
3. **A crate's `Match` can go through a block**: `c.get(i).map { |m|
   m.as_str }` gives a `Str?`. The examples show only `c.get(1)!.as_str` and
   `find_iter(..).map(_.as_str)`. Files: a03, a05. Looked:
   `examples/crate.lume`, `crates.md`.
4. **`replace_all(text, "$3/$2/$1")` expands groups** as Rust's `regex`
   does with a `&str` replacement, and `$` is plain in a Lume string. Files:
   a03. Looked: `examples/crate.lume` (only a plain replacement),
   `strings.md` (escapes).
5. **`.len` works straight on a crate iterator** (`digits.find_iter(t).len`),
   since `collections.md` says `len` ends any lazy chain and `crates.md` says
   a crate iterator is one; `examples/crate.lume` writes `.to_list.len`.
   Files: a03.
6. **`None -> unread += 1`**: a compound assignment as a match arm's body.
   `examples/healthcheck/main.lume` has `Some(w) -> why = w`, not `+=`.
   Files: a03.
7. **`hex.encode_upper` is offered**, being the same shape as `hex.encode`
   (`T: AsRef<[u8]>`). Files: a04. Looked: `examples/crates.lume`,
   `crates.md`.
8. **`hex.decode("abc")`'s message is `Odd number of digits`**: the crate's
   `Display` of `OddLength` becomes `e.message` as it is. Files: a04.
   Looked: `crates.md` ("the crate's error as the message"),
   `examples/crates.lume` (only `.error?`).
9. **`urlencoding.decode("%FF")` is an `Error`** (the crate's
   `FromUtf8Error`), not a replacement character. Files: a04. Looked:
   `examples/crates.lume`.
10. **`Captures.get` past the last group is `None`**, not a stop. Files: a05.
    Looked: `examples/crate.lume`, `crates.md` (`Option<T>` is `T?`).
11. **A stop on a two-digit line** keeps `  --> ` at two spaces, as compile
    messages do on line 20 (`examples/crate.expected`), with `   |` and
    `22 |` under it, no column and no caret, and the message is
    `error: ` and then the doc's text exactly. Files: a05. Looked:
    `errors.md` ("Where the program stopped"), `crates.md`, `numbers.md`.
12. **Methods that wait can sit inside an interpolation** in async code:
    `"#{laps(ann).max} #{laps(ann).min}"`. Every example puts them straight
    after `puts`. Files: a06. Looked: `collections.md`,
    `examples/async_generators.lume`.
13. **The block error on a chain over lines points at the step's own line**,
    at its `{` (25:15), not at the line the chain starts on. The example has
    the chain on one line. Files: a07. Looked:
    `examples/errors/async_chain_self.*`, `collections.md` (async chains),
    `errors.md` (a chain step over several lines stops on its own line).
14. **`lume check --json` with only warnings** prints just the JSON on
    standard output, keys in the order `examples/check_json.expected` shows,
    with each `!` warning's message and help as the text form gives them
    (`examples/crate.expected`), in line order, `ok: true`, exit 0. Files:
    a08. Looked: `install.md` ("Editors"), `examples/check_json.expected`,
    `errors.md`.
15. **No "did you mean" for `hex.dump`**: the help is the same as for
    `regex.nope`, with the crate's name. Files: a09. Looked:
    `examples/errors/crate_no_function.*`, `crate_no_method.*`,
    `methods.md` (suggestions for close names).

**Total: 15 guesses.**

# Reviewer C — notes

Set: whole programs. Six small tools built on what 0.8 settles. Every expected
output was worked out by hand from the docs; nothing was compiled or run.

## What I read

- `corpus/m116/BRIEF.md`.
- `docs/`: `tour.md`, `stability.md`, `install.md` ("Editors" through
  the end); reference `http.md`, `crates.md`, `collections.md`, `methods.md`,
  `strings.md`, `errors.md`, `io.md`, `concurrency.md`, `testing.md`,
  `printing.md`, `packages.md`, `operators.md`, `functions-and-blocks.md`,
  `names.md`, `patterns.md` (first half); tutorial lunches 17, 19 and 21;
  `docs/tutorial/http-requests/` (`lume.toml`, `main.lume`, `main.expected`).
- `packages/http/` (`lume.toml`, `lib.lume`, `test.expected`).
- `examples/`: `crate`, `crates`, `crate_builder`, `async_generators`,
  `chains_over_lines`, `unused_names`, the first 80 lines of `async.lume`
  (each `.lume` with its `.expected`); `check_json.expected`; `http_demo/`
  and `healthcheck/` (`main.lume`, `lume.toml`, `check.expected`);
  `fmt/chain_comments`, `fmt/messy`, `fmt/blocks`; `tests/async_tests`,
  `tests/parse`, `tests/blocks`; from `errors/`: `crate_no_method`,
  `crate_no_function`, `count_without_block`, `async_gen_print`,
  `dropped_failure`, `typo`, `missing_arm`, `async_chain_self`, `immutable`,
  `unknown_function`. I also searched every `examples/**/*.expected` for
  `warning:` lines, to see which warnings exist and how they are ordered.

## Programs

| file | tool | how it is checked | probes |
|---|---|---|---|
| `c01_uploads/` (`lume.toml`, `main.lume`) | upload planner | RUN, `lume run corpus/m116/c01_uploads/main.lume` | `http` builder steps, `with_body`, `with_file` (file there / not there), `send_parts` from a generator and setting aside a `with_file` body, `send` from blocking tasks, `await fetch`, `open`, a URL that does not parse, files under `/tmp/lume-review-c/` |
| `c02_indexer.lume` | text indexer | STOP (101) | `regex` (`find_iter`, `captures`, `get`), `hex` encode/decode, `urlencoding` encode/decode, crate types in signatures, and a `-1` given to a crate `usize` parameter |
| `c03_report.lume` | async sales report | RUN | async generators ranked with `join`, `max_by`, `min_by`, `sort_by`, `sort`, `max`, `min`, `last`, `reverse`, beside `fold`, `count`, `first`, `each`; ties; a sequence handed to an `async def`; one walked per task |
| `c04_logsum.lume` | log summariser | RUN | comments and a blank line between chain steps (at the steps' indent and deeper), steps inside a call's parentheses, in a `for` header, after a `def`'s `=`, mixed with `\|>`; bad lines with line numbers; a three-way tie in `max_by` |
| `c05_stock.lume` | stocktake tool | `lume check --json` | the settled fields of `lume check --json`, seven warning kinds, their order, and a column after non-ASCII text (characters, not bytes) |
| `c06_tests.lume` | feed tag linker | `lume test` | `test` and `async test` over crate calls and async sequences, `!` silent in tests, one failed `assert` on `[Int]`, and one test stopped inside a crate's integer check |

## Guesses

1. **A package run by naming its `main.lume` from outside the package.**
   `c01_uploads/` (header, `lume.toml`). I assumed
   `lume run corpus/m116/c01_uploads/main.lume` from the repository root
   finds `corpus/m116/c01_uploads/lume.toml` by looking upward from the
   file, and resolves `path = "../../../packages/http"` from that file's
   folder. packages.md ("Commands": "Naming a file still works in a
   package"; "looks upward for a `lume.toml`" is said of the *current
   folder*) does not say how a package is found when the folder you are in
   is outside it.
2. **The message for a URL that does not parse.** `c01_uploads/main.lume`
   (`not a url, named in the message: true`). I assumed ureq gives the
   `Error` when the request is made, not when `ureq::request` is called,
   so it goes through `open`'s `format!("{} {}: {}", m, u, e)` and starts
   `GET not a url: `. Looked at: http.md ("a URL that does not parse" is
   an `Error`, "the message names the method and the URL"),
   `packages/http/lib.lume` (`open`, and the test `get("not a url")` that
   checks only `error?`), lunch 19.2.
3. **How a crate's integer check stops a program.** `c02_indexer.lume`
   (the `EXPECTED STDERR` block). crates.md gives the words,
   ``-1 does not fit in the Rust `usize` the crate asks for``; I assumed
   it is reported like every other stop in errors.md ("Where the program
   stopped": `error: ` + message, `  --> file:line`, the gutter, the line,
   exit 101), on the line of the crate call inside `field`, not the line
   in `main` that called `field`.
4. **What `urlencoding.encode` leaves alone.** `c02_indexer.lume` (the
   three `Order%20…` lines). I assumed it keeps `A–Z a–z 0–9 - _ . ~`
   as they are and writes everything else as `%XX` with capital hex digits
   (`%2C`, `%3B`, `%26`). Looked at: `examples/crates.lume` and its
   `.expected` (`%20`, `%26`, `%3D` only); the rest is the crate's own
   behaviour, which no Lume page shows.
5. **`each do |s|` on an async sequence.** `c03_report.lume` (the
   `revenue` tally). collections.md and lunch 17.4 show `each` on an async
   sequence only with `{ |w| ... }`; I assumed the `do` form works too, as
   it does on a list (functions-and-blocks.md, "The three block forms").
6. **An async sequence's waiting methods inside `#{...}`.**
   `c03_report.lume` (`soups sold`, `first`, `last`, `least by name`). I
   assumed `"#{all_sales(stores).count { |s| ... }}"` in `async def main`
   waits as the same call does on its own line. Looked at: collections.md
   ("Methods that wait for the items"), lunch 17.4 ("These wait without
   `await` being written"), `examples/http_demo/main.lume` (blocks inside
   `#{}`, but no async sequence there).
7. **A comment indented deeper than the steps, in a program that runs.**
   `c04_logsum.lume` (`slow = entries`, the comment `# slowest first; …`
   at eight spaces). collections.md says a comment line between two steps
   is passed over but not at what indent; only `examples/fmt/chain_comments`
   shows a deeper one, as input to `lume fmt`, not a run.
8. **A comment inside a `for` header's chain.** `c04_logsum.lume`
   (`for e in entries` / `# the server's own failures` / `.filter …:`).
   `examples/chains_over_lines.lume` shows a `for` header over lines with
   its body at the steps' indent, but without a comment; I assumed the
   comment rule of collections.md holds there too, including between the
   receiver and the first step.
9. **The order of warnings of different kinds.** `c05_stock.lume` (the
   order of the seven diagnostics). install.md says `diagnostics` come "in
   the order `lume check` prints them" and the brief says "in the order the
   compiler finds them"; no example mixes kinds (`unused_names` has only
   unused names, `crate` only `!`, the async ones only `shared var`). I
   assumed plain line order, whatever the kind.
10. **The column of an unused block parameter.** `c05_stock.lume` (col 23,
    line 115). names.md says "the caret is on the name", but
    `examples/unused_names.expected` (28:22) puts it on the block's `{`. I
    followed the example.
11. **An unused parameter of a plain function.** `c05_stock.lume` (line
    99). I assumed it gets the same message and help as a method's
    (`examples/unused_names.expected`, 12:11 and 12:23), including "as an
    interface's method may be".
12. **No other diagnostics.** `c05_stock.lume`. I assumed nothing else is
    warned about: not the block that always gives `false`, not a
    `shared var` changed only outside tasks, not the `none` list. Looked
    at names.md ("A name that is never used"), concurrency.md ("Where the
    warnings go"), patterns.md (the unreachable-arm warning).
13. **A test stopped by a crate's integer check.** `c06_tests.lume`
    (`stops on purpose: capture -1`). testing.md shows a test that stops
    on a division by zero as `file:line: stopped: message`, with the line
    where it stopped; I assumed the crate's check (guess 3) is reported the
    same way, at `m = c.get(n)?` inside `part` (line 49).

### Where the docs promise something the reference does not show

- stability.md says to build a `Request` "with `http.request` and its
  steps, not by naming its fields", but the only way the docs show to look
  at a built request is to read its fields (`headers`, `timeout_ms`, in
  lunch 19.6 and `docs/tutorial/http-requests/`). `c01` reads them; whether
  reading is promised is not said.
- names.md says an unused-name warning's caret is "on the name"; the
  checked example puts a block parameter's on `{` (guess 10).
- install.md promises that `col` counts characters, not bytes, but no
  example or `.expected` file has a diagnostic after non-ASCII text;
  `c05` checks it (col 33 on line 109, where bytes would give 37).
- stability.md opens "Lume is at 0.7" while settling `http`, crates and
  `lume check --json` "in 0.8".
- lunch 19's step table (19.3) lists `.header`, `.with_body` and
  `.timeout`, but not `.with_file` or `send_parts`, which http.md has.

**Total: 13 guesses.**

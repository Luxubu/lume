# Reviewer B — the edges

## What I read

- The brief, `corpus/m116/BRIEF.md`.
- `docs/reference/http.md`, `crates.md`, `collections.md` (lazy chains,
  generators, async generators, the methods that wait and the ones that keep
  every item), `methods.md`, `printing.md`, `packages.md`, `numbers.md`
  ("What stops the program"), `errors.md` ("Where the program stopped",
  the `!` warning), `strings.md` (escapes), parts of `control-flow.md`
  (ranges) and `functions-and-blocks.md`.
- `docs/stability.md`, `docs/install.md` (the commands, "Editors" and the
  `lume check --json` fields), `docs/examples.md`, `docs/tutorial/19-http.md`.
- `packages/http/lib.lume`, `lume.toml` and `test.expected`.
- Examples: `crate.lume`, `crates.lume`, `crate_builder.lume`,
  `async_generators.lume`, `unused_names.lume`, `check_json.expected`,
  `http_demo/`, `healthcheck/lume.toml`, every file in `examples/fmt/`, and
  in `examples/errors/` the `crate_*`, `import_rust_no_crate`,
  `import_version_not_crate`, `async_gen_*`, `generator_used_twice`,
  `task_used_after_move`, `arity`, `typo`, `x`, `unknown_function`,
  `chain_no_items`, `count_without_block` and `sort_without_lt` pairs.

Nothing else: no compiler source, tests, tools, other corpus files, README,
HISTORY, CHANGELOG, bench or `.lume` folders (`docs/tutorial/http-requests/`
has one; I did not open it).

## The programs

| id | probes | meant to | checked with |
|---|---|---|---|
| `b01_request_steps/` | `http`: builder steps in every order, `with_body` after `with_file` and the reverse, two bodies, `timeout` first and last, a negative timeout, the request a step started from left unchanged | RUN | `lume run …/main.lume` |
| `b02_send_errors/` | `http`: a missing file's exact `Error` through `send`, `open` and `fetch`; `with_body` after it sets the file aside; `send_parts` sets it aside; port 9 with a timeout; a URL that does not parse | RUN | `lume run …/main.lume` |
| `b03_send_parts_moved/` | `http`: `send_parts` takes a generator by move; walking it afterwards | FAIL | `lume check …/main.lume` |
| `b04_crate_types.lume` | crates: a crate type in a field, `Option`, `Vec<u8>` back (and empty), `&[u8]` in, the crate's `Result` error text, `usize` and `Option<usize>` back, `Cow<str>`, a crate iterator as a chain | RUN | `lume run` |
| `b05_usize_stop.lume` | crates: `usize` parameters with values that fit, an index past the groups, then -1 | STOP | `lume run` |
| `b06_u8_list_stop.lume` | crates: the check on each item of an `[Int]` crossing as `&[u8]`: 0 and 255 fit, 256 stops | STOP | `lume run` |
| `b07_builder_twice.lume` | crates: a builder (ureq) used twice, then in a loop, then moved by its last use | RUN | `lume run` |
| `b08_used_up_refused.lume` | crates: a method that uses up a value that cannot be cloned while it is still needed | FAIL | `lume check` |
| `b09_json_warnings_only.lume` | `lume check --json`: warnings only, so `ok` is true and the exit is 0 | RUN | `lume check --json` |
| `b10_json_column.lume` | `lume check --json`: `col` after two- and three-byte characters; an unused local not shown beside an error | FAIL | `lume check --json` |
| `b11_json_order.lume` | `lume check --json`: five warnings of four kinds, interleaved, two on one line: their order | RUN | `lume check --json` |
| `b12_async_empty.lume` | async sequences: `join`, `max`, `min`, `max_by`, `min_by`, `sort`, `sort_by`, `last`, `reverse` with no items, directly and after chains that let nothing through | RUN | `lume run` |
| `b13_async_after_chain.lume` | async sequences: the same nine after chains, with ties (first on a tie, stable sort) | RUN | `lume run` |
| `b14_async_no_uniq.lume` | async sequences: `uniq` after `map`, the refusal and its help word for word | FAIL | `lume check` |
| `b15_fmt_chain_comments.lume` | `lume fmt --stdout`: two chains with comments between steps, indented oddly, a trailing comment, a block with no spaces | RUN | `lume fmt --stdout` |

Four programs fail with a whole error predicted (b03, b08, b10, b14), and two
stop (b05, b06).

Things I noticed in the docs while writing these (not guesses):

- `docs/stability.md` opens "Lume is at 0.7", and then settles `http`, Rust
  crates and `lume check --json` "in 0.8".
- `lume fmt --stdout`, which the brief names, is in no doc: `install.md`
  lists `lume fmt file.lume` only, `packages.md` gives `lume fmt [--check]`,
  and `examples/fmt/messy.lume` says "`lume fmt` has no options".
- `crates.md` promises the `-1 does not fit in the Rust `usize`` stop, and
  that a `u64` past the largest `Int` coming back stops too, but no example
  in `examples/` or `examples/errors/` shows either.
- `crates.md` gives the refusal for a value used up while still needed only
  as "… uses up `x`, and `x` is still needed after it", with no example.
- `http.md` says a request's `Error` message "names the method and the URL";
  the missing-file `Error` (`cannot read `…`: …`) names neither.
- `http.md` says nothing about two `Content-Type` headers: `with_body` and
  `with_file` each add one, so each step after the first adds a second
  (`packages/http/lib.lume`); only the package source shows it.
- `stability.md` says not to build a `Request` by naming its fields, while
  lunch 19 and the package's own test read them (reading is not ruled out).
- The settled-commands row of `stability.md` leaves out `lume crate`,
  `lume emit` and `lume lsp`, which `install.md` lists (`lume lsp` is
  covered under editors).

## Guesses

1. **`lume fmt --stdout` exists**, prints the formatted file to standard
   output and leaves the file alone. — b15. Looked in `install.md` (the
   commands), `packages.md` (Commands), `examples/fmt/messy.lume`, the
   tutorial (`01-hello.md`).
2. **`lume fmt` puts every step of a chain, and every comment between steps,
   two spaces in from the statement**, whatever their indent (10, 6, 3, 12
   or 7 spaces). — b15. The only example, `examples/fmt/chain_comments`,
   has 6 and 4 going to 4; `collections.md` says only "lume fmt keeps them
   one a line".
3. **A comment after a step on the same line gets two spaces before `#`.**
   — b15. `examples/fmt/messy` shows this for a struct field only.
4. **`lume fmt` leaves comment lines as they are**, including header lines
   holding code and `#` inside them. — b15. `examples/fmt/messy`,
   `chain_comments`.
5. **Warnings of different kinds come in source order** (line, then
   column), not grouped by kind or by the pass that finds them. — b11.
   `install.md` says only "in the order `lume check` prints them"; no
   example mixes kinds (`crate.expected`, `unused_names.expected`,
   `port/orders.expected` each have one kind).
6. **A generator handed to a function's `Iterator[T]` parameter is refused
   on its later use with "… is a generator, and it is given away here, so
   it cannot be used after"**, the caret on the argument, and the help of
   `generator_used_twice`. — b03. `examples/errors/generator_used_twice`
   ("used up here", on `e.sum`) and `task_used_after_move` ("given away
   here", on the argument).
7. **The whole refusal for a used-up value still needed**: "`into_string`
   uses up `r`, and `r` is still needed after it", the caret on `r`, and the
   help "make it the last use, or make a new value where one is needed
   again". — b08. `crates.md` "Builders" gives only the elided message and
   the advice as prose.
8. **ureq's `Response` cannot be cloned and `into_string` is offered** as a
   method that uses the value up, so b08 reaches that refusal rather than
   another. — b08. `crates.md`, `examples/crate_builder.lume`; the crate's
   API from memory.
9. **A local used by a using-up step inside a loop body, and again after
   the loop, is copied each time round**. — b07. `crates.md` "Builders" and
   `collections.md` "When a list is copied" (neither shows a loop with a
   crate value).
10. **hex 0.4's error texts**: "Odd number of digits" and "Invalid character
    'z' at position 0". — b04. `crates.md` says only "the crate's error as
    the message"; `examples/crates.lume` prints only `.error?`.
11. **`urlencoding.encode_binary` is offered** (`&[u8]` as `[Int]`) and
    encodes bytes as upper-case `%XX`: `[255, 0]` is `%FF%00`, `[72, 105]`
    is `Hi`. — b04, b06. The crate's API from memory; `crates.expected`
    shows `%3D` in upper case.
12. **regex 1.x has `captures_len` and `static_captures_len`**, and the
    second is `None` for `(a)|b`. — b04. The crate's API from memory;
    `crates.md` table for `usize` and `Option`.
13. **regex's `splitn` with a limit of 1 gives the whole text**, and with 2
    the text either side of the first match. — b05. The crate's API from
    memory.
14. **`.map { |m| m.as_str }` on a `regex.Match?` works**, giving a `Str?`.
    — b05. `examples/crate.lume` uses `c.get(1)!.as_str` and `?` on `find`,
    never `.map`.
15. **A negative `Int` made at run time (`0 - 1`, taken from a list) is
    stopped at run time, on the line of the Lume call**, in the
    no-column form of `errors.md`. — b05. `crates.md` table, `errors.md`
    "Where the program stopped".
16. **An `[Int]` crossing as `&[u8]` has each item checked**, the first
    that does not fit is named, and the message says `u8`. — b06.
    `crates.md` (the table names `usize` only in its example).
17. **A refused connection to port 9 with `.timeout(200)` is not reported
    as a timeout** (its message does not contain "timed out"). — b02.
    `packages/http/lib.lume` (`open` adds "(timed out)" only for an I/O
    timeout); ureq's own wording from memory.
18. **`http.Response or Error` is accepted as a parameter's type.** — b02.
    `stability.md` lists `T or Error` among types; the docs show it only as
    a result and a binding's type.
19. **A chain over an async sequence is named `async Iterator[Str]` after
    `map` to `Str`**, with the caret on the `.` before the missing method.
    — b14. `examples/errors/async_gen_chain` (a generator, not a chain).
20. **Messages name a package's file as `corpus/m116/<pkg>/main.lume`**
    when `lume check` is given that path, and `http = { path =
    "../../../packages/http" }` reaches the package. — b01, b02, b03.
    `packages.md`, `examples/http_demo/lume.toml`.

Also not a guess but a choice: b07 and b08 use ureq `"2"`, which the brief's
list of crates leaves out. The only crates the brief allows have no builder
whose steps take `self` by value (regex's `RegexBuilder` takes `&mut self`),
and ureq is already fetched for `examples/crate_builder.lume` and
`packages/http`, so nothing new is downloaded.

**Total: 20 guesses.**

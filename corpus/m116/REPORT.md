# Milestone 116 review corpus — REPORT (twenty-third review round)

This round tested milestones 112–115 and what 0.8 settles: the `http`
package with a body sent as it is made, Rust crates with their checked
integers, the fields of `lume check --json`, an async sequence's methods
that keep every item, and comments between a chain's steps. Three
reviewers wrote thirty programs from `docs/` and `examples/` alone, with
nothing run. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday code: requests built and failing, uploads from a file and a generator, regex on a log, `urlencoding` and `hex`, a capture group past the end, async rankings, a chain with comments, `check --json`, two refusals |
| `b01`–`b15` | the edges: builder steps in every order, each missing-file path, a generator moved into `send_parts`, a row of the crate table each, `usize` and `u8` that do not fit, a builder used twice, warnings-only and column and order in `--json`, the nine new methods on empty sequences and after chains, a method still missing, `fmt --stdout` on odd indents |
| `c01`–`c06` | whole programs: an upload planner, a text indexer on three crates, an async sales report, a log summariser with comments in its chains, a stocktake checked with `--json`, a tag linker with its own tests |

## The numbers

| | round twenty-two | round twenty-three |
|---|---|---|
| guesses | 45 | **48** (A 15, B 20, C 13) |
| meant to run, stop, test, format or check as JSON, right on the first run | 16 of 23 | **18 of 24** |
| refused as predicted | 7 of 7 | **3 of 6**, word for word (two others by their wording only) |
| rustc leak | 0 | **3** (b04, b06, c06: two causes) |
| wrong output | 0 | 0 |

No program printed a wrong answer. After the fixes, 21 of the 30 print
what their authors predicted; the other nine are guesses the docs now
answer the other way, or the changes this round made (below).

## What the programs found

1. **A list given to a crate that asks for bytes reached rustc** (b04, b06):
   `urlencoding.encode_binary(data: &[u8])` was offered as taking `[Int]`,
   and the list was passed as Lume's 64-bit integers. Milestone 114
   checked one integer crossing to a crate and not a list of them. Each
   item is converted and checked now, borrowed or owned, so 256 for a byte
   stops the program on its line, as b06 predicted.
2. **`xs == []`, with `xs` a list of `Str`, reached rustc** (c06, in an
   `assert`; it does outside one too): Rust cannot tell which of `String`'s
   `==` an empty `vec![]` means. In `==` and `!=` an empty `[]` or `{}` is
   of the other side's type now. Milestone 103's binding of an `assert`'s
   sides made it reachable in a test.
3. **`lume check --json main.lume` said there was no file** (a08): the file
   was looked for right after the command only, though `install.md` shows
   it written after `--json`. A `.lume` file after a leading option is the
   file now.
4. **`.uniq` on an async sequence said it "is not a value to print"**
   (b14), not that it has no `uniq`; the missing method says so first now.
5. **Warnings came by kind, not where they were written** (b11): every `!`
   before every unused name. They are in the order they are written now,
   whatever found them.
6. **`lume fmt` moved a comment at the end of a chain step's own line onto
   a line of its own** (b15) — milestone 115's fix took every comment
   between steps for one on its own line. One at the end of a step stays
   there.
7. **A request given two bodies sent two `Content-Type` headers** (b01, and
   a02 through `send_parts`): each body step added one. A body step takes
   the place of an earlier one's `Content-Type` now.
8. **A file that is not there was not named by method and URL** (A, B):
   `http.md` says every request's `Error` is, and the missing-file one was
   not. It is now: ``PUT http://example.com/up: cannot read `/no/such/file`:
   ...``.
9. **An unused block parameter's caret was on the `{`** (C, against
   `names.md`, which says the caret is on the name): it is on the name.

## What the docs now say

- `http.md`: a body step takes the place of an earlier one's
  `Content-Type`; the missing-file `Error`, whole (A, B).
- `stability.md`: a `Request`'s fields are for reading — build one with
  `http.request` and its steps; `lume crate` is among what is settled (A,
  B, C).
- `install.md`: `lume fmt --stdout`, which only examples showed; the order
  of `--json`'s diagnostics (B).
- `errors.md`: a stop's `-->` line is two spaces in (A).
- Lunch 19 and the `http` package's own comment: `.with_file` and
  `send_parts` (A).

## Left

- `docs/stability.md` opens with "Lume is at 0.7" beside parts "settled in
  0.8" (A, B); release 0.8.0, milestone 117, makes that true.

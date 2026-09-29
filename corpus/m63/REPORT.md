# Milestone 63 review corpus — REPORT (thirteenth review round)

This round tested milestones 60–62:
- the `json` package: reading, decoding into types of your own, writing
  through `ToJson`;
- dates and times;
- the error messages.

Three reviewers wrote twenty-seven programs from `docs/`, `examples/` and
`packages/` alone, with nothing run, and counted what they had to guess.
The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday programs: settings with defaults, an address book, writing structs out, an event log by day, birthdays, an invoice with due dates, editing a document, durations, a path explorer |
| `b01`–`b12` | the edges: every `need_…` on every wrong kind, big numbers, paths, eighteen texts that are not JSON, `ToJson` nested and missing, the orphan rule, date directives, leap days, times before 1970, parsing |
| `c01`–`c06` | whole programs: a log analyzer, a shop with nested orders, a scheduler, a command runner writing JSON, a JSON diff, a timeline over a library's generic decoder |

## The numbers

| | round twelve | round thirteen |
|---|---|---|
| guesses | 38 | **31** (A 5, B 20, C 6) |
| meant to run, right on the first run | 21 of 22 | **20 of 21** |
| refused as predicted | 7 of 7, 2 word for word | 6 of 6, **3 word for word** |
| rustc leak | 0 | 0 |
| wrong output | 0 | 0 |

After the fixes, all 21 programs print what their authors worked out,
except where a fix changed the answer (below). Reviewers A and C were right
on every program. The three refusals that were not word for word differ in
one thing only: the message names `json.Json` where the reviewer wrote
`Json`. That is now documented.

## What the programs found

1. **`Time.parse` did not read `%j` or a negative `%s`** (b12). The docs and
   the compiler's own pattern check both listed `%j` as a directive `parse`
   knows, but running it said "`%j` is not a date directive". Both work now.
   Text left over after the pattern now says where, like every other
   mismatch: ``unexpected ` extra` at position 10`` (the reviewer's wording,
   which was better than the old one).

Reviewer B read the package's source and found seven problems in the `json`
package. All are fixed, and b01–b03 now print the fixed answers:

2. A high surrogate followed by an ordinary character (`"\ud800A"`) was
   accepted as a character. It is now `lone surrogate`, and so is a lone low
   one, which used to say `bad code point 56320` with no position.
3. `bad hex digit` had no position. It has one now.
4. `01` was accepted. RFC 8259 forbids a leading zero.
5. `show` wrote control characters as they are, so its output could fail to
   parse back. They are now written as `\u00XX`.
6. `1e15` was written `1000000000000000.0`, which contradicted the docs. Whole
   numbers up to 2^53, which a `Float` holds exactly, are written without a
   fraction, and `int` reads up to the same limit.
7. `need_int` on `1e20` said "not a whole number". It now says ``too large a
   number to be an `Int` exactly``.
8. `path` took `" 1"` and `"+1"` as positions. A position is digits only now.

"`f` is a null" now reads "`f` is null".

## Where the guesses went

- **Dates** (B, 9): how `Time.parse` reads (literal characters, how many
  digits each directive takes, names in any case, a field the pattern leaves
  out), and its error texts. `io.md` now has all of it, with a table of the
  errors.
- **Optional fields with a default** (A): the package now has
  `opt_str`/`opt_int`/`opt_num`/`opt_bool`, giving `T? or Error` as serde
  reads an `Option` field: `j.opt_int("port")?.or(8080)`.
- **Whether `import json.ToJson` is needed** (A): it is not; `import json`
  brings the package's extends. Said in `json.md`.
- **The parser's error texts** (A, B): now a table in `json.md`.
- **A bare `"` in a `"""` string, and pushing onto an `[Interface]` list then
  calling a default through it** (C): both work, and both are now shown.

## Verdict (condensed from the three notes)

A: the package and the date functions covered all nine programs; the gaps
were in the docs, not the language. B: the package was readable enough to
predict every error position, which is how its seven defects were found.
C: none of my guesses was about the package.

One defect in the compiler, the newest one, and seven in the new package,
all found by reading, not running. No output was ever wrong, and nothing
leaked into rustc.

# Milestone 81 review corpus — REPORT (seventeenth review round)

This round tested text processing, where milestones 72 and 80 changed how
Lume works without changing what a program prints: lists from `split` and
`lines` that hold slices of the text, names bound to their items, and
`.slice` read in place. The brief asked the reviewers above all for programs
whose output would be wrong if any of these were read in place when it
should have been copied.

Three reviewers wrote thirty programs from `docs/` and `examples/` alone,
with nothing run. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday text tools: a CSV report, a log summary, an INI reader, a word index, fixed-width records, a template filler, a word diff, a passwd parser, one refusal |
| `b01`–`b15` | the edges of reading in place: a `var` source grown or replaced while its pieces are in use, the source or the list kept, returned or given away, items kept in sets, maps, fields, blocks and tasks, `.slice` at every position, nested splits, generators over lines, two refusals |
| `c01`–`c06` | whole programs: a Markdown renderer, a two-pass assembler, a bank reconciler, a log analyser, a mini spreadsheet, a make-like resolver |

## The numbers

| | round sixteen | round seventeen |
|---|---|---|
| guesses | 47 | **12** (A 5, B 2, C 5) |
| meant to run, right on the first run | 15 of 18 | **26 of 27** |
| refused as predicted | 11 of 11 | **3 of 3**, word for word |
| rustc leak | 3 | **1** (a01) |
| wrong output | 0 | 0 |

No program printed a wrong answer: every one of the fifteen edge programs
written to catch text read in place that should have been copied printed
the copied answer. c04's output was right; its author did not predict the
warning for a list of mixed values behind an interface.

## What the programs found

1. **`enumerate` and `zip` over split text handed out slices where Lume
   promised text of its own** (a01: `for i, name in header.split(",").enumerate`,
   then `cols[name] = i`, reached rustc). A borrowed item was "copied" with
   `.clone()`, which on a slice of text is still a slice. It is now
   `.to_owned()`, which gives text of its own, at all five places that copy
   an item into a pair. The `enumerate` one was older than milestone 72; the
   `zip` ones came with milestone 68.

## What the docs now say

- `strings.md`: a negative position or length in `.slice` counts as 0 (B);
  `to_int` reads leading zeros (C).
- `methods.md`: `lines` keeps an empty line between two others (C).

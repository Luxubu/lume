# Review corpus

Programs written by an independent reviewer after milestone 14, without
knowledge of the compiler: nine realistic programs (`p*.lume`) and small
edge-case probes (`edge/*.lume`). At the time of the review, none of the
nine compiled as first written, and about 40% of all probes ended in a
leaked rustc error. See the consolidated review for the ranked findings.

These are not in `tests/run.sh` yet. They are the bar for milestones
15–17: every file here either runs correctly or fails with a Lume error.

## m17/ — second review round

Thirty programs written by a second independent reviewer after milestone 17,
from the README and three examples only, aimed at the new surface: `|`
alternatives, the exhaustiveness checker, the new methods, statements in
inline blocks. Their first run: 20 ok, 5 rustc leaks (two causes: `map_values`
with a method call in the block, `Str` parameter `+` literal), 1 crash
(`clamp` with the bounds reversed spoke Rust), 4 correct compile errors,
**0 wrong outputs**. All three defects are fixed; `REPORT.md` is the
reviewer's report as written. Every file has an `.expected` and runs in
`tests/run.sh`.

## m18/ — third review round

Thirty programs by a third independent reviewer after milestone 18, from the
README and the examples only, on sets, character slicing, and how those
combine with interfaces, modules, async, tests and the formatter. Their first
run: 14 ok, 9 Lume errors, 7 rustc leaks, 1 crash, 2 silently wrong outputs
(a change landing on a temporary copy), 1 formatter defect — and, again, no
wrong set or slice semantics. All ten ranked problems are fixed; `REPORT.md`
is the reviewer's report. `s31_block_param_matrix.lume` is not the reviewer's:
it is the regression test written while fixing finding 3, every block method
over every item shape (225 cells). `*.exp` files are the reviewer's
hand-computed expectations, kept as evidence; `*.expected` is what the suite
compares.

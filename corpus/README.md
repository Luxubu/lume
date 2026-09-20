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

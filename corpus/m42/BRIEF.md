# Round eight — the brief every reviewer was given

This round tests milestones 37–41, which changed the language: functions of
a type (`def self.name`), blocks as values (kept in fields, lists and
bindings, returned, called as `f(x)(y)`), a module's function as a block,
tuple ordering, `await f()?`, and Rust's coherence rule for `extend`.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/`. Nothing else: not the compiler source, not
`tests/`, not `corpus/`, not `README.md`, not `HISTORY.md`.

**Do not run the compiler.** Every expected output is worked out by hand from
what the docs say. A guess only counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

Each program starts with a header: what it probes, `Meant to RUN.` or
`Meant to FAIL.` (with the error you expect), and `EXPECTED OUTPUT:` as
comment lines (a line that is just `#` is a blank output line).

# Round nine — the brief every reviewer was given

This round tests milestones 42–44: `?` inside `spawn:` (a task that uses `?`
gives back `T or Error`); interface methods that take a block, have type
parameters of their own, or are `async` (the last two callable only where the
value's own type is known); and what round eight fixed — a block kept in a
map or a tuple, `extend` of your own generic type at chosen arguments
(`Box[Int]`, `Box[Str]`), generic calls that work `T` out through blocks.

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

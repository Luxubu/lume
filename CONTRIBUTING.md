# How Lume changes

Lume moves one milestone at a time, and each milestone is one commit that
leaves `tests/run.sh` passing. A milestone is done when:

1. **The feature works**, with an example under `examples/` whose output is
   checked (`name.lume` beside `name.expected`).
2. **Every way to get it wrong has an example** in `examples/errors/`: the
   smallest program that makes `lume check` give the message, with its
   output in `name.expected`. `tests/message_coverage.py` counts the
   compiler's messages that some test shows, and the suite fails if that
   number falls below `tests/message_coverage.min`; raise it when you add
   examples. A message that cannot be reached goes in
   `examples/errors/UNREACHED.md` with the reason.
3. **The docs say it**, in the page a reader would look in first. Every
   ```` ```lume ```` example there is run by `tests/docs.sh` and must print what
   it claims.
4. **`HISTORY.md` has a row**: what was built, and what building it found.

Where Lume has to choose how something behaves, it does what Rust does,
unless there is a reason written down not to.

Every few milestones a **review round** checks the docs against people who
have not seen the compiler. Reviewers write programs from `docs/` and
`examples/` alone, predict every output by hand, and count what they had to
guess. See `corpus/*/BRIEF.md` and `corpus/*/REPORT.md`.

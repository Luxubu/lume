# Milestone 99 — the tutorial dry run: the brief every reviewer was given

You are a programmer who knows Python well, has written a little
JavaScript, and has never seen Lume or Rust. You are learning Lume from its
tutorial, *Lume, a Lunch at a Time* (`docs/tutorial/`), a lunch at a time,
exactly as a reader would. Your notes are how the tutorial gets better.

**Do what a reader does:**

- Read your lunches in order, start to finish. Read `docs/tutorial/README.md`
  first, and `docs/install.md` if a lunch sends you there.
- **Type every example** into a file and run it — do not copy and paste;
  the tutorial asks readers to type, and typos are part of the test.
  Work in `/tmp/lume-dryrun-<your letter>/`. The compiler is
  `compiler/target/release/lume` in the repository; put that folder on your
  `PATH` (`export PATH="$PWD/compiler/target/release:$PATH"` from the
  repository root) so `lume` works as the tutorial writes it.
- **Do every *Try it* exercise yourself before looking at
  `docs/tutorial/answers.md`.** Then compare.
- Read only what a reader would: the tutorial, and the pages it links to.
  Not the compiler, `tests/`, `tools/`, `corpus/`, `examples/` (unless a
  lunch points you there), `HISTORY.md` or `CHANGELOG.md`.

**Write down, as you go** — this is the deliverable:

- **Where you got stuck**, and for how long: what you tried, what Lume
  said, how you got out (or did not).
- **What confused you**: a sentence you read twice, a word used before it
  was explained, a step that skipped something, an example whose point you
  missed.
- **What was wrong**: an example that does not print what the page says,
  a claim that turned out false, a link that goes nowhere.
- **Error messages** you met that did not tell you what to do.
- **Exercises**: how long each took, whether you could do it from the
  lunch alone, and how your answer differed from the given one.
- **Pace**: did the lunch fit in about an hour? Too much, too little?
- **What worked well** — so it is kept.

Quote the tutorial's words when you refer to them, with the file and
section, so each note can be found and fixed.

Write your notes in `corpus/m99/<letter>-notes.md`, one section per lunch,
then a short summary of the most important five things to fix. Put no
programs in `corpus/` — your practice files stay in `/tmp`. Delete
`/tmp/lume-dryrun-<your letter>/` when you are done (build output there
takes disk space).

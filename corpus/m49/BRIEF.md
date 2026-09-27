# Round ten — the brief every reviewer was given

This round tests milestones 47–48: packages. A `lume.toml` names a package
and its dependencies; `lib.lume` is a library's root, and from outside only
what it makes `pub` or passes on with `pub import` can be reached; module
paths start at the package root; the orphan rule decides where an `extend`
may be written; crate versions go in `[rust]`; commands work with no file
named; dependencies may come from git, pinned by `lume.lock`.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/`. Nothing else: not the compiler source, not
`tests/`, not `corpus/` (except this brief), not `README.md`, not
`HISTORY.md`.

**Do not run the compiler**, and do not run `git`. Every expected output is
worked out by hand from what the docs say. A guess only counts as a guess if
you could not check it in the docs.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

**Each program is a folder** `corpus/m49/<id>_<name>/` holding two or more
packages side by side, each in its own folder with a `lume.toml`. The
program to run is always `app/main.lume`, and dependencies are given by
`path` (`../<name>`). `app/main.lume` starts with a header: what it probes,
`Meant to RUN.` or `Meant to FAIL.` (with the whole error you expect, file
and line included, as `lume check corpus/m49/<id>_<name>/app/main.lume`
run from the repository root would print it), and `EXPECTED OUTPUT:` as
comment lines (a line that is just `#` is a blank output line).

# Round thirteen — the brief every reviewer was given

This round tests milestones 60–62:
- **the `json` package** (`packages/json/`, documented in
  `docs/reference/json.md`): parsing and printing, typed reads, decoding into
  types of your own with `need_…`, and writing through `ToJson`;
- **dates and times**: `Time.format`, `Time.parse`, `Time.date` and the parts
  of a time (`docs/reference/io.md`, "Dates");
- the error messages.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/` and `packages/`. Nothing else: not the compiler
source, not `tests/`, not `corpus/` (except this brief), not `README.md`, not
`HISTORY.md`.

**Do not run the compiler**, and do not run the programs you write. Every
expected output is worked out by hand from what the docs say. A guess only
counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

**A program that uses JSON is a package**: a folder
`corpus/m63/<id>_<name>/` with a `lume.toml` whose `[dependencies]` has
`json = { path = "../../../packages/json" }`, and a `main.lume`. Any other
program may be one file, `corpus/m63/<id>_<name>.lume`. The file that holds
`main` starts with a header: what it probes, `Meant to RUN.` or
`Meant to FAIL.`, and `EXPECTED OUTPUT:` as comment lines (a line that is
just `#` is a blank output line). For a program meant to fail, give the whole
error, as `lume check corpus/m63/<path to main.lume>` run from the repository
root would print it.

Output must not depend on the date, the clock, timing or the machine: use
fixed times (`Time.date(..)`, numbers, `Time.parse(..)`), never `Time.now`
in what you print.

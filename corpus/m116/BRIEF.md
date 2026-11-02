# Round twenty-three — the brief every reviewer was given

This round tests milestones 112–115, and what 0.8 settles:
- **the `http` package**, now settled: the request builder, `send`,
  `fetch`, `open` with `lines` and `chunks`, and a body sent as it is made
  — `.with_file(path, content_type)` and `req.send_parts(parts)`
  (`docs/reference/http.md`). No network: a program may build requests and
  look at them, send to `http://127.0.0.1:9/`, where nothing listens, and
  send a file that is not there — each is an `Error` it can show;
- **Rust crates**, now settled: how a crate's types are read, builders,
  and the integers that cross to a crate, which are checked
  (`docs/reference/crates.md`). Use only the crates the examples already
  use — `regex` "1", `hex` "0.4", `urlencoding` "2" — so nothing new is
  fetched;
- **for editors and tools**: the fields of `lume check --json`
  (`docs/install.md`, "Editors");
- **an async sequence's methods that keep every item**: `join`, `max`,
  `min`, `max_by`, `min_by`, `sort`, `sort_by`, `last`, `reverse`
  (`docs/reference/collections.md`);
- **comments between the steps of a chain**, and `lume fmt`;
- **what is settled** (`docs/stability.md`): lean on what it promises, and
  say where the docs promise something the reference does not show;
- everything around them, and the error messages.

**Warnings** go to standard error before anything the program prints, in
the order the compiler finds them. Give each, whole, in the `EXPECTED
STDERR:` block, as `lume run <file>` from the repository root names the
file (`--> corpus/m116/a01_name.lume:7:3`).

**A program that stops** writes what it printed first, then the stop to
standard error, and exits with code 101: give it in `EXPECTED STDERR:` and
add `EXPECTED EXIT: 101`.

A program checked another way says so in its header, as **"Run with
`lume test corpus/m116/<file>`"**, **"Run with `lume check --json
corpus/m116/<file>`"** or **"Run with `lume fmt --stdout
corpus/m116/<file>`"**, and gives the whole output that command prints. A
program that uses another module may be a package: a folder
`corpus/m116/<id>_<name>/` with `lume.toml` and `main.lume`.

Programs every machine has may be run with `Process.run` and
`Process.spawn` — `echo`, `printf`, `cat`, `sort`, `sh`, `tr`, `wc` — but
keep arguments to plain ASCII, with no newline inside one, and output must
not depend on timing.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/` and `packages/`. Nothing else: not the compiler
source, not `tests/`, not `tools/`, not `corpus/` (except this brief), not
`README.md`, not `HISTORY.md`, not `CHANGELOG.md`, not `bench/`, and no
folder named `.lume` anywhere (build output).

**Do not run the compiler**, and do not run the programs you write. Every
expected output is worked out by hand from what the docs say. A guess only
counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

**Every program is one file**, `corpus/m116/<id>_<name>.lume` (or a folder,
as above). It starts with a header: what it probes, `Meant to RUN.`, `Meant
to STOP.` or `Meant to FAIL.`, and `EXPECTED OUTPUT:` as comment lines (a
line that is just `#` is a blank output line). For a program meant to
fail, give the whole error, as `lume check <file>` run from the repository
root would print it. Warnings, stops, and anything a program writes to
standard error, go in a separate `EXPECTED STDERR:` block after the output.

No network, no clock in what you print, no reading files that are not in
the repository (a program may write and read its own files under
`/tmp/lume-review-<your letter>/`).

Write `corpus/m116/<letter>-notes.md` too, with three sections: what you read,
a table of your programs, and your guesses, numbered, each with the files
it touches and where you looked.

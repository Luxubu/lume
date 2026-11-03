# Round twenty-four — the brief every reviewer was given

This round is the last before 1.0. It does not test one release's changes:
it tests **what 1.0 promises**, the whole of it, as someone meeting Lume
for the first time would. `docs/stability.md` lists the settled parts;
each is described in a reference page under `docs/reference/` and taught
in the tutorial, `docs/tutorial/`. Lean on what they promise, and say
wherever two pages disagree, or a page promises something no example
shows.

**Warnings** go to standard error before anything the program prints, in
the order they are written. Give each, whole, in the `EXPECTED STDERR:`
block, as `lume run <file>` from the repository root names the file
(`--> corpus/m120/a01_name.lume:7:3`).

**A program that stops** writes what it printed first, then the stop to
standard error, and exits with code 101: give it in `EXPECTED STDERR:` and
add `EXPECTED EXIT: 101`.

A program checked another way says so in its header, as **"Run with
`lume test corpus/m120/<file>`"**, **"Run with `lume check --json
corpus/m120/<file>`"** or **"Run with `lume fmt --stdout
corpus/m120/<file>`"**, and gives the whole output that command prints. A
program that uses another module may be a package: a folder
`corpus/m120/<id>_<name>/` with `lume.toml` and `main.lume`.

Programs every machine has may be run with `Process.run` and
`Process.spawn` — `echo`, `printf`, `cat`, `sort`, `sh`, `tr`, `wc` — with
arguments in plain ASCII and no newline inside one; output must not
depend on timing. Rust crates: only `regex` "1", `hex` "0.4",
`urlencoding` "2" and `ureq` "2", which the examples use already. `http`
only to `http://127.0.0.1:9/`, where nothing listens.

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

**Every program is one file**, `corpus/m120/<id>_<name>.lume` (or a folder,
as above). It starts with a header: what it probes, `Meant to RUN.`, `Meant
to STOP.` or `Meant to FAIL.`, and `EXPECTED OUTPUT:` as comment lines (a
line that is just `#` is a blank output line). For a program meant to
fail, give the whole error, as `lume check <file>` run from the repository
root would print it. Warnings, stops, and anything a program writes to
standard error, go in a separate `EXPECTED STDERR:` block after the output.

No network, no clock in what you print, no reading files that are not in
the repository (a program may write and read its own files under
`/tmp/lume-review-<your letter>/`).

Write `corpus/m120/<letter>-notes.md` too, with three sections: what you read,
a table of your programs, and your guesses, numbered, each with the files
it touches and where you looked.

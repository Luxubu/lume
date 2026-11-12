# Round twenty-six — the brief every reviewer was given

This round tests what milestones 128–130 made, on the way to 1.2:

- serving HTTP, settled now, and lunch 27, which teaches it with a café's
  orders service; `examples/notes/`, a whole service built on it;
- `http.value`, the value of a name in a query or a form;
- generators that are a loop around one `yield`, which now run as plain
  iterators — their work must still happen when, and in the order, the
  pages say;
- an `extend [T: I]` used on a list whose items are not `I`;
- a `() or Error` call that ends a branch at the end of a `() or Error`
  function.

`docs/reference/http.md`, `docs/stability.md`, `docs/reference/collections.md`
and the tutorial, `docs/tutorial/` (lunches 16, 21 and 27 above all),
describe them. Lean on what the pages promise, and say wherever two pages
disagree, or a page promises something no example shows.

**Warnings** go to standard error before anything the program prints, in
the order they are written. Give each, whole, in the `EXPECTED STDERR:`
block, as `lume run <file>` from the repository root names the file
(`--> corpus/m131/a01_name.lume:7:3`).

**A program that stops** writes what it printed first, then the stop to
standard error, and exits with code 101: give it in `EXPECTED STDERR:` and
add `EXPECTED EXIT: 101`. Any other exit code is given the same way, as
`EXPECTED EXIT: 1`.

A program checked another way says so in its header, on one line, as
**"Run with `lume test corpus/m131/<file>`"**, **"Run with `lume check
--json corpus/m131/<file>`"** or **"Run with `lume fmt --stdout
corpus/m131/<file>`"**, and gives the whole output that command prints. A
program that uses a package — `http` and `json` are packages — is a folder
`corpus/m131/<id>_<name>/` with `lume.toml` and `main.lume`; the
dependency is by path, `http = { path = "../../../packages/http" }`.

**HTTP**: a program may start its own server on `127.0.0.1:0` — a port the
system picks — and make requests to it from a task, as
`examples/http_server/` does. Nothing else: no other address, except
`http://127.0.0.1:9/`, where nothing listens. Never print the port, and
nothing whose order depends on timing.

Programs every machine has may be run with `Process.run` and
`Process.spawn` — `echo`, `printf`, `cat`, `sort`, `sh`, `tr`, `wc` — with
arguments in plain ASCII and no newline inside one.

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

**Every program is one file**, `corpus/m131/<id>_<name>.lume` (or a folder,
as above). It starts with a header: what it probes, `Meant to RUN.`, `Meant
to STOP.` or `Meant to FAIL.`, and `EXPECTED OUTPUT:` as comment lines (a
line that is just `#` is a blank output line). For a program meant to
fail, give the whole error, as `lume check <file>` run from the repository
root would print it. Warnings, stops, and anything a program writes to
standard error, go in a separate `EXPECTED STDERR:` block after the output.

No network, no clock in what you print, no reading files that are not in
the repository (a program may write and read its own files under
`/tmp/lume-review-<your letter>/`).

Write `corpus/m131/<letter>-notes.md` too, with three sections: what you read,
a table of your programs, and your guesses, numbered, each with the files
it touches and where you looked.

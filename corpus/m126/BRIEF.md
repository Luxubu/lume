# Round twenty-five — the brief every reviewer was given

This round tests what came after 1.0, in milestones 122–125:

- serving HTTP, a preview: `http.listen`, `Server`, `Incoming`, `Reply`;
- the URL and form helpers in the `http` package, settled: `http.encode`,
  `http.decode`, `http.form`, `http.parse_form`, and the request steps
  `.with_form` and `.with_query`;
- the tutorial's lunches 12, 13 and 14, which were one lunch before;
- a `for` over a list of tasks;
- what an optional or a result says about a method it does not have.

`docs/reference/http.md`, `docs/stability.md` and the tutorial,
`docs/tutorial/`, describe them. Lean on what the pages promise, and say
wherever two pages disagree, or a page promises something no example shows.

**Warnings** go to standard error before anything the program prints, in
the order they are written. Give each, whole, in the `EXPECTED STDERR:`
block, as `lume run <file>` from the repository root names the file
(`--> corpus/m126/a01_name.lume:7:3`).

**A program that stops** writes what it printed first, then the stop to
standard error, and exits with code 101: give it in `EXPECTED STDERR:` and
add `EXPECTED EXIT: 101`. Any other exit code is given the same way, as
`EXPECTED EXIT: 1`.

A program checked another way says so in its header, on one line, as
**"Run with `lume test corpus/m126/<file>`"**, **"Run with `lume check
--json corpus/m126/<file>`"** or **"Run with `lume fmt --stdout
corpus/m126/<file>`"**, and gives the whole output that command prints. A
program that uses a package — `http` and `json` are packages — is a folder
`corpus/m126/<id>_<name>/` with `lume.toml` and `main.lume`; the
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

**Every program is one file**, `corpus/m126/<id>_<name>.lume` (or a folder,
as above). It starts with a header: what it probes, `Meant to RUN.`, `Meant
to STOP.` or `Meant to FAIL.`, and `EXPECTED OUTPUT:` as comment lines (a
line that is just `#` is a blank output line). For a program meant to
fail, give the whole error, as `lume check <file>` run from the repository
root would print it. Warnings, stops, and anything a program writes to
standard error, go in a separate `EXPECTED STDERR:` block after the output.

No network, no clock in what you print, no reading files that are not in
the repository (a program may write and read its own files under
`/tmp/lume-review-<your letter>/`).

Write `corpus/m126/<letter>-notes.md` too, with three sections: what you read,
a table of your programs, and your guesses, numbered, each with the files
it touches and where you looked.

# Round twenty-two — the brief every reviewer was given

This round tests milestones 103–109, and what 0.7 promises:
- **warnings for unused names**: a local given a value and never read, a
  parameter, a `for` variable, a block's parameter, a name a pattern binds;
  `_` and names beginning with `_`; `_` more than once among a block's or a
  `for`'s names (`docs/reference/names.md`, "A name that is never used";
  `examples/unused_names.lume`);
- **an interface requiring an async generator**: `async def items ->
  Iterator[T]` or `def items -> async Iterator[T]`, each type writing its
  own, generic interfaces, `extend`, functions taking the interface value
  (`docs/reference/interfaces.md`, `docs/reference/collections.md`,
  "Async generators"; `examples/async_interfaces.lume`);
- **`Process.spawn` and `Child`**, now settled: `lines`, `error_lines`,
  `read_line`, `read(n)` (which never cuts a character in two), `write`,
  `close_input`, `wait`, `kill`, and the edges the reference now states
  (`docs/reference/io.md`, "A program running alongside");
- **tests**: `async test`, `lume test FILE NAME`, what a failure says, a
  test that stops (`docs/reference/testing.md`);
- **what is settled** (`docs/stability.md`): write programs that lean on
  what it promises, and say where the docs promise something the reference
  does not show;
- everything around them, and the error messages.

**Warnings** go to standard error before anything the program prints, in
the order the compiler finds them. Give each, whole, in the `EXPECTED
STDERR:` block, as `lume run <file>` from the repository root names the
file (`--> corpus/m110/a01_name.lume:7:3`).

**A program that stops** writes what it printed first, then the stop to
standard error, and exits with code 101: give it in `EXPECTED STDERR:` and
add `EXPECTED EXIT: 101`.

A program that probes `lume test` says so in its header ("Run with `lume
test`"), and gives the whole output that command prints. A program that
uses another module may be a package: a folder `corpus/m110/<id>_<name>/`
with `lume.toml` and `main.lume`.

Programs every machine has may be run with `Process.run` and
`Process.spawn` — `echo`, `printf`, `cat`, `sort`, `sh`, `tr`, `wc` — but
keep arguments to plain ASCII, and output must not depend on timing: print
what tasks and programs give back in a fixed order.

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

**Every program is one file**, `corpus/m110/<id>_<name>.lume` (or a folder,
as above). It starts with a header: what it probes, `Meant to RUN.`, `Meant
to STOP.` or `Meant to FAIL.`, and `EXPECTED OUTPUT:` as comment lines (a
line that is just `#` is a blank output line). For a program meant to
fail, give the whole error, as `lume check <file>` run from the repository
root would print it. Warnings, stops, and anything a program writes to
standard error, go in a separate `EXPECTED STDERR:` block after the output.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository (a program may write and read its own files
under `/tmp/lume-review-<your letter>/`).

Write `corpus/m110/<letter>-notes.md` too, with three sections: what you read,
a table of your programs, and your guesses, numbered, each with the files
it touches and where you looked.

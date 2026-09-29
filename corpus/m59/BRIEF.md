# Round twelve — the brief every reviewer was given

This round tests milestones 55–58:

- `Process.run(program, args)`, which runs another program: see
  `docs/reference/io.md`, "Process";
- `map_error` on a `T or Error`;
- a branch that ends in `Env.exit(code)`;
- a type of your own named `Task` (`docs/reference/names.md`);
- `for (i, x) in`;
- `lume check --json` (`docs/install.md`, "Editors").

The error messages matter more than before: `lume lsp` shows them in the
editor as the program is typed.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/`. Nothing else: not the compiler source, not
`tests/`, not `corpus/` (except this brief), not `README.md`, not
`HISTORY.md`.

**Do not run the compiler**, and do not run the programs you write. Every
expected output is worked out by hand from what the docs say. A guess only
counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

A program is one file `corpus/m59/<id>_<name>.lume`, or a folder
`corpus/m59/<id>_<name>/` with `main.lume` (and a `lume.toml` if it is a
package, dependencies beside it by `path`). The file that holds `main` starts
with a header: what it probes, `Meant to RUN.` or `Meant to FAIL.`, and
`EXPECTED OUTPUT:` as comment lines (a line that is just `#` is a blank
output line). For a program meant to fail, give the whole error, as
`lume check corpus/m59/<file>` run from the repository root would print it.

Programs that run other programs use only what every Unix system has:
`sh -c`, `echo`, `printf`, `true`, `false`, `cat`, `wc`, `sort`, `sleep`,
`test`. Output must not depend on timing, the date, or the machine.

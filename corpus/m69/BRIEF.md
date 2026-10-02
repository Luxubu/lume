# Round fourteen — the brief every reviewer was given

This round tests milestones 64–68:
- **iterating types of your own**: `def items -> [T]` (`Iterable[T]`) and
  `def next(var self) -> T?` (`Iterator[T]`), with `for`, chain methods,
  endless sequences, bounds such as `[S: Iterator[Int]]`, and values held as
  the interface (`docs/reference/collections.md`, "Types of your own");
- **interfaces whose required methods take `var self`**
  (`docs/reference/interfaces.md`);
- **integer overflow** under `lume run` (`docs/reference/numbers.md`);
- **tasks**: `spawn:` with and without `await` (`docs/reference/concurrency.md`);
- the error messages.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/` and `packages/`. Nothing else: not the compiler
source, not `tests/`, not `corpus/` (except this brief), not `README.md`, not
`HISTORY.md`, not `CHANGELOG.md`.

**Do not run the compiler**, and do not run the programs you write. Every
expected output is worked out by hand from what the docs say. A guess only
counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

**Every program is one file**, `corpus/m69/<id>_<name>.lume`. It starts with
a header: what it probes, `Meant to RUN.` or `Meant to FAIL.`, and
`EXPECTED OUTPUT:` as comment lines (a line that is just `#` is a blank output
line). For a program meant to fail, give the whole error, as
`lume check corpus/m69/<file>` run from the repository root would print it;
for one meant to stop at run time (an overflow, say), give what `lume run`
prints and say so.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository. Output must not depend on timing: tasks
whose results are printed must be awaited in a fixed order.

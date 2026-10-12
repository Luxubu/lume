# Round sixteen — the brief every reviewer was given

This round tests milestones 75–77:
- **generator methods**: a method of a struct or enum with `yield`, which
  walks its own copy of the value (`docs/reference/collections.md`,
  "Generators: `yield`");
- **lazy chains into generators**: a chain that starts from a generator, a
  range or a sequence of your own stays lazy when given to a generator
  ("Taking a sequence");
- **blocks over tuples**: a block naming one argument for each part of a
  tuple, three or more (`docs/reference/methods.md`);
- **interfaces in interface signatures**: interface methods that take an
  interface value, give one back, or give a sequence `-> Iterator[T]`, and
  generator defaults (`docs/reference/interfaces.md`, "An interface's own
  methods may use interfaces");
- **any `def` that gives `Iterator[T]`**, with or without `yield`;
- the error messages.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/` and `packages/`. Nothing else: not the compiler
source, not `tests/`, not `corpus/` (except this brief), not `README.md`, not
`HISTORY.md`, not `CHANGELOG.md`, not `bench/`.

**Do not run the compiler**, and do not run the programs you write. Every
expected output is worked out by hand from what the docs say. A guess only
counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

**Every program is one file**, `corpus/m78/<id>_<name>.lume`. It starts with
a header: what it probes, `Meant to RUN.` or `Meant to FAIL.`, and
`EXPECTED OUTPUT:` as comment lines (a line that is just `#` is a blank output
line). For a program meant to fail, give the whole error, as
`lume check corpus/m78/<file>` run from the repository root would print it.
Warnings, and anything a program writes to standard error, go in a separate
`EXPECTED STDERR:` block after the output, if you predict any.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository. Output must not depend on timing.

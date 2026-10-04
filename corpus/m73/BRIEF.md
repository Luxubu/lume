# Round fifteen — the brief every reviewer was given

This round tests milestones 70–72:
- **generators**: a `def` that gives `Iterator[T]` with `yield` in its body;
  what they can do (`for`, chains, pairs, `zip`, printing, `next` on a
  `var`), how they move, and where they are refused
  (`docs/reference/collections.md`, "Generators: `yield`");
- **sequence parameters**: `xs: Iterator[T]`, which takes a generator, a
  chain, a list, a set or a type with `next`, by move ("Taking a sequence");
- **`split` and `lines`**, whose lists hold slices of the text when only read
  by position ("When a list is copied, and when it moves") — nothing a
  program prints should depend on it;
- **lists, maps and calls written across lines** ("Lists");
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

**Every program is one file**, `corpus/m73/<id>_<name>.lume`. It starts with
a header: what it probes, `Meant to RUN.` or `Meant to FAIL.`, and
`EXPECTED OUTPUT:` as comment lines (a line that is just `#` is a blank output
line). For a program meant to fail, give the whole error, as
`lume check corpus/m73/<file>` run from the repository root would print it.
Warnings, and anything a program writes to standard error, go in a separate
`EXPECTED STDERR:` block after the output, if you predict any.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository. Output must not depend on timing.

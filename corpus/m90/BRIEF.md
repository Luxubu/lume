# Round nineteen — the brief every reviewer was given

This round tests milestones 87–89:
- **chains over async sequences**: `map`, `filter`, `reject`, `take_while`,
  `take`, `skip` and `enumerate` on an async generator's sequence, and what
  their blocks may use (`docs/reference/collections.md`, "Chains over an
  async sequence");
- **async generator methods**: a method of a struct or enum that is an
  `async def` with `yield` (`collections.md`, "Async generators");
- **`await tasks.next_done`**: whichever task in a `var` list finishes
  first (`docs/reference/concurrency.md`, "Whichever finishes first");
- **generators in an `extend`**, of your own types and of built-in ones
  (`docs/reference/interfaces.md`);
- **generators that take a block or an interface value**, which they keep
  (`collections.md`, "Generators");
- everything around them, and the error messages.

Output must not depend on timing: print what tasks give back in a fixed
order, and when a task or generator waits, make the order of what is
printed follow from the program, not from how long things take. With
`next_done`, the order things finish in may be printed only when the
program makes that order certain (sleeps far apart: 1 ms against 200 ms
is fine; 10 against 20 is not).

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

**Every program is one file**, `corpus/m90/<id>_<name>.lume`. It starts with
a header: what it probes, `Meant to RUN.` or `Meant to FAIL.`, and
`EXPECTED OUTPUT:` as comment lines (a line that is just `#` is a blank output
line). For a program meant to fail, give the whole error, as
`lume check corpus/m90/<file>` run from the repository root would print it.
Warnings, and anything a program writes to standard error, go in a separate
`EXPECTED STDERR:` block after the output, if you predict any.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository (a program may write and read its own files
under `/tmp/lume-review-<your letter>/`). Output must not depend on timing.

Write `corpus/m90/<letter>-notes.md` too, with three sections: what you read,
a table of your programs, and your guesses, numbered, each with the files
it touches and where you looked.

# Round eighteen — the brief every reviewer was given

This round tests milestones 82–85:
- **async generators**: an `async def` that gives `Iterator[T]` with `yield`
  and `await` in its body, walked with `for`, `next` and `.to_list` in async
  code (`docs/reference/collections.md`, "Async generators");
- **tasks and what they capture**: a `spawn:` copies what it uses, except a
  generator, an async sequence or another task, which it takes
  (`docs/reference/concurrency.md`);
- **`Process.run` and `Process.run_in`** (`docs/reference/io.md`, "Process"),
  with programs every machine has (`echo`, `sh -c`, `ls`, `cat`, `printf`);
- everything around them, and the error messages.

Output must not depend on timing: print what tasks give back in a fixed
order, and when a task or generator waits, make the order of what is
printed follow from the program, not from how long things take.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/` and `packages/`. Nothing else: not the compiler
source, not `tests/`, not `corpus/` (except this brief), not `README.md`, not
`HISTORY.md`, not `CHANGELOG.md`, not `bench/`.

**Do not run the compiler**, and do not run the programs you write. Every
expected output is worked out by hand from what the docs say. A guess only
counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

**Every program is one file**, `corpus/m86/<id>_<name>.lume`. It starts with
a header: what it probes, `Meant to RUN.` or `Meant to FAIL.`, and
`EXPECTED OUTPUT:` as comment lines (a line that is just `#` is a blank output
line). For a program meant to fail, give the whole error, as
`lume check corpus/m86/<file>` run from the repository root would print it.
Warnings, and anything a program writes to standard error, go in a separate
`EXPECTED STDERR:` block after the output, if you predict any.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository (a program may write and read its own files
under `/tmp/lume-review-<your letter>/`). Output must not depend on timing.

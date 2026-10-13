# Round seventeen — the brief every reviewer was given

This round tests text processing, where milestones 72 and 80 changed how
Lume works under the hood without changing what a program prints:
- `split` and `lines`, whose lists hold slices of the text when only read by
  position (`docs/reference/collections.md`, "When a list is copied, and
  when it moves");
- a name bound to an item of such a list, and `.slice` read by a text
  method, which are read in place too;
- everything around them: strings (`docs/reference/strings.md`), maps,
  sets, `group_by`, `match` on text, `Result` and `?`, generators reading
  lines, and functions that take and give text.

**What matters most is a program whose output would be wrong if one of these
were read in place when it should have been copied** — text kept in a list,
a field or a map after its source changed or went away, text given to a
function or a task, the same item used twice, a `var` changed while a piece
of it is in use. Write such programs on purpose.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/` and `packages/`. Nothing else: not the compiler
source, not `tests/`, not `corpus/` (except this brief), not `README.md`, not
`HISTORY.md`, not `CHANGELOG.md`, not `bench/`.

**Do not run the compiler**, and do not run the programs you write. Every
expected output is worked out by hand from what the docs say. A guess only
counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

**Every program is one file**, `corpus/m81/<id>_<name>.lume`. It starts with
a header: what it probes, `Meant to RUN.` or `Meant to FAIL.`, and
`EXPECTED OUTPUT:` as comment lines (a line that is just `#` is a blank output
line). For a program meant to fail, give the whole error, as
`lume check corpus/m81/<file>` run from the repository root would print it.
Warnings, and anything a program writes to standard error, go in a separate
`EXPECTED STDERR:` block after the output, if you predict any.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository. Output must not depend on timing.

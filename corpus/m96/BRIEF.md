# Round twenty — the brief every reviewer was given

This round tests milestones 92–95:
- **methods that wait for an async sequence's items**: `first`, `count`,
  `sum`, `any?`, `all?`, `find`, `each`, `fold` (`docs/reference/collections.md`,
  "Methods that wait for the items");
- **`Process.spawn` and `Child`**: a program running alongside — `lines`,
  `error_lines`, `read_line`, `read(n)`, `write`, `close_input`, `wait`,
  `kill` (`docs/reference/io.md`, "A program running alongside");
- **`async Iterator[T]` as a type**: a `def` that hands one on, and a
  parameter that takes one (`collections.md`, "Async generators");
- **the `http` package, streamed**: `http.request`, `.header`,
  `.with_body`, `.timeout`, `send`, `fetch`, `open`, `lines`, `chunks`
  (`docs/reference/http.md`). There is no network and no server: a program
  may build requests and look at them, and may make requests that cannot
  be made (a URL that does not parse; `http://127.0.0.1:9/`, where nothing
  listens), but nothing that needs an answer. A program using `http` is a
  package: a folder `corpus/m96/<id>_<name>/` with `lume.toml` and
  `main.lume`, depending on `packages/http` by path
  (`{ path = "../../../packages/http" }`).
- everything around them, and the error messages.

Programs every machine has: `echo`, `printf`, `cat`, `sort`, `tr`, `wc`,
`head`, `sh -c`, `sleep`, `ls`. Output must not depend on timing: print
what tasks and programs give back in a fixed order, and when something
waits, make the order of what is printed follow from the program, not
from how long things take.

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

**Every program is one file**, `corpus/m96/<id>_<name>.lume` (or, for
`http`, the folder above). It starts with a header: what it probes, `Meant
to RUN.` or `Meant to FAIL.`, and `EXPECTED OUTPUT:` as comment lines (a
line that is just `#` is a blank output line). For a program meant to
fail, give the whole error, as `lume check <file>` run from the repository
root would print it. Warnings, and anything a program writes to standard
error, go in a separate `EXPECTED STDERR:` block after the output, if you
predict any.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository (a program may write and read its own files
under `/tmp/lume-review-<your letter>/`).

Write `corpus/m96/<letter>-notes.md` too, with three sections: what you read,
a table of your programs, and your guesses, numbered, each with the files
it touches and where you looked.

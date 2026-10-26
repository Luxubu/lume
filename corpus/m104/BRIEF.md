# Round twenty-one — the brief every reviewer was given

This round tests milestones 99–102:
- **a chain over several lines**: a line starting with `.name` continues
  the line above, one step a line — after a binding, as an argument, in a
  `for` header, in a method, mixed with `|>`, with blocks, and what `lume
  fmt` does with it (`docs/reference/collections.md`, "A chain too long for
  one line"; `docs/reference/operators.md` for `|>`);
- **structs with no fields**: `struct Marker` alone, or a block of methods
  only, made with `Marker()` — behind an interface, in lists, sets and
  maps, compared, printed, as a field, in an `enum`'s payload
  (`docs/reference/structs-and-enums.md`, "A struct with no fields");
- **a program that stops says where**: `!` on a failure, `xs.at(i)` past
  the end, a division by zero, an overflow, an `assert` outside a test —
  in `main`, a method, a block, a generator, another module, a task, and
  in `lume test` (`docs/reference/errors.md`, "Where the program stopped");
- **the tutorial-round fixes**: `price = price` in a setter is refused,
  a keyword used as a name is called one, `take` and `skip` with a
  negative count, a `Char` key where a `Str` key is wanted;
- everything around them, and the error messages.

**A program that stops** writes what it printed first, then the stop to
standard error, and exits with code 101. Give the stop in the `EXPECTED
STDERR:` block, with the file named as `lume run` from the repository root
would name it (`--> corpus/m104/a01_name.lume:7`), and add `EXPECTED EXIT:
101`. A program in a package (below) names its files from the repository
root too.

A program that uses another module, or that probes `lume test`, may be a
package: a folder `corpus/m104/<id>_<name>/` with `lume.toml` and
`main.lume` (see `docs/reference/packages.md` and `modules.md`). For a
`lume test` probe, say in the header that it is run with `lume test` and
give the whole output that command prints.

Programs every machine has may be run with `Process.run` (`echo`, `printf`,
`cat`, `sort`), but this round is about the language: most programs need
none. Output must not depend on timing: print what tasks give back in a
fixed order.

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

**Every program is one file**, `corpus/m104/<id>_<name>.lume` (or a folder,
as above). It starts with a header: what it probes, `Meant to RUN.`, `Meant
to STOP.` or `Meant to FAIL.`, and `EXPECTED OUTPUT:` as comment lines (a
line that is just `#` is a blank output line). For a program meant to
fail, give the whole error, as `lume check <file>` run from the repository
root would print it. Warnings, stops, and anything a program writes to
standard error, go in a separate `EXPECTED STDERR:` block after the output.

No network, no Rust crates, no clock in what you print, no reading files
that are not in the repository (a program may write and read its own files
under `/tmp/lume-review-<your letter>/`).

Write `corpus/m104/<letter>-notes.md` too, with three sections: what you read,
a table of your programs, and your guesses, numbered, each with the files
it touches and where you looked.

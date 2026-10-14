# lumecheck — what writing it found

`lumecheck` is three of the test suite's own checks, rewritten from Python
into Lume, and now the ones the suite runs:

    tests/lumecheck.sh docs [-v] [-j N]    every example in docs/ against what it claims
    tests/lumecheck.sh coverage [--check N] which compiler messages some test shows
    tests/lumecheck.sh keywords            every keyword is in the VS Code grammar

It is a package of five modules (549 lines): `fences` reads the Lume blocks
of a Markdown file with a generator, `runner` runs one with the compiler and
judges it, `coverage` finds the message templates in the compiler's Rust
source and matches them against what the tests show — without regular
expressions — and `keywords` and `main` are the rest.

Its output was compared with the Python scripts' line for line: the same
415 examples, the same 365 of 380 messages and the same 15 missing, in the
same order. `docs` runs the examples eight at a time on tasks, so it takes
46 seconds where the Python took 2 minutes 19.

## What was wrong, and is fixed

1. **Two programs with the same file name, built at the same time, could
   run each other.** Every example `lumecheck docs` runs is a `doc.lume`,
   eight at once, and thirteen of them printed another example's output.
   Programs built through cargo — those with tasks or crates — share one
   cargo target folder, and the binary was named after the file, so the
   last build of a `doc` won. Two packages each with a `main.lume`, tested
   in parallel, would have done the same. Each program's binary is now
   named after its file and a hash of where its project lives.
2. **A generator from another module lost the type of its items.**
   `for b in fences.blocks(text)` gave a `b` of no known type, and
   `match b.kind:` was refused. A module's exported types were qualified
   with the module's name everywhere but inside a generator, a task, a
   future and a `shared` value; now there too.

## Left as it is

- **Positions in text count characters**, so finding position `i` walks
  the text from the start. Scanning the compiler's 700 KB source by
  `slice` and `index_of` from a position was quadratic and never finished;
  taking `chars.to_list` once and indexing it was instant.
  [Strings](../../docs/reference/strings.md) now says so.
- **`Process.run` has no working directory.** A multi-file example has to
  run in its own folder; the program runs `sh -c 'cd "$0" && ...'`.
- **`where` is a keyword**, so it cannot name a field, a parameter or a
  block's argument; the message said so each time. **There is no character
  literal**: `'_'` was refused with a message pointing at `"_"`, which is a
  `Char` where one is wanted.

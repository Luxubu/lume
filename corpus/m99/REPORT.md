# Milestone 99 — the tutorial dry run: REPORT

Three reviewers worked through *Lume, a Lunch at a Time* as a Python
programmer new to Lume would: in order, typing and running every example,
doing every exercise before reading its answer, and writing down what
slowed them, confused them, or was wrong. A took lunches 1–8, B 9–16, C
17–24. Their notes are `a-notes.md`, `b-notes.md` and `c-notes.md`; the
brief is `BRIEF.md`.

## What they found

**Nearly every example printed what its page shows** — every one in
lunches 9–24, including the network and language-server ones, which C ran
five times. Each lunch fit in an hour except lunch 12 (80–90 minutes) and,
nearly, 6, 10 and 16.

**The docs check had a hole.** An example inside a list item has an
indented fence, and the check read only fences at the start of a line, so
three examples in all of `docs/` were never run — one of them, in lunch
2.6, showing a message Lume does not give. The check now reads indented
fences too.

**Five compiler problems**, each found by doing what a reader would:

| found by | what a reader wrote | what happened | now |
|---|---|---|---|
| A, lunch 6 | a letter count in a `{Str: Int}` over `"banana".chars` | reached rustc | a `Char` key of a `Str`-keyed map is its text |
| A, lunch 8 | `price = price` in a setter, as in Python | compiled, changed nothing | refused; the help says `self.price = price` |
| A, lunch 2 | `for = 3` | "expected a loop variable" | "`for` is one of Lume's own words, so it cannot be a name" |
| B, lunch 12 | `xs.skip(xs.len - 2)` on a one-item list | `take(-1)` gave the whole list | a negative count is none, as `slice` reads one |
| C, lunch 17 | `async def f -> async Iterator[T]` with `yield` | compiled, then "`async async Iterator`" | the same as `-> Iterator[T]` |

**Messages that sent readers the wrong way**, now fixed:

- a missing `await` inside a `spawn:` said to use `spawn:`; in a plain
  `def` it said to `await`, which is not allowed there. Each now says what
  works where the call is.
- `spawn: n += 1` on one line said to wrap it in parentheses (which fails);
  it now says a one-line `spawn:` takes a value, and where the statement
  goes.
- a struct with a field named as an interface's method gave only "has no
  `price` method"; it now says a field is not a method, and what to do.
- a transform's help named the line of the transform, not of the name.

**False sentences**, corrected:

- lunch 13: `Path.join` "puts exactly one `/`" — it adds one when there is
  none, and does not remove a doubled one; now shown with a checked
  example.
- lunch 17: "at most one value ahead" — a value waits in one place while
  the generator makes the next, so it can be two ahead; the reference said
  the same, and both now say what happens (C saw it with four dishes).
- lunch 17's recap named the wrong four waiting methods as the ones that
  stop; lunch 21 said "nothing holds the whole log" while `File.read` does;
  lunch 20 credited list patterns and `map_error` to lunches that never
  taught them.
- lunch 24's exercise 3: completion after `puts double(4).` answers
  nothing — the server answers from the last version that checked, so it
  completes after a name that version had. The exercise and its answer now
  do that, run against `lume lsp`.

**Things used before being taught**, each now explained where it is
first met (Part 3 marks them *new here*): `Char` (3.6), `.to_list` on a set
(6.6), `Hashable` (12.2), `pub` on a struct's members (12.4), checking a
module through `main.lume` (12.4), `!`'s spelling (10.8), `Task[()]` and
`spawn: await` (16.3, 16.6), `Time.sleep` outside async code (16.2),
`zip`, `map_error`, list patterns, `+` on lists, `.error?`, `pad_right`,
`index_of`, `map` on a `T?`, and the `json` package's `Null`, `Arr`, `.str`,
`.int` and `.items`.

**Python habits**, now named where they bite: negative division and `%`,
`round` sending halves away from zero, `xs[-1]` being `None`, `s[a..b]`
including `b`, and a parameter hiding a field.

## Left as they are

- **Runtime errors say no Lume line.** Dividing by a zero that came from
  outside, or `.at` past the end, stops with the error but not where it
  was; the tutorial now says so. Pointing at the Lume line is a compiler
  feature for another milestone.
- **Lunch 12 runs long** (generics, modules, tests and packages). Splitting
  it would renumber every later lunch; that is the author's call.
- **Python's `lambda`** gets no hint toward blocks, where `&&` and `else if`
  get one.

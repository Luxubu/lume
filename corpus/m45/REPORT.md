# Milestone 45 review corpus — REPORT (ninth review round)

This round tested milestones 42–44: `?` inside `spawn:`; interface methods
that take a block, have type parameters of their own, or are `async`; and
what round eight fixed — blocks kept in maps and tuples, `extend` of your own
generic type at chosen arguments, generic inference through blocks. Three
reviewers wrote thirty programs from `docs/` and `examples/` alone — nothing
run — and counted what they had to guess. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a10` | everyday programs: command tables and dispatch maps of blocks, tasks that use `?`, a walker interface |
| `b01`–`b10` | exactly where interface methods with type parameters or `async` may and may not be called; `extend` at chosen arguments and the coherence rule |
| `c01`–`c10` | whole programs, eight of them over several modules, and concurrency — failing tasks, `async` interface methods across files |

## The numbers

| | round eight | round nine |
|---|---|---|
| guesses | 88 | **115** (A 44, B 36, C 35) |
| meant to run, right on the first run | 16 of 23 | **17 of 22** |
| refused as predicted | 6 of 7 | 6 of 8, word for word |
| accepted, then stopped at run time instead of refused | 0 | **2** |
| rustc leak | 2 | 3 |
| refused although correct | 5 | 2 |
| wrong output | 0 | 0 |

After the fixes: **22 of 22** print exactly what their authors worked out by
hand, and all eight refusals give the message their authors predicted —
two of them guessed by analogy, before any page showed them.

The count rose, and the reason is plain in the notes: this round aimed at
features only a few months old, with one example each, and every reviewer
asked for more than one. The guesses about the older language — printing,
strings, collections, modules — stayed near zero.

## What the programs found

1. **The refusal milestone 44 promised had a hole, and two programs fell
   through it** (b05, b10). A generic or `async` interface method is refused
   on a value held through a pointer — but whether a name was an interface
   *parameter* was looked up in a set of names that was never emptied
   between functions, so a loop variable or binding that shared its name
   with a parameter of an earlier function passed as one. Both programs
   compiled and then stopped at run time in the body Lume had called
   unreachable. The set now belongs to one function. This is the worst class
   of defect — accepted, then wrong — and it is why a round follows a
   milestone.
2. **A struct in one module conforming to an interface in another, where
   neither module imports the other**, got no Rust `impl` at all: only the
   interface's module or the type's module wrote one (c01, c08, both rustc
   leaks). Each module now records the impls it writes, and the entry file
   — which sees everything — writes any a conforming pair still lacks.
3. **A block written for a `(Str, ...)` parameter** held its text as a
   borrow, and storing it — `best = path`, `names.push(path)` — reached rustc
   (a04). An old defect, easy to reach now that interfaces take blocks.
4. **A method that calls itself with a block wrapping its own** — a tree
   walk handing each sub-folder a block that prefixes the path — made a new
   closure type at every level, which Rust cannot compile (a04). Such a
   function now takes its blocks as `&mut dyn FnMut`, as a Rust programmer
   would write it.
5. **`?` on a result taken from a list** applied it to a borrow (c03, c10).
6. **A list of `(name, block)` pairs** handed back from a function did not
   tell the blocks their type (c10); **a task whose value came from taking a
   tuple apart** could not tell its own type (c03).

## Where the guesses went

- **Blocks kept in maps and tuples, and `extend` at chosen arguments**
  (asked by all three): one sentence each in the docs, no example. Both now
  have checked examples.
- **Two pages contradicting each other** (asked by two): functions-and-blocks
  said a block returning `T?` gets no implied `Some` and "passes `lume check`
  and then fails to build" — no longer true: the bare value works — while concurrency
  said a task gets it "the same way". Fixed. printing.md said a space before
  `{` after `puts` is ambiguous; only `(` is. Fixed.
- **Why a parameter and a binding of one interface type follow different
  rules**: now said in one paragraph.
- **The mixed-list warning** fired for `[Walk]` and not for `[Mappable[Int]]`;
  it now fires for both.
- **Exact wording** of errors the docs do not quote: all predicted from
  `examples/errors/`, missing only on whether an imported interface is named
  with its module path (it is: `shelf.api.Mappable[Int]`).

## Verdict (condensed from the three notes)

A: everyday Lume is learnable from the docs; the newest features need
examples more than words. B: the rules for where a method may be called are
clear once stated, and were stated only by example. C: concurrency with
failing tasks is now predictable; modules held no surprises.

Nine defects, as in round eight, and again every one sat where features meet.
Two were the kind a user could not have caught: the program compiled.

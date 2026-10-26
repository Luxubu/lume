# Milestone 104 review corpus — REPORT (twenty-first review round)

This round tested what milestones 99–102 added: a chain written one step a
line, structs with no fields, a program that stops saying where, and the
fixes of the tutorial's dry run. Three reviewers wrote thirty programs from
`docs/` and `examples/` alone, with nothing run. The brief is in
`BRIEF.md`; it was written before milestone 103 and names the folder
`corpus/m103`, which became `corpus/m104` when milestone 103 took that
number. The paths in the programs were changed with it, and nothing else.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday code: an order report in long chains, pricing with field-less strategies, stops on two-digit lines, in a block of a chain, in a task and in a generator, two refusals |
| `b01`–`b15` | the edges: comments and blank lines inside a chain, chains in every kind of body, after an inline `if` and mixed with `|>`, a field-less struct in every place a struct goes, stops in a generator, a module, a task, a test and an `assert`, five refusals |
| `c01`–`c06` | whole programs: a pricing engine, a gradebook, a door state machine, a units converter, a stack calculator, a payroll package over two files |

## The numbers

| | round twenty | round twenty-one |
|---|---|---|
| guesses | 34 | **39** (A 11, B 16, C 12) |
| meant to run or stop, right on the first run | 20 of 22 | **17 of 23** |
| refused as predicted | 4 of 8 | **6 of 7**, word for word |
| rustc leak | 1 | **3** (c01, c05, c06: one cause) |
| wrong output | 0 | 0 |

No program printed a wrong answer. After the fixes, all thirty print what
their authors predicted, word for word, standard error and exit codes
included.

The most guesses (12 of 39) were about the new stops, whose docs showed one
example, on a one-digit line: the gutter on a two-digit line, where a
task's stop is told, what an `assert` outside a test says. Nine were about
a chain over lines meeting something else — a comment, a blank line, `|>`,
a call's parentheses, an inline `if`. Seven were about messages for
mistakes, and three about a field-less struct inside other values. The
docs now answer them.

## What the programs found

1. **A map literal of values behind an interface reached rustc** (c01, c05,
   c06 — all three of C's whole programs that hold strategies in a map):
   `Engine(discounts: {"TEN": TenPercent(), "BULK": BulkThree()})`, where
   the field is `{Str: Discount}`. A list literal was built as the
   interface's pointers; a map literal was not. Writing the fix found the
   same gap older and wider: a list or map literal of interface values
   *returned* from a function, or *passed* to a parameter, reached rustc
   too. All four are built as the wanted type now
   (`examples/interface_literals.lume`).
2. **A stop inside a step of a chain written over lines named the
   statement's first line** (a05): `total = lines` rather than `.map { |n|
   n.to_int! }`. Milestone 101 marked statements only. A step on a later
   line now has a line of the generated Rust of its own, marked — and a
   lazy chain, which is written out from its last step, marks its earlier
   steps' blocks back to their own lines.
3. **A task's stop depended on timing** (b08, found when its output was
   recorded): it was told when the task stopped, on its thread, so it could
   come before or after what `main` printed meanwhile, and a task that
   stopped and was never awaited printed `error:` and then exited 0. Both
   reviewers who wrote one (a06, b08) predicted it told where the task is
   awaited, in the form of any other stop — which is what Lume's docs had
   said. It is now: told once, at the `await` that stops the program, as a
   task's `T or Error` comes home there, with the line inside the task. A
   task that stopped and was never awaited is told when the program ends,
   as `error: a task that was never awaited stopped: ...`.
4. **`.name` as the first line of a `do |w|` block** (b11) was joined to
   the `do |w|` line, and said "expected an indented block". The line after
   `:` was already known to begin a block; the line after `do` or `do |x|`
   is now too, with the message the reviewer predicted.
5. **A failed `assert` outside a test said `error: assert not overdrawn?`**
   (b10). It says `error: assert failed: not overdrawn?` now, as the
   reviewer predicted and as Rust's `assertion failed:` reads.
6. **`lume test` named a file by its base name** (b09):
   `b09_test_stop.lume:16:`, where a compile error and a stop name it as it
   was given, `corpus/m104/b09_test_stop.lume:16`. It is named as given
   now, as the reviewer predicted. In a package, run from inside it, that
   is the same as before.

## What the docs now say

- `errors.md`, "Where the program stopped": no column and no caret; the
  gutter as wide as the line number; a type's own function and a step of a
  chain stop on their own lines; what an `assert` outside a test says; a
  task's stop told where it is awaited, and one never awaited told at the
  end (A, B, C).
- `concurrency.md`: a crash in a task, the same (A, B).
- `collections.md`: comments and blank lines between steps are passed over;
  steps inside a call's parentheses; `.name` and `|>` lines mixed; after an
  inline `if`, a `.name` line applies to the last branch; `do` and `do |x|`
  begin a block (A, B, C).
- `structs-and-enums.md`: a field-less struct goes everywhere a struct
  goes, and prints as `Marker()` inside other values (A, B).
- `testing.md`: a failure names the file as it was given (B).

## Left

- C's guess 7, that an unused parameter gets no warning, is right, and no
  page says so either way.
- `lume fmt` still moves a comment between two chain steps to after the
  statement (milestone 102).

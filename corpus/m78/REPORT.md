# Milestone 78 review corpus — REPORT (sixteenth review round)

This round tested milestones 75–77:
- generator methods, which walk their own copy of the value;
- lazy chains handed to generators;
- blocks naming each part of a tuple;
- interfaces whose methods take and give interface values and sequences,
  with generator defaults;
- `-> Iterator[T]` on any function;
- the error messages.

Three reviewers wrote twenty-nine programs from `docs/` and `examples/`
alone, with nothing run, and counted only real uncertainties. The brief is
in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday programs: disk usage over a folder enum, a bookshelf by pages, a cinema seat map, a seeded shuffle, carriers behind an interface, news feeds with an endless ticker, a warehouse manifest, two refusals |
| `b01`–`b14` | the edges: when a generator method copies, generic and enum generator methods, chains into generators with every kind of captured value, triples and quadruples in blocks, every form of interface in a signature, `-> Iterator[T]` from each kind of sequence, eight refusals |
| `c01`–`c06` | whole programs: a file tree, a chess-like board of interface pieces, a document model, an event scheduler over endless sources, a type checker over an expression tree, one refusal |

## The numbers

| | round fifteen | round sixteen |
|---|---|---|
| guesses | 37 | **47** (A 15, B 19, C 13) |
| meant to run, right on the first run | 20 of 21 | **15 of 18** |
| refused as predicted | 9 of 9, 6 word for word | **11 of 11, all word for word** |
| rustc leak | 1 | **3** (a05, b02, c02) |
| wrong output | 0 | 0 |

Every refusal was predicted word for word, the first round where that is
so. All three leaks were older gaps that the new features made easy to
reach, not mistakes in milestones 75–77's own code; each is fixed, and after
the fixes all 29 programs print what their authors worked out, warnings and
standard error included.

## What the programs found

1. **A struct or variant built at the head of an `if`, `while`, `for` or
   `match` reached rustc** (b02: `for x, y in Shape.Rect(w: 2, h: 3).corners`).
   Rust reads a struct literal there as the start of the body. Every
   struct and variant value is now written in brackets, and a `for`'s
   sequence too; `if P(x: 2).big?:` was the same gap, older than generators.
2. **A function giving `Carrier or Error` could not give a plain `Flat`**
   (a05). The implied `Ok` wrapped the value without first making it the
   interface. It is boxed first now.
3. **A list literal moved a value that a later item of it still read**
   (a05: `[post, courier, post.with_surcharge(1)]`). Call arguments already
   counted the arguments after them as later uses; list items do now, both
   for a list of interface values and for any other.
4. **A local moved into a `match`'s head while the arms still read it**
   (c02: `match (first, Some(side)):`, then `side` in an arm). A value is now
   copied when the statement it is given away in mentions it again.

## What the docs now say

- `names.md` listed `yield` among the Rust words that a program may use as
  a name (B). It is a keyword, and the table says so.

## Left open

- The warning for a list holding values of different types behind an
  interface is described only by example (A). Reviewers predicted it from
  the examples' `.expected` files, and all of them got it right.

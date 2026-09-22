# Milestone 30 review corpus — REPORT (sixth review round)

Thirty **multi-file** programs by three independent reviewers, written from
`README.md`, the top-level examples and the two multi-file examples only — no
compiler source, nothing run, every expected output computed by hand first.
Each program is a directory with a `main.lume` and the modules it imports.

| set | reviewer aimed at |
|---|---|
| `p01`–`p10` | module mechanics: import forms, `pub`, nesting, diamonds, cycles, qualified names, collisions |
| `q01`–`q10` | the milestone-29 rule that an `extend` travels with the import |
| `r01`–`r10` | real programs laid out across files the way a person would lay them out |

Plus `p-notes.md`, `q-notes.md` and `r-notes.md`: what each program probes and
ninety-five numbered places the README left them guessing.

## First run

| class | count |
|---|---|
| ok | 11 |
| lume error | 19 |
| rustc leak | 0 |
| crash | 0 |
| wrong output | 0 |

**No leaks and no wrong output on a first run** — the first round where that
has happened. Everything the round found, it found as a refusal: correct code
the compiler would not accept. That is a better failure mode than the last two
rounds and a worse one than it looks, because nineteen of thirty programs
written from the documentation did not run.

## After the fixes

| class | files |
|---|---|
| ok (20) | p01, p03, p04, p08, p10; q01–q06, q08, q09; r01–r03, r05, r06, r08, r10 |
| lume error (10) | p02 p05 p06 p07 p09 q07 q10 r04 r07 r09 |
| rustc leak (0) | — |
| crash (0) | — |

All twenty print exactly what their author worked out by hand. Six of the ten
rejections are the reviewers' deliberate-error programs, and every one of the
six now gives the error its author asked for, in the file and at the line they
named.

## Ranked problems, and what was done

1. **An `extend` travelled exactly one hop.** `main` imports `mid`, `mid`
   imports `base`, and `base`'s conformances did not reach `main`. Three
   reviewers bet a program each on transitivity and all three were right to:
   the impl really is program-wide in the emitted Rust, and the clash rule
   milestone 29 shipped already says *anywhere in one program*. One-hop
   visibility was the compiler contradicting itself. *Fixed:* an `extend` is
   not a name, so it keeps travelling; the interface behind it comes along so
   the conformance can still be honoured, under a key no file can write
   unless it imports that module. The name does not re-export — only the
   methods do.

2. **Two modules could each claim the same type and interface without
   meeting.** The clash rule only fired inside one file. With extends now
   travelling, `metric.lume` and `imperial.lume` both extending `Int` would
   have reached rustc as conflicting impls. *Fixed:* they meet at the import
   that brings them together, and the error names both files and lines —
   which is exactly what `r09`'s author asked for and did not expect to get.

3. **A type could cross two boundaries but not be used.** `report` exports a
   struct whose field is a `money.Money`; `main` imports `report` and never
   imports `money`, so it could hold the value and not call a method on it.
   Two independent programs hit this with the most ordinary layering there
   is. *Fixed:* the types a module names in what it exports travel with it.

4. **A `pub` constant could not be reached from outside** — not by name
   (`import geo.units.METRES_PER_MILE`) and not through an alias
   (`un.METRES_PER_MILE`). Constants were exported and registered, and then
   left out of both lookups. Three programs. *Fixed.*

5. **A private type said "unknown type".** `ledger.Signed` where `Signed` is
   there but not `pub` got the generic unknown-type error and a help about
   structs and enums. The single-name import path already said the right
   thing; the type path did not. *Fixed:* it now says it exists and is not
   `pub`, and names the file to add `pub` in.

6. **`Sized` was reserved with no Lume reason** — the third round running to
   lose a program to a name Rust wants and Lume does not. Every use was a
   `?Sized` bound in the prelude. *Fixed* by qualifying those, which also
   turned `corpus/m26/g05`, rejected two rounds ago for the same reason, into
   a program that now runs and matches its author's output exactly.

7. **A mixed-type list literal was refused where an interface was wanted.**
   `[Shape]` holds mixed types by design, but a literal took its element type
   from its first item and then rejected the rest. *Fixed* for the case it is
   for: a literal whose items disagree, in a position that wants an
   interface.

8. **An interface parameter could not be stored.** Rust takes an interface
   parameter as a generic and a stored one as a pointer to the trait, so
   putting a parameter into a field or a list did not compile. *Fixed* using
   the `lume_box` every interface has carried since the last round.

9. **`self` in an `extend` on a copied built-in came back as a reference.**
   `extend Int with Weighed: def grams -> Int = self` did not compile. *Fixed*
   in one place, which also removed two older double-dereferences that had
   been cancelling each other out.

## Left open, and why

- **`split` drops a trailing empty piece.** `"core:".split(":")` is one piece,
  not two, so the most natural `key: value` parser in the world falls through
  its own `match` — which is what `r04` does. An empty piece *between* two
  separators is kept, so the rule is "trailing only", and it is documented
  nowhere. It is also deliberate, and changing it silently changes what
  existing programs mean: making `split` behave like every other language
  breaks `examples/enums.lume`, which is the proof. **This is a language
  decision, not a bug**, and it wants deciding on purpose rather than inside
  a round aimed at modules. The README now at least states the behaviour.
- **No re-export of names.** Raised first in the p notes and unchanged: a
  consumer of `table` that wants `seq.map` must import `seq` itself, so a
  library's internal layout is part of its public surface. What this round
  added is narrower — the *types* a module names in its exports travel, and
  so do conformances — which is arguably the first third of the answer.
- **A bare zero-argument call** (`score(best_tag)` where `best_tag` is a
  function). Refused with a clear message, but a reviewer reached for it.
- **A qualified function name as a block** (`xs.map(seq.shout)`). A bare name
  works; a module-qualified one does not.
- **A bound naming an imported generic type** (`r07`), and a consumer adding a
  method an imported `extend` already supplies under a different interface
  (`q10`) — the case the clash rule does not cover.

## What the reviewers had to guess

Ninety-five numbered points. The ones raised by more than one:

- **Which side `pad` and `pad_right` fill.** Raised in every round since the
  third, and every table in every program rests on it.
- **Whether a `pub struct`'s fields are readable from an importing file** —
  "only `pub` items cross a file boundary", and a field is not an item. Six
  programs in one set depend on the answer.
- **When a lazy chain needs `.to_list`**, which the examples do both ways.
- **Whether an `extend` may live in a file other than its interface's.**
- **How a value prints** — an optional, a list, a map, a tuple, a `Float`.
  Raised in every round so far, and the reason all three reviewers routed
  their output through `join`, `len` and interpolation.

## Newcomer's verdict (condensed)

The module surface is smaller than it looks and mostly sound: nothing leaked,
nothing produced wrong output, and the errors that did fire pointed at the
right file and line. What it was missing was reach. Four of the nine fixes are
the same shape — something that should cross a boundary and stopped one file
short: a conformance, a type, a constant, an interface. Milestone 29 asked how
far an `extend` travels and answered "with the import"; this round found that
the honest answer is "as far as the program goes", and that three other things
were stopping short in exactly the same way.

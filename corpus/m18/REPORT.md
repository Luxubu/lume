# Milestone 18 review corpus — REPORT (third review round)

Written by an independent reviewer from `README.md` and `examples/` only (no
compiler source, no older corpus). Expected output for every program was
worked out by hand first (`sNN.exp`, `s15.texp` for `lume test`, `s18.exp`
for `mod1/main.lume`). Five times the hand calculation was wrong (s04, s07,
s16, s25, s27); each time the reviewer's arithmetic, not the compiler.

## First run

| class | files |
|---|---|
| ok (14) | s01 s02 s07 s15 s16 s18 s21* s22 s27* s28 s30; error tests s10 s11 s13 (good messages) |
| lume (9) | s06 s08 s09 s14 s19 s23 s24 s25 s26 |
| leak (7) | s03 s04 s05 s12 s17 s20 s29 |
| wrong (0 of 30, 2 repros) | s06b, s08c |
| crash (0 of 30, 1 repro) | s08b |
| fmt (1 repro) | s28b |

\* after rewriting one line each for a documented rule (block `unless`;
string inside a larger `|` alternative).

Checked by hand and correct: set literals, dedup, insertion order (also
after `remove`), `add` returning Bool, all set algebra, `==` ignoring order
(nested in structs, lists, `Some`, maps), reductions on empty and non-empty
sets, sets of tuples/structs/enums, struct/enum/tuple map keys, character
slicing on é, 日本語 and emoji with every clamping edge, slices of slices,
slices in interpolation/`+`/arguments/`match`, recursion over
`xs[1..xs.len]`, sets across modules, through `|>`, in `test` blocks (a
failing assert prints both sets), `shared var {Str}` changed by tasks.
Formatter: idempotent on every file; every formatted program printed the
same output as the original.

## Ranked problems (reviewer's order)

1. Two block `if`s in a row do not parse (s25, s25b) — breaks guard clauses.
2. `{}` is an empty set only in a typed binding or constructor argument;
   as an argument, return value, reassignment, nested literal, or `fold`
   seed it leaks as a map (s05, s05b–d, s24, s24b).
3. `==` on a block parameter leaks for struct/enum items, and for a
   destructured tuple part compared with a literal (s04, s04b, s04c).
4. `{Str}.contains?(w)` with a Str block parameter leaks (s20, s20b).
5. Mutations of temporaries are silently lost, and the help text suggests
   one: `idx[k].or({}).add(x)` (s06, s06b), `for var p in xs[0..1]`
   (s08c), `grid[0].push(9)` / `grid[0][2] = 5` refused (s26, s26b).
6. Built-in method arguments are not type-checked: `s.add("two")` on a
   `{Int}`, `push`, `contains?`, `remove`, `union`, `m[3] = 4` on a
   `{Str: Int}`, mixed list literal `[1, "a"]` (s12, s12b, s12c, s17, s17b).
7. Ownership leaks: `it.name.pad_right(8)` on a field (s29, s29b); changing
   a set/list while looping over it (s21b).
8. `join` on a non-Str set or list (s03, s03b).
9. "Do nothing": `each { |n| s.add(n) }` leaks (block value `bool`), a bare
   `None` arm leaks, and there is no `()` (s20c, s23, s23b).
10. Interface default methods cannot be called on the conforming type, even
    after `extend` (s14, s14b, s14c).

Also: the formatter moves a trailing comment out of a multi-line literal
(s28b); a slice ending at the largest Int stops with "Int overflow in `+`"
(s08b); the `fold({})` hint suggests `{Str: Int}` and says "`this value`"
(s24); `s2 = s.push(3)` on a set says "cannot tell the type"; `Box` is
reserved with no Lume reason given (s19); the mixed set literal error says
"a `Int`"; no tuple destructuring `(k, v) = pair` (s19); lists have no `+`
(s08); a string cannot be one alternative of a larger `|` pattern (s09,
s27); the `shared var` pointer note prints on every run (s30); set
operators `| & -` and `each_with_index` were reached for and missing.

## Newcomer's notes (condensed)

The set surface and character slicing were solid: no wrong set semantics
found, every slicing edge gave the obvious answer, deliberate-rule messages
consistently good, formatter never changed behaviour. Most friction came
from older features sets use heavily: block parameters in `filter`/`reject`
(3, 4) and `{}` outside a typed binding (2). Five of ten realistic programs
needed a workaround. Surprising gaps: no "do nothing", no tuple
destructuring, no list `+`, no way to change a set stored in a map in place
(and the help leads to lost data).

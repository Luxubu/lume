# What six review rounds could not find in the documentation

Eighteen independent reviewers wrote Lume programs from `README.md` alone,
across six rounds, and each recorded what they had to guess. Consolidated and
de-duplicated, it comes to **210 distinct questions**.

That list is the specification for `docs/`. A question is answered when the
answer is somewhere a person would look **and** a runnable example proves it —
`tests/docs.sh` runs every example in these pages and checks its output, so an
answer here cannot quietly stop being true.

The full list, with the reviewers' own phrasing, is in the round reports:
`corpus/m17/`, `corpus/m18/`, `corpus/m26/`, `corpus/m28/` and `corpus/m30/`,
each with a `REPORT.md` and per-reviewer notes. Below are the ones raised more
than once, which are the ones that shaped these pages.

## Raised in three or more rounds

| | question | answered in |
|---|---|---|
| 1 | How does a value print — an optional, a list, a map, a set, a tuple, a struct, a `Float`? *(3 rounds, 9 reviewers, and every report)* | [printing](reference/printing.md) |
| 2 | Which side does `pad` fill, and which does `pad_right`? *(every round since the third)* | [printing](reference/printing.md#pad-and-pad_right) |
| 3 | Does `Int.pad(n)` pad with spaces or zeros? | [printing](reference/printing.md#pad-and-pad_right) |
| 4 | When does a lazy chain need `.to_list`, and which methods end one? *(6 reviewers)* | [collections](reference/collections.md#lazy-chains-and-when-you-need-to_list) |
| 5 | Do `max_by` and `min_by` give `T` or `T?`? | [collections](reference/collections.md#lists) |
| 6 | Is `next` a real keyword, does it work in a `for`, and is there a `break`? | [control flow](reference/control-flow.md) |
| 7 | Does a bare struct *field* satisfy an interface's method requirement? | [interfaces](reference/interfaces.md) |
| 8 | May an interface be a struct *field* type? | [interfaces](reference/interfaces.md) |
| 9 | Where may `{}` and `[]` appear without a type? *(4 rounds)* | [collections](reference/collections.md#where--and--need-a-type) |
| 10 | Which indexing forms reach the item, and which give a `T?`? *(4 rounds)* | [collections](reference/collections.md#lists) |
| 11 | Which names are reserved by the Rust backend? *(4 rounds, and all three reviewers in round seven)* | [names](reference/names.md) |
| 12 | Does a method of your own shadow a built-in of the same name? | [interfaces](reference/interfaces.md) |
| 13 | Can you destructure a tuple — `(a, b) = expr`? | [structs and enums](reference/structs-and-enums.md) |
| 14 | Behaviour cannot be stored in a field or returned — what do you write instead? | since milestone 40 it can: [blocks as values](reference/functions-and-blocks.md#blocks-as-values) |

## Raised in two rounds

| | question | answered in |
|---|---|---|
| 15 | Does `join` work on a list that is not `[Str]`? *(6 reviewers)* | [printing](reference/printing.md#join) |
| 16 | How does `puts` render a `Bool`? | [printing](reference/printing.md) |
| 17 | Does `puts` need parentheses before a `{` or `(`? | [printing](reference/printing.md#a-space-before--or-) |
| 18 | What happens when a value is wider than the pad width? | [printing](reference/printing.md#pad-and-pad_right) |
| 19 | Is there zero-padding, `format`, or printf? | [printing](reference/printing.md#pad-and-pad_right) — no; `decimals` |
| 20 | Is a bare `#` in a string a comment? Does `"##{x}"` lex? | [strings](reference/strings.md) |
| 21 | How does `decimals(n)` round, and what about negatives? | [printing](reference/printing.md#pad-and-pad_right) |
| 22 | Is there a literal for a `Char`? | [strings](reference/strings.md) |
| 23 | What do the character predicates give on an empty string? | [strings](reference/strings.md) |
| 24 | Nested patterns through a recursive enum's field — allowed or an error? | [structs and enums](reference/structs-and-enums.md) |
| 25 | What are `sum`, `avg`, `first`, `max` on an empty list? | [collections](reference/collections.md#lists) |
| 26 | Is `sort` stable, is it ascending, and is the result lazy? | [collections](reference/collections.md#lists) |
| 27 | Is there a descending sort or a comparator? | [collections](reference/collections.md#lists) |
| 28 | Does `xs.sum` follow from a user type's `+`? | [generics](reference/generics.md) |
| 29 | Can a map be iterated with `for k, v in m`? | [collections](reference/collections.md#maps) |
| 30 | Does an `extend` travel one hop or all the way? *(5 reviewers)* | shipped in milestone 30 — all the way |
| 31 | Do interface defaults shadow a container's built-ins? | [interfaces](reference/interfaces.md) |
| 32 | Are two extends with different *bounds* one clash or two claims? | one clash — [generics](reference/generics.md#two-extends-of-one-interface-the-rust-rule) |
| 33 | Does a method on a `pub struct` need its own `pub`? | [interfaces](reference/interfaces.md) |
| 34 | Are import paths program-absolute or relative to the file? | [modules](reference/modules.md) |
| 35–42 | Module questions: aliases, namespaces, name collisions, re-export, `pub` fields, where an `extend` may live, dotted bounds, what the clash error prints | [modules](reference/modules.md) |
| 43 | Does `?` inside a block leave the block or the function? | [functions and blocks](reference/functions-and-blocks.md) |
| 44 | Can you chain onto a call that ends in a trailing block? | [functions and blocks](reference/functions-and-blocks.md) |
| 45 | Can an interface method take a block? | yes, since milestone 44: [interfaces](reference/interfaces.md#methods-that-take-a-block-have-type-parameters-or-are-async) |
| 46 | May a method add its own type parameters? | [generics](reference/generics.md) |
| 47 | Is there a block form of `unless`, and do two block `if`s parse? | [control flow](reference/control-flow.md) |
| 48 | Must a zero-argument *function* be called with `()` while a method need not? | [functions and blocks](reference/functions-and-blocks.md) |
| 49 | Does the `[Shape]` pointer note actually print? | no such note is printed |
| 50 | Is `m[k].push(x)` on a `{K: [V]}` supported? | [collections](reference/collections.md#maps) |
| 51 | What does `xs.at(i)` do when the position is not good? | [collections](reference/collections.md#lists) |
| 52 | Is there a precedence table? | [operators](reference/operators.md) |
| 53 | Is the README's own milestone table internally consistent? | it was not; it is now `HISTORY.md` |

## Still open

Nothing. The last two were decided in milestone 38, by Rust's coherence rule:
two `extend`s of one interface with different bounds are one clash, and when
`extend [T]` and `extend [[T]]` both fit a type neither wins — the program is
refused ([generics](reference/generics.md#two-extends-of-one-interface-the-rust-rule)).

Decided in milestone 37: a module's function is a block by its qualified name
(`xs.map(seq.shout)`, [functions and blocks](reference/functions-and-blocks.md#a-functions-name-as-a-block));
tuples order with `<` as they sort ([operators](reference/operators.md#comparison));
`await f()?` passes on the awaited failure ([concurrency](reference/concurrency.md)).

## Round seven

`corpus/m36/` measured these pages: three reviewers wrote thirty programs from
`docs/` alone. They made **no** guesses about anything in the tables above
that had a page, and 98 about what did not — chiefly the modules and
concurrency pages, which did not exist yet, and a method table, a list of
reserved words and the fate of an overwritten map key, which all three asked
for. Those are now [modules](reference/modules.md),
[concurrency](reference/concurrency.md), [methods](reference/methods.md),
[names](reference/names.md), [numbers](reference/numbers.md),
[patterns](reference/patterns.md) and [input and output](reference/io.md).
See `corpus/m36/REPORT.md`.

## Round eight

`corpus/m42/` measured milestones 37–41: three reviewers, thirty programs,
**88 guesses** — the first round whose count fell. The guesses clustered
where two new features meet (a block in a map or a tuple, a block in a
generic container, an `extend` of the program's own generic type) and on four
pages that contradicted each other about tuples and module functions; those
are fixed. See `corpus/m42/REPORT.md`.

## What writing the answers found

Four programs that the compiler accepted and then could not generate Rust for:
`(1..10).step(2)`, `s.split.first`, a block declared `(A) -> B?` ending in a
bare value, and an interface whose methods are all defaults applied to a
built-in. All four are fixed. Documentation you run is a test suite.

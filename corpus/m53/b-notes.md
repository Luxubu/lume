# Reviewer B, round eleven: break the move rules

## What I read

- `corpus/m53/BRIEF.md`
- `docs/reference/collections.md`, all of it, and closely "When a list is
  copied, and when it moves"
- `docs/reference/functions-and-blocks.md` (kept blocks, what a block copies,
  `?` in blocks), `concurrency.md` (what a task copies, `shared var`, the
  lock, `var self` under one lock, `for` over a `shared var`),
  `structs-and-enums.md`, `control-flow.md`, `patterns.md`, `errors.md`,
  `generics.md`, `interfaces.md`, `methods.md`, `printing.md`,
  `names.md` (self-transforms), `strings.md`, `operators.md`, `numbers.md`,
  `docs/tour.md`
- `examples/moves.lume`, `examples/moves_values.lume` and their
  `.expected`; `examples/ownership.lume`, `kept_blocks.lume`, `async.lume`,
  `structs.lume` (+ `.expected`, for how a struct looks in `#{}`),
  `enumerate_blocks.lume`, the `shared var` parts of `match/main.lume` and
  `port/app_async.lume`

I did not read `docs/QUESTIONS.md`, the compiler, `tests/`, or any other
file in `corpus/`. I did not run the compiler.

## The programs

Every one is Meant to RUN; the expected output is worked out by hand.

| file | what it attacks | leans on |
|---|---|---|
| `b01_while_moves.lume` | a list replaced in a `while` body and read by the condition; a list given to a new name in the body and read by the condition next time round; a list outside a `for` built on inside it; a list used in the iteration that `break`s and read after; a list replaced inside an `if` inside the loop | collections.md "moves ... even inside a loop"; control-flow.md `while`, `break if` |
| `b02_replace_named_twice.lume` | `x = f(x)` where `x` is also in an interpolation argument; `xs = xs.map { .. xs.len .. }`; `xs = xs + xs.map ..`; `s = s + "#{s.len}" + s.upcase`; `acc = acc + w + "#{acc.len}" + w` in a loop (the length must be the old one); an immutable self-transform reading the old value three times; a map and a set replaced by a filter whose block reads them; an `if` value reading the old list three times | collections.md "named twice", "a block uses it", `acc = acc + w`; names.md self-transforms |
| `b03_struct_later_branch.lume` | a struct's fields given away, then the struct printed in the branch not taken, passed on in a `match` arm that runs, given away whole in the other branch of an `if` value, and read afterwards | moves_values.lume; collections.md "a struct nothing needs afterwards" |
| `b04_field_twice.lume` | one field read twice in one statement: given away and upcased, given away and measured, read through a method (`shout`) that reads the same field, read with the whole struct in one interpolation, read inside a filter block and indexed outside it, indexed and given away | collections.md "reads each field of `p` once" |
| `b05_guards_where_q_or.lume` | values read by a `for ... where`, by a match guard, by a guard then replaced by the arm, around `?` on a `T?` and on `T or Error`, and given to two `.or(...)` defaults | control-flow.md `where`; patterns.md guards, one-line assignment arms; errors.md `?`, `.or` |
| `b06_tuples.lume` | a tuple destructured and printed after; `enumerate` pairs kept as a list, walked by `for (i, w)` and printed; a list split by a function and printed; a `var` tuple replaced by one built from its own parts, once each and then one part twice; `partition` destructured; a tuple inside a tuple | structs-and-enums.md tuple destructuring; methods.md tuples, `partition` |
| `b07_nested_paths.lume` | `o.mid.inner.words` taken while `o` is printed later; every leaf of a nested struct given once into a new one; `r.mid` given away whole while `r.mid.inner.note` is read in the same statement; `x.mid` then `x.mid.inner` then `x.title`; `z.title = z.mid.inner.note + z.title` | structs-and-enums.md, collections.md field rules |
| `b08_kept_blocks.lume` | a `var` captured by a kept block then replaced by a value built from it; a kept block whose whole result is the list it captured, called twice; a struct captured then its fields given away after; a parameter captured by a returned block; two kept blocks in a list that each build on one captured list | functions-and-blocks.md "It copies in what it uses when it is made"; collections.md "a block ... can never give away what it holds" |
| `b09_shared_var.lume` | a `shared var` struct replaced by one built from its own fields; a `var self` method that reads `label`, then replaces it by a value built from it; a `shared var` list replaced by a filter of itself, then grown while walked; a `shared var` map changed by key then replaced; the struct lent to a plain parameter; tasks calling the `var self` method | concurrency.md "What the lock covers", "A `shared var` struct and its methods", "Walking a `shared var`" |
| `b10_spawn_moves.lume` | one local mentioned by two tasks and read after; a local copied into a task and replaced there by a value built from it; a struct outside a loop whose fields every iteration's task builds on; a `var` replaced after a task captured it; one-line tasks made in a block, all reading the same list | concurrency.md "What a task sees: copies" |
| `b11_generic_iface_self.lume` | a struct handed to a generic that gives its parameter back twice, to one that gives a parameter back once, to an interface parameter, to an interface bound, and into a `[Named]` list, then printed; `var self` methods that read a field and then replace it, and that replace a list field by a value built from it that names it twice | generics.md, interfaces.md, structs-and-enums.md `var self` |
| `b12_early_return.lume` | `return acc` from inside a loop that goes on to build on `acc`; `return words.at(0)` from one `if`, with `words` read by the lines after; a set read by a `return` inside a loop and by the last line; a field bound to a name that one path gives away and the other reads twice; a `return Error` before a `?` | functions-and-blocks.md `return`; errors.md |

## Guesses

1. **b08: replacing a `var` outside a kept block after the block has
   captured it** (`prefix = prefix + "b"`, `seen = seen + ["q"]`). Looked in
   functions-and-blocks.md "Blocks as values": it says the block copies what
   it uses when it is made, and shows only the refusal of a block *changing*
   the `var`. concurrency.md shows `x = 5` after a `spawn:` captured `x`,
   and says a block copies "as `spawn:` does", so I assumed the same holds
   for a kept block.
2. **b09: `=` on a `shared var` in `main`, outside any `spawn:`**
   (`b = Box(..)`, `xs = xs.filter ..`, `counts = counts.filter ..`).
   concurrency.md shows `=` on a `shared var` only inside `spawn:`, and `+=`
   and methods outside. I assumed `=` works outside too, with the right side
   worked out first ("The right-hand side is worked out *before* the lock
   is taken").
3. **b09: a `shared var` struct passed to a plain `Box` parameter**
   (`show(b)`). concurrency.md shows lending only for a `shared var [Int]`
   passed as `[Int]` (`count(seen)`). I assumed a struct is lent the same way.
4. **b09: a `shared var` map replaced by `counts.filter { .. }`**, the block
   not naming `counts`. concurrency.md refuses only a block that mentions the
   value while a method of it runs; I assumed a block that does not mention
   it is fine, and that `counts["a"] += 1` works outside `spawn:` as it does
   inside (`match/main.lume` shows `per_block[key] = n` inside one).
5. **b02: `acc = acc + w + "#{acc.len}" + w` reads the old `acc.len`
   (`3`, not `4`).** collections.md says `acc = acc + w` "writes `w` onto the
   end of `acc`", which read literally would change `acc` before the
   interpolation is worked out. I took "None of this changes what a program
   prints" and "A list is a value" as the rule, so the result is `x0xy3y`.
6. **b11: a bare `T` parameter given back twice, `(x, x)`, and a `[T]`
   parameter given back as the result.** generics.md says a bare `T` "can
   be moved around"; collections.md says a parameter is borrowed, never
   moved. I assumed the compiler copies a bare `T` as needed (as the
   generics.md `keep` example does when it pushes `x` from a borrowed list),
   rather than refusing.

## Verdict

The rules on the page are enough to work every output out by hand: every
program above prints what it would print if nothing were ever moved, and the
page says that is the rule. What the page does not pin down is only at the
edges of other features: a `shared var` replaced with `=` outside a task,
a `shared var` struct lent to a plain parameter, and a `var` replaced after
a kept block captured it. Each of those is one sentence short in
concurrency.md or functions-and-blocks.md. One sentence in collections.md
invites a wrong reading: "`acc = acc + w` writes `w` onto the end of `acc`"
would be clearer as "reuses `acc`'s storage for the result", so nobody
thinks `acc` changes partway through the statement. The places I expect a
compiler that decides "is this needed again?" to get wrong are b02's
`acc.len` after `acc + w`, b07's `r.mid` given away with `r.mid.inner.note`
in the same statement, b04's field read through a method (`d.shout`) beside
the field itself, b08's kept block `h` that hands back what it holds, and
b01's list read by a `while` condition after being given away in the body.

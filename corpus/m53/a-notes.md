# Reviewer A: notes

## What I read

- `corpus/m53/BRIEF.md`.
- `docs/README.md`, `docs/tour.md`.
- All of `docs/reference/collections.md` (most closely "When a list is copied, and when it moves"), `methods.md`, `printing.md`, `control-flow.md`, `strings.md`, `names.md`, `structs-and-enums.md`, `functions-and-blocks.md`, `numbers.md`, `operators.md` and `patterns.md`. The first half of `errors.md`, and the "Constants" section of `modules.md`.
- Examples: `moves.lume`/`.expected`, `moves_values.lume`/`.expected`, `enumerate_blocks.lume`, `collections.lume`, `sets.lume` (and the head of `sets.expected`), `structs.lume`, `stdlib.lume`, `csvq/csvq/query.lume`, and the `for var` part of `ownership.lume`. I also grepped `examples/` for `+=` on text and for `x.field.push(..)`.
- I did not run the compiler or read anything else.

## The programs

| file | what it is | leans on |
|---|---|---|
| a01_report.lume | a sales report built line by line into one `var` string, then a summary built from a map | `for (i, s) in .enumerate`, `report = report + ..` also inside an `if`, `pad`/`pad_right`, `repeat`, reading `lines`/`first` on the finished text, a list passed to a function and read again, `max_by` on a map |
| a02_roster.lume | a player roster filtered and re-sorted step by step | `players = players.filter{..}.to_list` / `sort_by` repeated, reassignment inside an `if` branch whose block reads a `var`, `first` kept across a later replacement, a value from the list read in a block that replaces the list, `match` arms reassigning a `var` in a loop |
| a03_grouping.lume | orders grouped into maps, then the maps filtered | `group_by`, a tally with `.or(0) + ..`, `totals = totals.filter{..}`, a filtered copy while the original is still used, `map_values` to a map of sets then `cities = cities.filter`, `sort_by` on a map, tuple destructuring in a loop, the first map read at the end |
| a04_pipeline.lume | Raw -> Parsed -> Priced -> Label, each struct built from the fields of the one before | building a struct from another's fields, both where the source is read afterwards (`first.name`) and where it is not (`last`), a struct passed to a function and then read, list `+` inside a constructor, `for .. where` |
| a05_wordcount.lume | a word count with sets and a map | `out = out + c` (Str + Char), a top-level constant set used in a loop and in a block, the Bool that `add` returns, `counts = counts.reject{..}`, `keys.to_set`, set `==`/`diff`/`intersect`, the word list read again after everything built from it |
| a06_inventory.lume | an inventory: a struct holding a list of items and a log | `var self` methods doing `items = items.map{..}.to_list` / `filter` and reading `items` again, a guarded `match` arm, an argument pushed into a field and then read again, the caller's value read after it was passed |
| a07_recursion.lume | recursive functions that return lists: tree leaves and paths, collatz, subsets | `leaves(l) + leaves(r)`, `[n] + collatz(..)`, `without + taken` where a block also reads `without`, a `var out` extended by recursion, `evens = evens.sort` |
| a08_todo.lume | a todo board: a struct holding tasks, each holding tags | a `var self` method that pushes and returns a bumped field, `for var` over a field, an enum status, `var open` re-sorted into itself, `by_tag[tag].push(..)`, text built in both branches of an `if` |
| a09_csv.lume | CSV text -> rows -> a markdown table, a CSV summary, a raised copy | a list pattern on `split(",")`, returning a `([Row], Int)` tuple, `(rows, skipped) = ..` then `rows = rows.sort_by`, text built up and then `trim_right`, a new list of structs from the old one's fields with the old list read afterwards |
| a10_gradebook.lume | a gradebook: students with score lists, curved and regraded | a struct passed to functions and read again, a string interpolated, printed and then pushed, `match` reassigning a `var` that a later block reads, `students = students.map{ curve(..) }.to_list`, `while` with `at`, `group_by` |

## Guesses

1. **Assigning a whole field with plain `=` inside a `var self` method** (a06: `items = items.map { .. }.to_list` and `items = items.filter { .. }.to_list`). I looked in `structs-and-enums.md` under "var self", "Changing a field" and "Calling another method of the same type", and in `examples/structs.lume` and `sets.lume`. The docs only show `n += by`, `x += dx` and `items.push(..)` on a bare field, and `p.x = 9` on a `var` binding. I assumed a bare `field = value` assigns the field, as `+=` does, and does not bind a new local.
2. **`for var` over a list field inside a `var self` method, then assigning a field of each item** (a08: `for var t in tasks:` then `t.status = next_status(t.status)`). `control-flow.md` "`for var`" and `examples/ownership.lume` only show `for var` over a local `var` list, with a `var self` method called on each item. I assumed it also works on a field of a `var self`, and that `t.field = ..` counts as changing the item.
3. **A self-transform on a name bound by tuple destructuring** (a09: `(rows, skipped) = parse_rows(data)` and then `rows = rows.sort_by { .. }`). `names.md` "Binding a name again" allows `name = name.method` for a name bound without `var` in the same block. `structs-and-enums.md` "Tuple destructuring" says the names it binds are immutable, but neither page says whether they can be self-transformed. I assumed they can.
4. **The format of the expected-output header.** The brief says output goes in comment lines, with `#` alone for a blank line. I wrote every output line as `# ` plus the text. The brief says nothing more exact, and I looked at no other corpus files.

Everything else I checked against a doc or an example: the pad direction, `decimals`, `Str + Char`, `+=` on text, set `==` ignoring order, `group_by` key order, map order after `filter`, `max_by` on a map, `split` pieces, `"""` indent stripping, one-line assignment arms, and `x.field.push` on a `var` struct.

## Verdict

The docs were enough to write ordinary data-flow code with only 3 language guesses. All three are the same kind of gap: **changing things that live inside a struct**. That means a bare field assigned with `=`, `for var` over a field, and a destructured name transformed again. These are also the spots where the move rules matter most, since a field is always copied and never moved, so they are good probes. The move section in `collections.md` is clear on which reads cause a copy. But it does not say what happens to a *field* replaced by a value built from itself (`items = items.filter{..}` inside a method), and a06 depends on that answer. It is worth one line and one example there, plus one sentence in `structs-and-enums.md` saying that `field = value` works in a `var self` method.

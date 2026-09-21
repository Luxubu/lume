# m26 review round — notes

Twelve whole programs, written from `README.md` and the six example files only.
No compiler source read, nothing run. Every expected output in the headers was
worked out by hand (and the arithmetic/padding double-checked independently).

## What each program probes

| file | what it probes |
| --- | --- |
| `m01_wordfreq.lume` | a top-level `{Str}` constant, triple-quoted text, `lines`/`split`, `m[w] = m[w].or(0) + 1`, `for ... where`, `pad_right`, `at`, and `Float.decimals` on an exact binary half (9/8 = 1.125 → must be `1.13` if "halves away from zero" is real; Rust's and Python's own formatters say `1.12`) |
| `m02_ledger.lume` | operator methods `+`, `-`, `<` on a struct, the derived `>=`/`==`, `sort` on a user type that only defines `<`, an interface default reached by structural conformance with no `extend`, `if` as an expression across two lines |
| `m03_inventory.lume` | two enums matched together as a tuple, `\|` alternatives *inside* a tuple pattern, a guard that must not count as coverage, exhaustiveness over `(Category, Stock)`, `find` → `T?`, `?` in a `T?` function, `or_error` then `?`, `return Error(...)` as an early exit |
| `m04_report.lume` + `mod/units.lume` + `mod/table.lume` | three files importing each other, `import a.b.Name`, `import ... as`, a module-qualified type in a typed binding and in a struct field, a module-qualified keyword constructor, `pub def` beside plain `def` methods on a `pub struct` |
| `m05_dates.lume` | `test "...":` blocks with `assert` and `assert not`, `!` inside a test, a top-level `[Int]` constant, `at` in the "position already known good" role, `for ... where`, `return Error(...)`, a method calling a top-level function |
| `m06_config.lume` | `index_of` as an `Int?` matched with `Some`/`None`, `trim_left.trim_right` chained, a tuple destructured inside an `Ok(...)` pattern, `Char` predicates through `chars.all?`/`chars.count`, `next` in a `for`, insertion-ordered `{Str: Str}`, clamped and empty slices |
| `m07_life.lume` | **awkward**: Life on `[[Int]]`, `-1..1` as a range, `next` in doubly nested `for`s, a `var` list rebuilt each iteration, a block inside a block, a function name as the inner block, `match` on `(Bool, Int)` with `\|`, a `{(Int, Int)}` grown inside a `while`, `pop` → `T?` |
| `m08_sales.lume` | **awkward**: two functions of my own taking blocks, a block parameter handed straight on to a built-in (`xs.sum(f)`), `_` shorthand, `group_by`, `partition`, `fold(init) { }`, `sort_by(...).reverse`, `x \|> puts` and `x \|> f(y) \|> puts`, a `do \|x\|` block whose value is bound, a list used again after being passed |
| `m09_versions.lume` | `interface Comparable[T]` with a default written in terms of `T`, structural conformance without naming the interface, `[T: Comparable[T]]`, `extend Int with Comparable[Int]`, `struct Stack[T]` with `var self` methods, `?` on `xs.first` inside a `T?` function, `.map` on a `T?` |
| `m10_rooms.lume` | `{Str: {(Str, Int)}}` grown by `m[k].add(v)` from a missing key, tuples of mixed types as set items, 3-tuples as set items *and* list items, `intersect`/`diff`/`subset?`, `add`'s return value with a trailing `unless`, `for ... where`, `avg.decimals` |
| `m11_gradebook.lume` | **expects a compile error**: two immutable bindings assigned from inside an `if` inside a `for` — the exact case the README's "declare it `var`" rule names. Everything else in the file is meant to be valid, so the error should be those two lines and no others |
| `m12_split_bill.lume` | **awkward**: `trim_left`/`trim_right` on lines padded with spaces *and tabs*, slicing on both sides of a separator, integer `/` and `%` to split money with no float drift, `at` inside `enumerate`, a list used after being bound to another name, mutation in nested loops, name reuse across sibling scopes, and `decimals(0)` on `0.5`/`1.5`/`2.5`/`-2.5` |

## Things in the README I found ambiguous, surprising or under-specified

1. **`s[i]` is documented as two different types.** The `Char` paragraph says
   "`s[i]` is a `Char?`"; the positions paragraph says "`xs[i]` and `s[i]` are a
   `T?` / `Str?`". `examples/sets.lume` asserts `"héllo"[1] == Some("é")`, which
   fits either. I assumed `Char?` that compares equal to a one-character `Str`.
2. **Nested patterns through a recursive enum field: the README says both.**
   Milestone 20's row says nested patterns "reach inside the pointer a
   recursive field lives behind"; the Ownership paragraph says "a nested
   pattern on such a field is an error that says to `match` the field in the
   arm". I avoided relying on either, so no program here settles it — but one
   of those two sentences is wrong.
3. **The body of an `extend` on a built-in is never shown.** `extend Int with
   Measures[Str]:` appears with no body anywhere. There is no field to name
   bare, so `m09_versions.lume` writes `self`. Nothing in the README
   introduces `self` as an expression — it only ever appears as `var self` in a
   parameter list.
4. **Module import paths inside a module.** `import users.model` loads
   `users/model.lume`, but relative to what? `mod/table.lume` needs
   `mod/units.lume`; I wrote `import mod.units` (program-relative). If it is
   file-relative it should be `import units`. The README's `examples/modules/`
   layout does not disambiguate, since `app.lume` sits at the root.
5. **Does a method of a `pub struct` need its own `pub`?** "Only `pub` items
   cross a file boundary" plus milestone 24's "`pub def`" suggests it might.
   `mod/units.lume` uses plain `def` for methods on a `pub struct`.
6. **Block `unless`.** Only *trailing* `if`/`unless` is listed; `while` and
   `if`/`elif`/`else` have block forms. I avoided `unless cond:` with an
   indented body in `m10_rooms.lume` and used the trailing form instead.
7. **`fold`'s shape.** Listed only as a name. `m08_sales.lume` guesses
   `xs.fold(0) { |acc, x| ... }` — an init argument *plus* a trailing block,
   which is a form the README never shows (it only says the parentheses go
   "when the block is the only argument").
8. **`next` is never actually documented.** `continue` is listed among the
   rejected spellings "with the Lume form", and `examples/chars.lume` uses
   `next`, so I used it — including inside a `for` (the example only shows it
   inside a `while`) and inside two nested `for`s.
9. **`decimals` on values that are not exact halves in binary.** "Halves away
   from zero" is not implementable on a binary float for e.g. `1.005` or
   `2.345`, which are actually below the half. I stuck to exactly representable
   halves (`0.5`, `1.5`, `2.5`, `-2.5`, `1.125`) so the expectation is
   unambiguous — but those are exactly the values where Rust's own `{:.n}`
   disagrees with the README, so I expect at least one of these to differ.
10. **`round` on a `Float`.** Listed under "on `Int`/`Float`", so it is unclear
    whether it returns a `Float` or an `Int`. I avoided it entirely and used
    integer cents in `m12_split_bill.lume`.
11. **`slice(a, b)`.** `examples/stdlib.lume` uses `w.slice(0, 1)`, where
    "offset, length" and "start, end" give the same answer. I avoided it and
    used `s[a...b]` everywhere.
12. **`Str.pad` pads on the left, `pad_right` on the right** — clear from
    `examples/stdlib.lume`, but `Int.pad(n)` is listed with no example at all,
    so whether it pads with spaces or zeros is unknown. I avoided it and built
    `"0#{n}"` by hand in `m05_dates.lume` and `m02_ledger.lume`.
13. **What `puts` prints for a `T?`, a `T or E`, a list, a map, a set, a tuple
    or a `Float`** is never written down, even though the examples print all of
    them. I kept almost everything inside `#{}` or joined it by hand so the
    expected output does not depend on a derived format I had to invent. The
    one place I could not avoid it is `puts` on a `Bool` (assumed `true` /
    `false`) and on a bare `Int`.
14. **Multi-statement `match` arms.** Every example arm is a single expression.
    I wanted one in `m05_dates.lume` and rewrote it as an `if` chain because
    nothing says a `->` arm may take an indented block.
15. **`join` on a `[Int]`.** `join` is listed for lists without saying the items
    must be `Str`. `m12_split_bill.lume` writes `shares.join(" ")` on a `[Int]`
    (and `m09_versions.lume` deliberately writes the safe long form for
    contrast), so the pair should show which one is real.
16. **`keys` / `values` / `split` / `sort_by` — list or lazy?** `idx.keys.sort`
    works in `examples/collections.lume`, and `"a b".split.to_list` appears in
    `examples/stdlib.lume`, so `.to_list` is sometimes needed and sometimes
    not. I wrote `.to_list` before `at`/`join` and not before `sort`, which is
    a guess.
17. **Does `and`/`or` short-circuit?** Never stated. `m07_life.lume` relies on
    it only in a spot that is unreachable anyway, but a program that guards an
    index with `i >= 0 and xs.at(i) == v` is extremely common and the README
    does not promise it is safe.
18. **A range with a negative bound** (`for dr in -1..1`) is never shown;
    `m07_life.lume` uses it.
19. **The directory is called `mod/`**, which is a Rust keyword. If module
    paths become Rust module paths, that name is the first thing that will
    break — the task asked for this directory name, so it is worth flagging.
20. **Does a *field* satisfy an interface's method requirement?** The README
    says "a type conforms by having the methods". `m02_ledger.lume` therefore
    gives `Line` a real `def amount -> Money` rather than relying on a field
    named `amount`. If fields do count, that is a second, undocumented path to
    conformance.
21. **A multi-line `if`/`else` used as a value.** The README shows
    `if n < 2: n` / `else: ...` only as the tail of a function, where `else`
    sits at the statement's own indent. Assigning that to a binding
    (`extra = if cond: 1` / `else: 0`) needs a continuation rule that is not
    written down. `m12_split_bill.lume` guesses "align `else` under `if`"; the
    other eleven programs route around it through a one-line helper function,
    which is what I would actually do if I could not find the answer.
22. **Shadowing across sibling scopes.** "Nested rebinding is an error" and
    "no shadowing" are stated for outer names, but two sibling `for` loops each
    binding `i`, or a `var row` inside one loop and a `row` loop variable in the
    next, are not obviously covered. `m12_split_bill.lume` does both on purpose.

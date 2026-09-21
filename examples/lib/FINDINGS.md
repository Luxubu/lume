# What writing a library in Lume exposed

The method of milestones 19–21: write the real thing first, keep a log of
every workaround, then turn the log into the fix list. This time the
program is a *library* — code whose only job is to be imported — because
that is what generics (22) and blocks (23) were for.

Written: `seq.lume` (24 generic sequence helpers), `table.lume` (a typed
table over CSV, built on `seq`), `report.lume` (a program that uses
both). 380 lines, 19 tests.

The headline is that the two new features held: every helper in `seq` is
generic and takes a block, `table` calls them across a module boundary,
`report` calls both, blocks nest inside blocks, and methods that take
blocks chain (`t.where { }.sorted_by { }.render`). Nothing here needed a
workaround for generics or for blocks themselves.

What it did expose was the surface around them: the parser refusing
names a library wants, and three holes in the standard library that only
show up when the code cannot see its own types.

---

## 1. `pub def` on a method is refused, with a message about fields

The first ten lines of any library:

```ruby
pub struct Table:
  columns: [Str]

  pub def len -> Int:      # error: field `pub` needs a type
    rows.len
```

`pub` on a method is redundant — a method of a public struct is public
with it — but it is what an author writes, and the error blames a field
that was never there. Same for `pub` on a field.

**Fixed:** `pub def` on a method is accepted and means what it looks
like. `pub` on a field or an enum variant is now its own error: *a field
is public with its struct, so it takes no `pub`*.

## 2. `where` cannot be a method name

The most natural name for "the rows that match" is `where`, and `where`
is a keyword (`for x in xs where cond`). After `def` inside a struct, and
after a `.`, there is nothing a keyword could be confused with.

Renamed to `where_TMP` to keep going; the name is the one the library
wants.

**Fixed:** a method name may be any of the keywords a field name may
already be (`where`, `next`, `match`, `in`, `test`, `import`, `pub`,
`extend`, `interface`, `assert`). After `def` inside a type, and after a
`.`, there is nothing else it could mean. `table.where` is now its real
name.

## 3. No way to index a list you have already checked

`xs[i]` gives a `T?`, which is right. But in generic code there is no
value to fall back on: `.or(default)` needs a `T`, and `T` could be
anything. Both `seq.zip_with` and `seq.sort_with` ended up writing

```ruby
out.push(f(xs[i].or_error("unreachable")!, ys[i].or_error("unreachable")!))
```

after a `while` that had already checked the bound. Non-generic code
hides this, because there is always some default to write (`.or(0)`,
`.or("")`). A library cannot.

**Fixed:** `xs.at(i)` gives the item itself. It is the read that says "I
have checked this", and when that is not true it stops the program the
way an out-of-range write already does: `no item at 7 — the list has 3`.
Both helpers now read as they meant to:

```ruby
out.push(f(xs.at(i), ys.at(i)))
```

## 4. `!` warnings from a library land in the consumer's build

The line above warns — reasonably. But the warning appeared while
running `report.lume`, pointing at `seq.lume:156`, a file the author of
the report never opened and cannot change. Three of them, on every
build.

**Half fixed.** The warning already names the file, so the consumer can
see whose line it is. What was missing was something to do about it: the
warning on `xs[i]!` now reads *when the position is already known to be
good, write `xs.at(0)`*, and with `at` in the language the library has
no `!` left in it.

## 5. No `trim_left` / `trim_right`

`table.render` pads every cell and then wants the trailing spaces off
the end of the line. `trim` takes both ends. Written by hand in
`table.lume` as a five-line loop.

**Fixed:** `trim_left` and `trim_right` beside `trim`.

## 6. No way to print a number to two decimal places

Every report prints money. `f.to_s` gives `7360.0`, and the only way to
two places was `(f * 100.0).round.to_float / 100.0`, which still prints
`$7360.0`. This is the most-wanted missing method in the whole exercise.

**Fixed:** `x.decimals(n)` gives the number as text to `n` places, on
`Float` and on `Int`, rounding halves away from zero the way money is
expected to: `1234.5.decimals(2)` is `"1234.50"`, `0.125.decimals(2)` is
`"0.13"`. `money` is now `"$#{f.decimals(2)}"`.

---

## Noted, not fixed

- **No re-export.** `table` imports `seq`, but a consumer of `table` that
  wants `seq.map` must import `seq` itself, so a library's internal
  module layout is part of its public surface.
- **No interface over "anything you can iterate".** Every helper in `seq`
  takes a `[T]`. Making one work on a `{K: V}` or a `{T}` as well would
  need an interface with a type parameter, which interfaces cannot take.
  That is the next real gap, and it is bigger than anything above.

---

## What the fixes came to

Two in the parser (`pub def`, keywords as method names), three in the
standard library (`at`, `trim_left`/`trim_right`, `decimals`), one in a
warning's help text. Nothing in the type system: generics and blocks
carried a real library without a single workaround, which is the result
this milestone was looking for.

After the fixes the library reads the way it was meant to, and has no
`!`, no hand-written string trimming and no float arithmetic standing in
for a format.

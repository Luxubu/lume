# A library written in Lume

Three files, written as a library author would write them, to find out
what milestones 22 (generics) and 23 (blocks) left missing.

| file | what it is |
| --- | --- |
| `seq.lume` | 24 sequence helpers, generic over the item type, all taking blocks |
| `table.lume` | a typed table over CSV, built on `seq` |
| `report.lume` | a program that uses both: totals per team, the busiest people, checks |
| `FINDINGS.md` | every workaround the writing exposed, and what was done about it |

```
lume run examples/lib/report.lume              # the built-in sample
lume run examples/lib/report.lume -- hours.csv # a file of your own
lume test examples/lib/report.lume             # 14 tests, across all three files
```

`seq` knows nothing about what it holds:

```ruby
pub def group_with[T, K: Hashable](xs: [T], key: (T) -> K) -> {K: [T]}:
  var out: {K: [T]} = {}
  for x in xs:
    out[key(x)].push(x)
  out
```

`table` calls it across a module boundary, and hands a block straight
through:

```ruby
pub def where(ok: ([Str]) -> Bool) -> Table:
  Table(columns: columns, rows: seq.keep(rows, ok))
```

and `report` calls both:

```ruby
by_team = seq.group_with(people) { |p| p.team }
busiest = seq.sort_with(people) { |a, b| a.hours > b.hours }
puts Table(columns: ["name", "team", "cost"], rows: rows).render
```

What it cost: six findings, all fixed in milestone 24 — see `FINDINGS.md`.

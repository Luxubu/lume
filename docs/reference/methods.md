# Every method, by type

> Reviewers in every round asked "does a list have `last`?", "is there
> `insert`?", "does `max_by` work on a chain?" — and no page listed them.
> This one does.

Each section below is one type. The tables give the name, what it takes,
what it gives back, and one line on what it does. Examples follow for the
ones whose behaviour is not obvious from the name.

In the tables, `T` is the item type, `U` is whatever the block gives back, and
"block" means `{ |x| ... }` or the `_` shorthand.

## Asking the compiler

**Call a method that does not exist, and the error lists every method the
type has.** This works for any value, and it is the list the compiler itself
checks against:

```lume-bad
xs = [1, 2]
puts xs.frobnicate
#! `[Int]` values have no method `frobnicate`
#! `[Int]` has: len, empty?, any?, all?, first, last, max, min, sum, sort, sort_by, reverse, push, pop, contains?, join, map, filter, reject, each, count, find, take, skip, take_while, fold, min_by, max_by, enumerate, to_list, zip, flatten, uniq, index_of, insert, remove_at, avg, group_by, partition, flat_map, to_set, at, to_s, to_str
```

A name close to a real one gets a suggestion instead of the list:

```lume-bad
xs = [1, 2]
puts xs.revers
#! did you mean `reverse`?
```

Three types answer differently:

- A `T?` or a `T or Error` says to unwrap it first, because it may not hold a
  value. Their methods are in the tables below.
- A map's list leaves out the methods that take a block and work on its
  `(key, value)` pairs (`map`, `sort_by`, `max_by`, `min_by`, `fold`,
  `flat_map`, `group_by`, `partition`). They work. The [map](#k-v-maps)
  table has them all.
- A tuple has no methods. Asking gives an internal error, not a list.

## `Str`

[Strings](strings.md) covers `Str` in full, and "Everything `Str` has" there
lists its methods. In short:

| method | takes | gives | does |
|---|---|---|---|
| `len` | | `Int` | the number of characters, not bytes |
| `empty?` | | `Bool` | true for `""` |
| `upcase`, `downcase`, `capitalize` | | `Str` | changes case; `capitalize` lowercases the rest |
| `trim`, `trim_left`, `trim_right` | | `Str` | drops whitespace from both ends, the left, the right |
| `contains?`, `starts_with?`, `ends_with?` | `Str` | `Bool` | looks for a piece of text |
| `index_of` | `Str` | `Int?` | the character position of the first match |
| `replace` | `Str, Str` | `Str` | every match of the first replaced by the second |
| `slice` | `Int, Int` | `Str` | start and *length*, in characters |
| `repeat` | `Int` | `Str` | the text that many times |
| `reverse` | | `Str` | the characters backwards |
| `chars` | | `[Char]` | the characters, as a list |
| `split` | | chain of `Str` | the words, split on whitespace, empty pieces dropped |
| `split` | `Str` | chain of `Str` | every piece between separators, empties kept |
| `lines` | | chain of `Str` | the lines |
| `to_int` | | `Int or Error` | reads a whole number |
| `to_float` | | `Float or Error` | reads a number |
| `digit?`, `alpha?`, `space?` | | `Bool` | true when every character is one; false for `""` |
| `pad`, `pad_right` | `Int` | `Str` | spaces on the left, on the right ([printing](printing.md)) |
| `to_s`, `to_str` | | `Str` | the text itself |

`s[i]` gives a `Char?`, and `s[a..b]` a `Str`.

## `Char`

A `Char` comes from `s.chars` or `s[i]`. It is written like a one-character
`Str`.

| method | takes | gives | does |
|---|---|---|---|
| `digit?`, `alpha?`, `space?`, `alnum?` | | `Bool` | what kind of character it is |
| `upper?`, `lower?` | | `Bool` | its case |
| `upcase`, `downcase` | | `Str` | the other case, as text (one character can become two) |
| `code` | | `Int` | the Unicode code point |
| `pad`, `pad_right` | `Int` | `Str` | as for `Str` |
| `to_s`, `to_str` | | `Str` | a one-character `Str` |

The way back from a code is `Int.to_char`, below.

```lume
c = "Ab1".chars
puts c.map(_.code).join(",")      #=> 65,98,49
puts c.map(_.alnum?).join(",")    #=> true,true,true
puts c.map(_.upper?).join(",")    #=> true,false,false
puts 65.to_char                   #=> Some("A")
```

## `Int`

| method | takes | gives | does |
|---|---|---|---|
| `abs` | | `Int` | the value without its sign |
| `max`, `min` | `Int` | `Int` | the larger, the smaller of the two |
| `clamp` | `Int, Int` | `Int` | held between a low and a high bound |
| `pow` | `Int` | `Int` | raised to a power |
| `even?`, `odd?` | | `Bool` | |
| `to_float` | | `Float` | the same number as a `Float` |
| `to_int` | | `Int` | itself |
| `to_char` | | `Char?` | the character with this code, if there is one |
| `decimals` | `Int` | `Str` | the number as text with that many places |
| `pad`, `pad_right` | `Int` | `Str` | as for `Str` |
| `to_s`, `to_str` | | `Str` | the digits |

There is no `sqrt`, `round`, `floor` or `ceil` on an `Int`. Convert with
`.to_float` first. [Numbers](numbers.md) has the details: overflow, division,
`%`, and what stops the program.

## `Float`

| method | takes | gives | does |
|---|---|---|---|
| `round` | | `Float` | the nearest whole number; a half goes away from zero |
| `floor`, `ceil` | | `Float` | the whole number below, above |
| `to_int` | | `Int` | drops the fraction (toward zero) |
| `to_float` | | `Float` | itself |
| `sqrt` | | `Float` | the square root |
| `abs` | | `Float` | the value without its sign |
| `max`, `min` | `Float` | `Float` | the larger, the smaller of the two |
| `clamp` | `Float, Float` | `Float` | held between a low and a high bound |
| `pow` | `Float` | `Float` | raised to a power, which is a `Float` too |
| `decimals` | `Int` | `Str` | the number as text with that many places |
| `pad`, `pad_right` | `Int` | `Str` | as for `Str` |
| `to_s`, `to_str` | | `Str` | as `puts` prints it |

**`round`, `floor` and `ceil` give a `Float`.** Only `to_int` crosses over to
an `Int`. See [numbers](numbers.md) for how floats print and sort.

## `Bool`

| method | takes | gives | does |
|---|---|---|---|
| `pad`, `pad_right` | `Int` | `Str` | as for `Str` |
| `to_s`, `to_str` | | `Str` | `true` or `false` |

`and`, `or` and `not` are operators, not methods.

## `[T]`: lists

| method | takes | gives | does |
|---|---|---|---|
| `len` | | `Int` | how many items |
| `empty?` | | `Bool` | no items |
| `any?` | | `Bool` | at least one item |
| `any?`, `all?` | block | `Bool` | whether some, every item passes |
| `count` | block | `Int` | how many items pass |
| `first`, `last` | | `T?` | the first, last item |
| `at` | `Int` | `T` | the item at a position; stops the program if there is none |
| `xs[i]` | `Int` | `T?` | the item at a position, if there is one |
| `find` | block | `T?` | the first item that passes |
| `index_of` | `T` | `Int?` | the position of the first equal item |
| `contains?` | `T` | `Bool` | whether an equal item is there |
| `max`, `min` | | `T?` | the largest, smallest item |
| `max_by`, `min_by` | block | `T?` | the item with the largest, smallest key; the **first** one on a tie |
| `sum` | block, optional | `T` | the total — of the items, or of what the block gives each; `0` when empty |
| `avg` | | `Float` | the mean, of an `[Int]` or a `[Float]`; `0.0` when empty |
| `fold` | start, block | the start's type | a running total: the block takes the total and an item |
| `join` | `Str` | `Str` | the items as text with the separator between |
| `sort` | | `[T]` | ascending, and stable |
| `sort_by` | block | `[T]` | ascending by the key the block gives, and stable |
| `reverse` | | `[T]` | the items backwards |
| `uniq` | | `[T]` | the first of each equal item, in order |
| `map` | block | chain of `U` | each item changed |
| `filter`, `reject` | block | chain of `T` | the items that pass, that fail |
| `take`, `skip` | `Int` | chain of `T` | the first `n`, all but the first `n` |
| `take_while` | block | chain of `T` | items from the start while they pass |
| `enumerate` | | chain of `(Int, T)` | each item with its position, the position first |
| `zip` | `[U]`, a range or a sequence | `[(T, U)]`; a chain when called on one | pairs items up; stops at the shorter side, so an endless sequence on either side is fine |
| `flat_map` | block giving a list | `[U]` | each item becomes a list; the lists are joined |
| `flatten` | | `[U]` | a `[[U]]` joined into one list, one level deep |
| `group_by` | block | `{K: [T]}` | items gathered under the key the block gives |
| `partition` | block | `([T], [T])` | the items that pass, and the rest |
| `to_set` | | `{T}` | the distinct items |
| `to_list` | | `[T]` | a copy |
| `each` | block | | runs the block on each item |
| `push` | `T` | | adds to the end (`var` list only) |
| `pop` | | `T?` | removes and gives the last item (`var` list only) |
| `insert` | `Int, T` | | puts an item before a position; the length is allowed (`var` only) |
| `remove_at` | `Int` | `T` | removes and gives the item at a position (`var` only) |
| `to_s`, `to_str` | | `Str` | as `puts` prints it |

`at`, `insert` and `remove_at` stop the program when the position is out of
range. `take` and `skip` never do: `[1, 2].take(10)` is the whole list.

`count` needs a block. For the number of items, use `len`.

**A block with two names takes a pair apart**, on every method that takes a
block. That goes for the tuples in a `[(A, B)]`, for `enumerate`, and for a
map's `(key, value)`:

```lume
pairs = [("ann", 3), ("bo", 1), ("cy", 3)]
puts pairs.filter { |name, n| n > 1 }.map { |name, n| name }.join(",")    #=> ann,cy
puts pairs.max_by { |name, n| n }       #=> Some(("ann", 3))
puts pairs.min_by { |name, n| n }       #=> Some(("bo", 1))
puts pairs.sort_by { |name, n| n }      #=> [("bo", 1), ("ann", 3), ("cy", 3)]
```

`ann` and `cy` tie on 3: `max_by` gives `ann`, the first. `sort_by` keeps them
in the order they came, because it is stable.

**A tuple of three or more is taken apart the same way**, one name for each
part, as `for` does:

```lume
rows = [("ann", 3, true), ("bo", 5, false)]
puts rows.filter { |name, n, ok| ok }.map { |name, n, ok| "#{name}#{n}" }.to_list   #=> ["ann3"]
```

The ones that build something new:

```lume
xs = [3, 1, 3, 2]
puts xs.uniq                                #=> [3, 1, 2]
puts xs.enumerate.to_list                   #=> [(0, 3), (1, 1), (2, 3), (3, 2)]
puts xs.group_by { |n| n % 2 }              #=> {1: [3, 1, 3], 0: [2]}
puts xs.partition { |n| n > 2 }             #=> ([3, 3], [1, 2])
puts xs.flat_map { |n| [n, n * 10] }        #=> [3, 30, 1, 10, 3, 30, 2, 20]
puts [[1, 2], [3]].flatten                  #=> [1, 2, 3]
puts xs.zip(["a", "b"])                     #=> [(3, "a"), (1, "b")]
puts xs.take_while { |n| n > 2 }.to_list    #=> [3]
puts xs.to_set                              #=> {3, 1, 2}
puts xs.avg                                 #=> 2.25
```

`group_by` keeps its keys in the order they were first seen.

The ones that change a `var` list:

```lume
var ys = [1, 2, 3]
ys.insert(1, 10)
puts ys                  #=> [1, 10, 2, 3]
ys.insert(4, 99)
puts ys                  #=> [1, 10, 2, 3, 99]
puts ys.remove_at(0)     #=> 1
puts ys.pop              #=> Some(99)
puts ys                  #=> [10, 2, 3]
```

`sort`, `reverse` and `uniq` give a real list, so what they give can be
indexed and, once bound to a `var`, changed:

```lume
var s = [3, 1, 2].sort
puts s.remove_at(0)    #=> 1
puts s.at(1)           #=> 3
```

## Lazy chains

`map`, `filter`, `reject`, `take`, `skip`, `take_while` and `enumerate` give a
lazy chain, as do `Str.split` and `Str.lines`
([collections](collections.md) explains why).

**Every list method that does not change the list works directly on a chain.**
That includes `max_by`, `min_by`, `avg`, `to_set`, `find`, `group_by`,
`at` and `[i]`. `for` walks a chain too, and a `match` list pattern takes one
apart:

```lume
xs = [3, 1, 2]
puts xs.map { |n| n * 2 }.max_by { |n| 0 - n }    #=> Some(2)
puts xs.map { |n| n * 2 }.avg                     #=> 4.0
puts xs.map { |n| n % 2 }.to_set                  #=> {1, 0}
puts xs.map { |n| n * 2 }.at(0)                   #=> 6
for i, n in xs.map { |n| n * 10 }.enumerate:
  puts "#{i}: #{n}"      #=> 0: 30
                         #=> 1: 10
                         #=> 2: 20
for n in xs.take(2):
  puts n                 #=> 3
                         #=> 1
```

**The four that change a list do not work on a chain**: `push`, `pop`,
`insert` and `remove_at`. The change would be made to a copy nobody keeps, so
the compiler stops you:

```lume-bad
xs = [3, 1, 2]
xs.map { |n| n * 2 }.push(8)
#! would change a temporary copy
```

**Binding a chain to a name gives a list.** From then on it is a list in every
way. It can be walked twice, indexed, and changed if the binding is `var`:

```lume
xs = [3, 1, 2]
var doubled = xs.map { |n| n * 2 }
doubled.push(8)
puts doubled.len    #=> 4
puts doubled        #=> [6, 2, 4, 8]
```

In an error message a chain is named like a list (`[Int]`), and its "has:"
line is the list's without those four.

## `{K: V}`: maps

| method | takes | gives | does |
|---|---|---|---|
| `len` | | `Int` | how many keys |
| `empty?` | | `Bool` | no keys |
| `any?` | | `Bool` | at least one key |
| `m[k]`, `get` | `K` | `V?` | the value for a key, if there is one |
| `contains?` | `K` | `Bool` | whether the **key** is there |
| `keys` | | `[K]` | the keys, in order |
| `values` | | `[V]` | the values, in the same order |
| `to_list` | | `[(K, V)]` | the pairs, in order |
| `merge` | `{K: V}` | `{K: V}` | both; where a key is in both, the argument's value wins |
| `filter`, `reject` | block of `k, v` | `{K: V}` | the pairs that pass, that fail |
| `map_values` | block of `v` | `{K: U}` | the same keys with each value changed |
| `map` | block of `k, v` | `[U]` | a list of what the block gives |
| `flat_map` | block of `k, v` | `[U]` | as for a list |
| `any?`, `all?` | block of `k, v` | `Bool` | whether some, every pair passes |
| `count` | block of `k, v` | `Int` | how many pairs pass |
| `find` | block of `k, v` | `(K, V)?` | the first pair that passes |
| `max_by`, `min_by` | block of `k, v` | `(K, V)?` | the pair with the largest, smallest key; the first on a tie |
| `sort_by` | block of `k, v` | `[(K, V)]` | the pairs sorted by the key the block gives |
| `group_by`, `partition` | block of `k, v` | as for a list of pairs | |
| `fold` | start, block | the start's type | the block takes the total and a `(K, V)` pair |
| `each` | block of `k, v` | | runs the block on each pair |
| `remove` | `K` | `V?` | removes a key and gives its value (`var` map only) |
| `m[k] = v` | | | sets a key (`var` map only) |
| `to_s`, `to_str` | | `Str` | as `puts` prints it |

A map has no `first`, `sum`, `join`, `sort`, `take` or `enumerate`. Go
through `keys`, `values` or `to_list` for those.

**A map keeps the order keys were first put in.** Setting a key that is
already there changes its value and keeps its place. Removing a key closes the
gap, and putting it back puts it at the end:

```lume
var m: {Str: Int} = {}
m["a"] = 1
m["b"] = 2
m["a"] = 9
puts m                   #=> {"a": 9, "b": 2}
m["c"] = 3
puts m.remove("a")       #=> Some(9)
m["a"] = 4
puts m                   #=> {"b": 2, "c": 3, "a": 4}
```

The pair methods:

```lume
m = {"ann": 3, "bo": 1, "cy": 3}
puts m.max_by { |k, v| v }                 #=> Some(("ann", 3))
puts m.sort_by { |k, v| v }                #=> [("bo", 1), ("ann", 3), ("cy", 3)]
puts m.map { |k, v| "#{k}=#{v}" }.join(" ")    #=> ann=3 bo=1 cy=3
puts m.filter { |k, v| v > 1 }             #=> {"ann": 3, "cy": 3}
puts m.map_values { |v| v * 10 }           #=> {"ann": 30, "bo": 10, "cy": 30}
puts m.find { |k, v| v == 1 }              #=> Some(("bo", 1))
puts m.merge({"bo": 5, "dee": 2})          #=> {"ann": 3, "bo": 5, "cy": 3, "dee": 2}
```

## `{T}`: sets

| method | takes | gives | does |
|---|---|---|---|
| `len` | | `Int` | how many items |
| `empty?` | | `Bool` | no items |
| `any?` | | `Bool` | at least one item |
| `contains?` | `T` | `Bool` | whether the item is there |
| `add` | `T` | `Bool` | adds; `true` if it was new (`var` set only) |
| `remove` | `T` | `Bool` | removes; `true` if it was there (`var` set only) |
| `union`, `intersect`, `diff` | `{T}` | `{T}` | in either, in both, in this one only |
| `subset?`, `superset?` | `{T}` | `Bool` | whether every item is in the other, or the other way round |
| `first` | | `T?` | the item added earliest |
| `max`, `min` | | `T?` | the largest, smallest item |
| `sum` | | `T` | the total |
| `sort` | | `[T]` | a sorted list |
| `sort_by` | block | `[T]` | a list sorted by the key the block gives |
| `max_by`, `min_by` | block | `T?` | the first on a tie |
| `filter`, `reject` | block | `{T}` | a set of the items that pass, that fail |
| `map`, `flat_map` | block | `[U]` | a **list** of what the block gives |
| `any?`, `all?`, `count`, `find` | block | as for a list | |
| `fold`, `join`, `group_by`, `partition`, `take_while` | | as for a list | |
| `each` | block | | runs the block on each item |
| `to_list` | | `[T]` | the items, in the order they were added |
| `to_s`, `to_str` | | `Str` | as `puts` prints it |

A set keeps the order items were added, and `for`, `to_list`, `join` and
`puts` all walk it in that order. It has no position, so there is no `at`,
`last`, `take`, `skip`, `reverse` or `enumerate`. Use `to_list` or `sort`
first.

`filter` keeps a set a set. `map` gives a list, because two items may map to
the same value:

```lume
s = {3, 1, 2}
puts s.filter { |n| n > 1 }        #=> {3, 2}
puts s.map { |n| n % 2 }           #=> [1, 1, 0]
puts s.map { |n| n % 2 }.to_set    #=> {1, 0}
var t = {1, 2}
puts t.add(3)                      #=> true
puts t.add(3)                      #=> false
```

A `Float` cannot go in a set or be a map key. Floats have no exact equality.

## `T?`: optionals

| method | takes | gives | does |
|---|---|---|---|
| `or` | `T` | `T` | the value, or the default when it is `None` |
| `some?`, `none?` | | `Bool` | which it is |
| `map` | block | `U?` | changes the value if there is one |
| `or_error` | `Str` | `T or Error` | `None` becomes a failure with that message |
| `to_s`, `to_str` | | `Str` | `Some(...)` or `None` |

Anything else needs the value first: `match`, `.or(default)`, or `?`
([errors](errors.md)).

```lume
price: Int? = Some(4)
none: Int? = None
puts price.map(_ * 2)               #=> Some(8)
puts none.map(_ * 2)                #=> None
puts none.or_error("no price")      #=> Error("no price")
```

## `T or Error`: results

| method | takes | gives | does |
|---|---|---|---|
| `or` | `T` | `T` | the value, or the default when it failed |
| `ok?`, `error?` | | `Bool` | which it is |
| `ok` | | `T?` | the value, if it succeeded |
| `error` | | `Error?` | the failure, if it failed |
| `map` | block | `U or Error` | changes the value if it succeeded; a failure passes through |
| `map_error` | block | `T or Error` | replaces the failure with the `Error` the block gives; a value passes through (Rust's `map_err`) |
| `to_s`, `to_str` | | `Str` | `Ok(...)` or `Error(...)` |

An `Error` has one field, `message`, a `Str`.

```lume
good = "42".to_int
bad = "x".to_int
puts good.map { |n| n + 1 }    #=> Ok(43)
puts bad.map { |n| n + 1 }     #=> Error("`x` is not an integer")
puts good.ok                   #=> Some(42)
puts bad.ok                    #=> None
puts bad.error                 #=> Some(Error("`x` is not an integer"))
```

## Tuples

A tuple has no methods. Its parts are `.0`, `.1` and so on. It prints, it
compares with `==`, and it can be a map key or a set item when its parts can.

**A list of tuples sorts lexicographically**: by the first part, then the
second on a tie, and so on. `max` and `min` follow the same order:

```lume
t = (1, "two", 3.0)
puts t.1                                   #=> two
puts t == (1, "two", 3.0)                  #=> true
puts [(2, "b"), (1, "z"), (2, "a")].sort   #=> [(1, "z"), (2, "a"), (2, "b")]
puts [(2, "b"), (2, "a")].max              #=> Some((2, "b"))
```

`<`, `<=`, `>` and `>=` compare two tuples in the same order, part by part
([operators](operators.md#comparison)), and a tuple holding a `Float` sorts
like any other:

```lume
puts [(2.5, "b"), (0.5, "a")].sort         #=> [(0.5, "a"), (2.5, "b")]
p = (1, "a")
puts p < (1, "b")                          #=> true
```

A tuple is taken apart by `(a, b) = t`, by a block with a name for each part, or by
`match` ([patterns](patterns.md)).

## Everything

`to_s` and `to_str` are on every type and are the same: the value as
`puts` would print it.

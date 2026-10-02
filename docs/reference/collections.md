# Lists, maps and sets

## Lazy chains, and when you need `.to_list`

> Six reviewers across three rounds could not derive the rule and inserted
> `.to_list` defensively everywhere.

**You almost never need it.** `map`, `filter`, `reject`, `take`, `skip` and
friends build a chain that compiles to one Rust loop; anything that needs an
answer ends the chain itself. `join`, `sum`, `len`, `sort`, `first`, `max`,
`min`, `count`, `any?`, `all?` and `fold` all work directly on one:

```lume
xs = [3, 1, 2]
puts xs.map { |n| n * 2 }.join("-")        #=> 6-2-4
puts xs.map { |n| n * 2 }.sum              #=> 12
puts xs.map { |n| n * 2 }.len              #=> 3
puts xs.map { |n| n * 2 }.sort.join("-")   #=> 2-4-6
```

`.to_list` is for when you want the list itself — to store it, return it, or
index it. On something that is already a list it is a copy, not an error.

## Lists

```lume
xs = [3, 1, 2]
puts xs.len                #=> 3
puts xs.sort.join(",")     #=> 1,2,3
puts xs.sort.reverse.join(",")   #=> 3,2,1
puts xs.contains?(2)       #=> true
puts xs.index_of(2).or(0 - 1)    #=> 2
puts [1, 2].join("+")      #=> 1+2
```

**`sort` is ascending and stable.** There is no `sort_by_desc`; `.reverse`
after a sort is the descending form.

**A literal may span lines.** Inside `[...]`, `{...}` or the parentheses of
a call, a line break does not end the statement, so long lists, maps and
argument lists are written one item to a line. A comma after the last item
is fine, and `lume fmt` writes one:

```lume
moves = [
  ("north", 3),
  ("east", 1),
]
puts moves.len    #=> 2
```

A position may be out of range, so `xs[i]` gives a `T?`. When you know it is
good, `.at(i)` gives the item and stops the program if it is not:

```lume
xs = [10, 20, 30]
puts xs[1].or(0)    #=> 20
puts xs[9].or(0)    #=> 0
puts xs.at(1)       #=> 20
```

A list you can change is a `var`, and a position can be assigned:

```lume
var ys = [1, 2, 3]
ys[0] = 9
ys.push(4)
puts ys.join(",")   #=> 9,2,3,4
```

**`max_by` and `min_by` give a `T?`**, like `max` and `min`, because a list
may be empty:

```lume
words = ["bb", "a", "ccc"]
puts words.max_by { |w| w.len }    #=> Some("ccc")
puts words.min_by { |w| w.len }    #=> Some("a")
```

On an empty list, `sum` is `0` and `first`/`max`/`min` are `None`.

`fold` takes the starting value and a block of the running total and the item:

```lume
puts [1, 2, 3].fold(0) { |total, n| total + n }    #=> 6
puts [1, 2, 3].fold(1) { |total, n| total * n }    #=> 6
```

## Maps

**A map keeps the order you put things in**, and `keys`, `values`, `to_list`
and `for` all walk it in that order. A tally prints in the order it was built.

```lume
var m: {Str: Int} = {}
m["b"] = 2
m["a"] = 1
puts m.keys.join(",")      #=> b,a
puts m.len                 #=> 2
for k, v in m:
  puts "#{k}=#{v}"         #=> b=2
                           #=> a=1
```

`for k, v in m` works, and so does a block taking two names. A missing key
gives nothing rather than stopping the program, so reading one is `.or`:

```lume
ages = {"ada": 36}
puts ages["ada"].or(0)     #=> 36
puts ages["nobody"].or(0)  #=> 0
```

Writing through a missing key starts from an empty value, which is what makes
a tally or an index a single line:

```lume
words = ["a", "b", "a"]
var counts: {Str: Int} = {}
var index: {Str: [Int]} = {}
for i, w in words.enumerate:
  counts[w] = counts[w].or(0) + 1
  index[w].push(i)
puts counts["a"].or(0)        #=> 2
puts index["a"].or([]).len    #=> 2
```

`contains?` on a map asks about a **key**. `m.to_list` gives `[(K, V)]`.

## Sets

```lume
tags = {"red", "blue", "red"}
puts tags.len                    #=> 2
puts tags.contains?("red")       #=> true

a = {1, 2, 3}
b = {2, 3, 4}
puts a.union(b).len              #=> 4
puts a.intersect(b).len          #=> 2
puts a.diff(b).len               #=> 1
puts a.subset?({1, 2, 3, 4})     #=> true
```

A duplicate in a set literal is dropped silently. An empty set and an empty
map are both written `{}`, so an empty one needs its type said somewhere — a
typed binding, a field, or a constructor argument:

```lume
none: {Str} = {}
puts none.len    #=> 0
```

## Types of your own: `items`

**A type with `def items -> [T]` is looped over with `for`, and takes every
method a list has**, working on `items`. This is Lume's `Iterable[T]`, as
Rust's `IntoIterator`:

```lume
struct Shelf:
  books: [Str]

  def items -> [Str] = books.sort

def main:
  s = Shelf(books: ["pear", "fig", "apple"])
  for b in s:
    puts b                 #=> apple
                           #=> fig
                           #=> pear
  puts s.filter { |b| b.len > 3 }.to_list    #=> ["apple", "pear"]
  puts s.len                                 #=> 3
```

- **Its own methods come first:** a method or field of the type's own with
  the same name as a list method (`len`, `first`) is used instead.
- **Pairs:** `def items -> [(Str, Int)]` makes `for k, v in value` work, as
  over a map.
- **Generic code:** a bound `[C: Iterable[Int]]` takes any such type, and
  lists and sets as well. A value can be held as an `[Iterable[Int]]`.
- **Not `for var`:** `items` gives a new list, so there is nothing in place
  to change. Change the type itself with a `var self` method.
- **More:** `examples/iterable.lume` shows each of these.

## Types of your own: `next`

**A type with `def next(var self) -> T?` is a lazy sequence**: `for` and
every chain method work it out one item at a time, and it ends at the first
`None`. This is Lume's `Iterator[T]`, and in the Rust it compiles to, the
type *is* an `Iterator`:

```lume
struct Fib:
  a: Int
  b: Int

  def next(var self) -> Int?:
    x = a
    a = b
    b = x + b
    Some(x)

def main:
  fib = Fib(a: 0, b: 1)
  puts fib.take(8).to_list                          #=> [0, 1, 1, 2, 3, 5, 8, 13]
  puts fib.filter { |x| x % 2 == 0 }.take(4).to_list #=> [0, 2, 8, 34]
  puts fib.find { |x| x > 100 }                      #=> Some(144)
  for x in fib:
    if x > 3:
      break
    puts x                 #=> 0
                           #=> 1
                           #=> 1
                           #=> 2
                           #=> 3
```

- **Every method a list has works**, except the ones that change a list in
  place (`push`, `pop`, `insert`, `remove_at`). `len`, `count`, `last`,
  `max`, `uniq`, `group_by` and the rest work out the items first.
- **Endless is fine.** Nothing is worked out before it is asked for, so
  `take`, `take_while`, `skip`, `enumerate`, `find`, `first`, `any?`,
  `all?`, `zip` and `break` stop a sequence that never ends, after asking for
  only as many items as they need. A method that needs every item
  (`to_list`, `len`, `sum`, `sort`) on an endless one never returns.
- **`zip`** pairs a sequence with a list, a range, or another sequence, and
  stops with the shorter. A list zipped with an endless sequence stops with
  the list.
- **Pairs:** a `next` that gives `(A, B)?` makes `for a, b in value` and
  two-name blocks work, as `items` of pairs do.
- **A loop or chain walks a copy.** `fib` above is still at its start after
  each line, as a list is after a `for`. To move a value along yourself,
  call `next` on a `var`: `var f = Fib(a: 0, b: 1)`, then `f.next` gives
  `Some(0)` and `f` has moved on. A loop or chain after that copies `f` as
  it is now, so it starts at `1`. The same holds for a parameter, a value
  held as an `Iterator[Int]`, and a value a task takes with it: the walk
  never changes the original. A `var s: S` parameter changes the caller's
  value only through `s.next`.
- **After `None`**, `next` is called again only by you, and gives whatever
  your code gives: Lume does not remember that it ended.
- **The shape is exact:** `next` takes nothing but `var self` and gives an
  optional. A `next` of another shape is just a method, and a `for` over the
  type says what is wrong with it.
- **Not both:** a type with `items` and `next` is refused, since a `for`
  over it could mean either.
- **Not `for var`:** the items are worked out, not stored.
- **Generic code:** a bound `[S: Iterator[Int]]` takes any such type, and
  `for` and chains work on the parameter. A value can be held as an
  `Iterator[Int]`. An interface of your own may also ask for a `var self`
  method; each type that fits writes it.
- **Speed:** a chain over a sequence is the Rust iterator chain you would
  write by hand; `bench/b6_iterators.lume` measures it against a hand-written `impl Iterator`.
- **More:** `examples/iterators.lume` shows each of these.

## Generators: `yield`

**A function with `yield` in it is a generator.** It says it gives
`Iterator[T]`, and each `yield` hands out the next item. Nothing in it runs
until an item is asked for, and it stops where it is until the next one is,
so a generator may go on for ever. In Rust it is a function that gives
`impl Iterator`, with the body compiled to a state machine:

```lume
def naturals -> Iterator[Int]:
  var n = 0
  while true:
    yield n
    n += 1

def words(text: Str) -> Iterator[Str]:
  for w in text.split(" "):
    yield w.upcase if w.len > 2

def main:
  puts naturals().filter { |n| n % 3 == 0 }.take(4).to_list   #=> [0, 3, 6, 9]
  puts words("a cat sat on the mat").join(" ")                #=> CAT SAT THE MAT
  for n in naturals():
    break if n > 1
    puts n             #=> 0
                       #=> 1
```

What a sequence of your own does, a generator does: `for`, every chain
method, pairs, `zip`, printing (its items), and `next` on a `var`. Given
where a list is wanted, it gives its items. Where it differs:

- **It is used up as it is walked**, as a Rust iterator is moved. After
  `for x in g`, `g.sum` or `puts g`, the name `g` is gone, and using it again
  is refused. Keep the items with `.to_list` to walk them twice. A sequence
  of your own (`def next(var self)`) is copied instead, because its state is
  fields that can be copied; a generator's state is where its body stopped.
- **So it is kept in a local, or handed on.** Not in a list, a tuple or a
  field, and not as an `Iterator[Int]` value or a `[S: Iterator[Int]]`
  argument, which are copied. A parameter `xs: Iterator[T]` takes it (below).
- **It has its own copy of its arguments,** taken at the call, since it runs
  after the call has returned. A `var` parameter is refused.
- **`return` alone ends it.** Its items come only from `yield`.
- **`yield` belongs to the generator's own body**: its loops, `if`s and
  `match`es, not a block it hands to a method (`xs.each { .. }`) or a
  `spawn:`. Use `for` there instead.
- **Not yet:** a method with `yield`, an `async` generator, and a generator
  that takes a block or an interface value.

### Taking a sequence: `xs: Iterator[T]`

**A parameter declared `Iterator[T]` takes any sequence of `T`s, by move**,
as Rust's `impl Iterator<Item = T>` does: a generator, a lazy chain, a list,
a set, or a type of your own with `next`. Inside, it is walked like a
generator, once, and `xs.next` works on it as it is. So generators chain
into each other, lazily:

```lume
def naturals -> Iterator[Int]:
  var n = 0
  while true:
    yield n
    n += 1

def evens(xs: Iterator[Int]) -> Iterator[Int]:
  for x in xs:
    yield x if x % 2 == 0

def total(xs: Iterator[Int]) -> Int = xs.sum

def main:
  puts evens(naturals()).take(3).to_list    #=> [0, 2, 4]
  puts total([1, 2, 3])                      #=> 6
  puts total(evens(1..6))                    #=> 12
```

A generator moves in; a list, a set or a sequence of your own is copied in,
so the caller's value is as it was. A lazy chain stays lazy into a plain
function. Into a generator, which outlives the call, it is worked out first,
since what it reads might not live as long: pass a generator itself to stay
lazy there.
- **Speed:** `bench/b7_generators.lume` is `b6` as a generator; it runs at
  about 1.3 times the hand-written Rust iterator.
- **More:** `examples/generators.lume` shows each of these.

## Where `[]` and `{}` need a type

An empty literal has nothing to infer from. It is fine wherever the type is
already known — a typed binding, a struct field, an argument to a function
with a declared parameter type, or `.or([])` on a known map — and not fine
standing alone:

```lume-bad
xs = []
#! cannot tell the type
```

## Iterating and changing at once

Changing a collection while looping over it walks a copy, so it is safe and
does what you mean:

```lume
var seen = {1, 2}
for x in seen:
  seen.add(x * 10)
puts seen.len    #=> 4
```

## When a list is copied, and when it moves

A list is a value: giving it to another name never lets the two change each
other. What that costs depends on whether the old name is needed again.
**Where it is not, the list moves**, as Rust's values do: no item is copied.

```lume
var rows = [["b", "2"], ["a", "1"], ["d", "4"]]
rows = rows.filter { |r| r.at(0) != "d" }.to_list   # rows is replaced: moved
rows = rows.sort_by { |r| r.at(1) }                  # moved again
puts rows           #=> [["a", "1"], ["b", "2"]]

words = ["pear", "fig"]
short = words.filter { |w| w.len < 4 }.to_list      # words is used below: copied
puts short          #=> ["fig"]
puts words          #=> ["pear", "fig"]
```

A list moves when it is a local of your own (bound with `=`, `var`, a
`for`, or taken out of a tuple) and one of these holds:

- nothing after the statement uses it;
- the statement replaces it, as in `rows = rows.filter { .. }.to_list`,
  even inside a loop.

It is copied when anything later reads it, when the statement names it twice,
when it is a parameter (a function borrows what it is given), when it is a
field, and when a block uses it. A block may run many times, so it can never
give away what it holds. `+` extends a list that nothing else holds instead
of copying it.

**The same holds for every value that is not a number, a `Bool` or a
`Char`**: text, maps, sets and structs.

- `acc = acc + w` writes `w` onto the end of `acc`. It does not build a new
  string each time round a loop, so building text piece by piece costs as
  much as the text is long, not its square. Everything on the right is
  still read before `acc` changes: in `acc = acc + w + "#{acc.len}"`,
  `acc.len` is the old length.
- `m = m.filter { .. }` hands the map's entries over, and so does the same
  on a set.
- A struct nothing needs afterwards gives its fields away one by one, as a
  Rust struct does: in `q = Person(name: p.name, tags: p.tags, score: p.score + 1)`
  nothing is copied, provided the statement reads each field of `p` once
  and nothing later reads `p`.

None of this changes what a program prints. It changes only how much work
the program does. `examples/moves.lume` and `examples/moves_values.lume`
show each side of the line.

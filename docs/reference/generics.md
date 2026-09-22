# Generics

`[T]` after a name says "this works for any type; call it `T`". The caller
never writes `T` — it is read off the arguments, or off the type the result
goes into.

```lume
def first[T](xs: [T]) -> T?:
  xs[0]

def main:
  puts first([3, 1, 2])      #=> Some(3)
  puts first(["ada"])        #=> Some("ada")
  puts first([1.5])          #=> Some(1.5)
```

## Generic structs and enums

The brackets go in the same place. Inside, `T` is an ordinary type.

```lume
struct Stack[T]:
  items: [T]

  def push(var self, x: T):
    items.push(x)

  def pop(var self) -> T? = items.pop
  def size -> Int = items.len

struct Pair[A, B]:
  left: A
  right: B

  def swapped -> Pair[B, A] = Pair(left: right, right: left)

def main:
  var s: Stack[Str] = Stack(items: [])
  s.push("a")
  puts s.size                                 #=> 1
  puts s.pop                                  #=> Some("a")
  puts Pair(left: 1, right: "one").swapped    #=> Pair(left: "one", right: 1)
```

The empty stack above has nothing to read `T` off, so the binding says it.
When an argument says it, nothing needs writing down.

An enum works the same way, and a variant may hold the enum itself:

```lume
enum Tree[T]:
  Leaf
  Node(value: T, left: Tree[T], right: Tree[T])

  def size -> Int:
    match self:
      Leaf          -> 0
      Node(_, l, r) -> 1 + l.size + r.size

def main:
  t: Tree[Int] = Node(1, Leaf, Node(2, Leaf, Leaf))
  puts t.size    #=> 2
  puts t
  #=> Node(value: 1, left: Leaf, right: Node(value: 2, left: Leaf, right: Leaf))
```

## What a bare `T` can do

A bare `T` says nothing about the value, so it can be moved around, compared
with `==`, and **turned into text** — interpolation is available on every
type, bound or not:

```lume
def show[T](x: T) -> Str = "#{x}"

def same[T](a: T, b: T) -> Bool = a == b

def listed[T](xs: [T]) -> Str = xs.map { |x| "<#{x}>" }.to_list.join(",")

def main:
  puts show(1)              #=> 1
  puts show(Some(2))        #=> Some(2)
  puts same("a", "b")       #=> false
  puts listed([1, 2])       #=> <1>,<2>
```

`<` is not available, and the error says what to write:

```lume-bad
def bigger[T](a: T, b: T) -> T:
  if a < b: b else: a

def main:
  puts bigger(1, 2)
#! `T` could be any type, so `<` has nothing to compare
```

## Bounds

A bound is written `[T: Bound]`. There are two built-in ones and any
interface is a third kind.

**`Ordered` means `<` works**: `Int`, `Float`, `Str`, `Char`, or a type with
its own `def <`. **`Hashable` means the value can be a map key or a set
item**: `Int`, `Str`, `Bool`, `Char`, tuples of those, and structs and enums
made of them.

```lume
def largest[T: Ordered](xs: [T]) -> T?:
  xs.max

def tally[T: Hashable](xs: [T]) -> {T: Int}:
  var counts: {T: Int} = {}
  for x in xs:
    counts[x] = counts[x].or(0) + 1
  counts

def main:
  puts largest([3, 9, 2])              #=> Some(9)
  puts largest(["pear", "fig"])        #=> Some("pear")
  puts tally(["a", "b", "a"])          #=> {"a": 2, "b": 1}
```

A type that misses the bound is caught at the call:

```lume-bad
struct Box:
  v: Float

def largest[T: Ordered](xs: [T]) -> T?:
  xs.max

def main:
  puts largest([Box(1.0)])
#! `largest` needs `T: Ordered`, and `Box` is not Ordered
```

```lume-bad
def tally[T: Hashable](xs: [T]) -> Int:
  var seen: {T} = {}
  for x in xs:
    seen.add(x)
  seen.len

def main:
  puts tally([1.5, 2.5])
#! `tally` needs `T: Hashable`, and `Float` is not Hashable
```

### Defining `<` is enough for `Ordered`

> Asked in round after round. Yes — one operator, and the bound is met.

```lume
struct Money:
  cents: Int

  def <(o: Money) -> Bool = cents < o.cents

def largest[T: Ordered](xs: [T]) -> T?:
  xs.max

def main:
  puts largest([Money(5), Money(2), Money(9)])    #=> Some(Money(cents: 9))
  puts [Money(5), Money(2)].sort    #=> [Money(cents: 2), Money(cents: 5)]
```

`Hashable` needs nothing written at all: a struct or enum whose parts are
hashable already is.

### Bounds on a generic type's parameters

**They are allowed**, on a `struct` and on an `enum`, in exactly the same
spelling as on a `def`. A type with several parameters may bound some and not
others.

```lume
struct Sorted[T: Ordered]:
  items: [T]

  def best -> T? = items.max
  def in_order -> [T] = items.sort

struct Keyed[K: Hashable, V]:
  pairs: {K: V}

  def size -> Int = pairs.len

enum Maybe[T: Hashable]:
  Nothing
  Just(value: T)

def main:
  puts Sorted(items: [3, 1, 2]).best        #=> Some(3)
  puts Sorted(items: [3, 1, 2]).in_order    #=> [1, 2, 3]
  puts Keyed(pairs: {"a": 1}).size          #=> 1
  puts Just(3)                              #=> Just(value: 3)
```

The bound is checked where the value is built:

```lume-bad
struct Box:
  v: Float

struct Sorted[T: Ordered]:
  items: [T]

  def best -> T? = items.max

def main:
  puts Sorted(items: [Box(1.0)]).best
#! `Sorted` needs `T: Ordered`, and `Box` is not Ordered
```

### One bound per parameter

> Reviewers reached for `[T: Ordered + Hashable]` again and again. It does
> not parse.

**A parameter takes exactly one bound.** There is no `+` and no `&`:

```lume-bad
def uniq_sorted[T: Ordered + Hashable](xs: [T]) -> [T]:
  xs.sort

def main:
  puts uniq_sorted([2, 1])
#! expected `]` to close the type parameters, found `+`
```

A comma does not help either: it starts the *next* parameter, so
`[T: Ordered, Hashable]` declares a second type parameter named `Hashable`
and leaves `T` merely `Ordered`. The mistake shows up at the call, as a
parameter nothing can fix:

```lume-bad
def uniq_sorted[T: Ordered, Hashable](xs: [T]) -> [T]:
  xs.sort

def main:
  puts uniq_sorted([2, 1])
#! cannot tell what `Hashable` is in this call to `uniq_sorted`
```

When you find yourself wanting both, pick the one the work really needs.
De-duplicating with a list and `contains?` needs only `==`, which every `T`
has, so `Ordered` alone carries this:

```lume
def uniq_sorted[T: Ordered](xs: [T]) -> [T]:
  var out: [T] = []
  for x in xs:
    if not out.contains?(x):
      out.push(x)
  out.sort

def main:
  puts uniq_sorted([3, 1, 3, 2])          #=> [1, 2, 3]
  puts uniq_sorted(["b", "a", "b"])       #=> ["a", "b"]
```

## Interfaces as bounds

Any interface is a bound, and conformance stays structural — the conforming
type never names it.

```lume
interface Titled:
  def title -> Str

  def shout -> Str = title.upcase + "!"

struct Song:
  name: Str

  def title -> Str = name

def shout_all[T: Titled](xs: [T]) -> [Str]:
  xs.map { |x| x.shout }.to_list

def main:
  puts shout_all([Song(name: "one"), Song(name: "two")])
  #=> ["ONE!", "TWO!"]
```

## Interfaces with type parameters

An interface may take parameters of its own, which lets it say what a type
must be able to act *on*. `[T: Comparable[T]]` is the usual shape: "a type
that can be compared with others of its own kind".

```lume
interface Comparable[T]:
  def compare(other: T) -> Int

  def before?(other: T) -> Bool = compare(other) < 0

struct Version:
  major: Int
  minor: Int

  def compare(other: Version) -> Int:
    if major != other.major:
      return major - other.major
    minor - other.minor

def largest[T: Comparable[T]](xs: [T]) -> T?:
  var best = xs[0]?
  for x in xs:
    if best.before?(x):
      best = x
  Some(best)

def main:
  a = Version(major: 1, minor: 2)
  b = Version(major: 1, minor: 9)
  puts a.before?(b)      #=> true
  puts largest([a, b])   #=> Some(Version(major: 1, minor: 9))
```

`Version` never writes `Comparable`. It has `def compare(other: Version)`, so
the argument is worked out for it.

An interface's own parameter may carry a bound, and that bound then holds
inside its defaults:

```lume
interface Ranked[T: Ordered]:
  def all -> [T]

  def best -> T? = all.max
  def in_order -> [T] = all.sort

struct Scores:
  values: [Int]

  def all -> [Int] = values

def main:
  puts Scores(values: [3, 1, 2]).best        #=> Some(3)
  puts Scores(values: [3, 1, 2]).in_order    #=> [1, 2, 3]
```

## Generic `extend`

**An `extend` introduces a type parameter by using one**: a name in the
target that is no type of this program's is a parameter. That is what lets
the built-in containers conform to an interface of yours.

```lume
interface Bag[T]:
  def items -> [T]

  def size -> Int = items.len
  def listed -> Str = items.map { |x| "#{x}" }.to_list.join(", ")

extend [T] with Bag[T]:
  def items -> [T] = self

extend {T} with Bag[T]:
  def items -> [T] = self.to_list

extend {K: V} with Bag[V]:
  def items -> [V] = self.values.to_list

struct Stack[T]:
  vals: [T]

extend Stack[T] with Bag[T]:
  def items -> [T] = vals

def describe[B: Bag[Str]](b: B) -> Str = "#{b.size}: #{b.listed}"

def main:
  puts describe(["pear", "fig"])          #=> 2: pear, fig
  puts describe({"red"})                  #=> 1: red
  puts describe({1: "one", 2: "two"})     #=> 2: one, two
  puts describe(Stack(vals: ["only"]))    #=> 1: only
  puts [3, 1, 2].size                     #=> 3
```

The last line is the point: the defaults are now the list's own methods, with
no helper in between.

A bound may be written inside the `[...]` of an `extend` target, because a
`:` there can mean nothing else. Inside `{...}` a `:` already means a map, so
a set's item takes whatever bound the interface declared:

```lume
interface Ranked[T: Ordered]:
  def all -> [T]

  def best -> T? = all.max
  def in_order -> [T] = all.sort

extend [T: Ordered] with Ranked[T]:
  def all -> [T] = self

extend {T} with Ranked[T]:
  def all -> [T] = self.to_list

def main:
  puts [3, 1, 2].best            #=> Some(3)
  puts [3, 1, 2].in_order        #=> [1, 2, 3]
  puts({"pear", "fig"}.in_order) #=> ["fig", "pear"]
```

A bare target must still be a real type, so a bound on one is a mistake
rather than a silent parameter:

```lume-bad
interface Named:
  def label -> Str

extend [Int: Ordered] with Named:
  def label -> Str = "x"

def main:
  puts [1].label
#! `Int` is a type, so it takes no bound here
```

## What this costs

Nothing at run time. A generic becomes an ordinary Rust generic — `lume emit`
on `first` above shows `pub fn first<T: ...>(xs: &Vec<T>) -> Option<T>` —
which is resolved when the program is compiled. There is no boxing and no
lookup at run time.

`examples/generics.lume`, `examples/generic_interfaces.lume` and
`examples/generic_extend.lume` are longer runnable versions of this page.

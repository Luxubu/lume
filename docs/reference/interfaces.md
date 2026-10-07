# Interfaces

## A type conforms by having the methods

**There is nothing to declare.** No `impl`, no `derive`, no list of
interfaces after the struct name. An interface names some methods; any type
that has them is one.

```lume
interface Shape:
  def area -> Float

struct Circle:
  r: Float

  def area -> Float = 3.0 * r * r

struct Sq:
  s: Float

  def area -> Float = s * s

def describe(x: Shape) -> Str = "area #{x.area}"

def main:
  puts describe(Circle(1.0))    #=> area 3.0
  puts describe(Sq(2.0))        #=> area 4.0
```

Neither struct mentions `Shape`. Both have `area`, so both are one.

## Required methods and defaults

A method with no body is required. A method **with** a body is a default:
every conforming type gets it, and the default may call the required ones.
Either body shape works — `= expr` or `:` and an indented block.

```lume
interface Shape:
  def area -> Float

  def name -> Str = "shape"

  def report -> Str:
    "#{name}: #{area}"

struct Circle:
  r: Float

  def area -> Float = 3.0 * r * r
  def name -> Str = "circle"

struct Sq:
  s: Float

  def area -> Float = s * s

def main:
  puts Circle(1.0).report    #=> circle: 3.0
  puts Sq(2.0).report        #=> shape: 4.0
```

`Sq` never wrote `name`, so it got the default. The defaults are the type's
own methods afterwards — `Sq(2.0).report` is called directly, with no
interface named anywhere.

**A required method may take `var self`**, as a Rust trait method may take
`&mut self`, and a type fits only if its own method takes `var self` too. A
default only reads the value, so it cannot:

```lume
interface Ticker:
  def tick(var self) -> Int

struct Clock:
  t: Int

  def tick(var self) -> Int:
    t += 1
    t

def twice[T: Ticker](var x: T) -> Int:
  x.tick
  x.tick

def main:
  var c = Clock(t: 0)
  puts twice(c)    #=> 2
```

Either way round, a mismatch is refused: a `tick` that only reads does not
fit `def tick(var self)`, and a `tick(var self)` does not fit an interface
whose `tick` only reads. Calling a `var self` method needs something that may
change: a `var` binding, a `var` parameter (`var x: T`, as above), or a
`for var` over a `var` list of the interface.

The built-in `Iterator[T]` is one of these: `def next(var self) -> T?`. See
[Collections](collections.md#types-of-your-own-next).

## `extend T with I:`

`extend` gives a type methods it did not define, so a type you do not own can
conform. **It works on the built-ins too.**

```lume
interface Shape:
  def area -> Float

  def name -> Str = "shape"

extend Int with Shape:
  def area -> Float = self.to_float

extend Str with Shape:
  def area -> Float = self.len.to_float
  def name -> Str = "the string #{self}"

extend [Int] with Shape:
  def area -> Float = self.len.to_float

def main:
  puts 5.area            #=> 5.0
  puts 5.name            #=> shape
  puts "abc".area        #=> 3.0
  puts "abc".name        #=> the string abc
  puts [1, 2].area       #=> 2.0
  puts [1, 2].name       #=> shape
```

Inside an `extend` the whole value is `self`, which is what you need when the
target is a built-in. When the target is a struct, its fields are also there
bare, exactly as in a method written on the struct itself:

```lume
interface Named:
  def label -> Str

struct Dog:
  tag: Str
  n: Int

extend Dog with Named:
  def label -> Str = "#{tag}/#{n}"

def main:
  puts Dog("rex", 2).label    #=> rex/2
```

An `extend` must finish the job. If the interface asks for something the
target still lacks, the whole block is rejected:

```lume-bad
interface Shape:
  def area -> Float
  def sides -> Int

struct Sq:
  s: Float

extend Sq with Shape:
  def area -> Float = s * s

def main:
  puts Sq(2.0).area
#! `extend Sq with Shape` is missing `sides`
```

### An extend may replace a default

**Yes.** Overriding a default is the ordinary case, and needs no keyword:

```lume
interface Shape:
  def area -> Float
  def name -> Str = "shape"

struct Sq:
  s: Float

  def area -> Float = s * s

extend Sq with Shape:
  def name -> Str = "square"

def main:
  puts Sq(2.0).name    #=> square
```

### It may not replace a method the type already wrote

```lume-bad
interface Shape:
  def area -> Float

struct Sq:
  s: Float

  def area -> Float = s * s

extend Sq with Shape:
  def area -> Float = 0.0

def main:
  puts Sq(2.0).area
#! `Sq` already has an `area` method
```

### It may not add a method the interface never asked for

An `extend` block is scoped to one interface, so a spare method has no home
there. Put it on the type itself.

```lume-bad
interface Shape:
  def area -> Float

struct Sq:
  s: Float

extend Sq with Shape:
  def area -> Float = s * s
  def perimeter -> Float = s * 4.0

def main:
  puts Sq(2.0).perimeter
#! `perimeter` is not a method of `Shape`
```

## Where an interface can be used

**All four places work**: a parameter, a list's element type, a struct field,
and a return type. In each, the value is some conforming type and the
interface is all that is known about it.

```lume
interface Shape:
  def area -> Float

struct Circle:
  r: Float

  def area -> Float = 3.0 * r * r

struct Sq:
  s: Float

  def area -> Float = s * s

struct Scene:
  title: Str
  hero: Shape
  rest: [Shape]

def total(xs: [Shape]) -> Float = xs.map(_.area).sum

def biggest(xs: [Shape]) -> Shape:
  var best = xs.at(0)
  for x in xs:
    if x.area > best.area:
      best = x
  best

def main:
  mixed: [Shape] = [Circle(1.0), Sq(5.0)]
  puts total(mixed)              #=> 28.0
  puts biggest(mixed).area       #=> 25.0
  sc = Scene(title: "t", hero: Circle(2.0), rest: mixed)
  puts sc.hero.area              #=> 12.0
  puts sc.rest.map(_.area).to_list    #=> [3.0, 25.0]
```

A `var` list of an interface type takes any conforming value with `push`,
and a value held as the interface answers the interface's defaults as well
as its required methods:

```lume
interface Named:
  def name -> Str

  def greeting -> Str = "hello, #{name}"

struct Dog:
  called: Str

  def name -> Str = called

struct Ship:
  title: Str

  def name -> Str = title

def main:
  var all: [Named] = []
  all.push(Dog(called: "Rex"))
  all.push(Ship(title: "Argo"))
  for x in all:
    puts x.greeting    #=> hello, Rex
                       #=> hello, Argo
```

**The annotation on `mixed` is doing work.** A list literal of two different
structs has no single element type, so say `[Shape]` when you mean the
mixture:

```lume-bad
interface Shape:
  def area -> Float

struct Circle:
  r: Float

  def area -> Float = 3.0 * r * r

struct Sq:
  s: Float

  def area -> Float = s * s

def main:
  mixed = [Circle(1.0), Sq(5.0)]
  puts mixed.len
#! the items of a list must all be the same type
```

## An interface's own methods may use interfaces

**An interface method may take an interface value, give one back, or give a
sequence**, as a free function may. In Rust the trait takes `&dyn Shape`,
gives `Box<dyn Shape>`, and gives a boxed iterator for `Iterator[T]`, so it
stays usable through a pointer:

```lume
interface Shape:
  def area -> Float
  def scaled(k: Float) -> Shape
  def bigger?(o: Shape) -> Bool = area > o.area

struct Sq:
  s: Float
  def area -> Float = s * s
  def scaled(k: Float) -> Shape = Sq(s: s * k)

struct Circle:
  r: Float
  def area -> Float = 3.0 * r * r
  def scaled(k: Float) -> Shape = Circle(r: r * k)

def main:
  puts Sq(s: 2.0).bigger?(Circle(r: 1.0))    #=> true
  shapes: [Shape] = [Sq(s: 1.0), Circle(r: 1.0)]
  puts shapes.at(0).bigger?(shapes.at(1))   #=> false
  puts shapes.at(1).scaled(2.0).area        #=> 12.0
```

A default may be a generator, made from the interface's other methods, and
a required method may give a sequence, which each type makes as it likes —
with `yield`, or by handing back a chain:

```lume
interface Source[T]:
  def each_item -> Iterator[T]
  def numbered -> Iterator[(Int, T)]:
    var i = 1
    for x in each_item:
      yield (i, x)
      i += 1

struct Squares:
  upto: Int
  def each_item -> Iterator[Int] = (1..upto).map { |n| n * n }

def main:
  puts Squares(upto: 3).numbered.to_list    #=> [(1, 1), (2, 4), (3, 9)]
```

A generator default walks a copy of the value, taken through the same
`lume_box` that copies any value held as the interface. A method in an
`extend` cannot be a generator yet: write it as a default in the interface,
or on the type itself.

## Methods that take a block, have type parameters, or are `async`

**An interface method may take a block**, and is written as any other:

```lume
interface Walk:
  def each_item(f: (Int) -> ()) -> ()
  def total -> Int:
    var t = 0
    each_item { |x| t += x }
    t

struct Bag:
  xs: [Int]

  def each_item(f: (Int) -> ()) -> ():
    for x in xs:
      f(x)

extend [Int] with Walk:
  def each_item(f: (Int) -> ()) -> ():
    for x in self:
      f(x)

def main:
  walks: [Walk] = [Bag(xs: [1, 2, 3]), [10, 20]]
  puts walks.map { |w| w.total }.to_list    #=> [6, 30]
```

In Rust the block is lent as a `&mut dyn FnMut`, which keeps the interface
usable in every place above — a list, a field, a binding.

**An interface method may have type parameters of its own, or be `async`.**
Rust calls such a method one that only a known type can call, and so does
Lume: it is called through a parameter typed as the interface or a bound, and
refused on a value held through a pointer — a list item, a field, a binding
of the interface's type. The interface itself can still be held that way,
and its other methods called:

```lume
interface Mappable[T]:
  def map_all[U](f: (T) -> U) -> [U]
  def size -> Int

struct Box[T]:
  items: [T]

  def map_all[V](f: (T) -> V) -> [V] = items.map { |x| f(x) }.to_list
  def size -> Int = items.len

def labels[M: Mappable[Int]](m: M) -> [Str] = m.map_all { |n| "n#{n}" }

def main:
  puts labels(Box(items: [1, 2]))    #=> ["n1", "n2"]
  held: [Mappable[Int]] = [Box(items: [3, 4])]
  puts held.map { |m| m.size }.to_list    #=> [2]
```

The type's own method may name its parameter differently (`V` for `U`): they
are matched by position.

```lume-bad
interface Mappable[T]:
  def map_all[U](f: (T) -> U) -> [U]

extend [T] with Mappable[T]:
  def map_all[U](f: (T) -> U) -> [U] = self.map { |x| f(x) }.to_list

def main:
  held: [Mappable[Int]] = [[1, 2]]
  puts held.at(0).map_all { |x| x * 2 }
#! `map_all` has type parameters of its own (`U`), so it cannot be called on a `Mappable[Int]` held in a list, a field or a binding
```

Passing such a held value to a parameter typed as the interface is refused
for the same reason: the function could call `map_all` on it.

Why a parameter and a binding written the same way differ: a parameter
`m: Mappable[Int]` is compiled as a generic, so inside the function `m` is
the caller's own value with its own type; a binding, a field or a list item
of that type holds a value of some conforming type behind a pointer, and
the pointer only knows the methods every such type shares.

An `async` method is written `async def fetch -> Str` in the interface, and
awaited as any other: `await source.fetch()`. It may have a default body,
which awaits the type's own methods; in Rust it is a method returning a
future that can move between tasks:

```lume
interface Source:
  async def fetch -> Str

  async def both -> Str:
    a = await fetch()
    b = await fetch()
    "#{a}+#{b}"

struct Fixed:
  text: Str

  async def fetch -> Str = text

async def main:
  puts await Fixed(text: "yo").both()    #=> yo+yo
```

Two things stay with the type. A default that is neither `async` nor generic
cannot call an `async` method on `self` — Rust would not know `self` is a
known type there — so make that default `async` too. And an `async` default
cannot take a block: the block is lent for the call, and an `async` body may
still be running after it.

## `pub` and methods

**A method of a `pub struct` needs no `pub` of its own.** Visibility is
decided by the type: once the struct is `pub`, everything on it can be
reached by an importer. The same goes for a `pub interface` and its defaults.

```lume-skip
# users/model.lume
pub interface Shape:
  def area -> Float
  def name -> Str = "shape"

pub struct Sq:
  side: Float

  def area -> Float = side * side
```

An importer can then call `shapes.Sq(side: 3.0).area` and `.name`, and can
write `shapes.Shape` as a type. Writing `pub def area` inside the struct is
accepted and changes nothing. Dropping the `pub` from the struct is what
hides it, and the error says so directly: ``Sq` exists in module
`lib.shapes` but is not `pub``.

## Operator methods

An operator is a method with a punctuation name: `+`, `-`, `*`, `/`, `%`,
`<`, `<=`, `>`, `>=`, `==`, `!=`.

```lume
struct V:
  n: Int

  def +(o: V) -> V = V(n + o.n)
  def -(o: V) -> V = V(n - o.n)
  def *(k: Int) -> V = V(n * k)

def main:
  puts V(6) + V(2)    #=> V(n: 8)
  puts V(6) - V(2)    #=> V(n: 4)
  puts V(6) * 3       #=> V(n: 18)
```

**`<` alone is enough.** Define it and `<=`, `>`, `>=`, `sort`, `max` and
`min` all follow, and the type satisfies the `Ordered` bound:

```lume
struct M:
  c: Int

  def <(o: M) -> Bool = c < o.c

def main:
  puts M(1) < M(2)          #=> true
  puts M(1) <= M(1)         #=> true
  puts M(3) >= M(4)         #=> false
  puts [M(3), M(1)].sort    #=> [M(c: 1), M(c: 3)]
  puts [M(3), M(1)].max     #=> Some(M(c: 3))
```

`==` is already there field by field, so write one only to mean something
else. `!=` follows from whichever `==` is in force:

```lume
struct Ci:
  s: Str

  def ==(o: Ci) -> Bool = s.downcase == o.s.downcase

def main:
  puts Ci("Ada") == Ci("ada")    #=> true
  puts Ci("Ada") != Ci("ada")    #=> false
```

**An interface cannot require an operator.** The name is not accepted in an
interface body, so `Ordered` — which is a bound, not an interface — is how a
generic asks for `<`:

```lume-bad
interface Addable:
  def +(o: Int) -> Int

def main:
  puts "never gets here"
#! expected a method name, found `+`
```

`examples/interfaces.lume` is a longer runnable version of this page;
`docs/reference/generics.md` covers interfaces that take type parameters.

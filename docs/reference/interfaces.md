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
#! `Sq` already has a `area` method
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

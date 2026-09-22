# Structs and enums

A struct is a set of fields held together. An enum is a choice between named
shapes. Both get equality, printing and hashing without asking.

## A struct

Fields are indented under the name, one per line, each with a type. Methods
follow in the same block.

```lume
struct User:
  name: Str
  age: Int

  def adult? = age >= 18
  def greeting = "Hi, #{name}"

def main:
  u = User(name: "Ada", age: 36)
  puts u.name          #=> Ada
  puts u.adult?        #=> true
  puts u.greeting      #=> Hi, Ada
```

**Inside a method a field is written bare.** `name` means this value's
`name`; no `self.` is needed. `self` is still there for the whole value, and
`self.name` is accepted too:

```lume
struct P:
  x: Int
  y: Int

  def swapped -> P = P(x: y, y: x)
  def same?(o: P) -> Bool = self == o
  def just_x -> Int = self.x

def main:
  puts P(1, 2).swapped         #=> P(x: 2, y: 1)
  puts P(1, 2).same?(P(1, 2))  #=> true
  puts P(1, 2).just_x          #=> 1
```

## Two constructors, always

**Every struct can be built positionally or by keyword**, with no extra
declaration. Keywords may come in any order, and the two forms may be mixed.

```lume
struct P:
  x: Int
  y: Int

def main:
  puts P(1, 2)              #=> P(x: 1, y: 2)
  puts P(x: 1, y: 2)        #=> P(x: 1, y: 2)
  puts P(y: 2, x: 1)        #=> P(x: 1, y: 2)
  puts P(1, y: 2)           #=> P(x: 1, y: 2)
```

Every field must be given:

```lume-bad
struct P:
  x: Int
  y: Int

def main:
  puts P(1)
#! `P` is missing an argument: y
```

There are no field defaults to fall back on, and writing one says so:

```lume-bad
struct P:
  x: Int = 0
  y: Int

def main:
  puts P(y: 1)
#! field defaults are not supported yet
```

## Changing a field

`p.x = v` works when the binding is a `var`:

```lume
struct P:
  x: Int
  y: Int

def main:
  var p = P(1, 2)
  p.x = 9
  puts p    #=> P(x: 9, y: 2)
```

An immutable binding refuses, and names the line to change:

```lume-bad
struct P:
  x: Int
  y: Int

def main:
  p = P(1, 2)
  p.x = 9
#! `p` is immutable, so its field `x` cannot be changed
```

## `var self`

**A method that changes a field declares it: `def name(var self, ...)`.** The
`var self` is the first parameter and is never passed at the call.

```lume
struct Counter:
  n: Int
  label: Str

  def bump(var self, by: Int):
    n += by

  def show = "#{label}=#{n}"

def main:
  var c = Counter(n: 0, label: "hits")
  c.bump(2)
  c.bump(3)
  puts c.show    #=> hits=5
```

Without it, the assignment is refused where it happens:

```lume-bad
struct Counter:
  n: Int

  def bump(by: Int):
    n += by

def main:
  var c = Counter(n: 0)
  c.bump(1)
#! `bump` changes the field `n`, but its `self` is read-only
```

And a `var self` method needs a `var` binding to be called on, the same as an
assignment does:

```lume-bad
struct Counter:
  n: Int

  def bump(var self):
    n += 1

def main:
  c = Counter(n: 0)
  c.bump
#! `c` is immutable, but `bump` changes it
```

## `==`, printing and hashing come free

You never write `==` for a struct or an enum unless you want it to mean
something unusual. Equality is field by field, printing names the fields, and
a value made of hashable parts can be a set item or a map key.

```lume
struct P:
  x: Int
  y: Int

def main:
  puts P(1, 2) == P(1, 2)    #=> true
  puts P(1, 2) == P(1, 3)    #=> false
  puts P(1, 2)               #=> P(x: 1, y: 2)
  puts [P(1, 2)]             #=> [P(x: 1, y: 2)]
  puts {P(1, 2): "here"}     #=> {P(x: 1, y: 2): "here"}
```

## An enum: the exact shape

> Reviewers guessed this wrong round after round. There is one spelling.

**`enum Name:`, then each variant indented on its own line.** A variant with
no payload is a bare capitalised name. A variant with a payload writes its
fields in parentheses, and **every payload field needs a name** as well as a
type. No commas between variants, no `|`, no `case`.

```lume
enum Light:
  Red
  Yellow
  Green

enum Shape:
  Dot
  Circle(radius: Float)
  Rect(w: Float, h: Float)

def main:
  puts Red                      #=> Red
  puts Circle(radius: 1.0)      #=> Circle(radius: 1.0)
  puts Rect(2.0, 3.0)           #=> Rect(w: 2.0, h: 3.0)
```

The variants are what make it an enum, so there must be at least one and it
must be indented:

```lume-bad
enum L: Red | Green

def main:
  puts "never gets here"
#! enum `L` has no variants
```

An unnamed payload field is refused, because the name is what shows when the
value prints and what labels the argument when one is built:

```lume-bad
enum Shape:
  Circle(Float)

def main:
  puts "never gets here"
#! a field of `Circle` needs a name, not just the type `Float`
```

Methods go under the variants, in the same block, and `match self` is the
usual body:

```lume
enum Shape:
  Dot
  Circle(radius: Float)
  Rect(w: Float, h: Float)

  def area -> Float:
    match self:
      Dot           -> 0.0
      Circle(r)     -> 3.0 * r * r
      Rect(w, h)    -> w * h

def main:
  puts [Dot, Circle(1.0), Rect(2.0, 3.0)].map(_.area).to_list
  #=> [0.0, 3.0, 6.0]
```

## Bare variant names and qualified ones

**A bare `Circle` is enough whenever exactly one enum in scope has a variant
by that name.** Write `Shape.Circle` when two do — and the compiler tells you
which two rather than picking one.

```lume
enum A:
  One
  Two

enum B:
  One
  Three

def main:
  puts Two          #=> Two
  puts Three        #=> Three
  puts A.One        #=> One
  puts B.One        #=> One
```

```lume-bad
enum A:
  One
  Two

enum B:
  One
  Three

def main:
  puts One
#! `One` is a variant of more than one enum: A, B
```

Note that qualifying changes only how the value is *written*. It still prints
as the bare variant name.

## Recursive enums

A variant may hold the enum it belongs to. Nothing special is written; the
boxing happens underneath.

```lume
enum Expr:
  Num(value: Int)
  Add(left: Expr, right: Expr)
  Neg(inner: Expr)

  def eval -> Int:
    match self:
      Num(v)    -> v
      Add(l, r) -> l.eval + r.eval
      Neg(i)    -> 0 - i.eval

def main:
  e = Add(Num(1), Neg(Num(4)))
  puts e.eval    #=> -3
  puts e
  #=> Add(left: Num(value: 1), right: Neg(inner: Num(value: 4)))
```

## A field does not satisfy an interface

> Asked in three separate rounds. The answer is no.

**An interface asks for a method, and a field of the same name is not one.**
A struct with a `name: Str` field is not a `Named`:

```lume-bad
interface Named:
  def name -> Str

struct Dog:
  name: Str

def hello(x: Named) -> Str = "hi #{x.name}"

def main:
  puts hello(Dog(name: "Rex"))
#! `Dog` is used as a `Named` here, but it has no `name` method
```

The field is perfectly readable — `Dog(name: "Rex").name` works — but it is
not a method, so it does not conform, and the interface's defaults are not
there either.

There are two fixes. Give the method a field of its own to read:

```lume
interface Named:
  def name -> Str
  def shout -> Str = name.upcase

struct Dog:
  call_sign: Str

  def name -> Str = call_sign

def hello(x: Named) -> Str = "hi #{x.name} (#{x.shout})"

def main:
  puts hello(Dog(call_sign: "Rex"))    #=> hi Rex (REX)
```

Or, when the field is already called the right thing, let an `extend` hand it
over. Inside the extend, `name` is still the field:

```lume
interface Named:
  def name -> Str
  def shout -> Str = name.upcase

struct Dog:
  name: Str

extend Dog with Named:
  def name -> Str = self.name

def hello(x: Named) -> Str = "hi #{x.name}"

def main:
  puts hello(Dog(name: "Rex"))    #=> hi Rex
  puts Dog(name: "Rex").shout     #=> REX
```

## A field may be an interface type

It may, and so may a list of one. The field holds any conforming value, and
different values of different types can sit in the same list.

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

def main:
  sc = Scene(title: "t", hero: Circle(2.0), rest: [Sq(2.0), Circle(1.0)])
  puts sc.hero.area                 #=> 12.0
  puts sc.rest.map(_.area).to_list  #=> [4.0, 3.0]
```

## Tuple destructuring

**`(a, b) = expr` works**, on a literal or on anything that gives a tuple.

```lume
def split(p: (Int, Int)) -> (Int, Int) = (p.1, p.0)

def main:
  (a, b) = (1, "two")
  puts a                  #=> 1
  puts b                  #=> two
  (x, y) = split((3, 4))
  puts "#{x},#{y}"        #=> 4,3
```

The parentheses are part of it — a bare `a, b = ...` is not a statement:

```lume-bad
a, b = (1, 2)
#! expected the end of the line, found `,`
```

And the names it binds are immutable; `var (a, b)` is not a form:

```lume-bad
var (a, b) = (1, 2)
#! expected a variable name, found `(`
```

A tuple can also be taken apart by `match`, which is what you want when the
shape is one of several:

```lume
def main:
  match (1, "two"):
    (n, s) -> puts "#{n}/#{s}"    #=> 1/two
```

`examples/structs.lume` and `examples/enums.lume` are longer runnable
versions of this page.

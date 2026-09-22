# A tour of Lume

Half an hour, start to finish. Every example here is run by `tests/docs.sh`
and checked against the output shown, so what you read is what it does.

If you have not installed it yet, see [install.md](install.md).

## Hello

Put this in `hello.lume`:

```lume
def main:
  puts "hello"      #=> hello
```

```sh
lume run hello.lume
```

`def main:` is where a program starts. Indentation makes blocks — there are no
braces and no `end`. `puts` prints one value and a newline.

## Bindings

A name is written once and does not change. `var` makes one that can.

```lume
name = "Ada"
var count = 0
count += 1
puts "#{name} #{count}"    #=> Ada 1
```

`#{...}` puts a value inside a string. A name cannot be re-bound in a nested
block — that is an error, not a shadow — which is there so a typo in an inner
scope cannot quietly stop changing the outer one.

## Types, when you want them

Types are inferred and written only where you want them or where the compiler
cannot tell:

```lume
age: Int = 41
rate = 0.5           # a Float
ok = true            # a Bool
letters = ["a", "b"] # a [Str]
empty: [Int] = []    # nothing to infer from, so say it
puts "#{age} #{rate} #{ok} #{letters.len} #{empty.len}"   #=> 41 0.5 true 2 0
```

The scalar types are `Int`, `Float`, `Bool`, `Str`, `Char` and `()`. A `Float`
never mixes with an `Int` without saying so — `1 + 0.5` is an error, and
`1.to_float + 0.5` is not.

## Functions

```lume
def double(n: Int) -> Int:
  n * 2

def triple(n: Int) -> Int = n * 3

def main:
  puts double(4)    #=> 8
  puts triple(4)    #=> 12
```

The last expression is the result; there is no `return` needed, though
`return` exists for leaving early. The return type may be left off when it is
obvious, and argument types may not — they are the documentation.

## Lists, maps and sets

```lume
xs = [3, 1, 2]
puts xs.sort.join(",")            #=> 1,2,3
puts xs.map { |n| n * 2 }.sum     #=> 12

ages = {"ada": 36, "bob": 41}
puts ages["ada"].or(0)            #=> 36
for name, age in ages:
  puts "#{name} is #{age}"        #=> ada is 36
                                  #=> bob is 41

tags = {"red", "blue", "red"}
puts tags.len                     #=> 2
```

A map keeps the order you put things in. A missing key gives you nothing
rather than a crash, so `.or(default)` is how you read one.

Blocks come in three shapes and all three mean the same thing:

```lume
def shout(s: Str) -> Str = s.upcase

def main:
  names = ["ada", "bob"]
  puts names.map { |n| n.upcase }.join(" ")   #=> ADA BOB
  puts names.map(_.upcase).join(" ")          #=> ADA BOB
  puts names.map(shout).join(" ")             #=> ADA BOB
```

Chains are lazy and compile to one Rust loop, so `map` then `filter` then
`sum` walks the list once. You do not need `.to_list` to finish a chain —
`join`, `sum`, `len`, `sort` and the rest all work directly on one.

## Structs

```lume
struct Point:
  x: Int
  y: Int

  def away -> Int = x + y

def main:
  p = Point(x: 1, y: 2)
  puts p.x           #=> 1
  puts p.away        #=> 3
  puts p             #=> Point(x: 1, y: 2)
```

Inside a method, a field is used by its bare name. A method that changes a
field says so: `def move(var self, dx: Int)`. Printing and `==` come for free.

## Enums and `match`

```lume
enum Shape:
  Dot
  Circle(radius: Float)
  Box(w: Int, h: Int)

def area(s: Shape) -> Float:
  match s:
    Dot            -> 0.0
    Circle(r)      -> 3.14 * r * r
    Box(w, h)      -> (w * h).to_float

def main:
  puts area(Circle(radius: 2.0))   #=> 12.56
  puts area(Box(w: 2, h: 3))       #=> 6.0
```

`match` is an expression, and it must cover every case — leave a variant out
and the compiler names the one you missed. That is most of why enums are worth
reaching for.

## Absence and failure

A `T?` may be missing. A `T or Error` may have failed. Neither can be used
without saying what happens in the bad case.

```lume
def half(n: Int) -> Int?:
  if n % 2 == 0: Some(n / 2)
  else: None

def parse(s: Str) -> Int or Error:
  if s.empty?:
    return Error("empty")
  s.to_int

def main -> () or Error:
  puts half(8).or(0)          #=> 4
  puts half(7).or(0)          #=> 0
  puts parse("12")?           #=> 12
  match parse(""):
    Ok(n)    -> puts n
    Error(e) -> puts e.message    #=> empty
```

`?` after a call that can fail hands the failure up to your caller and carries
on with the value when it worked. A failure you never look at is an error —
if you really mean to drop one, write `_ = may_fail()`.

## Interfaces

A type conforms by having the methods; there is nothing to declare.

```lume
interface Sized:
  def size -> Int
  def big? -> Bool = size > 10

struct Crate:
  items: Int
  def size -> Int = items

def describe(s: Sized) -> Str = "#{s.size} (#{s.big?})"

def main:
  puts describe(Crate(items: 12))   #=> 12 (true)
```

`extend` gives an existing type — including a built-in — the methods an
interface asks for:

```lume
interface Sized:
  def size -> Int

extend Str with Sized:
  def size -> Int = self.len

def main:
  puts "hello".size    #=> 5
```

## Going further

Three things this tour leaves out, each with its own page:

- **Several files.** `import shop.items` reads `shop/items.lume`; `pub` marks
  what another file may use. See [Modules](reference/modules.md).
- **Constants.** `LIMIT = 10` at the top level of a file, no keyword. See
  [Names](reference/names.md) and [Modules](reference/modules.md).
- **Doing things at once.** `async def main`, `spawn:`, `await` and
  `shared var`. See [Concurrency](reference/concurrency.md).

And then:

- [The reference](README.md#reference) answers the specific questions:
  how a value prints, what `pad` fills, [every method of every
  type](reference/methods.md).
- [The examples](examples.md), in a reading order.
- `lume --help` lists the commands.

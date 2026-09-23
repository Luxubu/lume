# Functions and blocks

## Two shapes for a body

A `def` ends in `:` and an indented body, or in `= expr` on one line. They
mean the same thing. The last expression of a body is its result; there is no
`return` keyword at the end.

```lume
def add(a: Int, b: Int) -> Int:
  a + b

def mul(a: Int, b: Int) -> Int = a * b

def main:
  puts add(1, 2)    #=> 3
  puts mul(3, 4)    #=> 12
```

The `= expr` form may break after the `=` when the expression is long:

```lume
def shout(s: Str) -> Str =
  s.upcase + "!"

def main:
  puts shout("hi")    #=> HI!
```

`return` exists for leaving early, not for the last line:

```lume
def clamp(n: Int, lo: Int, hi: Int) -> Int:
  if n < lo:
    return lo
  if n > hi:
    return hi
  n

def main:
  puts clamp(0, 1, 3)    #=> 1
  puts clamp(2, 1, 3)    #=> 2
  puts clamp(5, 1, 3)    #=> 3
```

## Declared and inferred results

`-> T` is optional. Leave it off and the result is read from the body; this
works for a recursive function too.

```lume
def sub(a: Int, b: Int) = a - b

def neg(a: Int):
  0 - a

def fact(n: Int) = if n <= 1: 1 else: n * fact(n - 1)

def main:
  puts sub(9, 4)    #=> 5
  puts neg(7)       #=> -7
  puts fact(5)      #=> 120
```

Writing `-> T` makes the compiler check the body against it, and the mismatch
is reported at the body rather than at the call:

```lume-bad
def label(a: Int) -> Str:
  a

def main:
  puts label(1)
#! `label` returns `Str`, but this is a `Int`
```

A function whose body is only side effects has no result. Nothing is written
and nothing comes back:

```lume
def noisy(s: Str):
  puts s

def main:
  noisy("a")        #=> a
  puts "after"      #=> after
```

## A no-argument function needs `()`; a no-argument method does not

> Reviewers asked this round after round. The asymmetry is real.

**A function call always carries `()`, even with no arguments — a bare name
is a value, not a call. A method needs none, because the `.` already made it
a call; writing `()` on one anyway is allowed.**

```lume
def greet -> Str = "hi"

struct P:
  x: Int

  def double = x * 2

def main:
  puts greet()            #=> hi
  p = P(3)
  puts p.double           #=> 6
  puts p.double()         #=> 6
```

Dropping the `()` from the function is an error, and the compiler says so:

```lume-bad
def greet -> Str = "hi"

def main:
  puts greet
#! `greet` is a function; call it with `greet()`
```

## Changing an argument: `var` parameters

**A parameter marked `var` is the caller's own value, and the function may
change it.** Nothing is marked at the call — the declaration says it:

```lume
def add_one(var xs: [Int]):
  xs.push(1)

def main:
  var nums = [5]
  add_one(nums)
  puts nums    #=> [5, 1]
```

The caller's binding must itself be a `var`. Every other parameter is read
only, which is why most functions need no marking at all.

## The three block forms

A block is behaviour handed to a call. There are three spellings: an inline
`{ |x| ... }`, a `do |x|` with an indented body, and the `_` shorthand for
the simplest case.

```lume
xs = [1, 2, 3]
puts xs.map { |n| n * n }.join(",")    #=> 1,4,9
puts xs.map(_ * 10).join(",")          #=> 10,20,30
```

`do |x|` takes an indented body, for when one expression is not enough:

```lume
var total = 0
[1, 2, 3].each do |n|
  doubled = n * 2
  total += doubled
puts total    #=> 12
```

A block taking two names is written the same way:

```lume
puts [1, 2, 3].fold(0) { |acc, n| acc + n }    #=> 6
for i, w in ["a", "b"].enumerate:
  puts "#{i}:#{w}"                             #=> 0:a
                                               #=> 1:b
```

## How deep `_` goes

**`_` stands for the one argument, and everything after it is an ordinary
expression.** Method calls, chains of them, comparisons and arithmetic all
work; `_` is not limited to a single operator.

```lume
ws = ["ada", "bo"]
puts ws.map(_.upcase)            #=> ["ADA", "BO"]
puts ws.map(_.upcase.reverse)    #=> ["ADA", "OB"]
puts ws.filter(_.len == 2)       #=> ["bo"]
puts ws.map(_.len + 1)           #=> [4, 3]
puts [1, 2, 3].map(_ * 2 + 1)    #=> [3, 5, 7]
puts [1, 2, 3].filter(_ > 1)     #=> [2, 3]
```

The one rule is that **`_` may appear once**. A second one has nothing to
name it, so write the block out:

```lume-bad
puts [1, 2, 3].map(_ + _)
#! `_` may appear only once in a shorthand block
```

Once each, in two different blocks, is fine — an inner `_` belongs to the
inner block:

```lume
puts [[1, 2], [3]].map(_.map(_ * 2))    #=> [[2, 4], [6]]
```

## A function's name as a block

Where a block is expected, the name of a function that fits works instead.
No `()`, because the function is being handed over rather than called.

```lume
def double(n: Int) -> Int = n * 2

def main:
  puts [1, 2, 3].map(double)          #=> [2, 4, 6]
  puts ["a", "bb"].map(_.len).map(double)    #=> [2, 4]
```

A function of an imported module works the same way, by its qualified name:
with `import lib.text`, `words.map(text.shout)` and `words.filter(text.long?)`
hand over `text`'s functions, and so does passing `text.shout` to a block
parameter of your own. A module function that takes no arguments is still a
call when written bare — `text.banner` is its value — because there is nothing
to hand it.

```lume-skip
import lib.text

def main:
  puts ["ab", "lume"].map(text.shout)    # ["AB", "LUME"]
```

## Blocks of your own

A parameter typed `(A) -> B` takes a block, so the built-in methods are not a
closed set. `()` as the result means the block is run for what it does.

```lume
def keep[T](xs: [T], ok: (T) -> Bool) -> [T]:
  var out: [T] = []
  for x in xs:
    if ok(x):
      out.push(x)
  out

def times(n: Int, f: (Int) -> ()):
  var i = 0
  while i < n:
    f(i)
    i += 1

def main:
  puts keep([1, 2, 3, 4]) { |n| n % 2 == 1 }    #=> [1, 3]
  puts keep([1, 2, 3, 4], _ > 2)                #=> [3, 4]
  times(3) { |i| puts "run #{i}" }              #=> run 0
                                                #=> run 1
                                                #=> run 2
```

A block closes over what is around it, and may change it:

```lume
def each[T](xs: [T], f: (T) -> ()):
  for x in xs:
    f(x)

def main:
  factor = 10
  var total = 0
  each([1, 2, 3]) { |n| total += n * factor }
  puts total    #=> 60
```

Methods take blocks the same way:

```lume
struct Bag[T]:
  items: [T]

  def each(f: (T) -> ()):
    for x in items:
      f(x)

def main:
  Bag(items: [1, 2]).each { |n| puts n }    #=> 1
                                            #=> 2
```

## A block with no argument

There is one, written `() -> ()`. **It needs either the empty `()` on the
call or the `do` form** — a bare `name { ... }` with nothing before the brace
and nothing inside the bars is not a call the parser will take. (A block that
*does* take parameters is fine after a bare name: `find_first { |i| i > 3 }`
works, because the `|` says it is a block.)

```lume
def twice(f: () -> ()):
  f()
  f()

def main:
  twice() { puts "tick" }    #=> tick
                             #=> tick
  twice do
    puts "tock"              #=> tock
                             #=> tock
```

```lume-bad
def twice(f: () -> ()):
  f()
  f()

def main:
  twice { puts "tick" }
#! expected the end of the line, found `{`
```

When some other argument already puts parentheses on the call, `{ ... }`
follows it with nothing else needed:

```lume
def time_it(label: Str, work: () -> ()):
  work()
  puts "done #{label}"

def main:
  time_it("a") { puts "inside" }    #=> inside
                                    #=> done a
  time_it("b") do
    puts "inside b"                 #=> inside b
                                    #=> done b
```

## What fits inside `{ }`

**One expression.** Lume ends a statement at the end of a line, and an inline
block is one line, so there is nowhere for a second statement to go. There is
no `;`.

```lume-bad
puts [1, 2, 3].map { |n| n * 2; n + 1 }
#! unexpected character `;`
```

```lume-bad
puts [1, 2, 3].map { |n| n * 2
  n + 1 }
#! expected the end of the line, found `n`
```

That one expression may be as large as you like, including an `if`:

```lume
puts [1, 2, 3].map { |n| if n % 2 == 0: "even" else: "odd" }.join(",")
#=> odd,even,odd
```

For more than one statement, use `do`.

## Names inside a block

A block parameter may reuse a name from around it. The outer one is untouched
after the block:

```lume
x = 5
puts [1, 2, 3].map { |x| x * 2 }    #=> [2, 4, 6]
puts x                              #=> 5
```

## `?` inside a block

> Reviewers guessed at this across rounds. The answer is narrow.

**`?` inside a block leaves the block, never the function around it** — and it
is only allowed when the block's own result type can carry the failure. A
block handed to `map` or `each` returns a plain value, so `?` there is
refused outright:

```lume-bad
def parse_each(xs: [Str]) -> [Int] or Error:
  xs.map { |s| s.to_int? }.to_list

def main:
  puts parse_each(["1"])
#! `?` cannot be used inside a block
```

A parameter typed `(A) -> B or Error` is a different matter. There `?` is the
block's own early exit, and the caller of the block sees an `Error`:

```lume
def run(f: (Str) -> Int or Error) -> Str:
  match f("nope"):
    Ok(n)    -> "got #{n}"
    Error(e) -> "the block stopped: #{e}"

def caller -> Str:
  r = run do |s|
    n = s.to_int?
    n * 2
  "and we carried on — #{r}"

def main:
  puts caller()
  #=> and we carried on — the block stopped: Error("`nope` is not an integer")
```

Note the last line of that block: a bare value where `T or Error` is wanted is
the implied `Ok`, exactly as in a function. **A block whose result is a `T?`
has no matching implied `Some` — write it out.** A bare value there passes
`lume check` and then fails to build, so the compiler is no help; this is a
gap rather than a rule.

```lume
def pick(f: (Int) -> Str?) -> Str:
  f(0).or("nothing")

def main:
  puts pick do |n|
    s = ["a", "b"][n]?
    Some(s.upcase)
  #=> A
```

## Behaviour is not a value

**A block can be passed to a function and run there. It cannot be stored in a
binding, put in a field, or handed back.** Each of those is refused by name:

```lume-bad
struct Button:
  label: Str
  on_click: (Int) -> ()

def main:
  puts "never gets here"
#! a block cannot be a field
```

```lume-bad
def double(n: Int) -> Int = n * 2

def pick -> (Int) -> Int:
  double

def main:
  puts "never gets here"
#! a block cannot be a result
```

```lume-bad
fs: [(Int) -> Int] = []
#! a block cannot be a binding
```

```lume-bad
f = { |x| x + 1 }
#! a `{ |x| ... }` block goes after a method call
```

### What to write instead

**Name the choices in an `enum` and `match` on it.** The enum is an ordinary
value, so it goes in a field, in a list, and across a return, and it prints
and compares like any other:

```lume
enum Step:
  Double
  AddOne
  Square

def apply(s: Step, n: Int) -> Int:
  match s:
    Double -> n * 2
    AddOne -> n + 1
    Square -> n * n

struct Pipeline:
  steps: [Step]

  def run(n: Int) -> Int:
    var out = n
    for s in steps:
      out = apply(s, out)
    out

def next_step(s: Step) -> Step:
  match s:
    Double -> AddOne
    AddOne -> Square
    Square -> Double

def main:
  p = Pipeline(steps: [Double, AddOne, Square])
  puts p.run(3)              #=> 49
  puts p                     #=> Pipeline(steps: [Double, AddOne, Square])
  puts next_step(Double)     #=> AddOne
  puts Double == Double      #=> true
  puts p.steps.contains?(Square)    #=> true
```

This is more to write than a stored block, and it buys something back: the
pipeline above can be printed, compared and saved, which a field full of
behaviour could not be.

`examples/blocks.lume` and `examples/blocks_of_your_own.lume` are longer
runnable versions of this page.

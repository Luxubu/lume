# Names: keywords, reserved words, conventions

> All three reviewers of one round asked the same thing, independently:
> which words are reserved? This page answers that, and the questions that
> follow from it: which names mean something because of their capital
> letter, and when a name may be bound twice. Every answer was checked by
> running it.

## The keywords

**These 35 words are keywords. None of them can name a variable, a
parameter or a function.**

| | |
|---|---|
| defining | `def` `var` `const` `struct` `enum` `interface` `extend` `pub` `import` `test` |
| control | `if` `elif` `else` `unless` `while` `for` `in` `where` `match` `return` `break` `next` |
| values | `true` `false` `and` `or` `not` |
| output and checks | `puts` `warn` `assert` |
| concurrency | `async` `await` `spawn` `shared` |
| escape hatch | `rust` |

```lume-bad
in = 1
#! unexpected keyword `in` here
```

```lume-bad
def f(in: Int) -> Int = in

def main:
  puts f(1)
#! `in` is a keyword and cannot be used as a parameter name
```

The error differs from word to word, because each keyword starts its own
kind of line; `if = 1` says ``expected a value, found `=` ``.

**`const` is reserved, but Lume constants do not use it.** A constant is a
name and a value at the top level of a file, with no keyword:

```lume
LIMIT = 10

def main:
  puts LIMIT    #=> 10
```

```lume-bad
const LIMIT = 10

def main:
  puts LIMIT
#! found `const`
```

Constants are covered fully in [Modules](modules.md).

`self` is not a keyword, but it means something only inside a method:

```lume-bad
def helper -> Int = self.x

def main:
  puts helper()
#! `self` is only meaningful inside a method
```

## Rust's words are yours

Lume compiles to Rust, and Rust keeps many words that Lume does not.
**Those are renamed quietly in the Rust that Lume writes, so you can use
them.** A variable called `type` or `loop` works:

```lume
type = 1
loop = 2
fn = 3
impl = 4
mod = 5
move = 6
ref = 7
puts [type, loop, fn, impl, mod, move, ref].join(" ")    #=> 1 2 3 4 5 6 7
```

The same goes for `as`, `box`, `crate`, `dyn`, `let`, `mut`, `static`,
`super`, `trait`, `union`, `unsafe`, `use`, `yield`, `try`, `continue`,
`macro`, `extern`, `abstract` and `typeof`. They work as fields and
function names too:

```lume
struct Token:
  type: Str
  ref: Int

def loop(n: Int) -> Int = n + 1

def main:
  t = Token(type: "word", ref: 3)
  puts t.type     #=> word
  puts t          #=> Token(type: "word", ref: 3)
  puts loop(1)    #=> 2
```

Rust's type names are not a problem for a *variable* either:

```lume
Self = 1
String = 2
Vec = 3
Option = 4
Result = 5
Err = 6
puts [Self, String, Vec, Option, Result, Err].join(" ")    #=> 1 2 3 4 5 6
```

**`Some`, `Ok`, `None` and `Error` are the exception.** Each one builds a
value wherever it appears, so it cannot also be a name:

```lume-bad
def main:
  Some = 1
#! `Some` builds a value, so it cannot also be a name
```

**A field may be named with a keyword** that does not open a block, such as
`match` or `next`, and it can be set by name and read like any other:

```lume
struct Step:
  next: Int
  match: Str

def main:
  s = Step(next: 2, match: "all")
  puts "#{s.next} #{s.match}"    #=> 2 all
```

## Names that cannot be a type

**A `struct`, `enum` or `interface` cannot be called by a name the language
or the generated code already uses.** The error says which:

```lume-bad
struct Map:
  size: Int

def main:
  puts 1
#! `Map` cannot be a type name: this is a built-in Lume name
#! pick another name, like `MyMap`
```

```lume-bad
struct String:
  text: Str

def main:
  puts 1
#! `String` cannot be a type name: the compiler keeps this name for itself
```

```lume-bad
struct Error:
  message: Str

def main:
  puts 1
#! `Error` is the built-in error type
```

The whole list:

| refused as a type name | because |
|---|---|
| `Int` `Float` `Str` `Char` `Bool` | the scalar types |
| `List` `Map` `Set` `Option` | built-in Lume names |
| `Some` `None` `Ok` `Err` | the variants of `T?` and `T or Error` |
| `File` `Time` `Math` | built-in Lume names |
| `Error` | the built-in error type |
| `Vec` `String` `Rc` `Arc` `Mutex` `Clone` `Copy` `Iterator` `Ordering` `Self` | the compiler keeps these for itself |
| `Default` `Debug` `Display` `Eq` `PartialEq` `Ord` `PartialOrd` `Hash` `IntoIterator` `ToOwned` `ToString` `From` `Into` `LumeMap` `LumeSet` `LumeShow` | names the generated Rust uses by name |

`Result`, `Box` and `Iter` are **not** refused: `struct Result:` works.

`Dir`, `Path` and `Env` are not refused either, on purpose: `enum Dir:` for
a direction is one of the most natural names there is, and two independent
reviewers wrote exactly that. A type of yours with one of those names hides
the built-in in that file — after `enum Dir:`, `Dir.list(...)` means your
type, not the functions of [Input and output](io.md).

`Task` is not refused either. Reviewers in two review rounds running
reached for it for a to-do item, and Rust lets a type of yours shadow a name
from its prelude. A `struct Task` or `enum Task` of your own, or one you
import by name, is what `Task` means in that file, and `Task[..]` means
yours too:

```lume
struct Task:
  title: Str
  done: Bool

async def main:
  todo = [Task(title: "write", done: true), Task(title: "test", done: false)]
  names = todo.map { |t| spawn: t.title.upcase }.to_list
  puts await names     #=> ["WRITE", "TEST"]
```

`spawn:` still works in that file. What it gives back is still a task, but
there is no longer a name to write its type with. Leave the type out, as
above, and Lume works it out. `[Task[Int]]` in that file names your `Task`,
and the error says so.

## What the capital letter means

**A name that starts with a capital letter is a type, a variant or, by
convention, a constant. A lowercase name is a binding.** The compiler
relies on the first half:

```lume-bad
struct point:
  x: Int

def main:
  puts 1
#! struct names start with a capital letter: `point`
#! rename it `Point`
```

`enum`, `interface` and variant names are checked the same way.

In a pattern, the capital letter decides what a name *does*. **A lowercase
name in a pattern binds whatever is there; a capitalised one must be a
variant.**

```lume
enum Color:
  Red
  Green

def main:
  c = Green
  match c:
    Red -> puts "red"
    other -> puts "something else: #{other}"    #=> something else: Green
```

`other` is not compared with anything. It is a new name for the value, so
that arm catches everything the arms above it did not.

Because a capitalised name in a pattern must be a variant, **a constant
cannot be matched against by name**, even though it is capitalised by
convention:

```lume-bad
LIMIT = 10

def main:
  match 3:
    LIMIT -> puts "at the limit"
    _ -> puts "no"
#! unknown variant `LIMIT`
#! lowercase names are bindings
```

Compare with `==` instead: `if n == LIMIT:`.

For constants and functions the capital letter is only a convention: a
lowercase constant (`limit = 10` at the top level) and a capitalised
function (`def Foo`) are both accepted. Keep to the convention anyway,
because a reader relies on it.

## A name may end in `?`

**A function, method or variable name may end in `?`.** By convention it
answers yes or no. The built-in methods do this — `empty?`, `digit?`,
`contains?`, `File.exists?` — and so can yours:

```lume
struct Stack:
  items: [Int]

  def empty? -> Bool = items.empty?

def even_len?(s: Str) -> Bool = s.len % 2 == 0

def main:
  s = Stack(items: [])
  puts s.empty?          #=> true
  puts even_len?("ab")   #=> true
  done? = false
  puts done?             #=> false
```

`?` is the only mark a name can end with. There is no `!`.

## Binding a name again

**A name without `var` is bound once. The one exception is a
*self-transform*: in the same block, `name = name…` makes a new `name` from
the old one.**

The right-hand side must begin with the name itself, followed by a method
call, an operator, a range or `?`:

```lume
name = "  lume  "
name = name.trim
name = name.upcase
puts "[#{name}]"    #=> [LUME]

n = "21"
n = n.to_int.or(0)  # the type may change: a Str, then an Int
n = n * 2
puts n              #=> 42
```

`examples/shadow.lume` is exactly this. Anything else that binds a name
again is refused, because it is not a transform of the same value:

```lume-bad
s = "a"
s = "b" + s
#! `s` is immutable and cannot be reassigned
#! `s = s.something` is allowed as a transform of the same value
```

A function call does not start with the name, so `n = double(n)` is refused
the same way, and so is `n = n |> double`. For a value that changes, use
`var`.

**In a nested block, even a self-transform is refused.** A new name there
would hide the outer one and vanish when the block ends, and the outer one
would quietly stay as it was:

```lume-bad
def main:
  total = 0
  for i in [1, 2, 3]:
    total = total + i
  puts total
#! `total` is declared on line 2, outside this block; a new `total` here would hide it and be lost when the block ends
#! declare it `var total` on line 2
```

The fix is `var total = 0`; then `total = total + i` changes it. This is
what [the tour](../tour.md) means by "a name cannot be re-bound in a nested
block".

**Two sibling blocks may each bind the same name.** Neither is inside the
other, so neither hides anything:

```lume
for i in [1, 2]:
  puts i            #=> 1
                    #=> 2
for i in [3]:
  puts i            #=> 3
if true:
  label = "first"
  puts label        #=> first
label = "after"
puts label          #=> after
```

**A loop variable, a block parameter or a name in a pattern is always a new
name**, and it may reuse a name from outside. Inside, it hides the outer
one; after the loop, block or arm, the outer one is back:

```lume
i = 9
for i in [0, 1]:
  puts i          #=> 0
                  #=> 1
puts i            #=> 9

x = 1
match Some(3):
  Some(x) -> puts x    #=> 3
  None -> puts 0
puts x                 #=> 1
```

A constant is never bound again, not even at the top of `main`:

```lume-bad
LIMIT = 10

def main:
  LIMIT = 11
#! `LIMIT` is a constant, so it cannot be bound again
```

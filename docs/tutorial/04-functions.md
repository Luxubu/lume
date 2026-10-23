# Lunch 4: Functions

**In this lunch you will:**

- give a piece of work a name, and use it as often as you like;
- hand a function what it needs, by position or by name;
- say what a function gives back, and leave early when there is no more to do;
- read the messages Lume gives when a call does not fit.

## 4.1 Your first function

You have written one function already: `main`. Others are written the same
way, with `def`, a name, and an indented body:

```lume
def greet(name: Str) -> Str:
  "Hello, #{name}!"

def main:
  puts greet("Ada")     #=> Hello, Ada!
  puts greet("Grace")   #=> Hello, Grace!
```

Read the first line from left to right: a function named `greet` takes one
value, called `name`, which is a `Str`, and gives back (`->`) a `Str`.

**The last line of the body is what the function gives back.** There is no
need to say `return`: `"Hello, #{name}!"` is the last line, so that is the
result.

`main` can call `greet` although `greet` is written above it, and it could
just as well be written below: the order of functions in a file does not
matter.

## 4.2 Short functions on one line

A function whose body is a single expression can be written on one line,
with `=`:

```lume
def double(n: Int) -> Int = n * 2

def with_tip(bill: Float) -> Float = bill * 1.1

def main:
  puts double(21)                       #=> 42
  puts with_tip(20.0).decimals(2)       #=> 22.00
```

The two shapes mean exactly the same thing. Use whichever reads better.

## 4.3 Several parameters, and calling by name

Parameters are separated by commas, and so are the values you give a call:

```lume
def price(dish: Str, count: Int) -> Int:
  each = if dish == "soup": 4 else: 3
  each * count

def main:
  puts price("soup", 2)     #=> 8
  puts price("bread", 5)    #=> 15
```

The `if` there picks one of two values; [lunch 5](05-making-decisions.md) is about it.

**A call may also give each value by its parameter's name**, and then the
order does not matter:

```lume
def price(dish: Str, count: Int) -> Int = count * 4

def main:
  puts price(dish: "soup", count: 2)    #=> 8
  puts price(count: 3, dish: "tea")     #=> 12
```

Names make a call easier to read when the values alone would not say what
they are for — `book(2, 3)` is a puzzle, `book(guests: 2, nights: 3)` is
not.

## 4.4 What the types are for

**Every parameter says its type**, and a call that does not fit is caught
before the program runs:

```lume-bad
def price(dish: Str, count: Int) -> Int = count * 4

def main:
  puts price("soup", "two")
#! `price` takes `count: Int`, but this is a `Str`
```

```lume-bad
def price(dish: Str, count: Int) -> Int = count * 4

def main:
  puts price("soup")
#! `price` is missing an argument: count
```

A parameter without a type is refused too: the types of a function's
parameters are part of what it says to the reader, so Lume asks for them.

```lume-bad
def double(n) = n * 2

def main:
  puts double(2)
#! parameter `n` needs a type
```

**The result type may be left off**, and Lume works it out from the body.
Writing it is a promise the compiler checks:

```lume-bad
def label(seats: Int) -> Str:
  seats

def main:
  puts label(4)
#! `label` returns `Str`, but this is an `Int`
```

A good habit: write the result type on any function someone else might
call, and leave it off small helpers whose result is obvious.

Lume has no default values for parameters: a call gives every one. When
most calls want the same value, write a second, shorter function that
calls the first.

## 4.5 Functions with nothing to give

Some functions do their work and have nothing to hand back — printing, for
one. Leave off the `->`, and the result:

```lume
def announce(dish: Str):
  puts "Now serving: #{dish}"

def main:
  announce("soup")     #=> Now serving: soup
  announce("pie")      #=> Now serving: pie
```

## 4.6 Functions with no parameters

A function may take nothing at all. **Call it with empty parentheses**, so
that the call cannot be mistaken for a name:

```lume
def special -> Str = "today's soup"

def main:
  puts special()    #=> today's soup
```

Leave them off and Lume points it out:

```lume-bad
def special -> Str = "today's soup"

def main:
  puts special
#! `special` is a function; call it with `special()`
```

## 4.7 Leaving early with `return`

`return` leaves a function at once, with a value. It is for the cases that
are settled early, so the rest of the body can deal with the usual case:

```lume
def portion(guests: Int) -> Str:
  if guests <= 0:
    return "nothing to cook"
  if guests == 1:
    return "one bowl"
  "#{guests} bowls"

def main:
  puts portion(0)    #=> nothing to cook
  puts portion(1)    #=> one bowl
  puts portion(6)    #=> 6 bowls
```

## 4.8 A function may call itself

A function can call other functions — and itself. A function that calls
itself must have a case where it stops:

```lume
def countdown(n: Int) -> Str:
  if n == 0:
    return "lunch!"
  "#{n} " + countdown(n - 1)

def main:
  puts countdown(3)    #=> 3 2 1 lunch!
```

`countdown(3)` is `"3 "` followed by `countdown(2)`, which is `"2 "`
followed by `countdown(1)`, and so on, until `countdown(0)` gives
`"lunch!"`.

## Underneath, in Rust

Each `def` becomes a Rust `fn`. Lume passes an `Int`, a `Float` or a `Bool`
by value, and lends text and lists to the function rather than copying
them — in Rust terms, a `Str` parameter is a `&str` — so calling a function
with a long text costs nothing. Rust requires every function's result type
to be written; when you leave it off, Lume works it out and writes it for
you. A call by name, `price(count: 3, dish: "tea")`, is put back in the
parameters' order, so Rust sees an ordinary call.

## Try it

1. Write `square(n: Int) -> Int` on one line, and print `square(9)`.
2. Write `bill(dishes: Int, each: Int, tip: Int) -> Int`, and call it by name
   with the values in a different order from the parameters.
3. Write `size(guests: Int) -> Str` that gives `"small"` for up to 2 guests,
   `"medium"` for up to 6, and `"large"` otherwise, using `return` for the
   first two.
4. Write `stars(n: Int) -> Str` that calls itself and gives `n` stars, so
   that `stars(3)` is `***`. (Two stars are `"*" + stars(1)`.)

## Recap

- `def name(param: Type) -> Result:` and an indented body, or `= expr` on
  one line. The last line is the result.
- Every parameter has a type; the result type may be left off.
- A call gives values in order, or by name in any order.
- A function with no parameters is called with `()`.
- `return` leaves early with a value. A function may call itself, as long
  as it has a case where it stops.

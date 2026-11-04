# Lunch 12: Generics

**In this lunch you will:**

- write a function once, for values of any type;
- say what those types must be able to do;
- write a struct that holds values of any type.

## 12.1 Once, for any type

This function gives the second item of a list of `Int`s:

```lume
def second(xs: [Int]) -> Int? = xs[1]

def main:
  puts second([3, 1, 2])    #=> Some(1)
```

Nothing in its body cares that the items are `Int`s. Written for `[Str]`
it would be the same line again. Instead, **give the type a name in
brackets after the function's name**, and use the name where the type
goes:

```lume
def second[T](xs: [T]) -> T? = xs[1]

def main:
  puts second([3, 1, 2])           #=> Some(1)
  puts second(["soup", "pie"])     #=> Some("pie")
  puts second([1.5])               #=> None
```

`[T]` after the name says "this works for any type; call it `T`". The
caller never writes `T`: Lume reads it off the arguments — `Int` for the
first call, `Str` for the second. A function like this is called
**generic**.

## 12.2 What the type must be able to do

A bare `T` could be anything, so a generic function can do little with
it: pass it around, compare it with `==`, put it in text. To ask for more,
add a **bound** after a `:`. `Ordered` means the values can be compared
with `<`, which `max` needs:

```lume
def best[T: Ordered](xs: [T]) -> T? = xs.max

def main:
  puts best([3, 9, 2])           #=> Some(9)
  puts best(["fig", "pear"])     #=> Some("pear")
```

**`Hashable`** is the other built-in bound: the values can be a map's keys
or a set's items, which counting by value needs:

```lume
def tally[T: Hashable](xs: [T]) -> {T: Int}:
  var counts: {T: Int} = {}
  for x in xs:
    counts[x] = counts[x].or(0) + 1
  counts

def main:
  puts tally(["tea", "pie", "tea"])    #=> {"tea": 2, "pie": 1}
  puts tally([3, 3, 1])                #=> {3: 2, 1: 1}
```

Without the bound, Lume points out what is missing:

```lume-bad
def bigger[T](a: T, b: T) -> T = if a < b: b else: a

def main:
  puts bigger(1, 2)
#! `T` could be any type, so `<` has nothing to compare
```

**Any interface from lunch 11 is a bound too**, so a generic function can
call that interface's methods:

```lume
interface Priced:
  def price -> Int

struct Dish:
  name: Str
  cents: Int

  def price -> Int = cents

def total[T: Priced](items: [T]) -> Int = items.sum { |i| i.price }

def main:
  puts total([Dish("soup", 400), Dish("pie", 350)])    #=> 750
```

## 12.3 Generic types

A struct or an enum can take a type the same way. A queue of orders works
the same whatever is queued:

```lume
struct Queue[T]:
  waiting: [T]

  def join(var self, x: T):
    waiting.push(x)

  def next_up -> T? = waiting.first

  def size -> Int = waiting.len

def main:
  var q: Queue[Str] = Queue(waiting: [])
  q.join("table 4")
  q.join("table 9")
  puts q.size       #=> 2
  puts q.next_up    #=> Some("table 4")
```

The empty queue gives Lume nothing to read `T` off, so the name says it:
`Queue[Str]`.

## Underneath, in Rust

A generic function is a Rust generic function, and Rust compiles a copy
of it for each type it is used with — so `second` on `[Int]` runs as fast
as one written for `Int` alone. `Ordered` is Rust's `PartialOrd`, and an
interface bound is a trait bound. `Queue[T]` is a Rust `struct Queue<T>`.

## Try it

1. Write `last_two[T](xs: [T]) -> [T]` that gives the last two items of a
   list (or fewer, if it has fewer). `xs.skip(n)` drops the first `n`, and
   a negative `n` drops none.
2. Write `count_of[T](xs: [T], x: T) -> Int` that counts how many items
   equal `x`, and use it on a list of `Int`s and a list of `Str`s.
3. Make a generic `Pair[A, B]` struct with fields `left` and `right`, and a
   method `swap -> Pair[B, A]`.

## Recap

- `def name[T](...)` and `struct Name[T]:` work for any type `T`, read off
  the values given.
- A bound — `[T: Ordered]`, `[T: Hashable]`, or any interface — says what
  `T` must be able to do.
- When there is nothing to read `T` off, as with an empty `Queue`, the
  type says it: `Queue[Str]`.

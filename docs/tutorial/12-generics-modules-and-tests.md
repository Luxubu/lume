# Lunch 12: Generics, modules and tests

**In this lunch you will:**

- write a function or a type once, for values of any type;
- say what those types must be able to do;
- split a program into files, and choose what each one shares;
- write tests, and run them with `lume test`.

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

## 12.4 Splitting a program into files

A program grows. **Each file is a module**, and one file can use another's
work by importing it. The folder `docs/tutorial/cafe/` in Lume's
repository holds a small café program in three files:

```text
cafe/
  main.lume
  menu.lume
  kitchen/
    orders.lume
```

`menu.lume` keeps the menu. **Only what is marked `pub` can be reached from
another file**; the rest is the file's own business. `pub` goes on the
things at the top of a file — a struct, a function, a constant — and a
`pub struct`'s fields and methods come with it, with no `pub` of their own:

```lume-skip
# menu.lume — what the café sells. Only `pub` items can be reached from
# another file.

pub struct Dish:
  name: Str
  cents: Int

PRICES = {"soup": 400, "pie": 350, "stew": 600}

# A dish by name, or a free glass of water when there is no such dish.
pub def find(name: Str) -> Dish:
  match PRICES[name]:
    Some(c) -> Dish(name: name, cents: c)
    None -> water()

# Not `pub`: only this file can call it.
def water -> Dish = Dish(name: "water", cents: 0)
```

`kitchen/orders.lume` imports the one thing it needs from the menu, the
`Dish` type, by its path:

```lume-skip
# kitchen/orders.lume — an order for one table. Its import path starts from
# the folder that holds main.lume, as every path does.

import menu.Dish

pub KITCHEN = "the back kitchen"

pub struct Order:
  table: Int
  dishes: [Dish]

  def add(var self, d: Dish):
    dishes.push(d)

  def total -> Int = dishes.sum { |d| d.cents }

  def summary -> Str = "table #{table}: #{dishes.map(_.name).join(", ")} = #{total}"
```

And `main.lume`, the file you run, imports both:

```lume-skip
# The café from lunch 12, in three files: run it with `lume run main.lume`
# from this folder. Each `import` path starts from this folder.

import menu
import kitchen.orders
import kitchen.orders.Order

def main:
  var o = Order(table: 4, dishes: [])
  o.add(menu.find("soup"))
  o.add(menu.find("pie"))
  o.add(menu.find("steak"))
  puts o.summary
  puts "served by #{orders.KITCHEN}"
```

```text
table 4: soup, pie, water = 750
served by the back kitchen
```

There are two ways to import:

- **`import menu`** brings the module, and its things are used through its
  name: `menu.find("soup")`. A reader always sees where `find` came from.
- **`import kitchen.orders.Order`** brings one thing, used by its own name:
  `Order(...)`.

**A path is the file's place, with dots for slashes and no `.lume`**, and
every path starts from the folder that holds the file you run — inside
`kitchen/` too.

So check and run a program of several files **through the file that has
`main`**: `lume check main.lume` checks `menu.lume` and
`kitchen/orders.lume` with it. On its own, `lume check kitchen/orders.lume`
cannot find `menu`, because its paths start from where `main.lume` is.

## 12.5 Tests

A **test** is a block in a file, with a name, that checks the code does
what it should. `assert` takes a condition that must be `true`:

```lume
def split_bill(total: Int, people: Int) -> Int = total / people

test "a bill splits evenly":
  assert split_bill(12, 3) == 4

test "a remainder is dropped":
  assert split_bill(10, 3) == 3

def main:
  puts split_bill(12, 3)    #=> 4
```

`lume run` runs `main` and leaves the tests alone. **`lume test` runs the
tests**, and leaves `main` alone:

```sh
lume test bill.lume
```

```text
test a bill splits evenly ... ok
test a remainder is dropped ... ok

2 tests: 2 passed, 0 failed
```

A failing `assert` says which, and shows both sides of a comparison, so you
can see what it got. Change the second test to `assert split_bill(10, 4) ==
3`, and `lume test bill.lume` says what follows the program:

```lume
def split_bill(total: Int, people: Int) -> Int = total / people

test "a bill splits evenly":
  assert split_bill(12, 3) == 4

test "a remainder is dropped":
  assert split_bill(10, 4) == 3

def main:
  puts split_bill(12, 3)    #=> 4
```

```text
test a bill splits evenly ... ok
test a remainder is dropped ... FAILED
    bill.lume:7: assert split_bill(10, 4) == 3
      left:  2
      right: 3

2 tests: 1 passed, 1 failed
```

`lume test` exits with status 1 when any test fails, so a script or a
build server can tell. `lume test bill.lume remainder` runs only the tests
with `remainder` in their name. [Tests](../reference/testing.md) in the
reference has the rest, and how to test the async code of lunch 16 with
`async test`.

Write tests for the things that would be easy to get wrong and hard to
notice: the edge cases, like an empty list or a remainder. Running them
after every change takes a second, and catches the day a change breaks
something three files away.

## 12.6 Packages

A program you want to share with others, or a library many programs use,
is a **package**: a folder with a `lume.toml` file saying its name, and
which other packages it uses. `lume new cafe` makes one. In a package,
`lume run`, `lume test` and the rest need no file name. Packages are
fetched from a folder or a git repository; the
[packages reference](../reference/packages.md) has the details, and Part 3
of these lunches builds programs that use them.

## Underneath, in Rust

A generic function is a Rust generic function, and Rust compiles a copy
of it for each type it is used with — so `second` on `[Int]` runs as fast
as one written for `Int` alone. `Ordered` is Rust's `PartialOrd` and an
interface bound is a trait bound. Each Lume file becomes a Rust module, and
`pub` is Rust's `pub`. A test becomes a plain Rust function, and `lume
test` builds a small runner that calls each one and catches its failure, as
`cargo test` does for `#[test]` functions.

## Try it

1. Write `last_two[T](xs: [T]) -> [T]` that gives the last two items of a
   list (or fewer, if it has fewer). `xs.skip(n)` drops the first `n`, and
   a negative `n` drops none.
2. Write `count_of[T](xs: [T], x: T) -> Int` that counts how many items
   equal `x`, and use it on a list of `Int`s and a list of `Str`s.
3. Make a generic `Pair[A, B]` struct with fields `left` and `right`, and a
   method `swap -> Pair[B, A]`.
4. Write two tests for your `count_of`: one for a list where `x` appears
   twice, and one for an empty list.

## Recap

- `def name[T](...)` and `struct Name[T]:` work for any type `T`, read off
  the values given.
- A bound — `[T: Ordered]`, `[T: Hashable]`, or any interface — says what
  `T` must be able to do.
- Each file is a module. `pub` marks what other files may use; `import
  path` brings a module, `import path.Thing` one thing. Paths start from
  the folder of the file you run.
- `test "name":` blocks with `assert` check your code; `lume test` runs
  them.
- A package is a folder with a `lume.toml`, made with `lume new`.

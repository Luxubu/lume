# Lunch 14: Generators

**In this lunch you will:**

- write a function that hands out values one at a time, with `yield`;
- see that it does its work only as each value is asked for;
- make sequences that never end, and take as much of them as you need;
- connect generators into a pipeline.

## 14.1 A function that hands out values one at a time

A function gives back one value. A **generator** hands out several, one
at a time: each `yield` hands out the next. It says what kind of values,
with `Iterator[T]`:

```lume
def courses -> Iterator[Str]:
  yield "starter"
  yield "main"
  yield "dessert"

def main:
  for c in courses():
    puts c    #=> starter
              #=> main
              #=> dessert
```

A generator can be walked with `for`, as here, and it has every method a
list has: `map`, `filter`, `sum`, `to_list`, and the rest from lunch 7.

```lume
def courses -> Iterator[Str]:
  yield "starter"
  yield "main"
  yield "dessert"

def main:
  puts courses().map(_.upcase).to_list    #=> ["STARTER", "MAIN", "DESSERT"]
  puts courses().filter(_.len > 4).to_list    #=> ["starter", "dessert"]
```

## 14.2 Work done only when asked

What makes a generator different from a function giving a list is
**when** it does its work. A generator does nothing until the first value
is asked for, and stops at each `yield` until the next one is:

```lume
def cooked -> Iterator[Str]:
  for dish in ["soup", "pie"]:
    puts "  (cooking #{dish})"
    yield dish

def main:
  for dish in cooked():
    puts "serving #{dish}"
  #=>   (cooking soup)
  #=> serving soup
  #=>   (cooking pie)
  #=> serving pie
```

The soup is cooked, then served, *before* the pie is cooked. A function
that built a list would have cooked both first. For two dishes it makes
no difference; for a million lines of a log, it is the difference between
holding one line at a time and holding them all.

## 14.3 Sequences that never end

Because a generator only works when asked, it may go on for ever. Whoever
walks it decides when to stop — with `take`, `break` or `find`:

```lume
def ticket_numbers -> Iterator[Int]:
  var n = 100
  while true:
    yield n
    n += 1

def main:
  puts ticket_numbers().take(3).to_list    #=> [100, 101, 102]
  puts ticket_numbers().find { |n| n % 7 == 0 }    #=> Some(105)
  for n in ticket_numbers():
    break if n > 101
    puts n    #=> 100
              #=> 101
```

Each `ticket_numbers()` call starts a new sequence from 100.

## 14.4 Generators with parameters

A generator takes parameters like any function, and `yield` can go in its
loops and its `if`s:

```lume
def portions(guests: Int, per_pot: Int) -> Iterator[Str]:
  var left = guests
  var pot = 1
  while left > 0:
    served = left.min(per_pot)
    yield "pot #{pot}: #{served}"
    left -= served
    pot += 1

def main:
  for line in portions(10, 4):
    puts line    #=> pot 1: 4
                 #=> pot 2: 4
                 #=> pot 3: 2
```

`return` on its own ends a generator early: there will be no more values.

## 14.5 Pipelines

**A parameter of type `Iterator[T]` takes any sequence** — a generator, a
list, a range, a chain — so generators can feed one another. Each step
takes values from the one before as it needs them:

```lume
def ticket_numbers -> Iterator[Int]:
  var n = 100
  while true:
    yield n
    n += 1

def lucky(xs: Iterator[Int]) -> Iterator[Int]:
  for x in xs:
    yield x if x % 10 == 7

def labelled(xs: Iterator[Int]) -> Iterator[Str]:
  for x in xs:
    yield "ticket #{x}"

def main:
  puts labelled(lucky(ticket_numbers())).take(2).to_list
  #=> ["ticket 107", "ticket 117"]
  puts lucky(100..130).to_list    #=> [107, 117, 127]
```

The first line walks an endless sequence through two steps and still
ends: `take(2)` asks for two values, `labelled` asks `lucky` for two, and
`lucky` asks `ticket_numbers` for as many as it takes to find them —
eighteen — and no more.

## 14.6 Walked once

**A generator is used up as it is walked**: once the values have been
handed out, they are gone. So a name holding a generator can be walked
only once, and Lume refuses a second walk:

```lume-bad
def courses -> Iterator[Str]:
  yield "starter"
  yield "main"

def main:
  menu = courses()
  puts menu.to_list
  puts menu.to_list
#! `menu` is a generator, and it is used up here, so it cannot be used after
```

To walk the values twice, keep them in a list first, with `.to_list` — or
call the generator again, which starts it afresh.

## 14.7 One value at a time, by hand

`next` asks a generator held in a `var` for its next value. It gives
`Some(value)`, or `None` once the generator has ended:

```lume
def courses -> Iterator[Str]:
  yield "starter"
  yield "main"

def main:
  var menu = courses()
  puts menu.next    #=> Some("starter")
  puts menu.next    #=> Some("main")
  puts menu.next    #=> None
```

A `for` over `menu` after a `next` carries on from where the `next` left
it.

## 14.8 Methods that are generators

A method can be a generator too. This one walks a menu made of sections,
some inside others, handing out every dish at any depth:

```lume
struct Section:
  title: Str
  dishes: [Str]
  parts: [Section]

  def all_dishes -> Iterator[Str]:
    for d in dishes:
      yield d
    for p in parts:
      for d in p.all_dishes:
        yield d

def main:
  desserts = Section(title: "desserts", dishes: ["pie", "tart"], parts: [])
  menu = Section(title: "menu", dishes: ["soup"], parts: [desserts])
  puts menu.all_dishes.to_list    #=> ["soup", "pie", "tart"]
```

A generator method walks a copy of its value taken when it is called, so
changing the value afterwards does not change what it hands out.

## Underneath, in Rust

A generator is a Rust function giving `impl Iterator`. Rust has no `yield`
of its own yet, so Lume turns the body into a state machine: each `yield`
saves where the body is, and the next request resumes from there. The
result is an ordinary Rust iterator, so a pipeline like 14.5 compiles to
code much like a hand-written loop. Being used up as it is walked is
Rust's own rule: an iterator is *moved* into whatever walks it.

## Try it

1. Write `countdown(from: Int) -> Iterator[Int]` that yields from `from`
   down to `1`, and print it as a list for `5`.
2. Write an endless generator of the square numbers `1, 4, 9, ...` and print
   the first one over 50.
3. Write `only_long(xs: Iterator[Str], n: Int) -> Iterator[Str]` that hands
   on the words longer than `n`, and use it on the words of
   `"a big bowl of hot soup"`.
4. Make the program in 14.6 work, so it prints the list twice.

## Recap

- A function with `yield` is a generator, giving `Iterator[T]`. Each
  `yield` hands out the next value.
- A generator works only as each value is asked for, so it may be
  endless; `take`, `break` and `find` stop it.
- A parameter `Iterator[T]` takes any sequence, so generators chain into
  pipelines.
- A generator is walked once. `.to_list` keeps its values; `next` takes
  them one at a time from a `var`.
- A method can be a generator.

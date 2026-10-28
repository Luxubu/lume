# Lunch 17: Async sequences

> **A preview.** This lunch teaches one of Lume's newest parts. It works
> as shown — every example is checked — but details may change before 1.0.

**In this lunch you will:**

- write a generator whose values take time to arrive;
- walk it as each value comes, and stop when you have enough;
- chain and question such a sequence, as you would a list;
- hand one to a function or a task.

## 17.1 Values that arrive later

Lunch 14's generators hand out values as they are asked for, and each is
ready at once. Some values are not: each order from a kitchen, each line a
slow program prints, each page of a web search. **An `async def` with
`yield` in it is an *async generator*: between its values, it may `await`.**

```lume
async def orders -> Iterator[Str]:
  for dish in ["soup", "pie", "tea"]:
    await Time.sleep(10)
    yield dish

async def main:
  for dish in orders():
    puts "serving #{dish}"    #=> serving soup
                              #=> serving pie
                              #=> serving tea
```

`for` waits for each value as it comes: the soup is served as soon as it
is ready, before the pie is started. Because walking it waits, **an async
generator is walked in async code** — an `async def` or a `spawn:` body,
from lunch 16:

```lume-bad
async def orders -> Iterator[Str]:
  yield "soup"

def main:
  for dish in orders():
    puts dish
#! is an `async` generator, which runs as a task, so it is called from async code
```

## 17.2 A pass with room for one

An async generator runs as a task of its own, and hands each value over
through **one place** — like the pass between a kitchen and the dining
room, with room for a single plate. While a value waits there, the
generator goes on to make its next one, and then stops at its `yield`
until the waiting one is taken. Printing from inside it shows the turns:

```lume
async def orders -> Iterator[Str]:
  for dish in ["soup", "pie", "tea"]:
    puts "  kitchen: #{dish} ready"
    yield dish

async def main:
  for dish in orders():
    await Time.sleep(20)
    puts "waiter: served #{dish}"
  #=>   kitchen: soup ready
  #=>   kitchen: pie ready
  #=>   kitchen: tea ready
  #=> waiter: served soup
  #=> waiter: served pie
  #=> waiter: served tea
```

While the waiter serves the soup, the pie waits at the pass and the tea is
ready in the kitchen's hands — and there the kitchen stops. **It is never
more than two values ahead of what is being used**: one waiting, one made.
A kitchen that could run any number ahead would fill the pass with cold
food; one that could not run ahead at all would keep the waiter waiting.

**When the walker lets go, the generator stops** at its next `yield`. So an
endless one is fine, and `break` ends it:

```lume
async def ticket_numbers -> Iterator[Int]:
  var n = 100
  while true:
    yield n
    n += 1

async def main:
  for n in ticket_numbers():
    break if n > 101
    puts n    #=> 100
              #=> 101
```

## 17.3 Chains over values that arrive later

An async sequence has the chains of lunch 7 — `map`, `filter`, `reject`,
`take_while`, `take`, `skip` and `enumerate` — and each gives an async
sequence too, asking for values only as they are wanted:

```lume
async def ticket_numbers -> Iterator[Int]:
  var n = 100
  while true:
    yield n
    n += 1

async def main:
  lucky = ticket_numbers().filter { |n| n % 10 == 7 }.map { |n| "ticket #{n}" }
  puts lucky.take(2).to_list    #=> ["ticket 107", "ticket 117"]
```

`to_list` waits for every value, so it is only for a sequence that ends —
here, after `take(2)`.

A chain's block runs later, as each value arrives, so **it gets copies of
the names it uses**, as a task does. It cannot change them, nor use `self`
in a method; bind what it needs to a name first.

## 17.4 Questions that wait

The questions of lunch 7 work too, waiting for the values they need:
`first`, `count`, `sum`, `any?`, `all?`, `find`, `each` and `fold`. Each
handles a value as it arrives, without keeping them all, and `find`,
`any?`, `all?` and `first` stop as soon as they have their answer — so they
work on an endless sequence:

```lume
async def orders -> Iterator[Str]:
  for dish in ["soup", "pie", "stew", "tea"]:
    await Time.sleep(5)
    yield dish

async def ticket_numbers -> Iterator[Int]:
  var n = 100
  while true:
    yield n
    n += 1

async def main:
  puts orders().count { |d| d.len == 4 }          #=> 2
  puts orders().find { |d| d.starts_with?("s") }  #=> Some("soup")
  puts ticket_numbers().any? { |n| n % 50 == 0 }  #=> true
  var served = 0
  orders().each { |_| served += 1 }
  puts served                                     #=> 4
```

Their blocks run during the call, so — unlike a chain's — they can change
the names around them, as `each` changes `served`.

**These wait without `await` being written**, as `for` and `to_list` do:
walking an async sequence always waits, so saying so each time would add
nothing. `await` is for calling an `async def` (lunch 16), which is why
17.5 writes `await count_all(...)` but `orders().count`.

## 17.5 Methods, functions and parameters

**A method can be an async generator**, as a plain one could in lunch 14.
And the type of an async sequence can be written: **`async Iterator[T]`**.
A function that hands one on without `yield` says so, and a parameter of
that type takes one:

```lume
struct Kitchen:
  dishes: [Str]

  async def serve -> Iterator[Str]:
    for d in dishes:
      await Time.sleep(5)
      yield d

def long_ones(xs: async Iterator[Str]) -> async Iterator[Str] = xs.filter { |d| d.len > 3 }

async def count_all(xs: async Iterator[Str]) -> Int = xs.count

async def main:
  k = Kitchen(dishes: ["soup", "pie", "stew"])
  puts long_ones(k.serve).to_list    #=> ["soup", "stew"]
  puts await count_all(k.serve)      #=> 3
```

A parameter takes the sequence: afterwards, the caller's name for it is
used up, as a generator is after a walk.

An async generator itself may say either `-> Iterator[T]`, as the examples
here do, or `-> async Iterator[T]`; they mean the same, since its `async
def` already says its values arrive later.

## 17.6 A task that walks one

A task can walk an async sequence — **the task takes it**, as Rust moves a
value into a thread, so the name cannot be used after the `spawn:`:

```lume
async def orders -> Iterator[Str]:
  for dish in ["soup", "pie"]:
    await Time.sleep(5)
    yield dish

async def main:
  stream = orders()
  t = spawn: stream.map(_.upcase).to_list
  puts await t    #=> ["SOUP", "PIE"]
```

## Underneath, in Rust

An async sequence is what Rust calls a *stream*. Rust has no async
generators of its own yet; the popular `async-stream` crate gives them, and
Lume does the same thing: the body runs as a tokio task, and hands each
value over through a one-place channel, which is why it is never more than
two values ahead. The chains and questions are those of Rust's `StreamExt`, and
`async Iterator[T]` is Rust's `impl Stream<Item = T>`.

## Try it

1. Write an async generator `bells(n: Int) -> Iterator[Str]` that sleeps a
   little before each of `"ring 1"` … `"ring n"`, and print `bells(3)` as a
   list.
2. Take the first three even numbers from an endless async generator of
   `1, 2, 3, ...`, doubled, as a list.
3. Use `fold` on `bells(3)` to join its values with `" / "`.
4. Write `async def loudest(xs: async Iterator[Str]) -> Str?` that gives the
   longest value, and call it with an async generator of `"pie"`,
   `"minestrone"` and `"tea"`, in capitals.

## Recap

- An `async def` with `yield` is an async generator: it may `await` between
  values, and is walked in async code.
- It is never more than two values ahead, and stops when the walker lets go.
- It has the chains of a list, each giving an async sequence; their blocks
  get copies of the names they use.
- `first`, `count`, `sum`, `any?`, `all?`, `find`, `each` and `fold` wait
  for the values; `first`, `find`, `any?` and `all?` stop at their answer.
- Its type is `async Iterator[T]`. A method may be one, a function may hand
  one on, a parameter or a task takes one.

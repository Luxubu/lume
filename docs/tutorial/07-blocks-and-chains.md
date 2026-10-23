# Lunch 7: Blocks and chains

**In this lunch you will:**

- hand a piece of code to a method, as a *block*;
- change, keep and drop the items of a list with `map`, `filter` and
  `reject`;
- ask questions of a list with a block;
- chain methods one after another, and write blocks of your own.

## 7.1 A block is code handed to a method

Lunch 6 asked lists questions that needed nothing more: `sum`, `sort`,
`max`. Many questions need you to say *what* to do with each item. You say
it with a **block**: code in braces, with a name between bars for the item
it is given.

```lume
def main:
  prices = [4, 2, 7]
  puts prices.map { |p| p * 2 }    #=> [8, 4, 14]
```

Read it as "map each price `p` to `p * 2`". `map` runs the block once for
each item, and gives back a new list of what the block gave. The block is
not run when it is written — it is handed to `map`, which runs it when it
needs to.

## 7.2 `map`, `filter` and `reject`

These three are the ones you will use most:

```lume
def main:
  dishes = ["soup", "steak", "salad", "pie"]
  puts dishes.map { |d| d.upcase }          #=> ["SOUP", "STEAK", "SALAD", "PIE"]
  puts dishes.filter { |d| d.len > 3 }      #=> ["soup", "steak", "salad"]
  puts dishes.reject { |d| d.len > 3 }      #=> ["pie"]
```

- **`map`** changes each item into something else.
- **`filter`** keeps the items for which the block says `true`.
- **`reject`** drops them, keeping the rest.

The block of `filter` and `reject` must give a `Bool` — it is a yes-or-no
question:

```lume-bad
def main:
  puts [1, 2, 3].filter { |n| n * 2 }
#! the block of `filter` gives back an `Int`, but it must give a `Bool`
```

## 7.3 A shorter way: `_`

When a block only does something to its one item, `_` can stand for the
item and the bars can go:

```lume
def main:
  dishes = ["soup", "steak", "pie"]
  puts dishes.map(_.upcase)       #=> ["SOUP", "STEAK", "PIE"]
  puts dishes.filter(_.len > 3)   #=> ["soup", "steak"]
  puts [1, 2, 3].map(_ * 10)      #=> [10, 20, 30]
```

`_` may appear only once in such a block. When you need the item twice,
write the block out with a name.

## 7.4 Questions that take a block

Lunch 6's `sum` and `max` have partners that ask about each item first:

```lume
def main:
  prices = [4, 2, 7, 3]
  puts prices.any? { |p| p > 6 }      #=> true
  puts prices.all? { |p| p > 1 }      #=> true
  puts prices.count { |p| p > 3 }     #=> 2
  puts prices.find { |p| p > 3 }      #=> Some(4)
  puts prices.find { |p| p > 9 }      #=> None
  puts prices.sum { |p| p * 2 }       #=> 32
```

`find` gives the first item the block says yes to — if there is one, so
it gives `Some(...)` or `None`, like `max`.

Ordering by something other than the items themselves takes a block too.
`sort_by` sorts by what the block gives, and `max_by` and `min_by` find the
item with the largest or smallest answer:

```lume
def main:
  dishes = ["steak", "pie", "salad"]
  puts dishes.sort_by { |d| d.len }    #=> ["pie", "steak", "salad"]
  puts dishes.max_by { |d| d.len }     #=> Some("steak")
```

`"steak"` and `"salad"` are the same length; `sort_by` keeps them in the
order they came in, and `max_by` gives the first of them.

## 7.5 Chains

Each of these methods gives a list, so another can follow it. Read a chain
from left to right, one step at a time:

```lume
def main:
  orders = ["tea", "soup", "pie", "stew", "tea"]
  puts orders.filter { |o| o.len == 4 }.map(_.upcase).join(" and ")    #=> SOUP and STEW
```

When a chain grows too long for one line, put each step on a line of its
own, starting with `.`, indented under the line it continues:

```lume
def main:
  orders = ["tea", "soup", "pie", "stew", "tea"]
  loud = orders
    .filter { |o| o.len == 4 }
    .map(_.upcase)
    .join(" and ")
  puts loud    #=> SOUP and STEW
```

A line that begins with `.` always continues the line above. Or give the
steps names. Each name says what that step is for, which the chain alone
does not:

```lume
def main:
  prices = [12, 3, 8, 15, 4]
  dear = prices.filter { |p| p > 5 }
  doubled = dear.map { |p| p * 2 }
  puts doubled.sort    #=> [16, 24, 30]
```

Lume does not build a list at every step: the whole chain runs as one pass
over the items. Where a list is wanted — a name you keep, a function's
result, a field — Lume makes one, so you rarely need to say so. When you
do want to, `.to_list` makes it explicit.

## 7.6 Longer blocks: `do`

A block in braces holds one expression. When it needs more, write `do |x|`
at the end of the line, and the block's body indented below:

```lume
def main:
  var total = 0
  [4, 2, 7].each do |p|
    with_tip = p + 1
    total += with_tip
  puts total    #=> 16
```

`each` runs the block for each item and gives nothing back — it is for
blocks that *do* something, as this one does to `total`. A block can use,
and change, the names around it.

## 7.7 `fold`: everything into one value

`fold` builds one answer from all the items. It takes a starting value,
and a block given the answer so far and the next item:

```lume
def main:
  prices = [4, 2, 7]
  puts prices.fold(0) { |total, p| total + p }       #=> 13
  puts prices.fold("") { |s, p| s + "#{p};" }        #=> 4;2;7;
```

Starting from `0`, the block gives `0 + 4`, then `4 + 2`, then `6 + 7`.
`sum` is a `fold` with `+`; `fold` is for the answers no other method gives.

## 7.8 A function where a block goes

Where a block is expected, the name of a function that fits can go
instead — without `()`, because the function is handed over, not called:

```lume
def with_tip(p: Int) -> Int = p + 1

def main:
  puts [4, 2, 7].map(with_tip)    #=> [5, 3, 8]
```

## 7.9 Blocks of your own

A function can take a block too. Its parameter's type says what the block
is given and what it gives back: `(Int) -> Int` is "given an `Int`, gives an
`Int`", and `()` as a result means it gives nothing. The function calls the
block like a function:

```lume
def apply_twice(n: Int, f: (Int) -> Int) -> Int = f(f(n))

def repeat(times: Int, f: (Int) -> ()):
  for i in 1..times:
    f(i)

def main:
  puts apply_twice(3) { |n| n * 10 }    #=> 300
  repeat(3) { |i| puts "course #{i}" }  #=> course 1
                                        #=> course 2
                                        #=> course 3
```

The block comes after the call's parentheses. That is how `map` and the
rest are written too: they are methods with a block parameter, like these.

## Underneath, in Rust

A block is a Rust *closure*, and a chain is a chain of Rust *iterator
adapters* — `map`, `filter` — which the Rust compiler fuses into one loop,
so a chain of five steps costs no more than the loop you would have
written by hand. A function that takes a block is generic over the
closure's type, so the block is compiled straight into it, with no call
through a pointer.

## Try it

1. From `["ann", "bob", "cy"]`, print the names in capitals, and then only
   the names longer than two letters.
2. From `[3, 8, 1, 9, 4]`, print whether any is over 8, how many are even,
   and the first one over 5.
3. Sort `["stew", "pie", "salad", "tea"]` by length, longest first. (Sort by
   length, then `reverse`.)
4. Write `each_pair(xs: [Int], f: (Int, Int) -> ())` that calls `f` with
   each item and the one after it, and use it to print `1-2`, `2-3`, `3-4`
   from `[1, 2, 3, 4]`. (A block of two values is written `{ |a, b| ... }`.)

## Recap

- A block is code handed to a method: `{ |x| ... }`, `do |x|` with an
  indented body, or `_` for the simplest case.
- `map` changes each item, `filter` keeps and `reject` drops; `any?`,
  `all?`, `count`, `find`, `sum`, `sort_by`, `max_by` ask with a block.
- Methods chain, left to right, as one pass over the items; a line starting
  with `.` continues the chain.
- `fold` builds one value from all the items.
- A function's name can stand in for a block, and a function can take a
  block: `f: (Int) -> Int`.

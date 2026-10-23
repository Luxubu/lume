# Lunch 5: Making decisions

**In this lunch you will:**

- compare values, and combine the answers;
- do one thing or another with `if`, and pick a value with it;
- repeat work with `for` and `while`;
- skip ahead and stop early with `next` and `break`.

## 5.1 Questions with yes-or-no answers

Comparing two values gives a `Bool`, `true` or `false`:

```lume
def main:
  guests = 6
  puts guests == 6     #=> true
  puts guests != 6     #=> false
  puts guests > 4      #=> true
  puts guests <= 2     #=> false
  puts "apple" < "banana"    #=> true
```

`==` asks "are these equal?" — two signs, because one `=` already means
"name this value". Text compares letter by letter, the way a dictionary
orders it (with every capital letter before every small one).

**`and`, `or` and `not` combine answers**, and they are written as words:

```lume
def main:
  guests = 6
  booked = true
  puts guests > 4 and booked     #=> true
  puts guests > 10 or booked     #=> true
  puts not booked                #=> false
```

## 5.2 `if`, `elif`, `else`

`if` runs a block only when its condition is `true`. `elif` — short for
"else if" — tries another condition, and `else` catches everything left:

```lume
def table_for(guests: Int) -> Str:
  if guests <= 2:
    return "a small table"
  elif guests <= 6:
    return "a round table"
  else:
    return "the long table"

def main:
  puts table_for(2)     #=> a small table
  puts table_for(5)     #=> a round table
  puts table_for(9)     #=> the long table
```

The condition needs no parentheses, and the line ends in `:` because it
opens a block. Writing `else if`, as some languages do, gets a reminder:

```lume-bad
def main:
  guests = 4
  if guests > 6:
    puts "long"
  else if guests > 2:
    puts "round"
#! `else if` is written `elif` in Lume
```

## 5.3 `if` that gives a value

`if` is not only for doing things — it can *be* a value. Then every branch
gives one, and there must be an `else`, so there is always an answer:

```lume
def main:
  guests = 5
  table = if guests <= 2: "small" elif guests <= 6: "round" else: "long"
  puts table    #=> round
```

That is how lunch 4's `price` picked a price. When the branches are longer,
put each on its own line, starting the `if` on the line of the `=`:

```lume
def main:
  guests = 9
  note = if guests > 8:
    "book ahead"
  else:
    "just come in"
  puts note    #=> book ahead
```

Leave out the `else` and Lume asks for it:

```lume-bad
def main:
  guests = 9
  note = if guests > 8: "book ahead"
  puts note
#! an `if` used as a value needs an `else`
```

## 5.4 `if` and `unless` at the end of a line

For one short statement, the condition can come after it:

```lume
def main:
  guests = 9
  puts "we need more chairs" if guests > 8      #=> we need more chairs
  puts "plenty of room" unless guests > 8
```

`unless` means "if not". It reads well at the end of a line, and that is
the only place it goes.

## 5.5 `for`: once for each

`for` runs its block once for each item of a list, with a name for the item:

```lume
def main:
  for dish in ["soup", "salad", "pie"]:
    puts "one #{dish}"    #=> one soup
                          #=> one salad
                          #=> one pie
```

**A range is a list of numbers in order.** `1..3` is 1, 2 and 3 — two dots
include the last number — and `1...3` is 1 and 2, three dots stopping just
before it:

```lume
def main:
  for table in 1..3:
    puts "table #{table}"    #=> table 1
                             #=> table 2
                             #=> table 3
  for seat in 0...2:
    puts "seat #{seat}"      #=> seat 0
                             #=> seat 1
```

`0...n` is the one for counting `n` things from zero. A range never counts
down; for that, reverse it: `(1..3).reverse`.

**`where` keeps only the items you want**, and reads like a sentence:

```lume
def main:
  for n in 1..10 where n % 4 == 0:
    puts n    #=> 4
              #=> 8
```

## 5.6 `while`: until something changes

`while` repeats its block for as long as a condition holds — for when you
do not know in advance how many times:

```lume
def main:
  var soup = 10
  var bowls = 0
  while soup >= 3:
    soup -= 3
    bowls += 1
  puts "#{bowls} bowls, #{soup} left over"    #=> 3 bowls, 1 left over
```

Something in the block must change what the condition tests — here,
`soup` — or the loop never ends.

## 5.7 Skipping and stopping: `next` and `break`

Inside a loop, `next` skips to the next item, and `break` leaves the loop:

```lume
def main:
  for n in 1..10:
    next if n % 2 == 0
    break if n > 7
    puts n    #=> 1
              #=> 3
              #=> 5
              #=> 7
```

`while true` with a `break` is how to write a loop that decides for itself
when to stop:

```lume
def main:
  var plates = 0
  while true:
    plates += 4
    break if plates >= 10
  puts plates    #=> 12
```

In loops inside loops, `next` and `break` act on the innermost one.

## 5.8 Putting it together

Everything from this lunch, in one program — a lunch-bell that rings a
different bell for every third and every fifth minute:

```lume
def bell(minute: Int) -> Str:
  if minute % 15 == 0:
    "ding-dong"
  elif minute % 3 == 0:
    "ding"
  elif minute % 5 == 0:
    "dong"
  else:
    minute.to_s

def main:
  var rang: [Str] = []
  for minute in 1..15:
    rang.push(bell(minute))
  puts rang.join(" ")    #=> 1 2 ding 4 dong ding 7 8 ding dong 11 ding 13 14 ding-dong
```

Look at the order of the conditions in `bell`: `15` comes first, because a
minute like 15 is a multiple of 3 *and* of 5, and the first branch that
matches wins.

## Underneath, in Rust

Rust's `if` is a value too, which is where Lume's comes from. A range
translates directly, though the dots read the other way round: Lume's
`1..3` is Rust's `1..=3`, and Lume's `1...3` is Rust's `1..3`. `next` is
Rust's `continue`. A `for` with `where` becomes one Rust loop with an `if`
inside it, so the filter costs nothing extra.

## Try it

1. Write `kind(n: Int) -> Str` that gives `"none"` for `0`, `"one"` for `1`
   and `"many"` for anything else, as an `if` that gives a value.
2. Print the even numbers from 2 to 10 with a `for`, a range and `where`.
3. Start with `var coins = 1`, and double it in a `while` loop until it is
   at least 100. Print how many doublings it took, and the final amount.
4. Print the numbers from 1 to 20, stopping at the first one that is a
   multiple of both 3 and 4.

## Recap

- `== != < > <= >=` compare and give a `Bool`; `and`, `or` and `not`
  combine them.
- `if` / `elif` / `else` choose what to do. An `if` can also give a value,
  and then it needs an `else`.
- A statement can end in `if cond` or `unless cond`.
- `for x in list:` runs once for each item. `1..3` includes the 3, `1...3`
  stops before it. `where` keeps the items you want.
- `while cond:` repeats while the condition holds.
- `next` skips to the next round, `break` leaves the loop.

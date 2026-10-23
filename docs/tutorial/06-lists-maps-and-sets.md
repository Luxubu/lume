# Lunch 6: Lists, maps and sets

**In this lunch you will:**

- keep many values in a list, and add to it, read from it and change it;
- look things up by name in a map, and keep a tally;
- keep a set of values with no repeats;
- learn what Lume does when a value may not be there.

## 6.1 Lists

A list holds values of one type, in order. Lunch 2 wrote its type `[Str]`
for "a list of `Str`":

```lume
def main:
  menu = ["soup", "salad", "pie"]
  puts menu.len                   #=> 3
  puts menu.empty?                #=> false
  puts menu.contains?("pie")      #=> true
  puts menu.join(", ")            #=> soup, salad, pie
```

**Positions start at 0.** `menu[0]` is the first item. But a position might
be past the end of the list, so `menu[i]` does not give the item itself —
it gives the item *if there is one*:

```lume
def main:
  menu = ["soup", "salad", "pie"]
  puts menu[0]                  #=> Some("soup")
  puts menu[9]                  #=> None
  puts menu[0].or("nothing")    #=> soup
  puts menu[9].or("nothing")    #=> nothing
```

`Some("soup")` means "here it is"; `None` means "there is nothing there".
`.or(...)`, which you met in lunch 3, takes the item or the fallback. Lume
makes you say what happens when there is nothing, and that is why a Lume
program does not crash on a missing item it forgot about — [lunch 10](10-absence-and-failure.md)
tells the whole story.

When you know the position is good — because you just checked the length,
say — `.at(i)` gives the item, and stops the program if you were wrong:

```lume
def main:
  menu = ["soup", "salad", "pie"]
  puts menu.at(1)    #=> salad
```

## 6.2 Changing a list

A list you will change is a `var`. `push` adds to the end, a position can
be given a new item, and `pop` takes the last one off (giving it to you, if
there was one):

```lume
def main:
  var order = ["soup"]
  order.push("bread")
  order.push("tea")
  order[0] = "stew"
  puts order          #=> ["stew", "bread", "tea"]
  puts order.pop      #=> Some("tea")
  puts order          #=> ["stew", "bread"]
```

A list made without `var` cannot be changed:

```lume-bad
def main:
  order = ["soup"]
  order.push("bread")
#! `order` is immutable, but `push` changes it
```

## 6.3 Answers from a list

Many questions about a list are one method away:

```lume
def main:
  prices = [4, 2, 7, 3]
  puts prices.sum                 #=> 16
  puts prices.max                 #=> Some(7)
  puts prices.min.or(0)           #=> 2
  puts prices.sort                #=> [2, 3, 4, 7]
  puts prices.sort.reverse        #=> [7, 4, 3, 2]
  puts prices.first               #=> Some(4)
  puts prices.index_of(7)         #=> Some(2)
```

`max`, `min`, `first`, `last` and `index_of` may have no answer — an empty
list has no largest item — so they give one *if there is one*, like
`menu[i]`. `sum` of an empty list is `0`, so it gives a plain number.

`sort` and `reverse` give a new list and leave the old one alone, like the
text methods of lunch 3.

## 6.4 Walking a list

Lunch 5's `for` walks a list. When you need each item's position too, ask
the list to `enumerate` itself, and name both:

```lume
def main:
  menu = ["soup", "salad", "pie"]
  for i, dish in menu.enumerate:
    puts "#{i + 1}. #{dish}"    #=> 1. soup
                                #=> 2. salad
                                #=> 3. pie
```

## 6.5 Maps

**A map looks values up by a key.** Its type is written `{Key: Value}` — a
price list is a `{Str: Int}`:

```lume
def main:
  prices = {"soup": 4, "salad": 6, "pie": 3}
  puts prices.len                    #=> 3
  puts prices["soup"]                #=> Some(4)
  puts prices["steak"]               #=> None
  puts prices["steak"].or(0)         #=> 0
  puts prices.contains?("pie")       #=> true
```

A key might not be there, so `prices[key]` gives the value if there is one,
just as a list's position does.

A map you will change is a `var`. Giving a key a value adds it, or replaces
the old value; `remove` takes a key out:

```lume
def main:
  var prices = {"soup": 4}
  prices["pie"] = 3
  prices["soup"] = 5
  prices.remove("pie")
  puts prices    #=> {"soup": 5}
```

**A map keeps the order its keys were added in**, and `for` walks it in
that order, with a name for the key and one for the value:

```lume
def main:
  prices = {"soup": 4, "salad": 6, "pie": 3}
  for dish, price in prices:
    puts "#{dish}: #{price}"    #=> soup: 4
                                #=> salad: 6
                                #=> pie: 3
  puts prices.keys      #=> ["soup", "salad", "pie"]
  puts prices.values    #=> [4, 6, 3]
```

**Counting things is the map's most common job.** Read the count so far,
with `0` when there is none yet, and add one:

```lume
def main:
  orders = ["tea", "soup", "tea", "pie", "tea"]
  var counts: {Str: Int} = {}
  for o in orders:
    counts[o] = counts[o].or(0) + 1
  puts counts    #=> {"tea": 3, "soup": 1, "pie": 1}
```

## 6.6 Sets

**A set holds values with no repeats, and answers "is this in it?"**
quickly. Its type is `{Str}`; it is written like a map without the values:

```lume
def main:
  allergies = {"nuts", "milk", "nuts"}
  puts allergies.len                  #=> 2
  puts allergies.contains?("milk")    #=> true
  var seen: {Str} = {}
  seen.add("tea")
  seen.add("tea")
  puts seen.len                       #=> 1
```

The repeated `"nuts"` was dropped, and adding `"tea"` twice keeps one.
Two sets can be combined:

```lume
def main:
  ann = {"soup", "pie"}
  bob = {"pie", "tea"}
  puts ann.intersect(bob)    #=> {"pie"}
  puts ann.union(bob).len    #=> 3
  puts ann.diff(bob)         #=> {"soup"}
```

`intersect` keeps what both have, `union` what either has, and `diff`
what the first has and the second does not.

## 6.7 Empty ones need their type said

`[]` and `{}` on their own give Lume nothing to work out a type from — and
`{}` could be an empty map or an empty set. Say which, as `counts` and
`seen` did above:

```lume-bad
def main:
  var guests = []
#! cannot tell the type of `guests` from `[]` alone
```

## 6.8 A list is a value

Giving a list a second name gives the second name **its own list**. Changing
one does not change the other:

```lume
def main:
  var today = ["soup", "pie"]
  yesterday = today
  today.push("tea")
  puts today        #=> ["soup", "pie", "tea"]
  puts yesterday    #=> ["soup", "pie"]
```

The same holds for maps, sets and text: a value never changes behind your
back because some other name held it. (Lume does not actually copy when it
can tell nothing would notice — when the old name is not used again — so
this costs nothing in the usual case.)

## Underneath, in Rust

A list is a Rust `Vec`, and `menu[i]` is `Vec::get`, which gives an
`Option` — `Some` or `None` — as Lume's does. A map is a hash map that also
remembers the order its keys came in, and a set is the same without the
values. Giving a list a second name copies it with Rust's `clone` only when
the first name is used again afterwards; otherwise the list is *moved*, as
Rust always does, and nothing is copied.

## Try it

1. Make a list of four prices, and print the total, the most expensive (or
   `0`), and the prices from cheapest to dearest.
2. Make a `var` list of names, add two more, and print each with its
   position, starting from 1.
3. Count the letters in `"banana"` with a map: `.chars` gives the letters.
   Print the map.
4. Two friends list the dishes they like. Print, as a list, the dishes
   only the first one likes.

## Recap

- `[a, b]` is a list, `{k: v}` a map, `{a, b}` a set. Their types are
  `[T]`, `{K: V}` and `{T}`.
- `xs[i]` and `m[key]` give the value if there is one — `Some(...)` or
  `None` — and `.or(fallback)` takes it. `xs.at(i)` is for when you are sure.
- A collection you change is a `var`: `push`, `pop`, `xs[i] = v`,
  `m[key] = v`, `remove`, `add`.
- `for x in xs`, `for i, x in xs.enumerate`, `for k, v in m`. A map keeps
  the order keys were added in.
- An empty `[]` or `{}` needs its type said.
- Giving a collection another name gives that name its own copy.

# Printing and laying things out

> Every one of six review rounds asked this first. It is the most-repeated
> question in the project's history.

## The rule

**A value on its own prints as itself. A value *inside* another prints as you
would write it.** They differ only for text, and that difference is what lets
one item be told from two.

```lume
puts "plain text"     #=> plain text
puts ["a", "b"]       #=> ["a", "b"]
puts ["a, b"]         #=> ["a, b"]
```

`["a", "b"]` is two items and `["a, b"]` is one, and you can see which.

## What each kind of value prints

```lume
puts 42               #=> 42
puts 0 - 7            #=> -7
puts 3.5              #=> 3.5
puts 2.0              #=> 2.0
puts true             #=> true
puts "text"           #=> text
puts [1, 2, 3]        #=> [1, 2, 3]
puts {"x": 1}         #=> {"x": 1}
puts {"one", "two"}   #=> {"one", "two"}
puts Some("pear")     #=> Some("pear")
puts None             #=> None
puts Ok(5)            #=> Ok(5)
```

A `Float` keeps its point: `2.0` prints as `2.0`, not `2`. An empty list is
`[]`; an empty set and an empty map are both `{}`, because both are written
`{}`.

### A space before `(` or `{`

`puts` is a keyword, not a function, and **a space before an opening bracket
is ambiguous** — `puts (a, b)` could be a call with two arguments or one
tuple. Lume refuses to guess. Give the value a name, or close the space up:

```lume
pair = (1, "two", true)
puts pair            #=> (1, "two", true)
puts((1 + 2) * 3)    #=> 9
```

```lume-bad
puts (1, 2)
#! ambiguous
```

Your own types print with their fields named, in order:

```lume
struct Point:
  x: Int
  y: Int

enum Shape:
  Dot
  Circle(radius: Float)

def main:
  puts Point(x: 1, y: 2)      #=> Point(x: 1, y: 2)
  puts Dot                    #=> Dot
  puts Circle(radius: 1.5)    #=> Circle(radius: 1.5)
  puts [Point(x: 1, y: 2)]    #=> [Point(x: 1, y: 2)]
```

Interpolation is the value-as-itself form, so no quotes appear:

```lume
who = "Ada"
puts "hello #{who}"                #=> hello Ada
puts "the list is #{["a", "b"]}"   #=> the list is ["a", "b"]
```

A blank line is `puts ""`:

```lume
puts "above"    #=> above
puts ""         #=>
puts "below"    #=> below
```

## `pad` and `pad_right`

**`pad(n)` puts the spaces on the left. `pad_right(n)` puts them on the
right.** Neither ever cuts a value short: something wider than `n` comes back
whole, so a column can overflow but never lie.

```lume
puts "[" + "ab".pad(6) + "]"              #=> [    ab]
puts "[" + "ab".pad_right(6) + "]"        #=> [ab    ]
puts "[" + "too long for this".pad(4) + "]"   #=> [too long for this]
```

Both work on any scalar, not only text, so a number can be right-aligned in a
column without converting it first:

```lume
puts "[" + 42.pad(6) + "]"          #=> [    42]
puts "[" + 42.pad_right(6) + "]"    #=> [42    ]
```

**`pad` fills with spaces, never zeros.** There is no zero-padding method and
no `format`; for a two-digit money fraction use `decimals`:

```lume
puts 1234.5.decimals(2)    #=> 1234.50
puts 7.decimals(1)         #=> 7.0
puts 0.125.decimals(2)     #=> 0.13
```

`decimals` rounds half away from zero, so `0.125` gives `0.13` and `-0.125`
gives `-0.13`.

## `join`

**`join(sep)` works on any list whose items can be shown, not just text**, and
it uses the value-as-itself form, so no quotes appear:

```lume
puts [1, 2, 3].join("-")       #=> 1-2-3
puts [1.5, 2.5].join("-")      #=> 1.5-2.5
puts ["a", "b"].join("-")      #=> a-b
```

It works directly on a lazy chain — no `.to_list` first:

```lume
puts [3, 1, 2].map { |n| n * 2 }.join("-")    #=> 6-2-4
```

## `warn`

`warn` is `puts` to standard error, for the part of the output that is not
the result. It follows all the same rules.

## Why not prettier?

`puts names` reading as `[Ada, Bo]` is nicer than `["Ada", "Bo"]`, and two of
three reviewers assumed it would. But a program that wants prose already
writes `names.join(", ")` — every reviewer did — while `puts` on a container
is an act of inspection, and inspection wants to be unambiguous.

`examples/printing.lume` is this page as a runnable program.

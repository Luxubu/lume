# Operators and precedence

> No round could find a precedence table. One reviewer noticed that
> `not a and not b` has two readings, could not tell which Lume takes, and
> parenthesised everything in ten programs.

## Precedence, tightest first

| | operators | groups |
|---|---|---|
| 1 | `.` `[]` `()`, and postfix `?` `!` | left |
| 2 | `**` | **right** |
| 3 | `-x` `not x` `await x` | right |
| 4 | `*` `/` `%` | left |
| 5 | `+` `-` | left |
| 6 | `<` `<=` `>` `>=` | left |
| 7 | `==` `!=` | left |
| 8 | `and` | left |
| 9 | `or` | left |
| 10 | `\|>` | left |

So `not` binds tighter than `and`, and `**` groups to the right. `?` and
`!` belong to the call they follow, and `await` to the call it precedes, so
none of them needs brackets inside arithmetic — `parse(s)? + 1`,
`await fetch(2) + 1`.

```lume
a = false
b = false
puts not a and not b    #=> true
puts 2 + 3 * 4          #=> 14
puts 2 ** 3 ** 2        #=> 512
```

`not a and not b` is `(not a) and (not b)`. `2 ** 3 ** 2` is `2 ** (3 ** 2)`,
which is 512 rather than 64.

```lume
def parse(s: Str) -> Int or Error = s.to_int

async def get(n: Int) -> Int = n * 10

async def main -> () or Error:
  puts parse("7")? + 1       #=> 8
  puts await get(2) + 1      #=> 21
```

**`and` and `or` short-circuit**, so a guard before a lookup is safe:

```lume
xs = [1, 2]
i = 9
if i >= 0 and i < xs.len and xs.at(i) == 1:
  puts "in range"
else:
  puts "guarded"    #=> guarded
```

## Unary minus

`-x` works on a binding, not only on a literal:

```lume
x = 5
puts -x        #=> -5
puts 0 - x     #=> -5
```

## Arithmetic

Integer division truncates **toward zero**, so `-7 / 2` is `-3`, not `-4`:

```lume
a = 0 - 7
puts a / 2     #=> -3
puts 7 / 2     #=> 3
puts 7 % 2     #=> 1
```

An `Int` and a `Float` never mix silently — that is a compile error, not a
promotion:

```lume-bad
x = 1 + 0.5
#! cannot combine
```

Convert the side you mean:

```lume
puts 1.to_float + 0.5    #=> 1.5
puts 3.7.round           #=> 4.0
```

`round`, `floor` and `ceil` on a `Float` give a **`Float`**. `.to_int` is how
you cross over.

**Overflow stops the program under `lume run` and `lume test`.** A binary
made with `lume build` wraps around instead, as Rust's release builds do,
unless it is built with `--checked`; see
[Numbers](numbers.md#what-stops-the-program).

An `Int` raised to a power — `2 ** 10`, `2.pow(10)` — needs a power of 0 or
more, since the answer has to be a whole number. A negative power written as a
literal is refused before the program runs; one that is only known at run time
stops the program with `` `pow` on an `Int` needs a power of 0 or more ``. For
a fraction, use a `Float`: `2.to_float.pow(-1.0)` is `0.5`.

```lume
puts 2 ** 10               #=> 1024
puts 2.to_float.pow(-1.0)  #=> 0.5
```

## Comparison

`==` and `!=` work on anything built out of comparable parts, including your
own structs and enums, without writing anything.

`<` orders `Str` by code point, so uppercase sorts before lowercase:

```lume
puts "A" < "a"     #=> true
puts "b" < "a"     #=> false
```

Tuples compare part by part, the first part first and the next only on a tie —
the same order `sort`, `max` and `min` put them in. Every part must have an
order of its own: a number, text, a `Char`, a `Bool`, or another such tuple.

```lume
a = (1, "b")
b = (1, "c")
puts a < b                 #=> true
puts b > (0, "z")          #=> true
puts [b, a].sort           #=> [(1, "b"), (1, "c")]
```

Defining `<` on your own type is what `sort`, `max`, `min`, `<=`, `>` and `>=`
are built from — write the one and the rest follow.

## `|>`

`x |> f(y)` is `f(x, y)`: the value on the left becomes the first argument.
It is the one operator that may continue an expression onto the next line.

```lume
def add(a: Int, b: Int) -> Int = a + b

def main:
  puts 1 |> add(2) |> add(3)    #=> 6
```

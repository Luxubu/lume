# Numbers

> Reviewers guessed at almost everything here: whether `[Float]` sorts, how
> `7.5.round` rounds, whether `to_int` can fail, what `-7 % 3` gives, and how
> `0.1 + 0.2` prints. This page answers each one, and each answer is run.

Lume has two kinds of number. **`Int`** is a whole number, 64 bits and
signed. **`Float`** is a 64-bit floating-point number. They never mix
without you saying so.

The methods of each are listed in [methods](methods.md#int).

## Literals

```lume
puts 42           #=> 42
puts 1_000_000    #=> 1000000
puts 2.5          #=> 2.5
puts 1_000.5      #=> 1000.5
puts 1e3          #=> 1000.0
puts 1.5e-3       #=> 0.0015
```

A literal with a point or an exponent is a `Float`. A `Float` literal needs a
digit on both sides of the point. `.5` and `5.` are not numbers, so write
`0.5` and `5.0`. There are no hex or binary literals.

A negative number is `-` in front: `-7`, `-x`. **A method binds tighter than
`-`**, so `-3.2.floor` means `-(3.2.floor)`, which is `-3.0` and not `-4.0`.
Put a negative value in a name, or in parentheses, before calling a method
on it:

```lume
puts -3.2.floor      #=> -3.0
x = 0.0 - 3.2
puts x.floor         #=> -4.0
```

## Arithmetic

`+`, `-`, `*`, `/` and `%` work on two `Int`s or two `Float`s.

**`Int` division truncates toward zero, and `%` takes the sign of the left
side.** The two agree: `(a / b) * b + a % b` is always `a`.

```lume
a = 0 - 7
puts a / 2           #=> -3
puts a % 3           #=> -1
puts 7 % (0 - 3)     #=> 1
puts 7 / 2           #=> 3
```

So `-1 % 26` is `-1`, not `25`. For a remainder that is never negative, as in
a Caesar shift, add the divisor and take `%` again:

```lume
shift = 0 - 1
puts((shift % 26 + 26) % 26)    #=> 25
```

`Float` has `/` and `%` too, with the same sign rule:

```lume
f = 0.0 - 7.5
puts f / 2.0     #=> -3.75
puts f % 2.0     #=> -1.5
```

## `Int` and `Float` never mix

**Adding an `Int` to a `Float` is a compile error.** It is not a quiet
promotion. The same goes for every other operator, for comparison, for
`x.max(y)`, for a `Float` binding or parameter given an `Int`, and for `+=`:

```lume-bad
count = 4
total = 10.0
puts total / count
#! `/` cannot combine a `Float` and a `Int`
```

```lume-bad
var t = 0.0
t += 1
#! `+` cannot combine a `Float` and a `Int`
```

Convert the side you mean. `.to_float` turns an `Int` into a `Float`, and
`.to_int` turns a `Float` into an `Int`:

```lume
count = 4
total = 10.0
puts total / count.to_float    #=> 2.5
puts total.to_int / count      #=> 2
```

Do not compare an `Int` with a `Float` using `==` either. Convert one side
first.

## Converting

| from | to | write | gives |
|---|---|---|---|
| `Int` | `Float` | `n.to_float` | `Float` |
| `Float` | `Int` | `x.to_int` | `Int`, with the fraction dropped |
| `Str` | `Int` | `s.to_int` | `Int or Error` |
| `Str` | `Float` | `s.to_float` | `Float or Error` |
| a number | `Str` | `n.to_s`, `"#{n}"` | `Str` |
| a number | `Str` | `x.decimals(2)` | `Str`, to that many places |

**`Float.to_int` is a plain `Int`, and it truncates toward zero.** It cannot
fail. A value too large for an `Int` becomes the largest `Int`:

```lume
puts 3.9.to_int       #=> 3
x = 0.0 - 3.9
puts x.to_int         #=> -3
puts 7.to_float       #=> 7.0
```

Round first if you want the nearest `Int`: `x.round.to_int`.

**Reading text can fail, so `Str.to_int` and `Str.to_float` give a result.**
`to_int` takes whole numbers only. `to_float` takes anything that looks like
a number, including a whole number, a leading `.` and an exponent. Both allow
a sign and spaces around the number. Neither takes `_`:

```lume
puts "15".to_float      #=> Ok(15.0)
puts "-2.5".to_float    #=> Ok(-2.5)
puts ".5".to_float      #=> Ok(0.5)
puts "1e3".to_float     #=> Ok(1000.0)
puts " 42 ".to_int      #=> Ok(42)
puts "+4".to_int        #=> Ok(4)
puts "2.5".to_int       #=> Error("`2.5` is not an integer")
puts "1_000".to_int     #=> Error("`1_000` is not an integer")
puts "abc".to_float     #=> Error("`abc` is not a number")
```

[Absence and failure](errors.md) covers taking a result apart.

## `round`, `floor`, `ceil`

**All three give a `Float`.** `round` sends a half **away from zero**:

```lume
puts 7.5.round    #=> 8.0
puts 8.5.round    #=> 9.0
puts 2.5.round    #=> 3.0
puts 3.7.floor    #=> 3.0
puts 3.2.ceil     #=> 4.0
h = 0.0 - 7.5
puts h.round      #=> -8.0
puts h.floor      #=> -8.0
puts h.ceil       #=> -7.0
```

This is not the "round half to even" rule some languages use, so `2.5` gives
`3.0`, not `2.0`. None of the three exist on an `Int`.

## `decimals`: a number for display

**`x.decimals(n)` is the number as text with exactly `n` places.** It also
rounds a half away from zero, and it works on an `Int` too:

```lume
puts 3.14159.decimals(2)    #=> 3.14
puts 2.5.decimals(0)        #=> 3
puts 0.125.decimals(2)      #=> 0.13
puts 7.decimals(2)          #=> 7.00
puts((0.1 + 0.2).decimals(2))    #=> 0.30
```

It gives a `Str`, so do all the arithmetic first and call `decimals` last.

It rounds the number as it is actually stored. `1.005` is stored as a little
less than `1.005`, so it rounds down:

```lume
puts 1.005.decimals(2)    #=> 1.00
```

For amounts that must be exact, such as money, keep whole cents in an `Int`
and divide only when you print.

## `abs`, `max`, `min`, `clamp`, `pow`, `sqrt`

```lume
n = 0 - 7
puts n.abs                  #=> 7
puts 3.max(7)               #=> 7
puts 3.min(7)               #=> 3
puts 15.clamp(0, 10)        #=> 10
puts 2.pow(10)              #=> 1024
puts 1.5.max(0.5)           #=> 1.5
puts 1.5.clamp(0.0, 1.0)    #=> 1.0
puts 2.5.pow(2.0)           #=> 6.25
puts 16.0.sqrt              #=> 4.0
```

Both sides are always the same kind. `Float.pow` takes a `Float`, and
`Int.pow` takes an `Int`. `sqrt` is on `Float` only: write `n.to_float.sqrt`.

A few of these stop the program instead of giving an answer. `clamp` stops
when the low bound is above the high one, and `Int.pow` stops on a negative
exponent. The square root of a negative `Float` is `NaN`.

For the largest or smallest of a list, use the list's `max` and `min`, which
give a `T?` ([methods](methods.md#t-lists)).

## How a `Float` prints

**A `Float` prints as the shortest text that reads back as exactly the same
number.** It is not rounded for you, so a sum that is not exact in binary
shows every digit it needs:

```lume
puts 0.1 + 0.2        #=> 0.30000000000000004
puts 1.0 / 3.0        #=> 0.3333333333333333
puts [0.1, 0.2].sum   #=> 0.30000000000000004
puts 2.0              #=> 2.0
puts 12.5             #=> 12.5
```

This is the true value, and it is why `0.1 + 0.2 == 0.3` is `false`. **Use
`decimals(n)` whenever a float is shown to a person.**

A whole `Float` keeps its point (`2.0`, not `2`). Very large and very
small values switch to an exponent, from `1e16` up and from `1e-5` down:

```lume
puts 1e15       #=> 1000000000000000.0
puts 1e16       #=> 1e16
puts 0.0001     #=> 0.0001
puts 0.00001    #=> 1e-5
```

A `Float` divided by zero does not stop the program. `1.0 / 0.0` is `inf`,
and `0.0 / 0.0` is `NaN`:

```lume
zero = 0.0
puts 1.0 / zero    #=> inf
puts zero / zero   #=> NaN
```

## Ordering and sorting

**`Float` is ordered.** `<` works, and so do `sort`, `max`, `min`, `sort_by`,
`max_by` and `min_by` on a `[Float]` or with a `Float` key:

```lume
fs = [2.5, 0.5, 1.0]
puts fs.sort       #=> [0.5, 1.0, 2.5]
puts fs.max        #=> Some(2.5)
puts fs.min        #=> Some(0.5)
puts 1.5 < 2.0     #=> true
temps = [("mon", 21.5), ("tue", 18.0), ("wed", 21.5)]
puts temps.max_by { |day, t| t }      #=> Some(("mon", 21.5))
puts temps.sort_by { |day, t| t }     #=> [("tue", 18.0), ("mon", 21.5), ("wed", 21.5)]
```

On a tie, `max_by` and `min_by` give the first item, and `sort_by` keeps
items in the order they came.

The exception is `NaN`, which is neither smaller nor larger than anything. A
list holding one does not come out sorted, and `max` and `min` over it are
unreliable. Keep `NaN` out of anything you sort.

A tuple that holds a `Float` sorts, and compares with `<`, part by part like
any other tuple; the `NaN` caution above applies to its `Float` part.

A `Float` cannot go in a set or be a map key, because floats have no exact
equality. Use an `Int` (cents, thousandths) or a `Str`.

## What stops the program

**`Int` overflow stops the program.** It does not wrap around, and there is
no undefined behaviour, in a built binary as much as under `lume run`.
Division or `%` by an `Int` zero stops it too:

```lume-skip
big = 9223372036854775807
puts big + 1
# error: Int overflow in `+`

x = 0
puts 10 / x
# error: division by zero
```

The message goes to standard error and the program exits with a failure
status. `Float` arithmetic never stops the program: it gives `inf` or `NaN`
instead.

A literal too large for an `Int` is refused before the program runs:

```lume-bad
puts 9223372036854775808
#! integer `9223372036854775808` is too large
```

# Lunch 3: Numbers and text

**In this lunch you will:**

- do arithmetic with whole numbers and with fractions;
- show numbers to people, rounded the way they expect;
- turn text into numbers and numbers into text;
- take text apart and put it together.

## 3.1 Arithmetic

`+`, `-`, `*` and `/` work as you would expect, and `%` gives the remainder
of a division. `*`, `/` and `%` happen before `+` and `-`; parentheses
change that.

```lume
def main:
  puts 2 + 3 * 4      #=> 14
  first = (2 + 3) * 4
  puts first          #=> 20
  puts 17 % 5         #=> 2
```

(Why `first` gets a name of its own here, instead of going straight after
`puts`, is in 3.4.)

**Dividing two `Int`s gives an `Int`**, with any fraction dropped:

```lume
def main:
  puts 7 / 2      #=> 3
  puts 7.0 / 2.0  #=> 3.5
```

Seven people sharing two tables gives three to a table, and `7 % 2` people
left over. When you want the fraction, divide `Float`s.

Dividing an `Int` by zero has no answer, so it stops the program. When both
numbers are written out, Lume sees it before the program runs:

```lume-bad
def main:
  puts 7 / 0
#! division by zero
```

When the zero comes from somewhere else — what a person typed, a file — the
program stops when it gets there, with `error: division by zero`. Checking
for zero first is the way to avoid that; [lunch 5](05-making-decisions.md) shows how.

## 3.2 Fractions, and showing them to people

A `Float` prints as the shortest text that is exactly the number it holds.
That is usually what you expect — and sometimes it is not:

```lume
def main:
  puts 12.5           #=> 12.5
  puts 2.0            #=> 2.0
  puts 0.1 + 0.2      #=> 0.30000000000000004
```

`0.1` and `0.2` cannot be held exactly in binary, as `1/3` cannot be written
exactly in decimal, and their sum shows it. Every language with floating-
point numbers has this; Lume does not hide it. **When a number is for a
person to read, say how many places it should have:**

```lume
def main:
  bill = 0.1 + 0.2
  puts bill.decimals(2)    #=> 0.30
  puts 7.decimals(2)       #=> 7.00
  share = 10.0 / 3.0
  puts "each pays #{share.decimals(2)}"    #=> each pays 3.33
```

`decimals` gives text, so do all the arithmetic first and call it last. To
round a number and keep it a number, use `round`, `floor` (down) or `ceil`
(up), which give a `Float`:

```lume
def main:
  puts 7.5.round     #=> 8.0
  puts 7.5.floor     #=> 7.0
  puts 7.1.ceil      #=> 8.0
```

For money, a good habit is to count whole cents in an `Int` and divide only
when you print: `Int` arithmetic is always exact.

## 3.3 Between numbers and text

Lunch 2 met `to_float`. Its partners turn a `Float` into an `Int`, with the
fraction dropped, and any number into text:

```lume
def main:
  puts 3.9.to_int       #=> 3
  puts 7.to_float       #=> 7.0
  puts 42.to_s + "!"    #=> 42!
```

**Text is turned into a number with `to_int` or `to_float`** — but text might
not be a number at all, so the answer says whether it worked:

```lume
def main:
  puts "42".to_int      #=> Ok(42)
  puts "lots".to_int    #=> Error("`lots` is not an integer")
```

`Ok(42)` is "it worked, and here is the number". `Error(...)` is "it did
not, and here is why". [Lunch 10](10-absence-and-failure.md) is all about answers like
these. Until then, `.or(fallback)` takes the number when there is one, and
the fallback when there is not:

```lume
def main:
  seats = "12".to_int.or(0)
  guests = "many".to_int.or(0)
  puts seats + 1     #=> 13
  puts guests        #=> 0
```

## 3.4 A trap with `puts` and parentheses

`puts` takes its value after a space. Starting that value with a
parenthesis is ambiguous — is it `puts` given what is in the parentheses,
or the start of a bigger value? — so Lume asks you to say:

```lume-bad
def main:
  price = 4
  puts (price * 3).to_float
#! a space between `puts` and `(` is ambiguous
```

Either put the whole value in parentheses, or — usually clearer — name it
first:

```lume
def main:
  price = 4
  puts((price * 3).to_float)    #=> 12.0
  total = (price * 3).to_float
  puts total                    #=> 12.0
```

That is why 3.1 named `first` before printing it.

## 3.5 Asking text questions, and changing it

Text has many methods. These are the ones you will reach for most:

```lume
def main:
  dish = "Tomato soup"
  puts dish.len                     #=> 11
  puts dish.upcase                  #=> TOMATO SOUP
  puts dish.downcase                #=> tomato soup
  puts dish.contains?("soup")       #=> true
  puts dish.starts_with?("Tom")     #=> true
  puts dish.ends_with?("salad")     #=> false
  puts dish.replace("soup", "salad")    #=> Tomato salad
  puts "-".repeat(11)               #=> -----------
  puts "  side  ".trim              #=> side
```

None of these change `dish`. Each gives new text and leaves the old as it
was — the same idea as the names in lunch 2. To keep the result, name it.

Methods can follow one another, each working on what the last one gave:

```lume
def main:
  order = "  Iced TEA "
  puts order.trim.downcase.capitalize    #=> Iced tea
```

## 3.6 Taking text apart

**Positions in text count characters, starting at 0.** `s[a..b]` is the
text from position `a` to position `b`, both included:

```lume
def main:
  word = "sandwich"
  puts word[0..3]     #=> sand
  puts word[4..7]     #=> wich
  puts word.len       #=> 8
```

**`split` cuts text into a list of pieces.** Given a separator, it cuts at
each one; given nothing, it cuts at spaces. `join` puts a list back
together:

```lume
def main:
  order = "soup,bread,tea"
  items = order.split(",")
  puts items              #=> ["soup", "bread", "tea"]
  puts items.len          #=> 3
  puts items.join(" + ")  #=> soup + bread + tea
  puts "a  quick lunch".split    #=> ["a", "quick", "lunch"]
```

`chars` gives the characters themselves, one by one:

```lume
def main:
  puts "tea".chars    #=> ["t", "e", "a"]
```

## 3.7 Text over several lines

Inside quotes, `\n` is a new line and `\t` is a tab. For text of several
lines, three quotes are easier to read:

```lume
def main:
  menu = """
    Soup     4
    Bread    2
    """
  puts menu    #=> Soup     4
               #=> Bread    2
```

The indentation that every line shares is removed, so the text can sit
indented with the code around it. Triple-quoted text may hold `"` freely,
and `#{ }` works in it as in any text.

## Underneath, in Rust

An `Int` is Rust's `i64` and a `Float` is an `f64`; Lume's arithmetic is
Rust's, with one addition — under `lume run`, a number that grows past what
an `i64` holds stops the program instead of quietly wrapping around. Text
is a Rust `String`, stored as UTF-8, but every position and length in Lume
counts characters, so `"héllo".len` is `5` where Rust's `len()` would say
`6`. `to_int` gives Rust's `Result`, which `Ok` and `Error` are.

## Try it

1. A bill of `47` is shared by `5` people. Print what each pays as a whole
   number, then as a `Float` to two places: `9` and `9.40`.
2. Read the text `"18"` as a number, add `2`, and print `20`. What happens
   if the text is `"eighteen"` and you use `.or(0)`?
3. Take `"lunch box"` and print `LUNCH BOX`, then `9`, then `Lunch box`.
4. Split `"tea, coffee, juice"` into its three drinks, without the spaces,
   and print them joined with ` / `. (`split(", ")` is one way.)

## Recap

- `+ - * / %` on numbers. `Int / Int` drops the fraction; divide `Float`s
  to keep it.
- A `Float` prints exactly what it holds. Use `decimals(n)` to show it to a
  person; `round`, `floor` and `ceil` keep it a number.
- `to_int`, `to_float` and `to_s` convert. Text into a number gives `Ok` or
  `Error`; `.or(fallback)` takes the number or the fallback.
- Text methods give new text; they never change the old.
- Positions count characters from 0. `s[a..b]` takes a piece, `split` cuts,
  `join` joins.

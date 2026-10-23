# Lunch 10: Absence and failure

**In this lunch you will:**

- understand the `Some`, `None`, `Ok` and `Error` you have been seeing since
  lunch 3;
- write functions that may have no answer, and functions that may fail;
- take their answers apart, and pass a failure on with `?`;
- see why Lume will not let you forget a failure.

## 10.1 Two different things

Since lunch 3, some answers have come wrapped: `menu[9]` gave `None`, and
`"lots".to_int` gave `Error(...)`. They are two different things, and Lume
keeps them apart:

| | the type | means | its values |
|---|---|---|---|
| **absence** | `Int?` | there may be no value here | `Some(4)` or `None` |
| **failure** | `Int or Error` | this may have gone wrong, and there is a reason | `Ok(4)` or `Error("why")` |

Absence is not a problem: a list without a ninth item, a map without a key,
a search that found nothing. Failure is: text that should have been a
number and was not, a file that would not open. An absence has no reason;
a failure always has one.

```lume
def main:
  puts [10, 20][1]       #=> Some(20)
  puts [10, 20][9]       #=> None
  puts "42".to_int       #=> Ok(42)
  puts "lots".to_int     #=> Error("`lots` is not an integer")
```

## 10.2 Functions that may have no answer: `T?`

A function that may have nothing to give says so with `?` after its result
type. **Give back a plain value when there is one, and `None` when there is
not** — Lume wraps the plain value in `Some` for you:

```lume
def table_for(name: Str) -> Int?:
  bookings = {"Ada": 4, "Grace": 7}
  if not bookings.contains?(name):
    return None
  bookings[name].or(0)

def main:
  puts table_for("Ada")     #=> Some(4)
  puts table_for("Alan")    #=> None
```

## 10.3 Getting at the value

You have used the shortest way since lunch 3: **`.or(fallback)`** gives the
value, or the fallback when there is none. When the two cases need
different work, **`match` on `Some` and `None`**:

```lume
def table_for(name: Str) -> Int?:
  bookings = {"Ada": 4, "Grace": 7}
  bookings[name]

def main:
  for name in ["Grace", "Alan"]:
    match table_for(name):
      Some(t) -> puts "#{name}: table #{t}"
      None -> puts "#{name}: no booking"
  #=> Grace: table 7
  #=> Alan: no booking
```

`Some(t)` is a pattern, as in lunch 9: it fits when there is a value, and
names it `t`. Notice that `table_for` this time just gives back
`bookings[name]`, which is an `Int?` already.

What you cannot do is use an `Int?` as if it were an `Int`. It might be
`None`, and Lume will not guess what you want then:

```lume-bad
def main:
  bookings = {"Ada": 4}
  puts bookings["Ada"] + 1
#! it may be absent, so it cannot be used with `+` directly
```

The message's help line lists the ways out: `match`, `?` or `.or(...)`.
That is the point of the `?` in the type: everywhere there might be
nothing, you say what happens when there is.

## 10.4 Functions that may fail: `T or Error`

A function that can go wrong says `-> T or Error`. **Give back a plain value
on success, and `Error("reason")` on failure** — Lume wraps the plain value
in `Ok` for you:

```lume
def seats(text: Str) -> Int or Error:
  n = text.to_int.or(0 - 1)
  if n < 0:
    return Error("`#{text}` is not a number of seats")
  n

def main:
  puts seats("6")      #=> Ok(6)
  puts seats("six")    #=> Error("`six` is not a number of seats")
```

An `Error` holds a message, read with `.message`. `match` takes a result
apart as it does an optional:

```lume
def seats(text: Str) -> Int or Error:
  n = text.to_int.or(0 - 1)
  if n < 0:
    return Error("`#{text}` is not a number of seats")
  n

def main:
  for t in ["6", "six"]:
    match seats(t):
      Ok(n) -> puts "booked #{n}"
      Error(e) -> puts "could not book: #{e.message}"
  #=> booked 6
  #=> could not book: `six` is not a number of seats
```

## 10.5 A failure cannot be ignored

A call that can fail, standing on a line by itself, would throw the
failure away without anyone noticing. Lume refuses it:

```lume-bad
def save(order: Str) -> Int or Error:
  if order.empty?:
    return Error("nothing to save")
  1

def main:
  save("")
  puts "saved"
#! can fail, and nothing here looks at the result
```

When you really do mean to drop it, say so: `_ = save("")`. Then the choice
is written down where the next reader will see it.

## 10.6 Passing a failure on: `?`

Most of the time, a function that meets a failure cannot fix it, and the
right thing is to hand it to whoever called. **`?` after a value that may
fail gives the value — or, if it failed, returns from the function at once
with the failure:**

```lume
def seats(text: Str) -> Int or Error:
  n = text.to_int?
  if n < 0:
    return Error("seats cannot be negative")
  n

def main:
  puts seats("6")     #=> Ok(6)
  puts seats("six")   #=> Error("`six` is not an integer")
  puts seats("-2")    #=> Error("seats cannot be negative")
```

`text.to_int?` is "the number, or leave now with its error". A function
full of steps that may fail reads as the steps, with a `?` on each, and
the failure handling happens in one place: wherever someone finally
`match`es.

`?` needs the function it is in to be able to fail too. `main` can be:
write `def main -> () or Error:`. Then a failure that reaches the end of
`main` ends the program, printing its message:

```lume-skip
def main -> () or Error:
  puts seats("6")?     # prints 6
  puts seats("-2")?    # stops: error: seats cannot be negative
  puts "never printed"
```

```text
6
error: seats cannot be negative
```

The program's exit status is 1, so a script that runs it can tell it
failed.

`?` works on an optional too, in a function that gives an optional: there
it means "the value, or return `None`".

## 10.7 From absence to failure

Sometimes nothing being there *is* a failure — a booking you were told
exists. **`.or_error("reason")` turns a `T?` into a `T or Error`**, giving the
absence its reason, and then `?` passes it on:

```lume
def table_for(name: Str) -> Int or Error:
  bookings = {"Ada": 4, "Grace": 7}
  t = bookings[name].or_error("no booking for #{name}")?
  t

def main:
  puts table_for("Ada")     #=> Ok(4)
  puts table_for("Alan")    #=> Error("no booking for Alan")
```

## 10.8 Choosing

| when | use |
|---|---|
| there is a sensible value to use instead | `.or(fallback)` |
| the two cases need different work | `match` |
| this function cannot fix it, its caller might | `?` |
| a missing value is really a failure | `.or_error("why")?` |
| you truly do not care | `_ = ...` |

There is one more, `!`, which takes the value and stops the whole program
if there is none. Lume warns wherever you use it: it is for tests and
quick scripts, never for code anyone relies on.

## Underneath, in Rust

`Int?` is Rust's `Option<i64>`, and `Int or Error` is `Result<i64, Error>`,
with `Error` a struct holding a message. `?` is Rust's `?`, which Lume's
copies exactly. Lume adds two things Rust does not have: the plain value at
the end of a function is wrapped in `Some` or `Ok` for you, and a result
left unlooked-at is an error, not the warning Rust gives.

## Try it

1. Write `cheapest(prices: [Int]) -> Int?` that gives `None` for an empty
   list, and print its answer for `[4, 2, 7]` and for `[]`.
2. Write `parse_party(text: Str) -> Int or Error` that fails with `"a party
   needs at least one person"` when the number is below 1, and passes on
   `to_int`'s failure with `?`. Print it for `"4"`, `"0"` and `"four"`.
3. Print each of `["3", "x", "5"]` as a number, or `"skip"` when it is not
   one, using `match` on `to_int`.
4. Write a `main -> () or Error` that calls your `parse_party` with `"two"`
   using `?`, and run it. What does it print, and what is its exit status?
   (`echo $?` after running it shows the status.)

## Recap

- `T?` is a value that may be absent: `Some(x)` or `None`. `T or Error` is
  a value that may have failed: `Ok(x)` or `Error("why")`.
- In such a function, a plain value means `Some` or `Ok`; write `None` or
  `return Error("why")` for the other case.
- `.or(fallback)`, `match`, `?` and `.or_error` get at the value.
- A call that can fail cannot be left unlooked-at; `_ = ...` drops it on
  purpose.
- `def main -> () or Error:` lets `?` reach the top, where a failure ends
  the program with its message.

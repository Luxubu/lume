# Lunch 9: Enums and `match`

**In this lunch you will:**

- make a type whose value is *one of* a fixed set of choices;
- let some choices carry values of their own;
- handle each choice with `match`, and know Lume will tell you if you
  missed one;
- use `match` on numbers and text too.

## 9.1 One of several

A course on a menu is a starter, a main or a dessert — never two at once,
never anything else. A struct cannot say that; an **enum** can:

```lume
enum Course:
  Starter
  Main
  Dessert

def main:
  c = Main
  puts c             #=> Main
  puts c == Main     #=> true
  puts [Starter, Dessert]    #=> [Starter, Dessert]
```

`enum Course:` lists its **variants**, one to a line. Each is a value of
type `Course`, written by its name: `Main`. Like a struct, an enum prints
itself and compares with `==` for free.

## 9.2 `match`: one answer for each choice

`match` takes a value and picks the first **arm** that fits it. Each arm is
a pattern, `->`, and what to do:

```lume
enum Course:
  Starter
  Main
  Dessert

def plate(c: Course) -> Str:
  match c:
    Starter -> "small plate"
    Main -> "large plate"
    Dessert -> "bowl"

def main:
  puts plate(Starter)    #=> small plate
  puts plate(Dessert)    #=> bowl
```

`match` is a value, like an `if`: here it is the last line of `plate`, so
its answer is the function's.

**The arms must cover every variant**, and Lume checks. Leave one out — or,
more likely, add a variant to the enum months later and forget one of the
places that match on it — and the program does not compile until you deal
with it:

```lume-bad
enum Course:
  Starter
  Main
  Dessert

def plate(c: Course) -> Str:
  match c:
    Starter -> "small plate"
    Main -> "large plate"

def main:
  puts plate(Main)
#! does not cover: Dessert
```

This is the reason to use an enum and `match` rather than text like
`"main"` and a chain of `if`s: when the choices change, Lume finds every
place that needs to change too.

## 9.3 Choices that carry values

A variant can hold values of its own, written like a struct's fields in
parentheses. Each choice holds only what makes sense for it:

```lume
enum Payment:
  Cash
  Card(last4: Str)
  Voucher(amount: Int)

def main:
  puts Card(last4: "4242")    #=> Card(last4: "4242")
  puts Voucher(10)            #=> Voucher(amount: 10)
```

**A pattern takes the values back out**, naming each one:

```lume
enum Payment:
  Cash
  Card(last4: Str)
  Voucher(amount: Int)

def receipt(p: Payment) -> Str:
  match p:
    Cash -> "paid in cash"
    Card(digits) -> "card ending #{digits}"
    Voucher(amount) -> "voucher for #{amount}"

def main:
  puts receipt(Cash)                  #=> paid in cash
  puts receipt(Card(last4: "4242"))   #=> card ending 4242
  puts receipt(Voucher(amount: 10))   #=> voucher for 10
```

In `Card(digits)`, `digits` is a new name for the value inside — any name
will do, and the values are taken **in the order the variant lists them**:
a pattern names them by position, not by field name, so `Card(last4: d)`
is not a pattern. And the values inside a variant are reached only through
a pattern: `p.last4` does not work on a `Payment`, since a `Cash` has no
`last4` at all.

## 9.4 Methods on an enum

An enum can have methods, written under its variants, and `match self` is
the usual way to write one:

```lume
enum Payment:
  Cash
  Card(last4: Str)
  Voucher(amount: Int)

  def fee(bill: Int) -> Int:
    match self:
      Cash -> 0
      Card(_) -> bill / 50
      Voucher(_) -> 0

def main:
  puts Card(last4: "4242").fee(200)    #=> 4
  puts Cash.fee(200)                   #=> 0
```

`Card(_)` matches any card: `_` means "a value I do not need".

## 9.5 Matching other values

`match` works on numbers and text too. A pattern can be an exact value, a
range, several choices joined by `|`, or `_` for anything else:

```lume
def describe(guests: Int) -> Str:
  match guests:
    0 -> "an empty table"
    1 | 2 -> "a quiet table"
    3..6 -> "a busy table"
    _ -> "a party"

def main:
  for g in [0, 2, 5, 12]:
    puts describe(g)    #=> an empty table
                        #=> a quiet table
                        #=> a busy table
                        #=> a party
```

An `Int` has too many values to list them all, so a `match` on one needs a
last arm that catches the rest — `_`, or a name for the value. Lume asks
for it if it is missing.

**`if` after a pattern adds a condition**, called a *guard*. It can use the
names the pattern took out:

```lume
enum Payment:
  Cash
  Voucher(amount: Int)

def still_due(p: Payment, bill: Int) -> Str:
  match p:
    Voucher(amount) if amount >= bill -> "covered"
    Voucher(amount) -> "#{bill - amount} still to pay"
    Cash -> "paid at the till"

def main:
  puts still_due(Voucher(amount: 20), 15)    #=> covered
  puts still_due(Voucher(amount: 10), 15)    #=> 5 still to pay
```

The arms are tried from the top, and **the first that fits wins** — so the
arm with the guard goes before the plainer one.

## 9.6 Arms with more to do

An arm can be a whole block, indented under it. Its last line is the arm's
value:

```lume
def price(guests: Int) -> Int:
  match guests:
    0 -> 0
    n if n >= 8 ->
      full = n * 10
      full - full / 10
    n -> n * 10

def main:
  puts price(4)    #=> 40
  puts price(10)   #=> 90
```

A party of eight or more gets a tenth off.

## Underneath, in Rust

An enum is a Rust `enum`, and its variants with values are Rust's variants
with fields. `match` is Rust's `match`, and covering every case is Rust's
rule: Lume checks it first, so the message is Lume's. A variant that holds
its own enum — a tree, say — is boxed for you, which Rust makes you do by
hand.

## Try it

1. Make an enum `Size` with `Small`, `Medium` and `Large`, and a function
   giving each a price: 3, 4 and 5.
2. Make an enum `Order` with `Drink(name: Str)` and `Meal(name: Str, sides: Int)`,
   and a method `line -> Str` that gives `"drink: tea"` or
   `"meal: stew + 2 sides"`.
3. Write `rating(stars: Int) -> Str` with `match`: `5` is `"superb"`, `3` or `4`
   is `"good"`, `1..2` is `"poor"`, and anything else `"invalid"`.
4. Add a fourth variant to your `Size` from exercise 1, without changing the
   function, and read what Lume says.

## Recap

- `enum Name:` lists its variants, one per line. A variant may hold values:
  `Card(last4: Str)`.
- `match value:` picks the first arm that fits. A pattern can be a variant,
  with names for what it holds, an exact value, a range, `a | b`, or `_`.
- The arms must cover every case; Lume names what is missing.
- `if` after a pattern adds a guard.
- `match` is a value, and an arm can be a block whose last line is its
  value.

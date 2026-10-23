# Lunch 11: Interfaces

**In this lunch you will:**

- describe what a value can *do*, rather than what it *is*;
- write one function that works for every type that can do it;
- give an interface ready-made methods that each type gets;
- teach a type you did not write — even `Int` — a new trick.

## 11.1 What a value can do

A café sells dishes and drinks. They are different types, with different
fields — but both have a price, and a bill only needs that:

```lume
interface Priced:
  def price -> Int

struct Dish:
  name: Str
  cents: Int

  def price -> Int = cents

struct Drink:
  name: Str
  size: Int

  def price -> Int = 150 + size * 50

def with_tax(item: Priced) -> Int = item.price * 110 / 100

def main:
  puts with_tax(Dish(name: "soup", cents: 400))    #=> 440
  puts with_tax(Drink(name: "tea", size: 2))       #=> 275
```

`interface Priced:` names a method, with no body: anything that is `Priced`
has a `price` giving an `Int`. `with_tax` takes any `Priced` value. It
cannot read `name` or `cents` — it does not know the value is a `Dish` —
only what `Priced` promises.

## 11.2 Nothing to declare

Look again at `Dish` and `Drink`: **neither mentions `Priced`.** A type
fits an interface simply by having its methods. When you add a `Snack`
with a `price` method next month, it is `Priced` at once, and `with_tax`
takes it without a change.

A type that lacks the method is caught where it is used as one:

```lume-bad
interface Priced:
  def price -> Int

struct Napkin:
  colour: Str

def with_tax(item: Priced) -> Int = item.price * 110 / 100

def main:
  puts with_tax(Napkin(colour: "white"))
#! `Napkin` is used as a `Priced` here, but it has no `price` method
```

**A field is not a method**, even with the right name: a struct with a
field `price: Int` does not fit `Priced`, because the interface asks for a
method. Lume's message says so when it sees such a field. A method and a
field of one struct cannot share a name, so keep the number in a field
named for what it is — `cents`, as `Dish` does — and give the method the
interface's name.

## 11.3 Methods that come with the interface

An interface can also hold methods **with** a body. Every type that fits
gets them, ready-made, and they can use the methods the interface asks
for:

```lume
interface Priced:
  def price -> Int
  def label -> Str = "item"
  def tag -> Str = "#{label}: #{price}"

struct Dish:
  name: Str
  cents: Int

  def price -> Int = cents
  def label -> Str = name

struct Napkin:
  colour: Str

  def price -> Int = 0

def main:
  puts Dish(name: "soup", cents: 400).tag    #=> soup: 400
  puts Napkin(colour: "white").tag           #=> item: 0
```

`Dish` wrote its own `label`, so its `tag` uses that; `Napkin` did not, so
it got the interface's `"item"`. Neither wrote `tag`.

## 11.4 A list of different things

A list holds one type — but that type can be an interface. Then each item
may be any type that fits:

```lume
interface Priced:
  def price -> Int

struct Dish:
  name: Str
  cents: Int

  def price -> Int = cents

struct Drink:
  name: Str
  size: Int

  def price -> Int = 150 + size * 50

def main:
  order: [Priced] = [Dish(name: "soup", cents: 400), Drink(name: "tea", size: 1)]
  puts order.sum { |i| i.price }    #=> 600
```

The type has to be said — `[Priced]` — because Lume would otherwise
expect every item to be the same struct.

When you run this, Lume prints a **warning** first, starting with
``warning: `order` holds values of different types behind `Priced` ``. It
is not an error: the program runs. It says that calling `price` on such
an item costs a little more than calling it on a known struct, because the
program must look up, for each item, which `price` to call. That is fine
almost everywhere; the warning is there for the rare loop where every
nanosecond counts.

## 11.5 Teaching a type you did not write

Sometimes the type you want to fit an interface is not yours to change —
`Int`, `Str`, a list, or a type from someone else's code. **`extend` gives
it the methods**, from outside:

```lume
interface Priced:
  def price -> Int

extend Int with Priced:
  def price -> Int = self

def with_tax(item: Priced) -> Int = item.price * 110 / 100

def main:
  puts with_tax(300)    #=> 330
  puts 300.price        #=> 300
```

Now a plain number is `Priced`: its price is itself. Inside `extend`,
`self` is the value being extended.

## 11.6 Interface or enum?

Both let one name stand for several kinds of thing. They answer different
questions:

- **An enum** says: *these* are all the kinds there are. Each place that
  `match`es must handle every one, and adding a kind means visiting each
  of those places — which Lume finds for you.
- **An interface** says: *anything* that can do this. New kinds can be added
  anywhere, by anyone, without touching the code that uses them — but that
  code can only ask for what the interface names.

Payment methods a till must each handle: an enum. Things on a bill that
have a price: an interface.

## Underneath, in Rust

An interface is a Rust **trait**. Because Lume checks fit by the methods a
type has, it writes the `impl Priced for Dish` that Rust needs for you,
for every type that fits. A parameter of an interface type becomes a
generic one in Rust, so `with_tax(Dish(...))` is compiled specially for
`Dish` and costs nothing extra. A list of an interface type is a `Vec` of
`Box<dyn Priced>`: each item behind a pointer, which is the small cost the
warning in 11.4 mentions.

## Try it

1. Make an interface `Greeter` with `def greet -> Str`, and two structs that
   fit it, and a function `twice(g: Greeter) -> Str` that gives the greeting
   twice with a space between.
2. Give `Greeter` a method with a body, `shout -> Str`, that gives the
   greeting in capitals with `"!"` at the end, and print it for both
   structs.
3. Put one of each of your structs in a `[Greeter]` list and print all the
   greetings, joined with `" / "`.
4. `extend Str with Greeter` so that `"Ada".greet` is `"hello, Ada"`.

## Recap

- `interface Name:` lists methods. A type that has them fits — nothing to
  declare.
- A parameter or a list of an interface type takes any type that fits, and
  can use only what the interface names.
- Methods with a body in an interface come free to every type that fits.
- `extend Type with Interface:` fits a type you did not write, even `Int`
  or `Str`.
- An enum for a fixed set of kinds handled in one place; an interface for an
  open set of kinds that share what they can do.

# Lunch 2: Names and values

**In this lunch you will:**

- give values names, and change the ones that change;
- meet the kinds of value Lume has, and say which kind when you want to;
- put values inside text;
- learn the few rules Lume has about names.

## 2.1 Naming a value

`=` gives a value a name:

```lume
def main:
  dish = "soup"
  price = 4
  puts dish     #=> soup
  puts price    #=> 4
```

From then on, `dish` means `"soup"`. **A name made this way keeps its
value.** Try to give it another one and Lume refuses:

```lume-bad
def main:
  price = 4
  price = 5
#! `price` is immutable and cannot be reassigned
```

That is deliberate. In most programs most values never change after they
are worked out, and a name you know cannot change is one less thing to keep
in your head when you read the code later.

## 2.2 `var`, for values that change

When a value is meant to change, say so with `var`:

```lume
def main:
  var plates = 0
  plates = plates + 1
  plates += 2
  puts plates    #=> 3
```

`plates += 2` is short for `plates = plates + 2`. The same goes for `-=`,
`*=` and `/=`.

`var` is written once, where the name is made. After that, the name is
changed with `=` alone.

## 2.3 A new value from the old one

There is one case where a name without `var` can be given a new value:
when the new value is made *from* the old one, on the next lines of the
same block. The right-hand side starts with the name itself:

```lume
def main:
  order = "  Two Teas  "
  order = order.trim
  order = order.downcase
  puts order    #=> two teas
```

Each line makes a new `order` from the one before. Lume calls this a
*transform*. It reads naturally — "trim it, then lower-case it" — and the
value still cannot be changed from somewhere else in your program.

## 2.4 Kinds of value

Every value has a **type**, which says what kind of value it is and what
can be done with it. These are the ones you will use most:

| type | what it holds | written like |
|---|---|---|
| `Int` | whole numbers | `4`, `-12`, `1_000_000` |
| `Float` | numbers with a fraction | `4.5`, `0.25`, `-1.0` |
| `Str` | text | `"soup"`, `""` |
| `Bool` | yes or no | `true`, `false` |
| `[Int]` | a list of `Int`s — or of any type | `[1, 2, 3]`, `["a", "b"]` |

Lume works out the type of a name from its value, so you rarely write it.
When you want to — to make a program easier to read, or because there is
nothing to work it out from — write it after the name, with a `:`:

```lume
def main:
  seats: Int = 12
  tip: Float = 1.5
  orders: [Str] = []
  puts seats         #=> 12
  puts tip           #=> 1.5
  puts orders.len    #=> 0
```

`orders` needs its type said: `[]` is an empty list, but a list of what?
Leave it off and Lume asks:

```lume-bad
def main:
  orders = []
#! cannot tell the type of `orders` from `[]` alone
```

**An `Int` and a `Float` do not mix on their own.** `4 + 1.5` is refused,
because it is not clear which you want. Turn the `Int` into a `Float` when
you mean it:

```lume
def main:
  price = 4
  tip = 1.5
  puts price.to_float + tip    #=> 5.5
```

`price.to_float` asks the `Int` for itself as a `Float`. The dot asks a
value to do something; [lunch 3](03-numbers-and-text.md) is full of them.

## 2.5 Values inside text

Lunch 1 ended with text and a number that `+` would not join. **To put a
value inside text, write it in `#{ }` within the quotes:**

```lume
def main:
  dish = "soup"
  price = 4
  puts "#{dish} costs #{price}"          #=> soup costs 4
  puts "two of them cost #{price * 2}"   #=> two of them cost 8
```

Anything can go between the braces, even a calculation, and any value is
turned into text there — numbers, `true` and `false`, lists. This is how
most text in a Lume program is made.

## 2.6 What a name can be

- **Names are lowercase, with `_` between words**: `price`, `main_course`,
  `plates_left`.
- **A name may end in `?`.** By custom, such a name answers yes or no:

  ```lume
  def main:
    hungry? = true
    puts hungry?    #=> true
  ```

- **A name that starts with a capital letter is a type** — `Int`, `Str`,
  and the types you will make in [lunch 8](08-structs.md) — **or a constant**, a value
  named at the top of a file, outside any `def`:

  ```lume
  SEATS = 12

  def main:
    puts "we have #{SEATS} seats"    #=> we have 12 seats
  ```

- **A few words belong to the language** and cannot be names: `def`, `var`,
  `if`, `for`, `match`, `true`, `puts` and so on. Using one says so:

  ```lume-bad
  def main:
    for = 3
  #! expected a name to loop with
  ```

## Underneath, in Rust

A name made with `=` is a Rust `let`, and a `var` is a `let mut`: Rust
also makes names unchangeable unless you ask, for the same reason. A
transform (2.3) is what Rust calls *shadowing*: a new `let` with the same
name. `Int` is Rust's `i64`, `Float` is `f64`, `Str` is `String`, and a
list is a `Vec`. Text with `#{ }` in it becomes a `format!`.

## Try it

1. Make a name for your favourite meal and one for what it costs, and
   print a sentence that uses both.
2. Count the people at a table: start a `var` at `0` and add to it three
   times, then print it.
3. Take the text `"  LUNCH TIME  "`, trim it and lower-case it with
   transforms, and print the result in square brackets: `[lunch time]`.
4. Work out the price of three dishes at `4` each with a `1.5` tip, and
   print it. (You will need `to_float`.)

## Recap

- `name = value` names a value, and the name keeps it. `var name = value`
  makes a name whose value can change.
- `name = name.something`, on the next lines of the same block, makes a new
  value from the old one.
- Every value has a type: `Int`, `Float`, `Str`, `Bool`, lists like
  `[Int]`. Lume works it out; write it with `: Type` when you want to, or
  when it cannot tell.
- `Int` and `Float` do not mix without `to_float`.
- `"#{value}"` puts any value inside text.

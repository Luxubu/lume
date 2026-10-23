# Lunch 8: Structs

**In this lunch you will:**

- make a type of your own, with named fields;
- give it methods: things its values can answer and do;
- change a value's fields, and write methods that change them;
- give a type functions of its own, such as other ways to make one.

## 8.1 A type of your own

A dish on a menu has a name and a price. You could keep two lists, or a
map, but neither says what a dish *is*. A **struct** does:

```lume
struct Dish:
  name: Str
  price: Int

def main:
  soup = Dish(name: "soup", price: 4)
  puts soup.name     #=> soup
  puts soup.price    #=> 4
  puts soup          #=> Dish(name: "soup", price: 4)
```

`struct Dish:` makes a new type, `Dish`, and lists its **fields**, each
with a type, indented under it. A type's name starts with a capital letter —
that is how Lume tells a type from a value. `Dish(name: ..., price: ...)`
makes one, and `soup.name` reads a field.

A struct can be made with each field named, as above, or with the values
in the order of the fields:

```lume
struct Dish:
  name: Str
  price: Int

def main:
  puts Dish("pie", 3)                 #=> Dish(name: "pie", price: 3)
  puts Dish(price: 3, name: "pie")    #=> Dish(name: "pie", price: 3)
```

Every field must be given. Naming them is the clearer habit, as it was for
function calls in lunch 4.

## 8.2 Methods

A **method** is a function that belongs to a type, written inside the
struct after its fields. **Inside a method, a field is written by its name
alone**:

```lume
struct Dish:
  name: Str
  price: Int

  def label -> Str = "#{name} (#{price})"
  def cheap? -> Bool = price < 5
  def price_for(count: Int) -> Int = price * count

def main:
  pie = Dish(name: "pie", price: 3)
  puts pie.label           #=> pie (3)
  puts pie.cheap?          #=> true
  puts pie.price_for(4)    #=> 12
```

A method is called on a value with a dot, like the methods of text and
lists. One that takes nothing needs no `()` — the dot already says it is a
call. Inside a method, `self` is the whole value, when you need it, and a
method can call its type's other methods by name, as it reads fields.

## 8.3 Comparing and printing come free

Two structs are `==` when all their fields are, a struct prints with its
fields, and a struct can go in a list, a set, or be a map's key:

```lume
struct Dish:
  name: Str
  price: Int

def main:
  a = Dish(name: "tea", price: 2)
  b = Dish(name: "tea", price: 2)
  puts a == b                   #=> true
  menu = [a, Dish(name: "pie", price: 3)]
  puts menu.map(_.name)         #=> ["tea", "pie"]
  puts menu.sum { |d| d.price } #=> 5
```

Blocks, from lunch 7, work on lists of your own types as on any other.

## 8.4 Changing a field

A struct held in a `var` can have its fields changed:

```lume
struct Dish:
  name: Str
  price: Int

def main:
  var special = Dish(name: "soup", price: 4)
  special.price = 5
  puts special    #=> Dish(name: "soup", price: 5)
```

Without `var`, as with everything else, the value stays as it was made:

```lume-bad
struct Dish:
  name: Str
  price: Int

def main:
  special = Dish(name: "soup", price: 4)
  special.price = 5
#! `special` is immutable, so its field `price` cannot be changed
```

## 8.5 Methods that change their value: `var self`

**A method that changes its value's fields says so, with `var self`** as its
first parameter. `var self` is never written at the call:

```lume
struct Tab:
  table: Int
  total: Int

  def add(var self, price: Int):
    total += price

  def show -> Str = "table #{table}: #{total}"

def main:
  var t = Tab(table: 4, total: 0)
  t.add(3)
  t.add(5)
  puts t.show    #=> table 4: 8
```

Both halves are needed. A method that changes a field without `var self`
is refused where it changes it, and a `var self` method needs a `var` to be
called on:

```lume-bad
struct Tab:
  table: Int
  total: Int

  def add(price: Int):
    total += price

def main:
  var t = Tab(table: 4, total: 0)
  t.add(3)
#! `add` changes the field `total`, but its `self` is read-only
```

So reading a struct's methods tells you which can change it — and a value
held without `var` can be handed anywhere without fear that it will come
back changed.

## 8.6 Functions of the type: `def self.`

Sometimes you want another way to make a value, or a value the type
always has. **`def self.name` is a function of the type itself**, called on
the type, not on one value:

```lume
struct Dish:
  name: Str
  price: Int

  def self.free(name: Str) -> Dish = Dish(name: name, price: 0)
  def self.house_soup -> Dish = Dish(name: "soup of the day", price: 4)

def main:
  puts Dish.free("water")      #=> Dish(name: "water", price: 0)
  puts Dish.house_soup.name    #=> soup of the day
```

Such a function has no value to work on, so it cannot read a field — only
make values and call other functions.

## 8.7 Structs made of structs

A field can be any type: a list, a map, or another struct. That is how a
program models its world, one small type at a time:

```lume
struct Dish:
  name: Str
  price: Int

struct Order:
  table: Int
  dishes: [Dish]

  def total -> Int = dishes.sum { |d| d.price }
  def names -> Str = dishes.map(_.name).join(", ")

def main:
  o = Order(table: 2, dishes: [Dish("soup", 4), Dish("pie", 3)])
  puts "table #{o.table}: #{o.names} = #{o.total}"    #=> table 2: soup, pie = 7
```

## 8.8 A struct is a value

As lunch 6 showed for lists, a struct given a second name gives that name
its own copy:

```lume
struct Dish:
  name: Str
  price: Int

def main:
  var today = Dish(name: "stew", price: 5)
  yesterday = today
  today.price = 6
  puts today.price        #=> 6
  puts yesterday.price    #=> 5
```

## Underneath, in Rust

A Lume struct is a Rust `struct`, with `Debug`, `Clone` and `PartialEq` (and
`Hash`, when its fields allow) derived for you — that is where free
printing and `==` come from. Methods go in an `impl` block: a method takes
`&self`, a `var self` method takes `&mut self`, and `def self.free` is an
associated function, `Dish::free`. Field names without `self.` are Lume's
shorthand; the Rust says `self.price`.

## Try it

1. Make a struct `Guest` with a `name` and a `party` size, and a method
   `table -> Str` that gives `"small"` for parties of up to 2 and `"large"`
   otherwise.
2. Make a list of three `Guest`s, and print the total number of people and
   the names of the large parties.
3. Give a struct `Kitchen` a `var self` method `cook(dish: Str)` that adds
   the dish to a list field `done`, and print `done` after cooking two.
4. Give `Guest` a function of the type, `Guest.solo(name: Str)`, that makes a
   guest with a party of 1.

## Recap

- `struct Name:` with fields `name: Type` makes a type of your own.
  `Name(field: value, ...)` makes a value of it, and `value.field` reads one.
- Methods are written inside the struct; fields are read by name in them.
- `==`, printing, and use in sets and as map keys come free.
- A field changes only on a `var`, and a method that changes fields says
  `var self`.
- `def self.name` is a function of the type, called as `Name.name(...)`.
- Fields can be lists, maps and other structs.

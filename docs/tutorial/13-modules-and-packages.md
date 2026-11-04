# Lunch 13: Modules and packages

**In this lunch you will:**

- split a program into files, and choose what each one shares;
- import a whole module, or one thing from it;
- make a package — a library of your own — and use it from a program.

## 13.1 Each file is a module

A program grows. **Each file is a module**, and one file can use another's
work by importing it. The folder `docs/tutorial/cafe/` in Lume's
repository holds a small café program in three files:

```text
cafe/
  main.lume
  menu.lume
  kitchen/
    orders.lume
```

`menu.lume` keeps the menu. **Only what is marked `pub` can be reached from
another file**; the rest is the file's own business. `pub` goes on the
things at the top of a file — a struct, a function, a constant — and a
`pub struct`'s fields and methods come with it, with no `pub` of their own:

```lume-skip
# menu.lume — what the café sells. Only `pub` items can be reached from
# another file.

pub struct Dish:
  name: Str
  cents: Int

PRICES = {"soup": 400, "pie": 350, "stew": 600}

# A dish by name, or a free glass of water when there is no such dish.
pub def find(name: Str) -> Dish:
  match PRICES[name]:
    Some(c) -> Dish(name: name, cents: c)
    None -> water()

# Not `pub`: only this file can call it.
def water -> Dish = Dish(name: "water", cents: 0)
```

`PRICES` and `water` are not `pub`, so the menu can change how it keeps its
prices without any other file noticing. Keeping most things private is
what lets a module change its insides safely.

## 13.2 Importing

`kitchen/orders.lume` imports the one thing it needs from the menu, the
`Dish` type, by its path:

```lume-skip
# kitchen/orders.lume — an order for one table. Its import path starts from
# the folder that holds main.lume, as every path does.

import menu.Dish

pub KITCHEN = "the back kitchen"

pub struct Order:
  table: Int
  dishes: [Dish]

  def add(var self, d: Dish):
    dishes.push(d)

  def total -> Int = dishes.sum { |d| d.cents }

  def summary -> Str = "table #{table}: #{dishes.map(_.name).join(", ")} = #{total}"
```

And `main.lume`, the file you run, imports both:

```lume-skip
# The café from lunch 13, in three files: run it with `lume run main.lume`
# from this folder. Each `import` path starts from this folder.

import menu
import kitchen.orders
import kitchen.orders.Order

def main:
  var o = Order(table: 4, dishes: [])
  o.add(menu.find("soup"))
  o.add(menu.find("pie"))
  o.add(menu.find("steak"))
  puts o.summary
  puts "served by #{orders.KITCHEN}"
```

```text
table 4: soup, pie, water = 750
served by the back kitchen
```

There are two ways to import:

- **`import menu`** brings the module, and its things are used through its
  name: `menu.find("soup")`. A reader always sees where `find` came from.
- **`import kitchen.orders.Order`** brings one thing, used by its own name:
  `Order(...)`.

The first is the one to reach for: a call like `menu.find` says where
`find` lives. Bring a single thing in when it is used so often that its
module's name would only be noise, as a type often is.

## 13.3 Where a path starts

**A path is the file's place, with dots for slashes and no `.lume`**, and
every path starts from the folder that holds the file you run — inside
`kitchen/` too. That is why `orders.lume`, one folder down, still writes
`import menu.Dish`.

So check and run a program of several files **through the file that has
`main`**: `lume check main.lume` checks `menu.lume` and
`kitchen/orders.lume` with it. On its own, `lume check kitchen/orders.lume`
cannot find `menu`, because its paths start from where `main.lume` is.

The [modules reference](../reference/modules.md) has the rest: passing a
module on with `pub import`, constants, and what happens when two names
meet.

## 13.4 A package of your own

A library many programs use, or a program you want to share, is a
**package**: a folder with a `lume.toml` file saying its name, and which
other packages it uses. `lume new` makes one. `--lib` makes a library,
whose root is `lib.lume`; without it, a program, whose root is
`main.lume`:

```sh
lume new greet --lib
lume new hello
```

```text
made library `greet` in greet
made program `hello` in hello
```

`greet/lib.lume` starts with one function, and a test of it:

```lume-skip
# The package `greet`: what is `pub` here, other packages can use.

pub def greeting(name: Str) -> Str = "Hello, #{name}!"

test "greeting":
  assert greeting("greet") == "Hello, greet!"
```

To use it from `hello`, name it under `[dependencies]` in
`hello/lume.toml`, with the path to its folder:

```toml
[package]
name = "hello"
version = "0.1.0"

[dependencies]
greet = { path = "../greet" }
```

and import it **by its name**, as a module:

```lume-skip
import greet

def main:
  puts greet.greeting("Ada")
```

Inside a package, the commands need no file name: from the `hello` folder,
`lume run` runs `main.lume`.

```text
Hello, Ada!
```

What `greet` shares is what its `lib.lume` marks `pub`, as with any module;
its other files stay inside it.

## 13.5 Packages from git

A package can come from a git repository too, at a tag:

```toml
[dependencies]
json = { git = "https://github.com/Luxubu/lume", tag = "v1.0.0" }
```

The first `lume run` fetches it and writes the exact commit it used into
`lume.lock`, beside `lume.toml`, so the next build — yours or anyone's who
has the lock — uses the same code. `lume update` moves it on. Lume's own
`json` and `http` packages are shared this way, and lunch 21 uses the
second. The [packages reference](../reference/packages.md) has the
details.

## Underneath, in Rust

Each Lume file becomes a Rust module, and `pub` is Rust's `pub`: an item
without it is private to its module, as in Rust. A package is a crate, and
`lume.toml` does the work of `Cargo.toml`; `lume.lock` is `Cargo.lock`'s
counterpart, pinning each git package to one commit.

## Try it

1. Give `menu.lume` a `pub def cheapest -> Dish` that gives the cheapest
   dish, and print it from `main.lume`.
2. Run `lume check kitchen/orders.lume` from the `cafe` folder. Why does it
   fail when `lume check main.lume` does not?
3. Add `pub def farewell(name: Str) -> Str` to `greet`, and use both of its
   functions from `hello`.

## Recap

- Each file is a module. `pub` marks what other files may use; the rest
  stays inside.
- `import path` brings a module, used through its name; `import path.Thing`
  brings one thing. Paths start from the folder of the file you run.
- A package is a folder with a `lume.toml`, made with `lume new`: a library
  has `lib.lume`, a program `main.lume`.
- A package names what it uses under `[dependencies]`, by `path` or `git`,
  and imports each by its name.

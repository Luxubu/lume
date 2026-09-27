# Modules

> Reviewers writing their first multi-file program guessed twelve times here:
> where a path starts, whether `()` is needed through a prefix, how to write
> a variant of an imported enum, whether imports must come first. This page
> answers each one. Every answer was checked by running it.

The two programs this page quotes are real and are run by `tests/docs.sh`:
[`modules/basics/`](modules/basics/main.lume) and
[`modules/reexport/`](modules/reexport/main.lume).

## One file is one module

**A file is a module, and `import a.b` means the file `a/b.lume`.** There
is nothing to declare: the file's path is its name.

```lume-skip
import shop.items          # shop/items.lume, used as `items.…`
import shop.items.Kind     # one item from it, used as `Kind`
import shop.stock as st    # shop/stock.lume, used as `st.…`
import shop.items.Item as I   # one item under a name of your choosing
```

The name a module is used by is the last part of its path: `import
shop.items` gives `items`. With `as` it is the name you chose, and only that:
after `import shop.stock as st`, writing `stock.…` is ``unknown name `stock` ``.

A file may import a module and some of its items together, and may import an
item without the module. Both are shown above.

## Where a path starts

> Reviewer C wanted to know: relative to what?

**Every import path starts from the folder of the file you run.** Not from
the folder of the file that holds the `import`. A file in a subfolder
still names its neighbours in full:

```lume-skip
# shop/stock.lume — the neighbour is `shop.items`, not `items`
import shop.items.Item
```

Writing `import items.Item` there looks for `items.lume` next to the file
you ran, and the error says where it looked:

```
error: no module `items.Item`: neither `items/Item.lume` nor `items.lume` exists
  --> shop/stock.lume:4:1
  |
4 | import items.Item
  | ^
  help: module paths are relative to the directory of the file that holds `main`
```

"The file that holds `main`" means the file you run.

In a package — a folder with a `lume.toml` — paths start at the package
root instead, whichever file you run; see [Packages](packages.md).

One consequence: a module is found through the program that uses it. Running
`shop/stock.lume` on its own would look for `shop/shop/items.lume`.

The file you run is a module too; its name is its file name, so `main.lume`
is `main`. If a module you import has a `def main` of its own, it is ignored:
only the file you run starts the program.

## `pub`

**Nothing leaves a file unless it is `pub`.** `pub` goes in front of `def`,
`struct`, `enum`, `interface`, a constant, or `import`:

```lume-skip
# shop/items.lume
pub TAX_PERCENT = 20

pub enum Kind:
  Food
  Tool(grams: Int)

  def label -> Str = match self:
    Food    -> "food"
    Tool(g) -> "tool, #{g}g"

pub struct Item:
  sku: Str
  cents: Int
  kind: Kind

  def with_tax -> Int = cents + tax_on(cents)

# Not `pub`: the methods above may use it, an importer may not.
def tax_on(cents: Int) -> Int = cents * TAX_PERCENT / 100
```

**Everything inside a `pub` type is public with it.** That is fields,
variants, and methods, on structs, enums and interfaces alike. `label` above
is callable from any importer, and so are an interface's defaults. Writing
`pub def` on a method is accepted and changes nothing. Writing `pub` on a
field or a variant is an error that says so:

```lume-bad
pub struct Point:
  pub x: Int
  y: Int

def main:
  puts Point(x: 1, y: 2).x
#! a field is public with its struct, so it takes no `pub`
```

A method of a `pub` type may call a private function of its own module, as
`with_tax` calls `tax_on`. That is how a module keeps helpers to itself.

An `extend` takes no `pub`; see [below](#extend-travels-with-the-import) for
where it reaches.

```lume-bad
interface Sized:
  def size -> Int

pub extend Int with Sized:
  def size -> Int = 1

def main:
  puts 1
#! `pub` goes before `def`, `struct`, `enum`, `interface`, `import` or a constant
```

### Reaching something that is not `pub`

It is one error, whatever the item is — a function with or without
arguments, a struct, a constant — and whether you reach it through a prefix
or import it by name:

```
error: `tax_on` exists in module `shop.items` but is not `pub`
  --> main.lume:4:13
  |
4 |   puts items.tax_on(100)
  |             ^
  help: add `pub` in front of its definition in shop/items.lume
```

The error points at your line; the help names the file to change. A name the
module does not have at all is a different error, with a list of what it does
have: ``module `shop.items` has no `tax` `` and
``help: its public items are: …``.

**A surprise:** a `pub` function may return a type that is not `pub`. An
importer can hold and print that value, but using a field or method on it
fails with `` `Hidden` has no field or method named `x` ``, which does not
mention `pub`. If an importer will use what comes back, make its type `pub`.

## Using another module's types

**Through the prefix, a type is written exactly as at home, with the prefix
in front.** That includes keyword construction:

```lume-skip
# main.lume
import shop.items

a = items.Item(sku: "P1", cents: 250, kind: items.Kind.Food)   # by keyword
b = items.Item("H2", 1200, items.Kind.Tool(grams: 450))       # by position
def total(xs: [items.Item]) -> Int = xs.map(_.cents).sum         # as a type
```

Importing the type by name drops the prefix: after `import shop.items.Kind`,
`Kind.Tool(grams: 450)` works, and so does `Kind` as a type. An interface
works the same way: `[items.Priced]` or, imported by name, `[Priced]`.

### Variants: bare or qualified

**A bare variant name works wherever the type is known, whether or not the
enum was imported by name.** In a pattern, the type comes from the value
being matched. In an expression, it comes from a parameter, a field or a
type annotation:

```lume-skip
# main.lume — only `import shop.items`, not `import shop.items.Kind`
stock.add(items.Item(sku: "P1", cents: 250, kind: Food))   # the field says Kind

match i.kind:
  Food    -> puts "#{i.sku} is food"
  Tool(g) -> puts "#{i.sku} weighs #{g}g"
```

The qualified forms always work too: `items.Kind.Tool(3)` in an expression,
and `items.Kind.Tool(g) -> …` in a pattern. There is no `items.Food`:
a variant belongs to its enum, not to the module.

Bare names are looked up among every `pub enum` in the program, including
ones this file never imported. When two enums have a variant with the same
name, a bare one needs its type to be known, and the error says so:
``cannot tell the type of `x` from `Apple` alone``. Add the type
(`x: a.Fruit = Apple`) or qualify it.

## Calling a function through a prefix

> The tour says a function call always carries `()`. `examples/modules/app.lume`
> writes `st.empty` with none. Both are right.

**Through a module prefix, a function with no arguments may be called with
or without `()`.** `st.empty` and `st.empty()` are the same call. `lume fmt`
writes it without. The same holds for a function of an imported type:
`items.Item.blank` and `items.Item.blank()` are one call.

**Everywhere else the rule in [Functions and blocks](functions-and-blocks.md)
holds:** a function of your own file, or one imported by name
(`import shop.stock.empty`), needs `empty()`. Bare `empty` is then an error.

A function *with* arguments always needs them. `items.make` for a function
that takes a `sku` is ``missing an argument: sku``.

A module function reached through its prefix is a call when it takes no
arguments. One that takes arguments, written without them where a block is
wanted — `words.map(text.shout)` — is handed over as the block
([Functions and blocks](functions-and-blocks.md#a-functions-name-as-a-block)).

## Constants

A constant is a name and a value at the top level of a file, with no
keyword. `pub` in front lets importers read it as `items.TAX_PERCENT`, or
import it by name.

**Any value can be a constant:** numbers, text (including `"""` text), lists,
lists of lists, maps, sets, tuples, structs, variants, an interpolated string,
arithmetic on other constants, even a function call.

```lume
LIMIT = 3
RATE = 2.5
GREETING = "hi #{LIMIT}"
BATCHES = [[1, 2], [3, 4, 5]]
AGES = {"ana": 31}
PAIR = (1, "a")
NOTHING: Int? = None

def main:
  puts LIMIT * 2          #=> 6
  puts RATE               #=> 2.5
  puts GREETING           #=> hi 3
  puts BATCHES.len        #=> 2
  puts AGES["ana"].or(0)  #=> 31
  puts PAIR               #=> (1, "a")
  puts NOTHING            #=> None
```

`None` alone has no type, so it needs one written: `NOTHING: Int? = None`.

A constant is worked out once, the first time it is used. A constant may
use constants written above it, not below; a function may use any constant
in its file.

```lume-bad
B = A + 1
A = 1

def main:
  puts B
#! unknown name `A`
```

Capital names are the convention, and the one reviewers expect; the
compiler does not insist.

**A constant cannot be bound again or changed.** A local with a constant's
name, or a method that changes one, is refused:

```lume-bad
LIMIT = 3

def main:
  LIMIT = 4
#! `LIMIT` is a constant, so it cannot be bound again
```

```lume-bad
BATCHES = [[1]]

def main:
  BATCHES.push([6])
#! `BATCHES` is a constant, but `push` changes it
```

To change a copy, take one:

```lume
BASE = [1, 2]

def main:
  var xs = BASE
  xs.push(3)
  puts xs      #=> [1, 2, 3]
  puts BASE    #=> [1, 2]
```

## Where imports go

**An `import` may go anywhere at the top level of a file.** Before or after
constants and definitions, it makes no difference. The convention is to put
them first, after the file's opening comment. Inside a function it is an
error:

```lume-bad
def main:
  import shop
  puts 1
#! `import` goes at the top level of the file
```

**An import you never use is neither an error nor a warning.** That matters
for a file that imports a module only for what it passes on or the
`extend`s it brings; see the next two sections.

Importing a name twice is an error, whether it is the same module twice or
two items with the same name from different modules:

```
error: `Item` is imported twice
  --> main.lume:3:1
  |
3 | import shop.items.Item
  | ^
  help: use `as` to give one of them another name
```

## `pub import`: passing a module on

**`pub import` makes a module part of your own.** A file that imports
yours gets it too, under the name you gave it:

```lume-skip
# report.lume
pub import stats
pub import text.style as tx
import sizes

pub def line(xs: [Int]) -> Str = tx.shout("#{xs.size} values, mean #{stats.mean(xs)}")
```

```lume-skip
# main.lume
import report

def main:
  xs = [4, 8, 9]
  puts report.line(xs)         # 3 VALUES, MEAN 7!
  puts stats.mean([10, 20])    # 15 — passed on by report
  puts tx.shout("done")        # DONE! — passed on as `tx`, so not `style`
  puts xs.size                 # 3 — the extend in sizes.lume came along
```

The passed-on module is used by its own name, `stats`, and not as
`report.stats`. An aliased `pub import text.style as tx` arrives as `tx`.
One item can be passed on the same way: `pub import stats.mean`.

It passes on through any number of files: if `top` has `pub import report`,
a file that imports `top` gets `report`, `stats` and `tx`.

A file may also import the passed-on module itself; that is not "imported
twice". If it imports something *else* under the same name, its own import
wins. If two modules it imports pass on different modules under one name,
that is an error:

```
error: `core` is passed on by two modules and means something different in each
  help: `two` passes on `b` and `one` passes on `a`; import the one you want directly, or have them agree on a name
```

A crate cannot be passed on: `pub import rust.regex` is
``a crate import cannot be `pub` ``.

## `extend` travels with the import

**An `extend` reaches every file that imports its module, directly or
through other imports.** It has no name to import, so nothing is written
for it. In the example above, `main.lume` imports `report`, `report` imports
`sizes`, and `sizes` has `extend [T] with Sized`, so `xs.size` works in
`main.lume`.

It goes up the chain of imports, not sideways: a file that imports neither
`sizes` nor anything that imports it does not get `.size`.

Two `extend`s of the same interface that some type fits both of — `[T]` and
`[Int]`, or `[T]` and `[[T]]` — anywhere in one program is an error that names
both files, reported at the import that brings the second in. It is Rust's
rule for two impls of one trait; see
[Generics](generics.md#two-extends-of-one-interface-the-rust-rule) and
`examples/travel/`.

## Cycles

**Two modules may not import each other, directly or in a ring.** The error
names the whole ring, and points at the import that closes it:

```
error: circular import: a -> b -> a
  --> b.lume:1:1
  |
1 | import a
  | ^
  help: move the shared definitions into a third module that both import
```

## When names meet

- **Two modules may use the same names inside.** A private `helper`, or a
  `pub struct T`, in each of two modules is fine: `a.T` and `b.T` are
  different types. To import both by name, rename one:
  `import b.T as BT`.
- **A local variable or parameter hides a module of the same name** for the
  rest of its scope. After `items = [1, 2]`, `items.len` is the list's.
- **A struct field does not.** Inside a method of a struct with a field
  called `items`, bare `items` is the field, but `items.len` still means the
  module, and fails with ``module `shop.items` has no `len` ``. Write
  `self.items.len`, or better, give the field or the import another name.
- **An item imported by name cannot share its name with a definition in the
  same file.** `import lib.one` beside a `def one` of your own is an error
  naming both, rather than one quietly winning:
  `` `one` is imported from `lib` and also defined in this file ``. Rename
  one, or import the module and write `lib.one`.

## What errors look like

**An error is reported against the file it is in**, with that file's
line, even when you ran a different one:

```
error: `[shop.items.Item]` values have no method `size`
  --> shop/stock.lume:12:27
   |
12 |   def count -> Int = known.size
   |                           ^
```

A use of something that is not `pub` is reported at the use, in your file,
and its help names the file where the `pub` goes.

In messages, a module is called by its import path, dotted: `shop.items`,
and so are its types (`shop.items.Item`). A module next to the file you run
is just its file name: `lib.lume` is `lib`.

A missing module says both places it looked:

```lume-bad
import shop.nothing

def main:
  puts 1
#! no module `shop.nothing`
#! module paths are relative to the directory of the file that holds `main`
```

## A whole example

[`modules/basics/`](modules/basics/main.lume) is three files:
`shop/items.lume` (quoted above), `shop/stock.lume`, and `main.lume`:

```lume-skip
# shop/stock.lume
import shop.items.Item

pub struct Stock:
  known: [Item]

  def add(var self, item: Item):
    known.push(item)

  def count -> Int = known.len

  def find(sku: Str) -> Item? = known.find(_.sku == sku)

pub def empty -> Stock = Stock([])
```

```lume-skip
# main.lume
import shop.items
import shop.items.Kind
import shop.stock as st

def main:
  var stock = st.empty
  stock.add(items.Item(sku: "P1", cents: 250, kind: Food))
  stock.add(items.Item("H2", 1200, Kind.Tool(grams: 450)))
  stock.add(items.Item(sku: "S3", cents: 800, kind: Tool(90)))
  puts "#{stock.count} items, tax #{items.TAX_PERCENT}%"

  match stock.find("H2"):
    Some(i) -> puts "#{i.sku}: #{i.kind.label}, #{i.with_tax} with tax"
    None    -> puts "no H2"

  for i in stock.known:
    match i.kind:
      Food    -> puts "#{i.sku} is food"
      Tool(g) -> puts "#{i.sku} weighs #{g}g"

  fresh: st.Stock = st.empty()   # the same call; `lume fmt` drops the ()
  puts fresh.count
```

It prints:

```
3 items, tax 20%
H2: tool, 450g, 1440 with tax
P1 is food
H2 weighs 450g
S3 weighs 90g
0
```

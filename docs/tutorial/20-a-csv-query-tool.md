# Lunch 20: A CSV query tool

Part 3 builds programs. Each lunch is one small, whole program, made a step
at a time from what Parts 1 and 2 taught, with notes on why it is shaped
the way it is.

**In this lunch you will build `csvq`**, which answers questions about a
CSV file — a table kept as text, one row to a line, with commas between
the fields:

```text
table,dish,price
4,soup,4
9,pie,3
4,stew,6
2,soup,4
```

By the end it answers these:

```sh
lume run csvq.lume orders.csv count
lume run csvq.lume orders.csv sum price
lume run csvq.lume orders.csv where dish soup
lume run csvq.lume orders.csv tally dish
```

## 20.1 The shape of it

Before writing code, decide what the program *is made of*. Here, two
things:

- **a table** — a header, and rows of fields — that can be read from text
  and asked questions;
- **a command** — one of four questions — read from the program's
  arguments.

Each becomes a type: a struct for the table (lunch 8) and an enum for the
command (lunch 9). The rest of the program is functions between them.

## 20.2 Reading a table

A `Table` holds the header and the rows. Reading one from text can fail —
a row with too few fields, an empty file — so `parse` gives a
`Table or Error` (lunch 10), and each failure says where it is:

```lume-skip
struct Table:
  header: [Str]
  rows: [[Str]]

def parse(text: Str) -> Table or Error:
  lines = text.lines.reject { |l| l.trim.empty? }.to_list
  if lines.empty?:
    return Error("the file is empty")
  header = lines.at(0).split(",")
  var rows: [[Str]] = []
  for i, line in lines.skip(1).enumerate:
    fields = line.split(",")
    if fields.len != header.len:
      return Error("line #{i + 2} has #{fields.len} fields, but the header has #{header.len}")
    rows.push(fields)
  Table(header: header, rows: rows)
```

`lines.at(0)` is safe here: the line before made sure there is one. The
line number in the message is `i + 2` — one because people count from 1,
and one for the header.

(A real CSV may hold a comma inside quotes, `"soup, hot"`; this one
does not, which keeps `split(",")` honest. The `csv` package in
`examples/csvq/` handles quotes.)

## 20.3 Finding a column

Every question names a column, so finding one is a method of its own. A
name that is not in the header is a failure, and the message lists the
names there are — the most useful thing to tell someone who mistyped one:

```lume-skip
  def column(name: Str) -> Int or Error:
    header.index_of(name).or_error("no column `#{name}`; there are: #{header.join(", ")}")
```

`index_of` gives an `Int?`; `or_error` turns its `None` into a failure with
a reason (lunch 10).

## 20.4 The four questions

Each question is a method on `Table`, and each starts by finding its
column with `?`, so a bad column name stops it with that message:

```lume-skip
  def count -> Int = rows.len

  def sum(name: Str) -> Int or Error:
    col = column(name)?
    var total = 0
    for i, row in rows.enumerate:
      n = row.at(col).to_int.map_error { |e| Error("line #{i + 2}: #{e.message}") }?
      total += n
    total

  def matching(name: Str, value: Str) -> Table or Error:
    col = column(name)?
    Table(header: header, rows: rows.filter { |r| r.at(col) == value }.to_list)

  def tally(name: Str) -> {Str: Int} or Error:
    col = column(name)?
    var counts: {Str: Int} = {}
    for row in rows:
      counts[row.at(col)] = counts[row.at(col)].or(0) + 1
    counts

  def show -> Str = ([header] + rows).map { |r| r.join(",") }.join("\n")
```

Three choices worth a look:

- **`sum` says which line holds a field that is not a number.**
  `map_error` (lunch 10's table) keeps the failure but gives it a better
  reason, so `sum dish` says `line 2: ...` instead of only that `soup` is
  not an integer.
- **`matching` gives a `Table`**, not a list of rows, so its answer can be
  shown with the header — and could be asked another question.
- **`tally` is lunch 6's counting pattern**, with the map in the order the
  values first appear.

The method is `matching`, not `where`: `where` is one of Lume's words
(lunch 5's `for x in xs where ...`), so it cannot name a method.

## 20.5 Commands

The command line gives a list of words. An enum names the four commands,
and a function turns words into one — with list patterns (lunch 9), which
take a list apart by its length and its words:

```lume-skip
enum Command:
  Count
  Sum(column: Str)
  Matching(column: Str, value: Str)
  Tally(column: Str)

USAGE = "usage: csvq FILE (count | sum COLUMN | where COLUMN VALUE | tally COLUMN)"

def command(words: [Str]) -> Command or Error:
  match words:
    ["count"] -> Count
    ["sum", c] -> Sum(c)
    ["where", c, v] -> Matching(c, v)
    ["tally", c] -> Tally(c)
    _ -> Error(USAGE)

def answer(t: Table, cmd: Command) -> Str or Error:
  match cmd:
    Count -> t.count.to_s
    Sum(c) -> t.sum(c)?.to_s
    Matching(c, v) -> t.matching(c, v)?.show
    Tally(c) -> t.tally(c)?.to_list.map { |k, n| "#{k}: #{n}" }.join("\n")
```

Every case of `Command` is answered; add a fifth command to the enum, and
`answer` will not compile until it answers that one too.

## 20.6 The whole program, tried out

Here is everything so far, with a `main` that tries it on a table written
into the program — so you can run it, and see each answer, before it reads
any files:

```lume
struct Table:
  header: [Str]
  rows: [[Str]]

  def column(name: Str) -> Int or Error:
    header.index_of(name).or_error("no column `#{name}`; there are: #{header.join(", ")}")

  def count -> Int = rows.len

  def sum(name: Str) -> Int or Error:
    col = column(name)?
    var total = 0
    for i, row in rows.enumerate:
      n = row.at(col).to_int.map_error { |e| Error("line #{i + 2}: #{e.message}") }?
      total += n
    total

  def matching(name: Str, value: Str) -> Table or Error:
    col = column(name)?
    Table(header: header, rows: rows.filter { |r| r.at(col) == value }.to_list)

  def tally(name: Str) -> {Str: Int} or Error:
    col = column(name)?
    var counts: {Str: Int} = {}
    for row in rows:
      counts[row.at(col)] = counts[row.at(col)].or(0) + 1
    counts

  def show -> Str = ([header] + rows).map { |r| r.join(",") }.join("\n")

def parse(text: Str) -> Table or Error:
  lines = text.lines.reject { |l| l.trim.empty? }.to_list
  if lines.empty?:
    return Error("the file is empty")
  header = lines.at(0).split(",")
  var rows: [[Str]] = []
  for i, line in lines.skip(1).enumerate:
    fields = line.split(",")
    if fields.len != header.len:
      return Error("line #{i + 2} has #{fields.len} fields, but the header has #{header.len}")
    rows.push(fields)
  Table(header: header, rows: rows)

enum Command:
  Count
  Sum(column: Str)
  Matching(column: Str, value: Str)
  Tally(column: Str)

USAGE = "usage: csvq FILE (count | sum COLUMN | where COLUMN VALUE | tally COLUMN)"

def command(words: [Str]) -> Command or Error:
  match words:
    ["count"] -> Count
    ["sum", c] -> Sum(c)
    ["where", c, v] -> Matching(c, v)
    ["tally", c] -> Tally(c)
    _ -> Error(USAGE)

def answer(t: Table, cmd: Command) -> Str or Error:
  match cmd:
    Count -> t.count.to_s
    Sum(c) -> t.sum(c)?.to_s
    Matching(c, v) -> t.matching(c, v)?.show
    Tally(c) -> t.tally(c)?.to_list.map { |k, n| "#{k}: #{n}" }.join("\n")

ORDERS = """
  table,dish,price
  4,soup,4
  9,pie,3
  4,stew,6
  2,soup,4
  """

# Each question, as the command line would ask it, and its answer or
# what went wrong.
def try(t: Table, words: [Str]):
  match command(words):
    Error(e) -> puts "#{words.join(" ")} -> #{e.message}"
    Ok(cmd) ->
      match answer(t, cmd):
        Ok(text) -> puts "#{words.join(" ")} ->\n#{text}"
        Error(e) -> puts "#{words.join(" ")} -> #{e.message}"

def main:
  match parse(ORDERS):
    Error(e) -> puts "could not read the table: #{e.message}"
    Ok(t) ->
      try(t, ["count"])
      try(t, ["sum", "price"])
      try(t, ["where", "dish", "soup"])
      try(t, ["tally", "dish"])
      try(t, ["sum", "dish"])
      try(t, ["sum", "cost"])
      try(t, ["average", "price"])
  #=> count ->
  #=> 4
  #=> sum price ->
  #=> 17
  #=> where dish soup ->
  #=> table,dish,price
  #=> 4,soup,4
  #=> 2,soup,4
  #=> tally dish ->
  #=> soup: 2
  #=> pie: 1
  #=> stew: 1
  #=> sum dish -> line 2: `soup` is not an integer
  #=> sum cost -> no column `cost`; there are: table, dish, price
  #=> average price -> usage: csvq FILE (count | sum COLUMN | where COLUMN VALUE | tally COLUMN)
```

The last three are mistakes on purpose, and each says what went wrong in a
way a person can act on — the measure of a tool other people will use.

## 20.7 Reading a file from the command line

The tool itself replaces the trying-out `main` with one that reads its
arguments: the file first, then the command. A missing argument is a usage
error, with exit status 2 (lunch 13); anything that fails after that ends
the program through `main`'s `?`, with exit status 1 (lunch 10):

```lume-skip
def main -> () or Error:
  args = Env.args
  if args.len < 2:
    warn USAGE
    Env.exit(2)
  t = parse(File.read(args.at(0))?)?
  cmd = command(args.skip(1).to_list)?
  puts answer(t, cmd)?
```

Save the table at the start of this lunch as `orders.csv`, and:

```sh
lume run csvq.lume orders.csv where dish soup
```

```text
table,dish,price
4,soup,4
2,soup,4
```

```sh
lume run csvq.lume orders.csv sum cost
```

```text
error: no column `cost`; there are: table, dish, price
```

`lume build csvq.lume -o csvq` makes it a program of its own: `./csvq
orders.csv tally dish`.

## 20.8 Testing it

The questions most likely to go wrong unnoticed are the edges — an empty
file, a short row. A few tests (lunch 12) at the bottom of `csvq.lume` keep
them right as the tool grows:

```lume-skip
test "a short row says which line":
  match parse("a,b\n1,2\n3"):
    Ok(_) -> assert false
    Error(e) -> assert e.message == "line 3 has 1 fields, but the header has 2"

test "an empty file is an error":
  assert parse("\n\n").error?

test "where with no match keeps the header":
  t = parse("a,b\n1,2").or(Table(header: [], rows: []))
  assert t.matching("a", "9").or(t).show == "a,b"
```

`lume test csvq.lume` runs them.

## Underneath, in Rust

Nothing here needs anything beyond the Rust that earlier lunches showed:
the struct and the enum become Rust's, `?` and `map_error` are Rust's `?`
and `map_err`, and the list patterns of `command` become a Rust `match` on
a slice. Built with `lume build`, `csvq` is a native program a few hundred
kilobytes big, which starts in a millisecond and reads a large file about
as fast as the same tool written in Rust by hand.

## Try it

1. Add a command `max COLUMN` that gives the largest number in a column.
   Add the variant first, and let `answer` tell you what else to change.
2. Make `where` compare without minding capitals, so `where dish SOUP`
   finds the soups.
3. Add `columns`, which lists the header's names one per line.
4. Write a test for `sum` on a column with a field that is not a number.

## Recap

- Decide what a program is made of first; here, a `Table` struct and a
  `Command` enum, with functions between them.
- Give each failure a message a person can act on: which line, which
  names there are.
- List patterns turn command-line words into a command.
- Try a program out on data written into it, then give it a `main` that
  reads the world.

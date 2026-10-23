# Answers to *Try it*

One way to do each; yours may differ and still be right. Like every
example in these chapters, each answer here is run by the test suite.

## Lunch 1

**1.**

```lume
def main:
  puts "Ada"    #=> Ada
  puts 7        #=> 7
```

**2.**

```lume
def main:
  puts ["soup", "bread", "an apple"]    #=> ["soup", "bread", "an apple"]
```

**3.** Leaving off the `:` after `def main`:

```lume-bad
def main
  puts "hi"
#! expected `:`
```

**4.** `lume fmt` puts one space between `puts` and its value, and indents
each block by two spaces.

## Lunch 2

**1.**

```lume
def main:
  meal = "noodles"
  cost = 6
  puts "#{meal} cost #{cost}"    #=> noodles cost 6
```

**2.**

```lume
def main:
  var people = 0
  people += 1
  people += 1
  people += 1
  puts people    #=> 3
```

**3.**

```lume
def main:
  text = "  LUNCH TIME  "
  text = text.trim
  text = text.downcase
  puts "[#{text}]"    #=> [lunch time]
```

**4.**

```lume
def main:
  dishes = 3
  each = 4
  tip = 1.5
  total = (dishes * each).to_float + tip
  puts total    #=> 13.5
```

## Lunch 3

**1.**

```lume
def main:
  puts 47 / 5                       #=> 9
  each = 47.0 / 5.0
  puts each.decimals(2)             #=> 9.40
```

**2.** Text that is not a number gives the fallback, `0`, so the sum is
`2`:

```lume
def main:
  puts "18".to_int.or(0) + 2          #=> 20
  puts "eighteen".to_int.or(0) + 2    #=> 2
```

**3.**

```lume
def main:
  box = "lunch box"
  puts box.upcase        #=> LUNCH BOX
  puts box.len           #=> 9
  puts box.capitalize    #=> Lunch box
```

**4.**

```lume
def main:
  drinks = "tea, coffee, juice".split(", ")
  puts drinks.join(" / ")    #=> tea / coffee / juice
```

## Lunch 4

**1.**

```lume
def square(n: Int) -> Int = n * n

def main:
  puts square(9)    #=> 81
```

**2.**

```lume
def bill(dishes: Int, each: Int, tip: Int) -> Int = dishes * each + tip

def main:
  puts bill(tip: 2, each: 4, dishes: 3)    #=> 14
```

**3.**

```lume
def size(guests: Int) -> Str:
  if guests <= 2:
    return "small"
  if guests <= 6:
    return "medium"
  "large"

def main:
  puts size(2)     #=> small
  puts size(5)     #=> medium
  puts size(10)    #=> large
```

**4.**

```lume
def stars(n: Int) -> Str:
  if n == 0:
    return ""
  "*" + stars(n - 1)

def main:
  puts stars(3)    #=> ***
```

## Lunch 5

**1.**

```lume
def kind(n: Int) -> Str = if n == 0: "none" elif n == 1: "one" else: "many"

def main:
  puts kind(0)    #=> none
  puts kind(1)    #=> one
  puts kind(7)    #=> many
```

**2.**

```lume
def main:
  for n in 2..10 where n % 2 == 0:
    puts n    #=> 2
              #=> 4
              #=> 6
              #=> 8
              #=> 10
```

**3.**

```lume
def main:
  var coins = 1
  var doublings = 0
  while coins < 100:
    coins *= 2
    doublings += 1
  puts "#{doublings} doublings, #{coins} coins"    #=> 7 doublings, 128 coins
```

**4.** The first number that is a multiple of both is 12:

```lume
def main:
  for n in 1..20:
    puts n
    break if n % 3 == 0 and n % 4 == 0
  #=> 1
  #=> 2
  #=> 3
  #=> 4
  #=> 5
  #=> 6
  #=> 7
  #=> 8
  #=> 9
  #=> 10
  #=> 11
  #=> 12
```

## Lunch 6

**1.**

```lume
def main:
  prices = [5, 2, 8, 3]
  puts prices.sum           #=> 18
  puts prices.max.or(0)     #=> 8
  puts prices.sort          #=> [2, 3, 5, 8]
```

**2.**

```lume
def main:
  var names = ["Ada", "Grace"]
  names.push("Alan")
  names.push("Edsger")
  for i, name in names.enumerate:
    puts "#{i + 1} #{name}"    #=> 1 Ada
                               #=> 2 Grace
                               #=> 3 Alan
                               #=> 4 Edsger
```

**3.**

```lume
def main:
  var counts: {Str: Int} = {}
  for c in "banana".chars:
    counts[c] = counts[c].or(0) + 1
  puts counts    #=> {"b": 1, "a": 3, "n": 2}
```

**4.** A set's `diff` answers this; `to_list` turns the set into a list:

```lume
def main:
  ann = {"soup", "pie", "tea"}
  bob = {"pie", "salad"}
  puts ann.diff(bob).to_list    #=> ["soup", "tea"]
```

## Lunch 7

**1.**

```lume
def main:
  names = ["ann", "bob", "cy"]
  puts names.map(_.upcase)         #=> ["ANN", "BOB", "CY"]
  puts names.filter(_.len > 2)     #=> ["ann", "bob"]
```

**2.**

```lume
def main:
  xs = [3, 8, 1, 9, 4]
  puts xs.any? { |n| n > 8 }           #=> true
  puts xs.count { |n| n % 2 == 0 }     #=> 2
  puts xs.find { |n| n > 5 }           #=> Some(8)
```

**3.** `pie` and `tea` have the same length; `sort_by` keeps them in the
order they came in, and `reverse` turns that round too:

```lume
def main:
  dishes = ["stew", "pie", "salad", "tea"]
  puts dishes.sort_by { |d| d.len }.reverse    #=> ["salad", "stew", "tea", "pie"]
```

**4.**

```lume
def each_pair(xs: [Int], f: (Int, Int) -> ()):
  last = xs.len - 1
  for i in 0...last:
    f(xs.at(i), xs.at(i + 1))

def main:
  each_pair([1, 2, 3, 4]) { |a, b| puts "#{a}-#{b}" }    #=> 1-2
                                                         #=> 2-3
                                                         #=> 3-4
```

## Lunch 8

**1. and 2.**

```lume
struct Guest:
  name: Str
  party: Int

  def table -> Str = if party <= 2: "small" else: "large"

def main:
  guests = [Guest(name: "Ann", party: 2), Guest(name: "Bo", party: 5), Guest(name: "Cy", party: 4)]
  puts guests.sum { |g| g.party }                             #=> 11
  puts guests.filter { |g| g.table == "large" }.map(_.name)    #=> ["Bo", "Cy"]
```

**3.**

```lume
struct Kitchen:
  done: [Str]

  def cook(var self, dish: Str):
    done.push(dish)

def main:
  var k = Kitchen(done: [])
  k.cook("soup")
  k.cook("pie")
  puts k.done    #=> ["soup", "pie"]
```

**4.**

```lume
struct Guest:
  name: Str
  party: Int

  def self.solo(name: Str) -> Guest = Guest(name: name, party: 1)

def main:
  puts Guest.solo("Di")    #=> Guest(name: "Di", party: 1)
```

## Lunch 9

**1.**

```lume
enum Size:
  Small
  Medium
  Large

def price(s: Size) -> Int:
  match s:
    Small -> 3
    Medium -> 4
    Large -> 5

def main:
  puts price(Medium)    #=> 4
```

**2.**

```lume
enum Order:
  Drink(name: Str)
  Meal(name: Str, sides: Int)

  def line -> Str:
    match self:
      Drink(n) -> "drink: #{n}"
      Meal(n, sides) -> "meal: #{n} + #{sides} sides"

def main:
  puts Drink(name: "tea").line              #=> drink: tea
  puts Meal(name: "stew", sides: 2).line    #=> meal: stew + 2 sides
```

**3.**

```lume
def rating(stars: Int) -> Str:
  match stars:
    5 -> "superb"
    3 | 4 -> "good"
    1..2 -> "poor"
    _ -> "invalid"

def main:
  for s in [5, 4, 1, 9]:
    puts rating(s)    #=> superb
                      #=> good
                      #=> poor
                      #=> invalid
```

**4.** With a fourth variant, `Huge`, the function no longer covers every
case, and Lume says which one it misses:

```lume-bad
enum Size:
  Small
  Medium
  Large
  Huge

def price(s: Size) -> Int:
  match s:
    Small -> 3
    Medium -> 4
    Large -> 5

def main:
  puts price(Medium)
#! does not cover: Huge
```

## Lunch 10

**1.** A list's `min` already gives an `Int?`, `None` for an empty list:

```lume
def cheapest(prices: [Int]) -> Int? = prices.min

def main:
  puts cheapest([4, 2, 7])    #=> Some(2)
  puts cheapest([])           #=> None
```

**2.**

```lume
def parse_party(text: Str) -> Int or Error:
  n = text.to_int?
  if n < 1:
    return Error("a party needs at least one person")
  n

def main:
  puts parse_party("4")       #=> Ok(4)
  puts parse_party("0")       #=> Error("a party needs at least one person")
  puts parse_party("four")    #=> Error("`four` is not an integer")
```

**3.**

```lume
def main:
  for t in ["3", "x", "5"]:
    match t.to_int:
      Ok(n) -> puts n
      Error(_) -> puts "skip"
  #=> 3
  #=> skip
  #=> 5
```

**4.**

```lume-skip
def parse_party(text: Str) -> Int or Error:
  n = text.to_int?
  if n < 1:
    return Error("a party needs at least one person")
  n

def main -> () or Error:
  puts parse_party("two")?
```

It prints the failure and stops, with exit status 1:

```text
error: `two` is not an integer
```

## Lunch 11

**1., 2. and 3.**

```lume
interface Greeter:
  def greet -> Str
  def shout -> Str = greet.upcase + "!"

struct Waiter:
  name: Str

  def greet -> Str = "welcome, I am #{name}"

struct Sign:
  text: Str

  def greet -> Str = text

def twice(g: Greeter) -> Str = "#{g.greet} #{g.greet}"

def main:
  puts twice(Sign(text: "open"))    #=> open open
  puts Waiter(name: "Al").shout    #=> WELCOME, I AM AL!
  puts Sign(text: "open").shout    #=> OPEN!
  all: [Greeter] = [Waiter(name: "Al"), Sign(text: "open")]
  puts all.map { |g| g.greet }.join(" / ")    #=> welcome, I am Al / open
```

**4.**

```lume
interface Greeter:
  def greet -> Str

extend Str with Greeter:
  def greet -> Str = "hello, #{self}"

def main:
  puts "Ada".greet    #=> hello, Ada
```

## Lunch 12

**1.**

```lume
def last_two[T](xs: [T]) -> [T] = xs.skip(xs.len - 2)

def main:
  puts last_two([1, 2, 3, 4])    #=> [3, 4]
  puts last_two(["a"])           #=> ["a"]
```

**2. and 4.**

```lume
def count_of[T](xs: [T], x: T) -> Int = xs.count { |y| y == x }

test "an item that appears twice":
  assert count_of([1, 2, 1], 1) == 2

test "an empty list has none":
  empty: [Str] = []
  assert count_of(empty, "a") == 0

def main:
  puts count_of([1, 2, 1], 1)                #=> 2
  puts count_of(["tea", "pie", "tea"], "tea")    #=> 2
```

**3.**

```lume
struct Pair[A, B]:
  left: A
  right: B

  def swap -> Pair[B, A] = Pair(left: right, right: left)

def main:
  puts Pair(left: 1, right: "one").swap    #=> Pair(left: "one", right: 1)
```

## Lunch 13

**1. and 2.**

```lume
def main -> () or Error:
  dir = "/tmp/lume-lunch-13-answers"
  Dir.make(dir)?
  path = Path.join(dir, "notes.txt")
  File.write(path, "one\ntwo\nthree\n")?
  puts File.read(path)?.lines.len         #=> 3
  File.append(path, "four\n")?
  puts File.read(path)?.lines.last.or("")  #=> four
  Dir.remove(dir)?
```

**3.**

```lume
def main:
  p = "/srv/cafe/menu.v2.json"
  puts Path.dir(p)     #=> /srv/cafe
  puts Path.base(p)    #=> menu.v2.json
  puts Path.ext(p)     #=> json
  puts Path.stem(p)    #=> menu.v2
```

Only the last extension counts, so the name without it is `menu.v2`.

**4.**

```lume-skip
def main:
  if Env.args.empty?:
    warn "give me some words"
    Env.exit(2)
  puts Env.args.max_by { |a| a.len }.or("")
```

`lume run longest.lume soup minestrone pie` prints `minestrone`.

## Lunch 14

**1.**

```lume
def countdown(from: Int) -> Iterator[Int]:
  var n = from
  while n >= 1:
    yield n
    n -= 1

def main:
  puts countdown(5).to_list    #=> [5, 4, 3, 2, 1]
```

**2.**

```lume
def squares -> Iterator[Int]:
  var n = 1
  while true:
    yield n * n
    n += 1

def main:
  puts squares().find { |s| s > 50 }    #=> Some(64)
```

**3.**

```lume
def only_long(xs: Iterator[Str], n: Int) -> Iterator[Str]:
  for x in xs:
    yield x if x.len > n

def main:
  puts only_long("a big bowl of hot soup".split, 3).to_list    #=> ["bowl", "soup"]
```

**4.** Keep the values with `.to_list` once, and use the list twice:

```lume
def courses -> Iterator[Str]:
  yield "starter"
  yield "main"

def main:
  menu = courses().to_list
  puts menu    #=> ["starter", "main"]
  puts menu    #=> ["starter", "main"]
```

## Lunch 15

**1.**

```lume
def main -> () or Error:
  (_, out, _) = Process.run("echo", ["Ada"])?
  puts out.trim    #=> Ada
```

**2.**

```lume
def min_max(xs: [Int]) -> (Int, Int) = (xs.min.or(0), xs.max.or(0))

def main:
  (low, high) = min_max([4, 9, 2])
  puts "#{low} to #{high}"    #=> 2 to 9
```

**3.**

```lume
def main -> () or Error:
  (_, out, _) = Process.run("sh", ["-c", "printf 'cherry\\nbanana\\napple\\n' | sort"])?
  puts out.lines    #=> ["apple", "banana", "cherry"]
```

**4.** The program wrote nothing to standard error, so the message ends
after the code:

```lume
def run_ok(program: Str, args: [Str]) -> Str or Error:
  (code, out, err) = Process.run(program, args)?
  if code != 0:
    return Error("`#{program}` failed with #{code}: #{err.trim}")
  out

def main:
  match run_ok("sh", ["-c", "exit 7"]):
    Ok(_) -> puts "it worked?"
    Error(e) -> puts e.message.trim    #=> `sh` failed with 7:
```

## Lunch 16

**1.**

```lume
async def boil(minutes: Int) -> Str:
  await Time.sleep(minutes * 10)
  "boiled for #{minutes}"

async def main:
  pots = [3, 1, 2].map { |m| spawn: await boil(m) }.to_list
  puts await pots    #=> ["boiled for 3", "boiled for 1", "boiled for 2"]
```

**2.**

```lume
async def main:
  puts await (1..5).map { |n| spawn: n * n }.to_list    #=> [1, 4, 9, 16, 25]
```

**3.**

```lume
async def main:
  shared var four = 0
  var checks: [Task[()]] = []
  for w in ["soup", "pie", "stew", "tea"]:
    t = spawn:
      four += 1 if w.len == 4
    checks.push(t)
  await checks
  puts four    #=> 2
```

**4.**

```lume
async def cook(dish: Str, minutes: Int) -> Str:
  await Time.sleep(minutes * 10)
  dish

async def main:
  var kitchen = [spawn: await cook("roast", 30), spawn: await cook("toast", 1)]
  while true:
    match await kitchen.next_done:
      Some(d) -> puts d
      None -> break
  #=> toast
  #=> roast
```

## Lunch 17

**1. and 3.**

```lume
async def bells(n: Int) -> Iterator[Str]:
  for i in 1..n:
    await Time.sleep(5)
    yield "ring #{i}"

async def main:
  puts bells(3).to_list    #=> ["ring 1", "ring 2", "ring 3"]
  joined = bells(3).fold("") { |acc, b| if acc.empty?: b else: acc + " / " + b }
  puts joined              #=> ring 1 / ring 2 / ring 3
```

**2.**

```lume
async def counting -> Iterator[Int]:
  var n = 1
  while true:
    yield n
    n += 1

async def main:
  puts counting().filter { |n| n % 2 == 0 }.map { |n| n * 2 }.take(3).to_list    #=> [4, 8, 12]
```

**4.** An async sequence has no `max_by` of its own; keep its values with
`to_list` and use the list's:

```lume
async def dishes -> Iterator[Str]:
  for d in ["pie", "minestrone", "tea"]:
    yield d

async def loudest(xs: async Iterator[Str]) -> Str?:
  all = xs.to_list
  all.max_by { |x| x.len }

async def main:
  puts await loudest(dishes().map(_.upcase))    #=> Some("MINESTRONE")
```

## Lunch 18

**1.**

```lume
async def main -> () or Error:
  c = Process.spawn("sh", ["-c", "echo a; echo b; echo c"])?
  puts c.lines.map(_.upcase).to_list    #=> ["A", "B", "C"]
```

**2.**

```lume
async def main -> () or Error:
  t = Process.spawn("tr", ["a-z", "A-Z"])?
  t.write("quiet please\n")?
  t.close_input
  puts t.lines.to_list    #=> ["QUIET PLEASE"]
```

**3.**

```lume
async def main -> () or Error:
  sizer = Process.spawn("sh", ["-c", "while read x; do echo ${#x}; done"])?
  for word in ["soup", "minestrone"]:
    sizer.write(word + "\n")?
    answer = await sizer.read_line
    puts answer.or("?")
  #=> 4
  #=> 10
```

**4.**

```lume
async def main -> () or Error:
  c = Process.spawn("sh", ["-c", "exit 5"])?
  puts await c.wait    #=> 5
```

## Lunch 19

These need the network, so the suite does not run them; each was run by
hand against `example.com`. Each is the `main.lume` of a package that
depends on `http`.

**1.**

```lume-skip
import http

def main -> () or Error:
  r = http.get("https://example.com/")?
  puts r.status              # 200
  puts r.body.lines.len      # 1 — example.com sends its page as one line
```

**2.**

```lume-skip
import http

def main -> () or Error:
  r = http.get("https://example.com/no-such-page")?
  puts(if r.ok?: "found it" else: "not there: #{r.status}")    # not there: 404
```

**3.**

```lume-skip
import http

def main:
  r = http.request("GET", "https://example.com/").timeout(1).send
  puts r.error?    # true
```

**4.**

```lume-skip
import http

async def main:
  reqs = [1, 2, 3].map { |_| spawn: await http.request("GET", "https://example.com/").fetch }.to_list
  statuses = await reqs
  puts statuses.map { |r| r.map { |x| x.status }.or(0) }.to_list    # [200, 200, 200]
```

## Lunch 20

Each answer is a change to `csvq.lume`.

**1.** A new variant, and `answer` then refuses to compile until it has an
arm for it — `does not cover: Max(_)`. The method and the arm:

```lume-skip
  # in Table
  def max(name: Str) -> Int or Error:
    col = column(name)?
    var best = 0
    for i, row in rows.enumerate:
      n = row.at(col).to_int.map_error { |e| Error("line #{i + 2}: #{e.message}") }?
      best = n if i == 0 or n > best
    best

  # in Command, `Max(column: Str)`; in `command`, `["max", c] -> Max(c)`;
  # in `answer`:
    Max(c) -> t.max(c)?.to_s
```

**2.** Compare the lowered forms:

```lume-skip
    Table(header: header, rows: rows.filter { |r| r.at(col).downcase == value.downcase }.to_list)
```

**3.** In `Command`, `Columns`; in `command`, `["columns"] -> Columns`; in
`answer`:

```lume-skip
    Columns -> t.header.join("\n")
```

**4.**

```lume-skip
test "sum on a column of words says which line":
  t = parse("a,b\n1,x").or(Table(header: [], rows: []))
  match t.sum("b"):
    Ok(_) -> assert false
    Error(e) -> assert e.message == "line 2: `x` is not an integer"
```

## Lunch 21

Each answer adds to `loglens.lume`.

**1.** Keep the good entries, then sort the slowest first:

```lume-skip
def slowest(text: Str, count: Int) -> [Entry]:
  var good: [Entry] = []
  for item in entries(text):
    match item:
      Ok(e) -> good.push(e)
      Error(_) -> ()
  good.sort_by { |e| 0 - e.ms }.take(count).to_list
```

**2.** A tally keyed by the minute, and the busiest from its pairs:

```lume-skip
def busiest_minute(text: Str) -> (Str, Int)?:
  var per_minute: {Str: Int} = {}
  for item in entries(text):
    match item:
      Ok(e) ->
        if e.status >= 500:
          minute = e.time[0...5]
          per_minute[minute] = per_minute[minute].or(0) + 1
      Error(_) -> ()
  per_minute.to_list.max_by { |m, n| n }
```

**3.** `analyse` can give a `Report or Error`, and end with:

```lume-skip
  if r.bad_lines.len > 10:
    return Error("#{r.bad_lines.len} lines could not be read")
  r
```

**4.** Give `analyse` a `status: Int?` parameter, and skip the entries that
do not match it, before counting: `next if status.some? and Some(e.status) != status`.

## Lunch 22

**1.** `sh` starts, but cannot find the command, and exits with `127`; the
report shows `exit 127` and the shell's own message, such as
`sh: no-such-tool: command not found`.

**2.** In `run_all`, remember whether a finished job failed, and only start
new jobs while none has:

```lume-skip
    while started < jobs.len and running.len < at_once and not stop:
      ...
    match await running.next_done:
      Some(result) ->
        stop = true if not result.1.ok? and fail_fast
        done.push(result)
      None -> ()
```

with `var stop = false` before the outer loop, and `fail_fast: Bool` as a
parameter.

**3.**

```lume-skip
  for o in outcomes:
    puts line_for(o)
    for l in o.err.lines.skip(1):
      puts "        #{l}"
```

**4.** With `-j 1` two `nap` jobs take about 0.4 seconds between them, one
after the other; with `-j 4` about 0.2, side by side.

## Lunch 23

**1.** In `pretend`, an arm `"https://cafe.test/hours" -> Ok((200, "Closed today"))`,
and a check expecting `"Open"`; it reports `DOWN  hours: no "Open" on the
page (2 tries)`.

**2.** With a field `ok_statuses: [Int]` on `Check`, `judge` begins:

```lume-skip
  if not c.ok_statuses.contains?(status):
    return Some("status #{status}")
```

**3.** `2.pow(n)` is 2 to the `n`:

```lume-skip
    Time.sleep(20 * 2.pow(attempt - 1)) if attempt < ch.tries
```

**4.** A block can hold a `shared var` (lunch 16), and a plain function can
take one as a parameter: `calls: shared var {Str: Int}`. The block counts,
then fetches as before:

```lume-skip
def count_and_fetch(calls: shared var {Str: Int}, url: Str) -> (Int, Str) or Error:
  calls[url] = calls[url].or(0) + 1
  pretend(url)

async def main:
  shared var calls: {Str: Int} = {}
  counting = Checker(tries: 2, fetch: { |url| count_and_fetch(calls, url) })
  ...
  puts calls
```

The pages that were down are fetched twice — once, and once more on the
retry; the page that was up, once.

## Lunch 24

Each is a few lines added to `main`, before the `shutdown`.

**1.**

```lume-skip
  s.send(request(4, "textDocument/definition", json.obj([("textDocument", json.obj([("uri", uri.to_json)])), ("position", at)])))?
  def_at = (await s.answer(4) { |m| () })?
  line = def_at.path("result.range.start.line").map { |x| x.int.or(0) }.or(0)
  puts "defined on line #{line + 1}"    # defined on line 1
```

**2.** A `didChange`, as in 24.5, with text such as
`def main:\n  puts nope\n`; the diagnostics that come back say
``line 2: unknown name `nope` ``, printed by `show_diagnostics` while the next
answer is awaited.

**3.** A third version of the text, with `puts total.` as its last line,
and a completion request at its end — line 4 (from 0), character 13:

```lume-skip
  typing = r"""
    def double(n: Int) -> Int = n * 2

    def main:
      total = double(4)
      puts total.
    """
  typed = json.obj([("textDocument", json.obj([("uri", uri.to_json), ("version", 3.to_json)])), ("contentChanges", Arr([json.obj([("text", (typing + "\n").to_json)])]))])
  s.send(notice("textDocument/didChange", typed))?
  spot = json.obj([("line", 4.to_json), ("character", 13.to_json)])
  s.send(request(6, "textDocument/completion", json.obj([("textDocument", json.obj([("uri", uri.to_json)])), ("position", spot)])))?
  done = (await s.answer(6) { |m| () })?
  items = done.path("result.items").map { |x| x.items }.or([])
  puts items.take(5).map { |i| i.path("label").map { |x| x.str.or("") }.or("") }.to_list
  # ["to_float", "to_int", "abs", "pad", "pad_right"]
```

**4.**

```lume-skip
  s.send(request(5, "textDocument/rename", json.obj([("textDocument", json.obj([("uri", uri.to_json)])), ("position", at), ("newName", "twice".to_json)])))?
  refused = (await s.answer(5) { |m| () })?
  puts refused.path("error.message").map { |x| x.str.or("") }.or("")
  # `textDocument/rename` is not supported
```

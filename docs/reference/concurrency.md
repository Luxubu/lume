# Concurrency: `async`, `spawn:` and `shared var`

> Reviewers guessed more here than anywhere else, and these guesses were
> about what a program *does*, not how it is spelled: what a task may
> contain, when a lock is held, what gets copied, where the warnings go.
> A wrong guess here can compile and then behave differently. Every answer
> below was checked by running it.

Tasks run on a pool of threads, so work started with `spawn:` really does
run at the same time as other work.

## The shape of a concurrent program

**`main` is `async def main`. `spawn:` starts a task. `await` waits for
it.**

```lume
async def fetch(id: Int) -> Str:
  await Time.sleep(10)
  "item #{id}"

async def main:
  puts await fetch(1)                  #=> item 1

  t = spawn: await fetch(2)
  puts "started"                       #=> started
  puts await t                         #=> item 2
```

`spawn:` gives back a `Task[T]` straight away, where `T` is what its body
ends with. Here `t` is a `Task[Str]`. `await t` waits until the task is done
and gives the `Str`.

**`spawn:` needs `async def main`.** Tasks run on the runtime an async
`main` starts, so in a program whose `main` is a plain `def`, a `spawn:` is
refused:

```lume-bad
def main:
  t = spawn: 1 + 1
#! `spawn:` needs the program to start with `async def main`
```

## `async def` and `await`

**An `async def` is a function whose result arrives later, so every call to
it is written `await f()`.** Leaving `await` off is an error, and so is
using `await` outside an `async def` or a `spawn:` block:

```lume-bad
async def f -> Int:
  1

async def main:
  x = f()
  puts x
#! `f` is an `async def`: its result arrives later
#! write `await` in front of the call
```

An `async def` another module may call is written `pub async def`, in that
order, and awaited through the prefix like any call: `await shop.price(sku)`.

```lume-bad
async def f -> Int:
  1

def main:
  puts await f()
#! `await` only works inside an `async def` or a `spawn:` block
```

A plain `def` can be called from anywhere, async or not, and a method can be
`async def` too:

```lume
def square(n: Int) -> Int = n * n

struct Worker:
  n: Int

  async def slow -> Int:
    await Time.sleep(1)
    square(n)

async def main:
  puts await Worker(3).slow      #=> 9
  t = spawn: square(4)
  puts await t                   #=> 16
```

`await` covers everything to its right, so `await (1..3).map { … }.to_list`
waits for the whole list. A `?` or `!` at the very end is the exception: it
applies to what `await` gives back, so `await fetch(url)?` waits and then
passes a failure on. To call a method on what comes back, put the `await` in
brackets: `(await tasks).sort`.

## Many tasks at once, and the order results come back in

**`await` on a list of tasks waits for all of them, and gives the results in
the order the tasks were started** — not the order they finished:

```lume
async def fetch(id: Int, ms: Int) -> Str:
  await Time.sleep(ms)
  "item #{id}"

async def main:
  var tasks: [Task[Str]] = []
  tasks.push(spawn: await fetch(1, 60))    # finishes last
  tasks.push(spawn: await fetch(2, 30))
  tasks.push(spawn: await fetch(3, 1))     # finishes first
  puts await tasks                         #=> ["item 1", "item 2", "item 3"]
```

So results can be paired back with what started them, by position:

```lume
def work(n: Int) -> Int = n * n

async def main:
  inputs = [3, 1, 2]
  results = await inputs.map { |n| spawn: work(n) }.to_list
  for n, r in inputs.zip(results):
    puts "#{n} -> #{r}"
  #=> 3 -> 9
  #=> 1 -> 1
  #=> 2 -> 4
```

`spawn:` and `await` work in any loop, including a `while` that starts a
fresh list of tasks each time round:

```lume
async def main:
  var round = 0
  while round < 3:
    var tasks: [Task[Int]] = []
    for k in 1..2:
      tasks.push(spawn: round * 10 + k)
    puts await tasks
    round += 1
  #=> [1, 2]
  #=> [11, 12]
  #=> [21, 22]
```

A task that is never awaited still runs, but nothing waits for it: when
`main` ends, the program ends, and an unfinished task is cut off without a
word. Await every task whose work matters.

## What a `spawn:` body may contain

**A `spawn:` body is a block of statements, like a function body.** A task
may declare its own `var`s, loop, `match`, and call anything. Its value is
its last line:

```lume
BATCHES = [[1, 2], [3, 4, 5], [6]]

async def main:
  var tasks: [Task[Str]] = []
  for i, batch in BATCHES.enumerate:
    t = spawn:
      var total = 0
      for n in batch:
        total += n
      size = match batch.len:
        1 -> "one"
        _ -> "many"
      "batch #{i}: #{size}, #{total}"
    tasks.push(t)
  for line in await tasks:
    puts line
  #=> batch 0: many, 3
  #=> batch 1: many, 12
  #=> batch 2: one, 6
```

`BATCHES` is a top-level constant: **constants can be used inside `spawn:`**
like anywhere else.

**A one-line `spawn: …` holds an expression, not a statement.**
`tasks.push(spawn: work(n))` is fine, but `spawn: hits += 1` is refused
(``expected the end of the line, found `+=` ``). An assignment, or anything
longer than one expression, goes on its own indented lines under `spawn:`.
An indented body cannot be written inside a call, so bind it to a name and
push the name, as above.

**`return` inside `spawn:` ends the task**, and its value is the task's
result:

```lume
async def main:
  t = spawn:
    if true:
      return 5
    6
  puts await t    #=> 5
```

**`?` inside `spawn:` leaves the task, not the function around it.** A task
whose body uses `?` gives back `T or Error` — the value its last line gives,
or the failure `?` met — as an `async` block does in Rust. The failure waits
in the task until it is awaited, where `?` or `match` handles it:

```lume
def half(n: Int) -> Int or Error:
  if n % 2 == 1:
    return Error("#{n} is odd")
  n / 2

async def main:
  t = spawn:
    a = half(20)?
    b = half(a)?
    a + b
  u = spawn:
    half(3)? * 2
  puts await t    #=> Ok(15)
  match await u:
    Ok(v)    -> puts v
    Error(e) -> puts "failed: #{e.message}"    #=> failed: 3 is odd
```

`?` on an optional value makes the task give back a `T?` the same way. A
task that uses `?` on both an optional value and a `T or Error` is refused,
as a function would be. Inside a block within the task — `.map { ... }` — `?`
is refused as it is in any block; see
[Functions and blocks](functions-and-blocks.md#-inside-a-block).

## What a task sees: copies

**A local that a `spawn:` body mentions is copied in when the task starts.**
The task has its own copy; changing the original afterwards does not reach
it:

```lume
async def main:
  var x = 1
  t = spawn: x * 10
  x = 5
  puts await t    #=> 10
  puts x          #=> 5
```

For the same reason, a task may not change an outside `var`: the change
would be made to its copy and lost. That is an error that tells you what to
write instead:

```lume-bad
async def main:
  var count = 0
  t = spawn:
    count = count + 1
  await t
  puts count
#! `count` inside `spawn:` is a copy
#! declare it `shared var count`
```

Anything can be copied in or sent back: numbers, text, lists, maps, your own
structs holding lists. There is no special rule for what may cross.

Inside a method, a `spawn:` cannot use `self` or its fields. Bind what the
task needs to a name first:
``a `spawn:` block inside a method cannot use `self` or its fields``.

## `await` takes the task

**A task is the work itself, so `await` uses it up.** A task cannot be
awaited twice, and a list of tasks cannot be used after it is awaited.
Keep what came back instead:

```lume-bad
async def main:
  t = spawn: 1
  puts await t
  puts await t
#! `await` takes this task, and `t` is used again below
```

```lume-bad
async def main:
  var ts: [Task[Int]] = []
  for i in 1..3:
    ts.push(spawn: i)
  puts await ts
  puts ts.len
#! `await` takes these tasks, and `ts` is used again below
```

If you need `ts.len` afterwards, read it before the `await`.

## A failure inside a task

**A task whose body gives `T or Error` is a `Task[T or Error]`, and the
failure comes home intact.** `?` and `return Error(...)` work in an
`async def` exactly as in a plain one:

```lume
async def parse_slowly(s: Str) -> Int or Error:
  await Time.sleep(5)
  if s.empty?:
    return Error("empty input")
  n = s.to_int?
  n * 2

async def main:
  tasks = ["21", "x", ""].map { |s| spawn: await parse_slowly(s) }.to_list
  for r in await tasks:
    match r:
      Ok(n)    -> puts "ok #{n}"
      Error(e) -> puts "failed: #{e.message}"
  #=> ok 42
  #=> failed: `x` is not an integer
  #=> failed: empty input
```

`main` can be `async def main -> () or Error`, and then `await t?` passes
a task's failure straight up. The program then stops with the message on
standard error and exit status 1:

```lume
def half(n: Int) -> Int or Error:
  if n % 2 == 1:
    return Error("#{n} is odd")
  n / 2

async def main -> () or Error:
  t = spawn: half(10)
  n = await t?
  puts n    #=> 5
```

The `?` belongs to the `await`, not to `t`: a task has not failed until it has
been waited for. `(await t)?` means the same and is still accepted.

A crash inside a task — `.at(9)` on a short list, say — stops the whole
program when the task is awaited, as a crash anywhere else would.

## `shared var`: one value that many tasks change

**`shared var` puts a value behind a lock, and every task that mentions it
works on the same one.**

```lume
async def main:
  shared var hits = 0
  var workers: [Task[()]] = []
  for i in 1..100:
    w = spawn:
      hits += 1
    workers.push(w)
  await workers
  puts hits    #=> 100
```

A `shared var` list or map is changed the same way, with methods and
assignments to a key: `seen.push(x)`, `counts[key] += 1`.

**`shared` without `var` is read-only.** It needs no lock, so it is cheap to
hand to every task. Changing it is an error:

```lume
async def main:
  shared greeting = "hello"
  hellos = await (1..3).map { |i| spawn: "#{greeting} #{i}" }.to_list
  puts hellos    #=> ["hello 1", "hello 2", "hello 3"]
```

A plain local is copied into each task anyway, so `shared` matters for a
large value you do not want copied once per task.

### What the lock covers

**Every mention of a `shared var` takes the lock and releases it again.**
Nothing holds it longer than that one use: not a whole line, not a task, and
never across an `await`.

- `hits += 1` is one use, so it is safe: no task can slip in between the
  read and the write. A hundred tasks doing it give 100.
- `hits += hits` mentions `hits` twice, so it is two uses, and another task
  can change `hits` between them.
- `total = hits` then `hits = total + 1` is two separate uses too, and can
  lose updates. Do the change in one use: `hits += 1`, or a `var self` method
  of a struct (see below), which runs entirely under one lock.

The right-hand side is worked out *before* the lock is taken. So a line
that waits, or that reads the same value, cannot wait forever:

```lume
async def peek(n: shared var Int) -> Int:
  await Time.sleep(10)
  n + 10

async def main:
  shared var n = 1
  n += await peek(n)    # peek reads n while this line waits for it
  puts n                #=> 12
```

The one case that could wait forever is a block that mentions the value
while a method of that same value is running, and that is refused:

```lume-bad
async def main:
  shared var xs = [1, 2, 3]
  ys = xs.map { |x| x + xs.len }
  puts ys
#! `xs` is used inside a block while `xs` is locked
#! bind what the block needs from `xs` to a name before this line
```

**Inside `spawn:`, `=` on a `shared var` replaces the value under the lock**,
the same as `+=` or a method changes it:

```lume
async def main:
  shared var best = 0
  t = spawn:
    best = 42
  await t
  puts best    #=> 42
```

### A `shared var` struct and its methods

**A `var self` method on a `shared var` struct runs under one lock, from
start to end.** That makes a method the way to do a change that must not be
interrupted. The method's result, including a `() or Error`, comes back as
usual and can be matched in the task:

```lume
struct Ledger:
  balances: {Str: Int}

  def apply(var self, who: Str, cents: Int) -> () or Error:
    now = balances[who].or(0)
    if now + cents < 0:
      return Error("#{who} would go below zero")
    balances[who] = now + cents

  def total -> Int = balances.values.sum

async def main:
  shared var book = Ledger({"ana": 500, "bo": 100})
  moves = [("ana", 0 - 200), ("bo", 0 - 300), ("cy", 50)]
  var tasks: [Task[Str]] = []
  for who, cents in moves:
    t = spawn:
      match book.apply(who, cents):
        Ok(_)    -> "#{who} ok"
        Error(e) -> e.message
    tasks.push(t)
  for line in await tasks:
    puts line
  #=> ana ok
  #=> bo would go below zero
  #=> cy ok
  puts book.total    #=> 450
  for who, cents in book.balances:
    puts "#{who} #{cents}"
  #=> ana 300
  #=> bo 100
  #=> cy 50
```

It works the same when the struct comes from another module.

### Walking a `shared var` with `for`

**A `for` over a `shared var` list or map — or a field of one, like
`book.balances` above — copies it out under one lock, and the loop walks the
copy.** The lock is not held while the loop body runs, so the body may do
anything, including change the same value:

```lume
async def main:
  shared var xs = [1, 2]
  for x in xs:
    xs.push(x * 10)
  puts xs    #=> [1, 2, 10, 20]
```

### Handing a `shared var` to a helper

**A parameter can be typed `shared var T`, and the helper then works on the
same value.** Passing a plain value there is an error.

```lume
def bump(n: shared var Int):
  n += 1

def count(xs: [Int]) -> Int = xs.len

async def main:
  shared var failures = 0
  shared var seen: [Int] = []
  var tasks: [Task[()]] = []
  for i in 1..4:
    t = spawn:
      seen.push(i)
      if i % 2 == 0:
        bump(failures)
    tasks.push(t)
  await tasks
  puts failures       #=> 2
  puts count(seen)    #=> 4
```

A `shared var` passed where a plain `[Int]` is expected, as `count(seen)`
is, is lent to the function under the lock for the length of the call.

A struct field can be `shared var T` as well. Copies of the struct then
share the value, which is how `examples/port/app_async.lume` hands one store
to every request.

## `Time.sleep`

**`Time.sleep(ms)` waits a whole number of milliseconds.** Pass an `Int`.
Inside async code — an `async def` or a `spawn:` — it is itself async and
must be awaited, `await Time.sleep(10)`, which lets other tasks run
meanwhile. In a plain `def` it takes no `await` and holds up the whole
thread.

## Where the warnings go

Every `shared var` declaration gives a warning:

```
warning: `hits` is `shared var`: every use of it takes a lock, so tasks change it one at a time
  --> main.lume:2:3
  |
2 |   shared var hits = 0
  |   ^
  help: fine for a store or a counter; keep the work done while it is locked short
```

**It goes to standard error, never standard output.** It comes from the
compiler, not from your program: `lume run`, `lume check` and `lume build`
print it once per `shared var`, each time they compile, and a program built
with `lume build` never prints it. It is not a sign that something is wrong.

The `.expected` files of `examples/async.lume`, `examples/match/` and
`examples/port/app_async.lume` show the warnings because the test suite
records both streams together. The output of those programs, on standard
output, starts after them.

## Further reading

- `examples/async.lume` — every form on this page in one short program.
- `examples/match/` — a whole program: fan out across tasks, collect, and
  report what failed, with two `shared var` tallies.
- `examples/port/app_async.lume` — one store shared by concurrent request
  handlers through a `shared var` struct field.

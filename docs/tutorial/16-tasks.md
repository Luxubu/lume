# Lunch 16: Tasks

**In this lunch you will:**

- start work that runs at the same time as other work, and wait for it;
- start many pieces of work and gather their results in order;
- understand what a task can and cannot see of the rest of your program;
- share one value between tasks safely.

## 16.1 Why do two things at once

A kitchen does not boil the soup, wait, then start the pie. It puts the
soup on, starts the pie while the soup boils, and serves each when it is
ready. Programs wait too — for a file, a program, the network — and while
one piece of work waits, another can go on.

A **task** is a piece of work that runs alongside the rest of your program.
A program with tasks starts with **`async def main`**. **`spawn:` starts a
task, and `await` waits for its result**:

```lume
def soup -> Str = "soup is ready"

async def main:
  t = spawn: soup()
  puts "started the soup"    #=> started the soup
  puts await t               #=> soup is ready
```

`spawn:` gives back a `Task[Str]` at once — a handle on work that is
running — and the program carries on. `await t` waits until the task is
done and gives its result.

## 16.2 Functions that wait: `async def`

Some work is waiting: for a timer, for another task. A function that waits
for such things is an **`async def`**, and every call to it is awaited.
`Time.sleep(ms)` waits for a number of milliseconds:

```lume
async def cook(dish: Str, minutes: Int) -> Str:
  await Time.sleep(minutes * 10)
  "#{dish} after #{minutes} minutes"

async def main:
  puts await cook("toast", 2)    #=> toast after 2 minutes
```

(A minute here is ten milliseconds, so the examples are quick.)

**Outside async code — in a plain `def` — `Time.sleep(ms)` is written
without `await`, and simply pauses**: there is nothing else that code could
get on with meanwhile. Inside an `async def` it must be awaited, so that
other tasks run while it waits. Leave off
the `await` and Lume says the result is not there yet:

```lume-bad
async def cook(dish: Str, minutes: Int) -> Str:
  await Time.sleep(minutes * 10)
  dish

async def main:
  puts cook("toast", 2)
#! `cook` is an `async def`: its result arrives later
```

## 16.3 Many at once

Start several tasks, and they all run together. **`await` on a list of
tasks waits for all of them, and gives their results in the order the
tasks were started** — whichever finishes first:

```lume
async def cook(dish: Str, minutes: Int) -> Str:
  await Time.sleep(minutes * 10)
  "#{dish} after #{minutes} minutes"

async def main:
  pie = spawn: await cook("pie", 9)
  soup = spawn: await cook("soup", 5)
  tea = spawn: await cook("tea", 1)
  puts await [pie, soup, tea]
  #=> ["pie after 9 minutes", "soup after 5 minutes", "tea after 1 minutes"]
```

The three cook together, so this takes about as long as the pie, not as
long as all three.

Notice **`spawn: await cook(...)`**: `cook` is an `async def`, so even
inside a task its result has to be waited for, with `await`, like any call
to one. A `spawn:` on one line takes a value — `spawn: await cook("pie", 9)`
or `spawn: sum_to(n)` — and for statements, such as `served += 1`, its body
goes on the lines below it, indented. The results come back in the order they were started,
so each one can be matched with what started it.

Tasks are usually made from a list, with `map`:

```lume
def sum_to(n: Int) -> Int = (1..n).sum

async def main:
  sizes = [10, 100, 1000]
  totals = await sizes.map { |n| spawn: sum_to(n) }.to_list
  puts totals    #=> [55, 5050, 500500]
```

## 16.4 Blocking work in tasks

`sum_to` above is a plain `def` — it does not wait for anything, it just
works. So is `Process.run`, from lunch 15. **A task whose body awaits
nothing runs on a pool of threads of its own**, so plain work like this,
or a program run, holds up no other task, and many run at once:

```lume
def say(word: Str) -> Str:
  (_, out, _) = Process.run("echo", [word]).or((1, "", ""))
  out.trim

async def main:
  words = await ["soup", "pie", "tea"].map { |w| spawn: say(w) }.to_list
  puts words    #=> ["soup", "pie", "tea"]
```

Three programs run at the same time, and the results still come back in
order.

## 16.5 What a task sees: copies

**A task gets its own copy of the names it uses**, made when it starts. So
changing a name afterwards does not reach the task:

```lume
async def main:
  var dish = "soup"
  t = spawn: "cooking #{dish}"
  dish = "stew"
  puts await t    #=> cooking soup
```

For the same reason, a task cannot change a `var` from outside: it would
only change its copy, and the change would be lost. Lume refuses it, and
says what to write instead:

```lume-bad
async def main:
  var served = 0
  t = spawn:
    served += 1
  await t
  puts served
#! `served` inside `spawn:` is a copy
#! declare it `shared var served`
```

## 16.6 One value for every task: `shared var`

**`shared var` makes one value that every task works on.** Each use of it
takes a lock — only one task touches it at a time — so tasks cannot
trample on each other's changes:

```lume
async def main:
  shared var served = 0
  # a task whose body ends with a statement gives back nothing, `()`
  var waiters: [Task[()]] = []
  for _ in 1..20:
    w = spawn:
      served += 1
    waiters.push(w)
  await waiters
  puts served    #=> 20
```

Twenty tasks each add one, and the total is twenty, every time.

When you run this, Lume prints a **warning** about the `shared var` first,
to standard error. It is a reminder, not a fault: every use of `served`
waits its turn for the lock, so tasks that do little but change a `shared
var` run one at a time. Keep the work done on a shared value short, and
most of the work in each task's own names.

## 16.7 Whichever finishes first

`await` on a list gives every result, in order. Sometimes you want each
result as soon as it is ready instead — to show progress, or to start
something new in its place. **`await tasks.next_done` waits for whichever
task in a `var` list finishes first**, takes it out of the list, and gives
its result — or `None` once the list is empty:

```lume
async def cook(dish: Str, minutes: Int) -> Str:
  await Time.sleep(minutes * 10)
  dish

async def main:
  var kitchen = [spawn: await cook("pie", 30), spawn: await cook("tea", 1)]
  first = await kitchen.next_done
  puts first           #=> Some("tea")
  puts kitchen.len     #=> 1
  puts await kitchen   #=> ["pie"]
```

## 16.8 Tasks that may fail

A task's body may give a `T or Error`, as any function may. Awaiting it
gives the result, to `match` or pass on with `?`:

```lume
def portion(text: Str) -> Int or Error = text.to_int

async def main:
  results = await ["4", "lots", "2"].map { |t| spawn: portion(t) }.to_list
  for r in results:
    match r:
      Ok(n) -> puts "#{n} portions"
      Error(e) -> puts "skipped: #{e.message}"
  #=> 4 portions
  #=> skipped: `lots` is not an integer
  #=> 2 portions
```

## Underneath, in Rust

`async def main` starts Rust's **tokio** runtime, the most widely used one
in Rust. `spawn:` is `tokio::spawn`, a task is a `JoinHandle`, and a body
that awaits nothing becomes `spawn_blocking`, on tokio's pool for blocking
work. The copies a task gets are Rust's `move` closures: a task owns what
it holds, which is how Rust makes sure two threads never change the same
value unguarded. A `shared var` is an `Arc<Mutex<T>>` — the guarded value
Rust would make you write by hand — and `next_done` is tokio's
`JoinSet::join_next`.

## Try it

1. Write `async def boil(minutes: Int) -> Str` that sleeps and says how
   long it took, and run three at once with different times, printing all
   the results.
2. Compute the squares of `1..5` in five tasks, one each, and print the
   list.
3. Count, in a `shared var`, how many of `["soup", "pie", "stew", "tea"]` are
   four letters long, each word checked in its own task.
4. Start two tasks, a slow one and a quick one, in a `var` list, and print
   their results in the order they finish with `next_done`.

## Recap

- `async def main` lets a program have tasks. `spawn:` starts one and gives
  a `Task[T]`; `await` gives its result.
- An `async def` may wait (`await Time.sleep(ms)`); every call to one is
  awaited.
- `await` on a list of tasks gives all the results, in the order the tasks
  started. A task that awaits nothing runs on a pool of its own.
- A task gets copies of the names it uses. `shared var` is one value every
  task shares, behind a lock.
- `await tasks.next_done` gives whichever task finishes first.

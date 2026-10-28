# Tests

A test is a named block in a file, beside the code it checks. `lume test`
runs a file's tests; `lume run` leaves them alone. This page says what a
test can do, what `lume test` prints, and how to run some tests and not
others.

## A test and `assert`

**`test "what it checks":` and an indented body.** In it, `assert` takes a
`Bool` that must be `true`. A test passes when its body runs to the end:

```lume
def tip(bill: Int, percent: Int) -> Int = bill * percent / 100

test "a tenth of a round bill":
  assert tip(50, 10) == 5

test "a tip is never negative":
  assert tip(0, 15) >= 0

def main:
  puts tip(80, 15)    #=> 12
```

```sh
lume test tip.lume
```

```text
test a tenth of a round bill ... ok
test a tip is never negative ... ok

2 tests: 2 passed, 0 failed
```

The tests run in the order they are written. A test may call anything in
its file, `pub` or not, and may use `!` without the warning it gets in
other code, since stopping is what a test should do when a value is
missing.

## What a failure says

**A failed comparison shows both sides**: the file as it was named on the
command line and the line, the `assert` as you wrote it, and the two
values, `left` and `right`:

```lume
def tip(bill: Int, percent: Int) -> Int = bill * percent / 100

test "a tenth of a round bill":
  assert tip(50, 10) == 5

test "rounded to the nearest":
  assert tip(55, 10) == 6

def main:
  puts tip(80, 15)    #=> 12
```

```text
test a tenth of a round bill ... ok
test rounded to the nearest ... FAILED
    tip.lume:7: assert tip(55, 10) == 6
      left:  5
      right: 6

2 tests: 1 passed, 1 failed
```

An `assert` that is not a comparison — `assert xs.empty?` — shows the line
and the `assert`. **Each side is worked out once**, as Rust's `assert_eq!`
works out each side once, so the value shown is the value that was
compared — which matters when a side moves something on, as `next` does:

```lume
def courses -> Iterator[Str]:
  yield "starter"
  yield "main"

test "the second course":
  var it = courses()
  assert it.next == Some("main")

def main:
  puts courses().to_list    #=> ["starter", "main"]
```

```text
test the second course ... FAILED
    menu.lume:7: assert it.next == Some("main")
      left:  Some("starter")
      right: Some("main")

1 test: 0 passed, 1 failed
```

**A test that stops** some other way — `xs.at(9)`, `!` on a failure, a
division by zero — fails too, with what stopped it and the line:

```lume
def share(total: Int, people: Int) -> Int = total / people

test "nobody to share with":
  assert share(12, 0) == 0

def main:
  puts share(12, 4)    #=> 3
```

```text
test nobody to share with ... FAILED
    share.lume:1: stopped: division by zero

1 test: 0 passed, 1 failed
```

The line is where it stopped, inside `share`, not the `assert` that called
it. The other tests still run: one failing test does not stop the rest.

**`lume test` exits with status 1 when a test fails**, and 0 when all
pass, so a script or a build server can tell.

## Some tests and not others

**`lume test tip.lume rounded` runs only the tests whose name has
`rounded` in it**, as `cargo test rounded` does:

```text
test rounded to the nearest ... FAILED
    tip.lume:7: assert tip(55, 10) == 6
      left:  5
      right: 6

1 test: 0 passed, 1 failed (1 not matching `rounded`)
```

A name that matches no test says so — ``none of the 2 tests has `zzz` in
its name`` — and is not a failure: `lume test` exits with 0, as `cargo
test` does.

## Testing async code: `async test`

**A test that waits is an `async test`**, as `async def main` is the `main`
that waits. It runs on its own tokio runtime, as Rust's `#[tokio::test]`
does, and can `await` an `async def`, a task, or an async sequence:

```lume
async def cook(dish: Str, ms: Int) -> Str:
  await Time.sleep(ms)
  "#{dish} is ready"

async test "two dishes at once":
  pie = spawn: await cook("pie", 10)
  tea = spawn: await cook("tea", 1)
  assert await [pie, tea] == ["pie is ready", "tea is ready"]

async def main:
  puts await cook("soup", 1)    #=> soup is ready
```

`await` in a plain `test` is refused, with the `async test` to write:

```lume-bad
async def cook(dish: Str) -> Str = dish.upcase

test "a dish is cooked":
  assert await cook("pie") == "PIE"

def main:
  puts 1
#! make it `async test "a dish is cooked":`
```

## Tests in a package

In a package, `lume test` with no file runs the tests of every file in the
package — `main.lume` or `lib.lume` and the modules they import — and not
the tests of its dependencies, as `cargo test` does. A test in a module is
named with the module first, `menu: prices add up`, and the modules' tests
run before the entry file's. `lume test prices` filters a package's tests
by that whole name, so `lume test menu` runs every test of `menu`.

## Underneath, in Rust

A test becomes a plain Rust function, and `lume test` builds the program
with a small runner in place of `main`, which calls each one and catches
its failure, as `cargo test` does for `#[test]` functions. A test is built
with overflow checks on, as under `lume run`. An `async test` is a function
that starts a tokio runtime and blocks on its body, which is what
`#[tokio::test]` expands to.

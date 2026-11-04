# Lunch 14: Tests

**In this lunch you will:**

- write tests beside your code, and run them with `lume test`;
- read what a failing test says;
- run some tests and not others;
- test code that may fail, and the tests of a package.

## 14.1 A test and `assert`

A **test** is a block in a file, with a name, that checks the code does
what it should. `assert` takes a condition that must be `true`:

```lume
def split_bill(total: Int, people: Int) -> Int = total / people

test "a bill splits evenly":
  assert split_bill(12, 3) == 4

test "a remainder is dropped":
  assert split_bill(10, 3) == 3

def main:
  puts split_bill(12, 3)    #=> 4
```

`lume run` runs `main` and leaves the tests alone. **`lume test` runs the
tests**, in the order they are written, and leaves `main` alone:

```sh
lume test bill.lume
```

```text
test a bill splits evenly ... ok
test a remainder is dropped ... ok

2 tests: 2 passed, 0 failed
```

A test sits in the same file as what it checks, so it may call anything
there, `pub` or not.

## 14.2 When a test fails

A failing `assert` says which, and shows both sides of a comparison, so you
can see what it got. Change the second test to `assert split_bill(10, 4) ==
3`, and `lume test bill.lume` says what follows the program:

```lume
def split_bill(total: Int, people: Int) -> Int = total / people

test "a bill splits evenly":
  assert split_bill(12, 3) == 4

test "a remainder is dropped":
  assert split_bill(10, 4) == 3

def main:
  puts split_bill(12, 3)    #=> 4
```

```text
test a bill splits evenly ... ok
test a remainder is dropped ... FAILED
    bill.lume:7: assert split_bill(10, 4) == 3
      left:  2
      right: 3

2 tests: 1 passed, 1 failed
```

`left` is what the code gave, `right` what the test expected. One failing
test does not stop the others: each runs, and the last line counts them.

## 14.3 Some tests and not others

`lume test bill.lume remainder` runs only the tests with `remainder` in
their name, which is quicker when you are working on one thing:

```text
test a remainder is dropped ... FAILED
    bill.lume:7: assert split_bill(10, 4) == 3
      left:  2
      right: 3

1 test: 0 passed, 1 failed (1 not matching `remainder`)
```

**`lume test` exits with status 1 when any test fails**, and 0 when all
pass, so a script or a build server can tell. A name that matches no test
says so — ``none of the 2 tests has `zzz` in its name`` — and is not a
failure.

## 14.4 Testing work that may fail

The work of lunch 10 — a `T or Error` — is tested the same way. **In a
test, `!` takes the value without the warning** it gets elsewhere: if the
value is not there, stopping is what the test should do. And `.error?`
asks whether a failure is what came back:

```lume
def split_bill(total: Int, people: Int) -> Int = total / people

def parse_people(s: Str) -> Int or Error:
  n = s.to_int?
  return Error("a table has at least one person") if n < 1
  Ok(n)

test "a bill splits evenly":
  assert split_bill(12, 3) == 4

test "people are read from text":
  assert parse_people("3")! == 3

test "nobody is no table":
  assert parse_people("0").error?

test "a table of none":
  assert split_bill(12, 0) == 0

def main:
  puts split_bill(12, parse_people("3").or(1))    #=> 4
```

```text
test a bill splits evenly ... ok
test people are read from text ... ok
test nobody is no table ... ok
test a table of none ... FAILED
    bill.lume:1: stopped: division by zero

4 tests: 3 passed, 1 failed
```

The last test finds a mistake: `split_bill` divides by the number of
people. A test that stops some other way fails too, with what stopped it
and where — the line inside `split_bill`, not the `assert` that called
it.

`parse_people` already refuses a table of none; the program never calls
`split_bill` with 0. Whether to make `split_bill` safe too, or drop the
test, is a decision a failing test makes you take on purpose.

## 14.5 The tests of a package

In a package (lunch 13), `lume test` needs no file name: it runs the tests
of every file in the package. From the `greet` folder:

```text
test greeting ... ok

1 test: 1 passed, 0 failed
```

A test in another module of the package is named with its module first —
`menu: prices add up` — and the filter matches that whole name, so `lume
test menu` runs every test of `menu`. A package's dependencies are tested
in their own folders, not with it: from `hello`, `lume test` says `no
tests`.

Write tests for the things that would be easy to get wrong and hard to
notice: the edge cases, like an empty list, a remainder, or nobody at the
table. Running them after every change takes a second, and catches the day
a change breaks something three files away. [Tests](../reference/testing.md)
in the reference has the rest, and how to test the async code of lunch 18
with `async test`.

## Underneath, in Rust

A test becomes a plain Rust function, and `lume test` builds the program
with a small runner in place of `main`, which calls each one and catches
its failure, as `cargo test` does for `#[test]` functions. A failed
comparison is Rust's `assert_eq!`, which shows both sides the same way.

## Try it

1. Write two tests for lunch 12's `count_of`: one for a list where `x`
   appears twice, and one for an empty list.
2. Write `average(xs: [Int]) -> Int or Error`, which fails for an empty
   list, and test both what it gives and that it fails.
3. Run your tests with a name that matches only one of them.

## Recap

- `test "name":` blocks with `assert` check your code; `lume test` runs
  them, and `lume run` leaves them alone.
- A failed comparison shows `left` and `right`; a test that stops shows
  where. The exit status is 1 when any test fails.
- `lume test file.lume word` runs the tests with `word` in their name.
- In a test, `!` gives the value with no warning, and `.error?` checks for
  a failure.
- In a package, `lume test` runs every file's tests, a module's named with
  the module first.

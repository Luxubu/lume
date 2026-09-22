# Absence and failure

Lume has two different things, and keeping them apart is most of what there is
to learn.

| | written | means | built from |
|---|---|---|---|
| absence | `T?` | there may be nothing here | `Some(x)` / `None` |
| failure | `T or Error` | this may have gone wrong, and there is a reason | `Ok(x)` / `Error("why")` |

**`T?` when nothing is wrong** — a lookup that missed, a list that was empty.
**`T or Error` when something went wrong and you want to say what.**

```lume
puts [10, 20][1]     #=> Some(20)
puts [10, 20][9]     #=> None
puts "42".to_int     #=> Ok(42)
puts "x".to_int      #=> Error("`x` is not an integer")
```

## `.or(default)`

The shortest way past either one. It takes the value or the default, and gives
you a plain `T` you can go on using:

```lume
puts [10, 20][1].or(0)     #=> 20
puts [10, 20][9].or(0)     #=> 0
puts "42".to_int.or(0)     #=> 42
puts "x".to_int.or(0)      #=> 0
puts "x".to_int.or(0).abs  #=> 0
```

## `match` on `Ok(x)` / `Error(e)`

```lume
def main:
  for s in ["42", "x"]:
    match s.to_int:
      Ok(n) -> puts "got #{n}"
      Error(e) -> puts "failed: #{e.message}"

#=> got 42
#=> failed: `x` is not an integer
```

`Some(x)` / `None` is the same shape for a `T?`:

```lume
def main:
  for i in [1, 9]:
    match [10, 20][i]:
      Some(n) -> puts "item #{n}"
      None -> puts "nothing there"

#=> item 20
#=> nothing there
```

## `Error` carries a message, and nothing else

`Error` has exactly one field:

```lume-bad
match "x".to_int:
  Ok(n) -> puts n
  Error(e) -> puts e.code
#! `Error` has fields message
```

`e.message` is the text. Printing the `Error` itself shows it quoted, the way
[printing](printing.md) shows any value inside another:

```lume
def main:
  match "x".to_int:
    Ok(n) -> puts n
    Error(e) ->
      puts e              #=> Error("`x` is not an integer")
      puts e.message      #=> `x` is not an integer
```

The built-in failures say what they were given:

```lume
puts "x".to_int      #=> Error("`x` is not an integer")
puts "x".to_float    #=> Error("`x` is not a number")
```

## Returning a failure

A function that may fail says `-> T or Error`.

**A bare `T` returned from such a function is wrapped in `Ok` for you.** You
never write `Ok(...)` yourself:

```lume
def half(n: Int) -> Int or Error:
  if n % 2 != 0:
    return Error("#{n} is odd")
  return n / 2

def main:
  puts half(10)   #=> Ok(5)
  puts half(7)    #=> Error("7 is odd")
```

**A value that is already a `T or Error` passes through — it is not wrapped
again.** So a function can simply hand back what it got:

```lume
def parse(s: Str) -> Int or Error:
  return s.to_int

def main:
  puts parse("5")   #=> Ok(5)
  puts parse("x")   #=> Error("`x` is not an integer")
```

There is no `Ok(Ok(5))` anywhere in that.

The last expression of the function is its value, as usual, so the `return` is
optional at the end — including on an `if`/`else` where one branch fails:

```lume
def half(n: Int) -> Int or Error:
  if n % 2 != 0:
    Error("#{n} is odd")
  else:
    n / 2

def main:
  puts half(10)   #=> Ok(5)
  puts half(7)    #=> Error("7 is odd")
```

## A failure nothing looks at is a compile error

> New in milestone 34, and the rule most likely to catch you out if you have
> written Go or Rust: there is no dropping a failure by accident.

Lume will not let a failure go unlooked-at. There are three forms of the rule.

**An `Error(...)` written as a statement does nothing, so it is rejected.** You
meant `return`:

```lume-bad
def check(n: Int) -> Int or Error:
  if n < 0:
    Error("negative")
  n * 2

def main:
  puts check(3)
#! this failure is thrown away, because nothing returns it
#! write `return Error(...)` to stop here with it
```

The fix is `return Error("negative")`. An `Error(...)` that *is* the function's
value — the last expression, or the last expression of a branch — is fine, as
in `half` above; that one is returned.

This first form is checked in a function whose success type is a real type. In
a function declared `-> () or Error` a stray `Error(...)` statement is
currently accepted and silently discarded, so write the `return` there by
habit rather than by the compiler's insistence.

**A call that can fail, standing on its own line, is rejected**:

```lume-bad
def may_fail() -> Int or Error:
  return Error("nope")

def main:
  may_fail()
  puts "done"
#! can fail, and nothing here looks at the result
#! pass the failure on with `?`, handle it with `match`, or say you mean to drop it: `_ = ...`
```

**`_ = ...` is how you say you mean to drop it.** Nothing is hidden: the
intent is right there in the line.

```lume
def may_fail() -> Int or Error:
  return Error("nope")

def main:
  _ = may_fail()
  puts "done"    #=> done
```

## `?` — pass the failure up

`?` unwraps the value, or returns from the current function with the failure.
It is how a function that calls failing things stays readable:

```lume
def double(s: Str) -> Int or Error:
  n = s.to_int?
  return n * 2

def main:
  puts double("21")   #=> Ok(42)
  puts double("x")    #=> Error("`x` is not an integer")
```

**`?` only works where the failure has somewhere to go** — in a function whose
own return type can carry it. In a plain `main` it is rejected, with the fix
spelled out:

```lume-bad
n = "x".to_int?
puts n
#! `?` returns the error early, but `main` does not return `T or E`
#! write `def main(...) -> Type or Error`, or handle the error with `match` or `.or(default)`
```

`main` may declare it, and then a failure ends the program with that message
and a non-zero exit status:

```lume
def load(s: Str) -> Int or Error:
  return s.to_int

def main -> () or Error:
  n = load("21")?
  puts n * 2     #=> 42
  return ()
```

`?` works on a `T?` too, in a function that returns a `T?`:

```lume
def head(xs: [Int]) -> Int?:
  first = xs.first?
  return first + 0

def main:
  puts head([5, 6])   #=> Some(5)
  puts head([])       #=> None
```

The two do not mix, and the compiler names the conversion for each direction.
`?` on a failure inside a `T?` function:

```lume-bad
def f(s: Str) -> Int?:
  n = s.to_int?
  return n

def main:
  puts f("3")
#! `?` on a `Int or Error` returns the error, but `f` returns `Int?`
```

`?` on an absence inside a `T or Error` function:

```lume-bad
def f(xs: [Int]) -> Int or Error:
  n = xs.first?
  return n

def main:
  puts f([3])
#! `?` on an optional value returns `None`, but `f` returns `Int or Error`
#! turn the absence into an error first: `.or_error("what went wrong")?`
```

`.or_error(msg)` is that bridge — it turns a `T?` into a `T or Error` by
giving the absence a reason:

```lume
def f(xs: [Int]) -> Int or Error:
  n = xs.first.or_error("empty list")?
  return n

def main:
  puts f([3])   #=> Ok(3)
  puts f([])    #=> Error("empty list")
```

## `Error(e) -> e`, not `Error(e) -> Error(e)`

> Reviewers who wrote a `match` over a failure reached for `Error(e)` first,
> round after round.

When you `match` on a failure, the `e` bound by `Error(e)` **is already an
`Error`**. To pass it on, give it back as it is:

```lume
def parse(s: Str) -> Int or Error:
  match s.to_int:
    Ok(n) -> n * 2
    Error(e) -> e

def main:
  puts parse("21")   #=> Ok(42)
  puts parse("x")    #=> Error("`x` is not an integer")
```

Writing `Error(e)` wraps an `Error` in an `Error`, and the compiler stops you:

```lume-bad
def parse(s: Str) -> Int or Error:
  match s.to_int:
    Ok(n) -> n * 2
    Error(e) -> Error(e)

def main:
  puts parse("21")
#! `Error` field `message` is `Str`, but this is a `Error`
```

To add context, build a new `Error` from the text:

```lume
def parse(s: Str) -> Int or Error:
  match s.to_int:
    Ok(n) -> n * 2
    Error(e) -> Error("parsing #{s}: #{e.message}")

def main:
  puts parse("21")   #=> Ok(42)
  puts parse("x")    #=> Error("parsing x: `x` is not an integer")
```

## `!` — unwrap or stop

`!` takes the value out, and **stops the program** if there is none. It is the
blunt instrument, and the compiler warns every time you use it:

```lume
puts "7".to_int!    #=> 7
```

That runs and prints `7`, but the compiler writes this to standard error on
the way past:

```
warning: `!` stops the program if the value is missing or an error
  --> quick.lume:2:18
  |
2 |   puts "7".to_int!
  |                  ^
  help: fine in tests and quick scripts; elsewhere use `match`, `?` or `.or(default)`
```

Give it a failure instead and the run ends there, with the message and a
non-zero status:

```
$ lume run bad.lume       # bad.lume says: puts "x".to_int!
error: `x` is not an integer
  (set LUME_BACKTRACE=1 to see where in the generated Rust)
```

Use it in a test or a throwaway script. In anything that has to keep running,
use `.or`, `match` or `?`.

## The other methods

On a `T?`:

```lume
def main:
  x: Int? = Some(3)
  y: Int? = None
  puts x.some?                  #=> true
  puts y.none?                  #=> true
  puts x.map { |n| n * 2 }      #=> Some(6)
  puts x.or_error("nothing")    #=> Ok(3)
  puts y.or_error("nothing")    #=> Error("nothing")
```

On a `T or Error`:

```lume
def main:
  r = "x".to_int
  puts r.ok?                 #=> false
  puts r.error?              #=> true
  puts r.ok                  #=> None
  puts r.or(0)               #=> 0
  puts r.map { |n| n + 1 }   #=> Error("`x` is not an integer")
```

`r.ok` is a `T?` — the value if it succeeded. `r.error` is an `Error?`, the
other way round:

```lume
def main:
  puts "x".to_int.error    #=> Some(Error("`x` is not an integer"))
  puts "7".to_int.error    #=> None
```

Neither `T?` nor `T or Error` can be used as if it were the value. A method or
some arithmetic straight on one is an error, and the message tells you how to
get past it:

```lume-bad
x = "abc"[1]
puts x.upcase
#! it may be absent, so `.upcase` cannot be called on it directly
#! unwrap it first: `match` on `Some(x)`/`None`, or `.or(default).upcase`
```

# Patterns: what `match` can take apart

> Reviewers wrote list patterns with literals in them, nested patterns
> through recursive enums, tuples of two enums and one-line assignment arms,
> and could not find a page that said any of it worked. It all does. This
> page shows each one.

A `match` tries its arms **from top to bottom, and the first arm that fits
wins**. An arm is a pattern, an optional `if` guard, `->`, and what to do.
The arms together must cover every value. The compiler checks this and names
what is missing.

## The forms at a glance

| pattern | fits | binds |
|---|---|---|
| `_` | anything | nothing |
| `name` | anything | the whole value, as `name` |
| `42`, `-3`, `"text"`, `1.5`, `true` | that exact value | nothing |
| `1..5`, `1...5` | an `Int` in the range (`..` includes the end, `...` does not) | nothing |
| `Circle(r)`, `Box(w, _)` | that variant | its fields, by position |
| `Circle(..)`, `Dot` | that variant, whatever it holds | nothing |
| `Shape.Box(w, h)` | the same, with the enum named | as above |
| `Some(x)`, `None`, `Ok(x)`, `Error(e)` | an optional or a result | the value, the `Error` |
| `(a, b)` | a tuple, part by part | each part |
| `[]`, `[x]`, `[a, b]` | a list of exactly that length | each item |
| `[first, ..rest]`, `[first, ..]` | a list of at least that length | the items, and the rest as a list |
| `p1 \| p2` | either pattern | the same names in each |

Every pattern inside another can be any of these, to any depth.

## Literals, ranges and a catch-all

```lume
def size(n: Int) -> Str:
  match n:
    0 -> "none"
    -1 -> "minus one"
    1 | 2 -> "a couple"
    3..9 -> "a few"
    10...100 -> "some"
    _ -> "lots"

def main:
  for n in [0, 0 - 1, 2, 9, 100]:
    puts size(n)
#=> none
#=> minus one
#=> a couple
#=> a few
#=> lots
```

`10...100` stops before 100, so `100` falls through to `_`.

A `Str` literal matches a `Str`, and it matches a `Char` too:

```lume
for c in "hé!".chars:
  match c:
    "h" -> puts "aitch"
    "é" -> puts "e acute"
    _ -> puts "other"
#=> aitch
#=> e acute
#=> other
```

**A catch-all may bind a name** in place of `_`. The name holds the whole
value:

```lume
n = 7
match n:
  0 -> puts "zero"
  other -> puts "not zero: #{other}"    #=> not zero: 7
```

**A name in a pattern is always a new name.** It never compares against a
variable that already exists, even one with the same name. To compare, use a
guard:

```lume
want = 3
x = 5
match x:
  want -> puts "bound #{want}"    #=> bound 5
match x:
  n if n == want -> puts "equal"
  _ -> puts "not equal"           #=> not equal
```

A top-level constant is not a pattern either. The compiler reads a capitalised
name as a variant:

```lume-bad
LIMIT = 3

def main:
  match 5:
    LIMIT -> puts "at the limit"
    _ -> puts "other"
#! unknown variant `LIMIT`
```

## The first arm that fits wins

Arms may overlap. The earlier one takes the value:

```lume
for n in [5, 50]:
  match n:
    5 -> puts "exactly five"
    n if n < 10 -> puts "small"
    _ -> puts "big"
#=> exactly five
#=> big
```

An arm that can never be reached, because the arms above it already take
everything it would, is a warning, not an error. The warning goes to standard
error, and the program still runs.

## Guards

`if` after a pattern adds a condition. It can use the names the pattern
binds:

```lume
def label(o: Int?) -> Str:
  match o:
    Some(n) if n > 100 -> "large #{n}"
    Some(n) -> "#{n}"
    None -> "none"

def main:
  puts label(Some(500))    #=> large 500
  puts label(Some(5))      #=> 5
  puts label(None)         #=> none
```

**A guarded arm does not count toward covering the value**, because the
guard might be false. Something after it must still cover the case:

```lume-bad
o = Some(3)
match o:
  Some(n) if n > 2 -> puts "big"
  None -> puts "none"
#! does not cover: Some(_)
```

`_ if cond` is a guard on its own, with no pattern to take apart:

```lume
c = "Q".chars[0].or(" ")
match c:
  "a" | "e" -> puts "vowel"
  _ if c.upper? -> puts "capital"    #=> capital
  _ -> puts "other"
```

## Enums, nested to any depth

A variant pattern takes the fields by position, and each field is a pattern
of its own. **That includes literals and variants inside a recursive enum**:

```lume
enum Expr:
  Num(v: Int)
  Add(l: Expr, r: Expr)
  Neg(e: Expr)

def simplify(e: Expr) -> Expr:
  match e:
    Add(Num(0), r) -> simplify(r)
    Add(l, Num(0)) -> simplify(l)
    Neg(Neg(inner)) -> simplify(inner)
    other -> other

def main:
  puts simplify(Add(Num(0), Neg(Neg(Num(5)))))    #=> Num(v: 5)
  puts simplify(Add(Num(2), Num(0)))              #=> Num(v: 2)
  puts simplify(Add(Num(1), Num(2)))              #=> Add(l: Num(v: 1), r: Num(v: 2))
```

`Circle(..)` matches a variant without naming its fields. `Shape.Box(w, _)`
names the enum too, which you need when two enums share a variant name:

```lume
enum Shape:
  Circle(r: Float)
  Box(w: Float, h: Float)
  Dot

def main:
  for s in [Circle(1.0), Box(2.0, 3.0), Dot]:
    match s:
      Circle(..) -> puts "round"
      Shape.Box(w, _) -> puts "box #{w} wide"
      Dot -> puts "dot"
#=> round
#=> box 2.0 wide
#=> dot
```

Optionals and results nest the same way:

```lume
o: Int?? = Some(Some(3))
match o:
  Some(Some(v)) -> puts "inner #{v}"    #=> inner 3
  Some(None) -> puts "empty inside"
  None -> puts "nothing"
```

A struct is not a pattern. Bind it and read its fields, or use a guard.

## Tuples

A tuple pattern takes each part apart, and **each part can bind a variant's
payload**. Matching on a tuple you build on the spot is how two values are
matched together:

```lume
enum State:
  Idle
  Playing(track: Str, at: Int)

enum Event:
  Play(track: Str)
  Tick(secs: Int)
  Stop

def step(s: State, e: Event) -> State:
  match (s, e):
    (Playing(t, at), Tick(n)) -> Playing(t, at + n)
    (_, Play(t)) -> Playing(t, 0)
    (_, Stop) -> Idle
    (state, _) -> state

def main:
  var s = Idle
  for e in [Play("song"), Tick(5), Tick(3), Stop, Tick(1)]:
    s = step(s, e)
    puts s
#=> Playing(track: "song", at: 0)
#=> Playing(track: "song", at: 5)
#=> Playing(track: "song", at: 8)
#=> Idle
#=> Idle
```

The last arm binds the whole state with a name. The coverage check reads the
tuple as a whole, so it knows `(state, _)` covers every pair left.

A tuple of a `T?` and a `T or Error` works the same way:

```lume
match (Some(1), "x".to_int):
  (Some(a), Ok(b)) -> puts "#{a} and #{b}"
  (Some(a), Error(e)) -> puts "#{a}, then #{e.message}"    #=> 1, then `x` is not an integer
  (None, _) -> puts "none"
```

## Lists

**`[a, b]` fits a list of exactly two items.** `[first, ..rest]` fits a list
of one or more and binds the rest as a list. `[first, ..]` does the same
without naming the rest. **Literals can go inside**, and a list pattern
takes a `split` apart directly, with no `.to_list`:

```lume
def run(line: Str) -> Str:
  match line.split:
    ["push", n] -> "push #{n}"
    [src, "->", dst] -> "move #{src} to #{dst}"
    ["pop"] -> "pop"
    [cmd, ..rest] -> "#{cmd}? (#{rest.len} more)"
    [] -> "blank"

def main:
  puts run("push 3")         #=> push 3
  puts run("a -> b")         #=> move a to b
  puts run("pop")            #=> pop
  puts run("jump 1 2 3")     #=> jump? (3 more)
  puts run("")               #=> blank
```

`split(",")` works the same way. A line with no comma is a one-item list,
so `[k, v]` does not fit it:

```lume
for line in ["a,1", "b"]:
  match line.split(","):
    [k, v] -> puts "#{k} is #{v}"
    [one] -> puts "just #{one}"
    _ -> puts "other"
#=> a is 1
#=> just b
```

The rest has to come last:

```lume-bad
match [1, 2, 3]:
  [a, .., z] -> puts a
  _ -> puts "short"
#! `..rest` must be the last thing in a list pattern
```

If a list match has no `_`, the compiler says which lengths are missing:

```lume-bad
xs = [1]
match xs:
  [] -> puts "empty"
  [a] -> puts "one"
#! does not cover: [_, _, ..rest]
```

## `|`: alternatives

`p1 | p2` fits if either does. **Every alternative must bind the same
names**, so the arm can use them whichever one fitted:

```lume
enum Pair:
  Two(a: Int, b: Int)
  Nothing

def main:
  for p in [Two(0, 4), Two(4, 0), Two(1, 1), Nothing]:
    match p:
      Two(x, 0) | Two(0, x) -> puts "one side is zero, the other #{x}"
      Two(..) | Nothing -> puts "no zero"
#=> one side is zero, the other 4
#=> one side is zero, the other 4
#=> no zero
#=> no zero
```

```lume-bad
enum Pair:
  Two(a: Int, b: Int)
  Nothing

def main:
  match Two(1, 2):
    Two(x, 0) | Nothing -> puts "x"
    _ -> puts "y"
#! every alternative of `|` must bind the same names
```

One more rule. When an alternative has a `Str` or `Float` literal inside
something else, all the alternatives must have the same shape, differing
only in their literals. The same goes for any literal inside a recursive
enum's fields. Split such an arm in two:

```lume-bad
match ("a", 1):
  ("a", _) | (_, 2) -> puts "hit"
  _ -> puts "miss"
#! alternatives with a string or float inside must otherwise look the same
```

```lume
for pair in [("a", 1), ("b", 2), ("c", 3)]:
  match pair:
    ("a", _) -> puts "hit"
    (_, 2) -> puts "hit"
    _ -> puts "miss"
#=> hit
#=> hit
#=> miss
```

## What an arm can do

**An arm can be one line or an indented block.** On one line it can be a
value, a `puts`, an assignment, `next`, `break`, `return` or `()`:

```lume
var done = 0
var failed = 0
for s in ["1", "x", "2", "stop", "9"]:
  match s:
    "stop" -> break
    _ -> ()
  match s.to_int:
    Ok(_) -> done += 1
    Error(_) -> failed += 1
puts "#{done} read, #{failed} failed"    #=> 2 read, 1 failed
```

An indented block holds as many statements as you like. When the `match` is
a value, **the last line of each arm is that arm's value**, and `return`
leaves the whole function:

```lume
def grade(n: Int) -> Str:
  match n:
    90..100 -> "A"
    s if s >= 80 ->
      plus = if s >= 85: "+" else: ""
      "B" + plus
    s if s < 0 -> return "invalid"
    _ -> "C"

def main:
  puts grade(95)          #=> A
  puts grade(87)          #=> B+
  puts grade(81)          #=> B
  puts grade(0 - 1)       #=> invalid
  puts grade(40)          #=> C
```

## Covering everything

The compiler lists what the arms leave out, down to nested cases:

```lume-bad
enum Shape:
  Circle(r: Float)
  Box(w: Float, h: Float)

def main:
  match Circle(1.0):
    Circle(r) -> puts r
#! does not cover: Box(_, _)
```

```lume-bad
match (true, 1):
  (true, _) -> puts "yes"
#! does not cover: (false, _)
```

An `Int`, a `Str` or a `Float` has too many values to list, so a `match` on
one always needs a final `_` or name:

```lume-bad
n = 3
match n:
  1 -> puts "one"
#! add a final `_ ->` arm
```

`examples/patterns.lume` has more of these as a runnable program, and
[control flow](control-flow.md) covers `match` as a value alongside `if`.

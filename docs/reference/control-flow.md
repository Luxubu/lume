# `if`, `for`, `while`, `match`

## `if` / `elif` / `else`

As a statement it is what you expect. The keyword is `elif`, not `else if`:

```lume
def classify(n: Int) -> Str:
  if n < 0:
    return "negative"
  elif n == 0:
    return "zero"
  else:
    return "positive"

def main:
  puts classify(0 - 4)   #=> negative
  puts classify(0)       #=> zero
  puts classify(4)       #=> positive
```

There are no parentheses around the condition, and `not` spells negation:

```lume
n = 3
if not n > 5:
  puts "small"    #=> small
```

`else if` is spelled `elif`, and the compiler says so if you forget:

```lume-bad
n = 3
if n > 5:
  puts "a"
else if n > 1:
  puts "b"
#! `else if` is written `elif` in Lume
```

## `if` as a value

Every branch produces the value, and **there must be an `else`**:

```lume
n = 7
label = if n < 0: "negative" elif n == 0: "zero" else: "positive"
puts label    #=> positive
```

```lume-bad
n = 7
label = if n > 5: "big"
puts label
#! an `if` used as a value needs an `else`
```

### A multi-line `if` can be a value

> Reviewers repeatedly assumed it could not be, and wrote a `var` and a
> statement `if` instead.

It can. The rule is only about where the line breaks go: **the `if` must start
on the same line as the `=`**, and the branches are indented under it.

```lume
def main:
  n = 7
  label = if n < 0:
    "negative"
  elif n == 0:
    "zero"
  else:
    "positive"
  puts label    #=> positive
```

Breaking the line after the `=` does not work — the parser wants a value on
that line:

```lume-bad
n = 7
label =
  if n < 0:
    "negative"
  else:
    "positive"
puts label
#! expected a value here, but the line ended
```

## Trailing `if` and `unless`

**A trailing `if` attaches to the whole statement in front of it**, not to the
last expression on the line.

```lume
n = 7
puts "big" if n > 5          #=> big
puts "small" unless n > 5
puts "a" + "b" if n > 5      #=> ab
```

`puts "a" + "b" if n > 5` prints `ab` or prints nothing; it never prints
`a` and then decides about `b`.

It works on an assignment, and on `next` and `break`:

```lume
def main:
  var msg = "none"
  n = 7
  msg = "yes" if n > 5
  puts msg              #=> yes
  for i in 1..5:
    next if i % 2 == 0
    puts i              #=> 1
                        #=> 3
                        #=> 5
```

A *binding* under a trailing `if` does not survive it. The name belongs to the
conditional, so there is nothing to read on the next line:

```lume-bad
n = 7
x = 1 if n > 5
puts x
#! unknown name `x`
```

Use a `var` declared before it, as `msg` is above.

**`unless` only exists in the trailing form.** There is no `unless:` block; the
compiler says so:

```lume-bad
n = 3
unless n > 5:
  puts "small"
#! `unless` cannot start an expression
```

## `while`

```lume
var i = 0
while i < 3:
  puts i        #=> 0
                #=> 1
                #=> 2
  i = i + 1
```

`while true` with a `break` is the way to write a loop that decides when to
stop:

```lume
var total = 0
while true:
  total = total + 1
  break if total == 3
puts total    #=> 3
```

`while` is the only loop of this kind. `loop`, `until` and `do ... while` are
not keywords in Lume; a line starting with one of them is read as an ordinary
name and fails to parse.

## `for` over a range

**`1..10` includes `10`. `1...10` stops at `9`.** Two dots is inclusive; three
dots is exclusive.

```lume
for i in 1..3:
  puts i      #=> 1
              #=> 2
              #=> 3
```

```lume
for i in 1...3:
  puts i      #=> 1
              #=> 2
```

The bounds can be any `Int` expression:

```lume
n = 4
for i in 0..n:
  puts i    #=> 0
            #=> 1
            #=> 2
            #=> 3
            #=> 4
```

### Descending and stepping

**A range never counts down.** `3..1` is empty — it does not walk backwards, it
walks nothing:

```lume
for i in 3..1:
  puts i
puts "nothing above"    #=> nothing above
```

To count down, `.reverse` the range:

```lume
for i in (1..5).reverse:
  puts i    #=> 5
            #=> 4
            #=> 3
            #=> 2
            #=> 1
```

**There is no step.** A range always moves by one. For every second item, walk
all of them and skip:

```lume
for i in 1..10 where i % 2 == 1:
  puts i    #=> 1
            #=> 3
            #=> 5
            #=> 7
            #=> 9
```

Do not write `(1..10).step(2)`. It is not a method Lume has; `lume check`
lets it through and the build then fails inside the generated Rust, which is
a gap in the compiler, not a feature waiting for you:

```lume-skip
for i in (1..10).step(2):    # does not build
  puts i
```

## `for var`: changing the items

**`for var x in xs` walks the list's own items, so changing `x` changes the
list.** It is for items with fields or methods that change them:

```lume
struct Counter:
  n: Int
  def bump(var self):
    n += 1

def main:
  var cs = [Counter(n: 1), Counter(n: 5)]
  for var c in cs:
    c.bump
  puts cs.map(_.n)    #=> [2, 6]
```

Over numbers it is refused, because what would change is a copy of each
number, not the list:

```lume-bad
def main:
  var xs = [3, 4]
  for var x in xs:
    x += 1
#! would change copies
```

To change numbers in a list, build the new list: `xs = xs.map { |x| x + 1 }`
on a `var xs`, or assign by position, `xs[i] = ...`.

## `for x in xs where cond`

A `where` clause on a `for` skips the items that do not match. It reads as a
filter and compiles as one loop.

```lume
for i in 1..10 where i % 3 == 0:
  puts i    #=> 3
            #=> 6
            #=> 9
```

```lume
xs = ["a", "bb", "ccc"]
for w in xs where w.len > 1:
  puts w    #=> bb
            #=> ccc
```

`where` and `break` work together; `where` decides what the body sees, `break`
still ends the loop:

```lume
xs = [1, 2, 3, 4, 5, 6]
for x in xs where x % 2 == 0:
  break if x > 4
  puts x    #=> 2
            #=> 4
```

## `next` and `break`

> Reviewers could not find out whether `next` was real, and some assumed it
> worked only in a `while`.

**Both are real keywords, and both work in a `for` and in a `while`.** `next`
goes to the next item; `break` leaves the loop.

```lume
for i in 1..5:
  if i == 3:
    next
  if i == 5:
    break
  puts i      #=> 1
              #=> 2
              #=> 4
```

```lume
var j = 0
while j < 10:
  j = j + 1
  if j % 2 == 0:
    next
  if j > 7:
    break
  puts j     #=> 1
             #=> 3
             #=> 5
             #=> 7
```

**In nested loops, both apply to the innermost loop.** There are no labels, so
`break` in an inner loop returns to the outer one and the outer one keeps
going:

```lume
for i in 1..3:
  for j in 1..3:
    break if j == 2
    puts "#{i},#{j}"    #=> 1,1
                        #=> 2,1
                        #=> 3,1
```

```lume
for i in 1..2:
  for j in 1..3:
    next if j == 2
    puts "#{i},#{j}"    #=> 1,1
                        #=> 1,3
                        #=> 2,1
                        #=> 2,3
```

To leave both, set a `var` the outer loop tests, or move the inner loop into a
function and `return` from it:

```lume
def main:
  var done = false
  for i in 1..3:
    for j in 1..3:
      if i * j >= 4:
        done = true
        break
      puts "#{i},#{j}"    #=> 1,1
                          #=> 1,2
                          #=> 1,3
                          #=> 2,1
    break if done
```

## `match`

A `match` arm can be a single expression after the `->`, **or an indented
block of as many statements as you like** on the following lines:

```lume
def main:
  for n in [1, 2, 3]:
    match n:
      1 ->
        puts "one"
        puts "still one"
      2 -> puts "two"
      _ ->
        x = n * 10
        puts x

#=> one
#=> still one
#=> two
#=> 30
```

`match` is also a value, with the same rule as `if`: every arm gives one, and
the arms must cover everything.

```lume
def main:
  n = 2
  name = match n:
    1 -> "one"
    2 -> "two"
    _ -> "many"
  puts name    #=> two
```

Alternatives in one arm are separated by `|`:

```lume
for c in "hello".chars:
  match c:
    "a" | "e" | "i" | "o" | "u" -> puts "#{c} vowel"
    _ -> puts "#{c} other"
#=> h other
#=> e vowel
#=> l other
#=> l other
#=> o vowel
```

## Doing nothing

**`()` is the statement that does nothing.** It is there for the branch or arm
you want to leave empty — an empty block is a parse error, so `()` is how you
say "this case is handled, by not acting".

```lume
n = 7
if n > 5:
  ()
else:
  puts "no"
puts "done"    #=> done
```

```lume
n = 2
match n:
  1 -> ()
  _ -> puts "not one"    #=> not one
```

`()` is also the empty return type: `def main -> () or Error` is a `main` that
returns nothing or fails. See [errors](errors.md).

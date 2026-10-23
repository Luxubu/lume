# Lunch 1: Hello, Lume

**In this lunch you will:**

- write a Lume program and run it;
- learn how the examples in these chapters show what they print;
- read what the compiler says when a program is wrong, and fix it;
- meet the four commands you will use every day.

## 1.1 Your first program

Make a folder for these lunches, and in it a file called `hello.lume`:

```lume
def main:
  puts "Hello, Lume!"    #=> Hello, Lume!
```

Now run it:

```sh
lume run hello.lume
```

It prints:

```text
Hello, Lume!
```

That is a whole program. `def main:` says where it starts: when you run a
file, Lume runs its `main`. `puts` prints one value and then a new line —
the name is short for "put string", from Ruby, where Lume borrows it.

The first time you run a program it takes a second or so, because Lume
turns it into a native program before running it. Run it again without
changing anything and it starts at once: an unchanged program is not built
again.

## 1.2 How to read the examples

In these chapters, **`#=>` at the end of a line shows what that line
prints.** You do not type it — though you can, because `#` starts a comment,
and Lume ignores everything from `#` to the end of the line.

A line that prints more than one line shows each, one under the other:

```lume
def main:
  puts "one"      #=> one
  puts "two"      #=> two
  puts 1 + 2      #=> 3
```

Every example in these chapters is run by Lume's own test suite, which
checks that it prints exactly what the `#=>` lines say. So when your output
differs from the page, the difference is in what you typed — which is
worth finding, because that is most of what learning a language is.

## 1.3 `puts` prints anything

`puts` takes any value, not only text:

```lume
def main:
  puts 42               #=> 42
  puts 3.5              #=> 3.5
  puts true             #=> true
  puts [1, 2, 3]        #=> [1, 2, 3]
  puts ["tea", "cake"]  #=> ["tea", "cake"]
```

Text on its own prints as it is, with no quotes. Inside a list, text keeps
its quotes, so you can tell `["1"]` — a list holding the text `1` — from
`[1]`, a list holding the number.

## 1.4 Lines and indentation

Lume has no braces and no `end`. **A line ending in `:` opens a block, and
the lines indented under it are that block.** Two spaces is the custom.

```lume
def main:
  puts "first"     #=> first
  puts "second"    #=> second
```

Both `puts` lines are indented the same, so both belong to `main`. Indent a
line further for no reason and Lume stops you:

```lume-bad
def main:
  puts "first"
    puts "second"
#! this line is indented more than the one above it, but nothing opened a block
```

Examples like that one, which do not compile, are marked in these
chapters: the line starting `#!` is the message Lume gives. They are there
on purpose, so you can see what each mistake looks like before you make it
yourself.

## 1.5 When something is wrong

Change `hello.lume` so it says hello to someone, but misspell the name:

```lume-bad
def main:
  name = "Ada"
  puts "Hello, " + nmae
#! unknown name `nmae`
#! did you mean `name`?
```

`lume run` does not run it. It prints this instead:

```text
error: unknown name `nmae`
  --> hello.lume:3:20
  |
3 |   puts "Hello, " + nmae
  |                    ^
  help: did you mean `name`?
```

Read a message from the top:

1. **What is wrong**: there is no name `nmae`.
2. **Where**: the file, line 3, column 20, and the line itself with a `^`
   under the place.
3. **What to do**: the `help` line. Here it noticed a name that is
   nearly the same.

Fix the typo, and it runs. Lume finds mistakes like this one before your
program starts — the whole program is checked first — so a typo in a part
of your program that rarely runs is still caught the first time.

Some messages explain a rule. Adding a number to text is one:

```lume-bad
def main:
  puts "Lunch at " + 12
#! `+` cannot combine a `Str` and an `Int`
```

`+` adds two numbers or joins two pieces of text, but not one of each. The
help line says what to write instead; [lunch 2](02-names-and-values.md)
shows it.

## 1.6 The commands you will use

| command | what it does |
|---|---|
| `lume run hello.lume` | checks the program, builds it, and runs it |
| `lume check hello.lume` | only checks it: the fastest way to find mistakes |
| `lume fmt hello.lume` | rewrites it in the standard layout: spacing, indentation |
| `lume build hello.lume -o hello` | builds a program you can give to someone, fully optimised |

`lume build` makes a single file, `hello`, which runs on a machine that has
never heard of Lume. `lume --help` lists the rest.

## Underneath, in Rust

Lume turns your program into Rust, and the Rust compiler turns that into a
native program. `lume emit` shows the Rust it writes:

```sh
lume emit hello.lume
```

Near the end, after the support code every program carries, is your
`main`:

```rust
pub fn main() {
    lume_install_panic_hook();
    println!("{}", String::from("Hello, Lume!"));
}
```

`puts` became `println!`. The built program, and the Rust it came from, are
kept in a folder named `.lume` next to your file; `lume clean hello.lume`
removes it. You never need to read this Rust, but it is there, and it is
why a Lume program runs about as fast as one written in Rust by hand.

## Try it

1. Write a program that prints your name on one line and your favourite
   number on the next.
2. Print a list of three things you might have for lunch.
3. Make the mistake from 1.5 on purpose, with a different name, and read
   what Lume says. Then make a different mistake — leave off the `:` after
   `def main` — and read that message too.
4. Write a program with untidy spacing — `puts    "hi"` — and run
   `lume fmt` on it. What changed?

## Recap

- A program starts at `def main:`. `lume run file.lume` runs it.
- `puts` prints any value and a new line.
- `#` starts a comment. In these chapters, `#=>` shows what a line prints.
- A line ending in `:` opens a block; the block is the lines indented
  under it.
- Lume checks the whole program before it runs. Its messages say what is
  wrong, where, and usually what to do.

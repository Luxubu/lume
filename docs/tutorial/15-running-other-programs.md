# Lunch 15: Running other programs

**In this lunch you will:**

- run another program from yours, and read what it printed;
- tell a program that failed from one that could not start;
- meet tuples: several values handed back at once;
- build pipes with `sh`, and run a program in a folder of your choice.

## 15.1 Running a program

Your computer is full of programs that already do useful things: `sort`,
`ls`, `git`, compilers. **`Process.run(program, args)` runs one, waits for
it to finish, and gives back what happened**:

```lume
def main -> () or Error:
  (code, out, err) = Process.run("echo", ["hello from echo"])?
  puts code          #=> 0
  puts out.trim      #=> hello from echo
  puts err.len       #=> 0
```

It gives back three things: the program's **exit code** (lunch 13: `0`
for success), what it wrote to standard output, and what it wrote to
standard error. The output usually ends with a newline, which `trim`
removes.

## 15.2 Tuples: several values at once

Those three values come together as a **tuple**: values side by side, in
parentheses, each of its own type. The type of `Process.run`'s answer is
`(Int, Str, Str) or Error`. You can make tuples yourself:

```lume
def split_bill(total: Int, people: Int) -> (Int, Int) = (total / people, total % people)

def main:
  share = split_bill(50, 4)
  puts share          #=> (12, 2)
  puts share.0        #=> 12
  (each, left) = split_bill(50, 4)
  puts "#{each} each, #{left} over"    #=> 12 each, 2 over
```

`share.0` is the first part, `share.1` the second. Writing names in
parentheses on the left, as `(each, left) = ...` does, takes a tuple apart
in one line — which is what `(code, out, err) = ...` did above. A tuple is
for a few values that belong together for a moment; when they are worth
naming for good, make a struct.

## 15.3 When the program fails

A program that runs and fails is not an `Error` in Lume: it ran, it gave an
exit code that is not `0`, and that is its answer. `sh -c` runs a line of
shell, which makes it easy to show:

```lume
def main -> () or Error:
  (code, out, err) = Process.run("sh", ["-c", "echo working; echo broke >&2; exit 3"])?
  puts code          #=> 3
  puts out.trim      #=> working
  puts err.trim      #=> broke
```

**Only a program that could not be started at all is an `Error`** — one
that does not exist, say:

```lume
def main:
  match Process.run("no-such-program", []):
    Ok(_) -> puts "it ran"
    Error(e) -> puts e.message    #=> cannot run `no-such-program`: No such file or directory (os error 2)
```

The difference matters. `grep` exits with `1` when it finds nothing —
which is an answer, not a failure — and only you know what each program's
codes mean.

## 15.4 Arguments go one by one

The program and its arguments are given apart: `["-r", "file.txt"]`, a list
of separate words. No shell reads them, so a space inside an argument is
just a space, and characters like `*`, `|` and `$` mean nothing special:

```lume
def main -> () or Error:
  (_, out, _) = Process.run("echo", ["two words", "*", "$HOME"])?
  puts out.trim    #=> two words * $HOME
```

That makes it safe to pass a file name or text you were given, whatever it
contains. When you *want* a shell — for a pipe, `|`, or `&&` — run `sh -c`
with the whole line as one argument:

```lume
def main -> () or Error:
  (_, out, _) = Process.run("sh", ["-c", "printf 'pie\\nsoup\\ntea\\n' | sort -r"])?
  puts out.lines    #=> ["tea", "soup", "pie"]
```

## 15.5 Running in a folder

A program runs in the folder your program was started from. **`Process.run_in(dir,
program, args)` runs it in another one**:

```lume
def main -> () or Error:
  dir = "/tmp/lume-lunch-15a"
  Dir.make(dir)?
  File.write("#{dir}/b.txt", "")?
  File.write("#{dir}/a.txt", "")?
  (_, out, _) = Process.run_in(dir, "ls", [])?
  puts out.lines    #=> ["a.txt", "b.txt"]
  Dir.remove(dir)?
```

A folder that does not exist is an `Error`, with a message that says so.

## 15.6 Making failure an `Error`

Often you want "run this, and if it fails, stop": a non-zero exit should
be an `Error` after all. A small function says so once, and lunch 10's `?`
does the rest:

```lume
def run_ok(program: Str, args: [Str]) -> Str or Error:
  (code, out, err) = Process.run(program, args)?
  if code != 0:
    return Error("`#{program}` failed with #{code}: #{err.trim}")
  out

def main:
  puts run_ok("echo", ["fine"])                             #=> Ok("fine\n")
  puts run_ok("sh", ["-c", "echo no tables left >&2; exit 1"])
  #=> Error("`sh` failed with 1: no tables left")
```

A program made of steps like `run_ok(...)?` stops at the first step that
fails, with a message saying which and why — what you want from a build
script or a deploy tool.

## Underneath, in Rust

`Process.run` is Rust's `std::process::Command`, with `output()` to wait
for the program and collect what it wrote. Rust also gives the program
and its arguments apart and uses no shell, which is where Lume's rule
comes from. A tuple is a Rust tuple, and `share.0` is Rust's spelling too.
Inside a task — next lunch — a running program does not hold up the other
tasks while it waits.

## Try it

1. Run `echo` with your name, and print what it printed, trimmed.
2. Write `min_max(xs: [Int]) -> (Int, Int)` and take its answer apart into
   two names. (Use `.or(0)` on `min` and `max`.)
3. Use `sh -c` and `sort` to print `["apple", "banana", "cherry"]` from the
   unsorted text `"cherry\nbanana\napple"`.
4. Use `run_ok` to run `sh -c "exit 7"`, and print the error's message.

## Recap

- `Process.run(program, args)` runs a program and gives `(code, out, err)`,
  or an `Error` if it could not start. `Process.run_in(dir, ...)` runs it
  in a folder.
- A non-zero exit code is an answer, not an `Error`; decide what it means,
  and turn it into an `Error` when that is right.
- A tuple holds a few values side by side: `(a, b)`. `t.0` reads a part;
  `(x, y) = t` takes it apart.
- Arguments are separate words and no shell reads them; `sh -c "..."` when
  you want one.

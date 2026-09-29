# Input and output: `File`, `Dir`, `Path`, `Env`, `Time`, `Process`

> A reviewer asked about `Time`, `File`, `Env`, `Dir` and `Path`: they
> appear in the examples and on no page. This is that page. Every answer
> was checked by running it.

These six are built in. There is nothing to import: `File.read(path)` works
in any file.

**Everything that touches the disk and can fail gives `T or Error`.** The
compiler will not let you ignore it, so each call is followed by `?`, taken
apart with `match`, or dropped on purpose with `_ = ...`. The rules for that
are in [Absence and failure](errors.md). Asking a question (`File.exists?`)
and working on a path as text (`Path`) cannot fail, and give a plain value.

## `File`

| call | gives | |
|---|---|---|
| `File.read(path)` | `Str or Error` | the whole file as text |
| `File.write(path, text)` | `() or Error` | creates the file, or replaces what was there |
| `File.append(path, text)` | `() or Error` | adds to the end; creates the file if it is missing |
| `File.exists?(path)` | `Bool` | true for a file **or a directory** |
| `File.remove(path)` | `() or Error` | deletes one file |
| `File.size(path)` | `Int or Error` | in bytes |
| `File.modified(path)` | `Int or Error` | when it last changed, in seconds since 1970 |

Every `path` and `text` is a `Str`.

```lume
def main -> () or Error:
  dir = "/tmp/lume-docs-io"
  Dir.make(dir)?
  path = Path.join(dir, "notes.txt")

  File.write(path, "one\n")?
  File.append(path, "two\n")?
  text = File.read(path)?
  for line in text.lines:
    puts "> #{line}"            #=> > one
                                #=> > two
  puts File.size(path)?         #=> 8
  puts File.modified(path)? > 0 #=> true
  puts File.exists?(path)       #=> true

  File.remove(path)?
  puts File.exists?(path)       #=> false
  Dir.remove(dir)?
  return ()
```

**`File.read` keeps the file's last newline.** `puts File.read(path)?` on a
file ending in `\n` prints a blank line after it. Go through `.lines`, as
above, or `.trim` it.

## `Dir`

| call | gives | |
|---|---|---|
| `Dir.make(path)` | `() or Error` | makes it and any missing parents; fine if it already exists |
| `Dir.exists?(path)` | `Bool` | true only for a directory |
| `Dir.list(path)` | `[Str] or Error` | the names inside it, files and directories, sorted |
| `Dir.walk(path)` | `[Str] or Error` | every *file* below it, at any depth, as full paths, sorted |
| `Dir.remove(path)` | `() or Error` | deletes it **and everything in it** |

`list` gives bare names; `walk` gives paths you can hand straight to
`File.read`:

```lume
def main -> () or Error:
  root = "/tmp/lume-docs-io"
  Dir.make(Path.join(root, "notes/old"))?
  File.write(Path.join(root, "notes/a.txt"), "a")?
  File.write(Path.join(root, "notes/old/b.md"), "b")?

  puts Dir.list(Path.join(root, "notes"))?  #=> ["a.txt", "old"]
  for path in Dir.walk(root)?:
    puts path     #=> /tmp/lume-docs-io/notes/a.txt
                  #=> /tmp/lume-docs-io/notes/old/b.md

  puts Dir.exists?(Path.join(root, "notes"))        #=> true
  puts Dir.exists?(Path.join(root, "notes/a.txt"))  #=> false
  Dir.remove(root)?
  puts Dir.exists?(root)                            #=> false
  return ()
```

## `Path`

**`Path` works on the text of a path only.** It never looks at the disk, so
nothing in it can fail, and every function gives a `Str`.

| call | gives | for `"/home/ada/notes/todo.txt"` |
|---|---|---|
| `Path.join(a, b)` | `a` and `b` with one `/` between | |
| `Path.dir(p)` | everything before the last `/` | `/home/ada/notes` |
| `Path.base(p)` | everything after the last `/` | `todo.txt` |
| `Path.ext(p)` | the extension, without the dot | `txt` |
| `Path.stem(p)` | the base without its extension | `todo` |

```lume
p = "/home/ada/notes/todo.txt"
puts Path.dir(p)                        #=> /home/ada/notes
puts Path.base(p)                       #=> todo.txt
puts Path.ext(p)                        #=> txt
puts Path.stem(p)                       #=> todo
puts Path.join("/home/ada", "notes")    #=> /home/ada/notes
puts Path.join("/home/ada/", "notes")   #=> /home/ada/notes
```

What is missing comes back as `""`, never as a failure:

```lume
puts "[" + Path.dir("todo.txt") + "]"   #=> []
puts "[" + Path.ext("Makefile") + "]"   #=> []
puts "[" + Path.ext(".bashrc") + "]"    #=> []
puts Path.stem(".bashrc")               #=> .bashrc
puts Path.ext("site.tar.gz")            #=> gz
puts Path.stem("site.tar.gz")           #=> site.tar
```

A leading dot is part of the name, not an extension, so `.bashrc` has none.
Only the last extension counts.

**`Path.join` does not treat an absolute second part specially.**
`Path.join("a", "/etc")` is `a/etc`, not `/etc`, and `..` is kept as it is,
not resolved.

```lume
puts Path.join("a", "/etc")    #=> a/etc
puts Path.join("a", "../b")    #=> a/../b
```

## `Env`

| call | gives | |
|---|---|---|
| `Env.args` | `[Str]` | the program's arguments, without the program's own name |
| `Env.get(name)` | `Str?` | an environment variable, `None` if it is not set |
| `Env.stdin` | `Str or Error` | all of standard input, read to the end |
| `Env.exit(code)` | nothing | ends the program at once with that exit status |

`Env.args` and `Env.stdin` take no arguments, so the parentheses are usually
left off.

`lume run` passes on whatever follows the file name. A `--` before the
arguments is allowed and not passed on, which is how to pass one that starts
with `-`:

```lume-skip
# args.lume
def main:
  puts Env.args
```

```sh
$ lume run args.lume a "b c"
["a", "b c"]
$ lume run args.lume -- -v out.txt
["-v", "out.txt"]
```

`Env.get` gives a `T?`, so say what to use when the variable is not set:

```lume
puts Env.get("LUME_DOCS_NOT_SET")                  #=> None
puts Env.get("LUME_DOCS_NOT_SET").or("default")    #=> default
```

`Env.stdin` waits for the input to end, then gives all of it. For a program
used in a pipe:

```lume-skip
# upper.lume
def main -> () or Error:
  for line in Env.stdin?.lines:
    puts line.upcase
```

```sh
$ printf 'one\ntwo\n' | lume run upper.lume
ONE
TWO
```

**`Env.exit(code)` stops the program where it stands**, with the status you
give. Nothing after it runs:

```lume
puts "before"    #=> before
Env.exit(0)
puts "after"
```

To end with an error from `main`, returning a failure through `?` (below) is
usually clearer than `warn` and `Env.exit(1)`.

**`Env.exit` never comes back, so it can end a branch whose other branches
give a value**, the way Rust's `exit` has the type `!`. That holds for a
`match` or `if` that is a binding's value, and for one that is a function's
result:

```lume
def main:
  n = match "12".to_int:
    Ok(n) -> n
    Error(e) ->
      warn e.message
      Env.exit(2)
  puts n * 2     #=> 24
```

## `Process`

| call | gives | |
|---|---|---|
| `Process.run(program, args)` | `(Int, Str, Str) or Error` | runs it and waits: its exit code, then what it wrote to standard output and to standard error |

**Only failing to start the program is an `Error`.** Its message names the
program and what the system said, as in ``cannot run `no-such-program`: No
such file or directory (os error 2)``. A program that runs and exits with a
non-zero code has still run: that is the code, and what it printed. A program ended by a signal gives `-1`. This is Rust's
`std::process::Command`: the program and its arguments are given apart, and
no shell is involved, so for pipes and `&&` run `sh -c`:

```lume
def main -> () or Error:
  (code, out, _) = Process.run("echo", ["hello"])?
  puts "#{code} #{out.trim}"            #=> 0 hello
  (code2, out2, err) = Process.run("sh", ["-c", "echo one; echo two >&2; exit 3"])?
  puts "#{code2} #{out2.trim} #{err.trim}"    #=> 3 one two
  puts Process.run("no-such-program", []).error?    #=> true
  ()
```

**Inside a task, other tasks go on while one waits for its program.** Three
programs started from three `spawn:` bodies, or from `async def`s they call,
run at the same time. A plain `def` called from a task works too, but it
holds on to its thread while the program runs, so make a helper that runs
programs an `async def`, as `examples/jobr/` does.
`examples/jobr/` is a job runner built on this.

## `Time`

| call | gives | |
|---|---|---|
| `Time.now` | `Int` | seconds since 1970 |
| `Time.now_ms` | `Int` | milliseconds since 1970 |
| `Time.sleep(ms)` | nothing | waits that many **milliseconds** |
| `Time.format(t, pattern)` | `Str` | a time as text, by the directives below |
| `Time.parse(text, pattern)` | `Int or Error` | text back into a time, by the same directives |
| `Time.date(year, month, day)` | `Int or Error` | midnight at the start of that day; an error for a day that does not exist |
| `Time.year(t)`, `month`, `day`, `hour`, `minute`, `second` | `Int` | one part of a time |
| `Time.weekday(t)` | `Int` | 1 for Monday to 7 for Sunday, as ISO 8601 counts |
| `Time.day_of_year(t)` | `Int` | 1 to 366 |

`Time.now` and `Time.now_ms` take no arguments either. The clock is
different on every run, so only a comparison is shown here:

```lume
start = Time.now_ms
Time.sleep(20)
puts Time.now_ms - start >= 20    #=> true
puts Time.now > 0                 #=> true
```

**Pass `Time.sleep` a whole number of milliseconds.** `Time.sleep(1000)` is
one second.

**Inside async code, `Time.sleep` must be awaited.** In an `async def` or a
`spawn:` block it is itself async: write `await Time.sleep(10)`, and other
tasks run while it waits. In a plain `def` it takes no `await` and holds up
the whole thread.

```lume
async def main:
  t = spawn:
    await Time.sleep(10)
    "slept"
  puts await t    #=> slept
```

```lume-bad
async def main:
  Time.sleep(10)
  puts "done"
#! `sleep` is an `async def`: its result arrives later
#! write `await` in front of the call
```

More on this in [Concurrency](concurrency.md).

### Dates

**A time is seconds since 1970, in UTC.** `Time.now` gives one, and the
functions above read it, write it and make one from a date. There are no
time zones: Rust's standard library has none either, and UTC gives the same
answer on every machine. A day is 86400 seconds, so dates are added and
compared as whole numbers.

```lume
def main -> () or Error:
  t = Time.date(2024, 9, 29)? + 8 * 3600 + 53 * 60
  puts Time.format(t, "%Y-%m-%d %H:%M")        #=> 2024-09-29 08:53
  puts Time.format(t, "%A, %d %B %Y")          #=> Sunday, 29 September 2024
  puts Time.weekday(t)                         #=> 7
  due = Time.parse("25 Dec 2024", "%d %b %Y")?
  days = (due - t) / 86400
  puts days                                    #=> 86
  puts Time.date(2023, 2, 29)                  #=> Error("February 2023 has days 1 to 28, not 29")
  ()
```

| directive | writes | | directive | writes |
|---|---|---|---|---|
| `%Y` | `2024` | | `%a` | `Sun` |
| `%m` | `09` | | `%A` | `Sunday` |
| `%d` | `29` | | `%b` | `Sep` |
| `%H` | `08`, 00 to 23 | | `%B` | `September` |
| `%M` | `53` | | `%j` | `273`, the day of the year |
| `%S` | `00` | | `%s` | seconds since 1970 |
| `%%` | `%` | | | |

`Time.parse` reads what `Time.format` writes, and its error says where the
text stopped matching: ``Error("`29/09/2024` does not match `%Y-%m-%d`:
expected `-` at position 2")``. **A pattern written out in the program is
checked when it is compiled**, so `%Q` is an error before anything runs:

```lume-bad
def main:
  puts Time.format(Time.now, "%Y-%Q")
#! `%Q` is not a date directive
```

## When I/O fails

**A failure is an `Error` whose message names the path and gives the
system's reason.** Take it apart with `match` to handle it:

```lume
match File.read("/no/such/file"):
  Ok(text) -> puts text
  Error(e) -> puts e.message    #=> cannot read `/no/such/file`: No such file or directory (os error 2)
```

Or use it as a value, like any other `T or Error`:

```lume
puts File.read("/no/such/file").or("fallback")    #=> fallback
puts File.read("/no/such/file").ok?               #=> false
```

Each function names what it was doing: `cannot read`, `cannot write`,
`cannot append to`, `cannot remove`, `cannot make`, `cannot list`.

**With `?` in a `main` declared `-> () or Error`, a failure ends the
program.** The message goes to standard error, after `error: `, and the exit
status is 1:

```lume-skip
# load.lume
def main -> () or Error:
  text = File.read("/no/such/file")?
  puts text
```

```sh
$ lume run load.lume
error: cannot read `/no/such/file`: No such file or directory (os error 2)
$ echo $?
1
```

**A call that can fail cannot stand on its own line.** Writing
`File.write(path, text)` with nothing to look at the result is refused:

```lume-bad
File.write("/tmp/lume-docs-io/x.txt", "x")
#! `write` can fail, and nothing here looks at whether it did
#! pass the failure on with `?`, handle it with `match`, or say you mean to drop it: `_ = ...`
```

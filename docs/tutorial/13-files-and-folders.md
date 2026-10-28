# Lunch 13: Files and folders

**In this lunch you will:**

- read and write files, and add to the end of one;
- make, list and walk folders;
- take paths apart without touching the disk;
- read a program's arguments and its input, and write to its error output.

This lunch starts Part 2: Lume and the world outside your program. The
world is less tidy than a list — a file may be missing, a folder may be
read-only — so almost everything here may fail, and lunch 10's `?` is
everywhere.

## 13.1 Reading and writing a file

**`File.write` puts text in a file, and `File.read` gives the whole file
back.** Both can fail, so both give a result, and `?` passes a failure on:

```lume
def main -> () or Error:
  Dir.make("/tmp/lume-lunch-13a")?
  path = "/tmp/lume-lunch-13a/menu.txt"
  File.write(path, "soup 4\npie 3\n")?
  text = File.read(path)?
  for line in text.lines:
    puts "> #{line}"    #=> > soup 4
                        #=> > pie 3
  Dir.remove("/tmp/lume-lunch-13a")?
```

`main` says `-> () or Error`, so a file that cannot be written ends the
program with the reason instead of carrying on. The program makes a folder
of its own to work in, and removes it at the end.

`File.write` replaces whatever was in the file. **`File.append` adds to the
end** instead, making the file if it is not there yet — the way to keep a
log:

```lume
def main -> () or Error:
  Dir.make("/tmp/lume-lunch-13b")?
  log = "/tmp/lume-lunch-13b/orders.log"
  File.append(log, "table 4: soup\n")?
  File.append(log, "table 9: pie\n")?
  puts File.read(log)?.lines    #=> ["table 4: soup", "table 9: pie"]
  Dir.remove("/tmp/lume-lunch-13b")?
```

`lines` cuts text at each line ending, so the file's last newline leaves no
empty line behind.

## 13.2 When a file is not there

Reading a file that does not exist is a failure, with a reason that names
the file:

```lume
def main:
  match File.read("/tmp/lume-lunch-13-no-such-file.txt"):
    Ok(text) -> puts text
    Error(_) -> puts "could not read it"    #=> could not read it
```

When a missing file is normal — the first run of a program, say — ask
first with `File.exists?`, which simply answers yes or no:

```lume
def main:
  path = "/tmp/lume-lunch-13-settings.txt"
  setting = if File.exists?(path): File.read(path).or("") else: "default"
  puts setting    #=> default
```

`File` also has `remove`, `size` (in bytes) and `modified` (when the file
last changed, in seconds since 1970).

## 13.3 Folders

**`Dir.make` makes a folder**, and any missing folders above it. **`Dir.list`
gives the names in a folder**, sorted, and **`Dir.walk` gives every file
below it**, at any depth, as full paths you can hand to `File.read`:

```lume
def main -> () or Error:
  root = "/tmp/lume-lunch-13c"
  Dir.make("#{root}/recipes/old")?
  File.write("#{root}/recipes/soup.txt", "boil")?
  File.write("#{root}/recipes/old/stew.txt", "simmer")?
  puts Dir.list("#{root}/recipes")?    #=> ["old", "soup.txt"]
  for path in Dir.walk(root)?:
    puts path    #=> /tmp/lume-lunch-13c/recipes/old/stew.txt
                 #=> /tmp/lume-lunch-13c/recipes/soup.txt
  Dir.remove(root)?
```

**`Dir.remove` removes a folder and everything in it**, so be careful what
you give it. `Dir.exists?` answers whether a folder is there.

## 13.4 Paths

A path is text, and **`Path` takes it apart and puts it together without
looking at the disk** — so nothing in it can fail:

```lume
def main:
  p = "/home/ada/recipes/soup.txt"
  puts Path.dir(p)     #=> /home/ada/recipes
  puts Path.base(p)    #=> soup.txt
  puts Path.stem(p)    #=> soup
  puts Path.ext(p)     #=> txt
  puts Path.join("/home/ada", "recipes")    #=> /home/ada/recipes
```

`Path.join` puts a `/` between its two parts when there is none — but it
does not take one away, so a slash on both sides stays doubled:

```lume
def main:
  puts Path.join("a", "b")      #=> a/b
  puts Path.join("a/", "b")     #=> a/b
  puts Path.join("a", "/b")     #=> a/b
  puts Path.join("a/", "/b")    #=> a//b
```

## 13.5 A program's arguments

The words after the file name in `lume run` are the program's
**arguments**, and `Env.args` gives them as a list:

```lume-skip
# greet.lume
def main:
  for name in Env.args:
    puts "hello, #{name}"
```

```sh
lume run greet.lume Ada Grace
```

```text
hello, Ada
hello, Grace
```

The same works for a program made with `lume build`: `./greet Ada Grace`.

## 13.6 Input, error output and exit codes

**`Env.stdin` reads everything a program is given on its input**, which is
how a program works in a pipe:

```lume-skip
# shout.lume
def main -> () or Error:
  for line in Env.stdin?.lines:
    puts line.upcase
```

```sh
printf 'soup\npie\n' | lume run shout.lume
```

```text
SOUP
PIE
```

Programs have two outputs. `puts` writes to the first, *standard output*,
which is where the program's results go — what a pipe passes on. **`warn`
writes to the second, *standard error*,** for messages about the work rather
than the work itself, so they reach the person even when the results go
into a file.

When a program ends, it tells whoever started it whether it went well, with
an **exit status**: `0` for yes, anything else for no. A `main` that ends
with a failure, as in lunch 10, exits with `1`. **`Env.exit(code)` ends the
program at once with the status you give:**

```lume-skip
# check.lume
def main:
  if Env.args.empty?:
    warn "usage: check <file>..."
    Env.exit(2)
  for path in Env.args:
    puts "#{path}: #{File.exists?(path)}"
```

**`Env.get` reads an environment variable** — settings the shell hands to
every program — and gives a `Str?`, since the variable may not be set:

```lume
def main:
  puts Env.get("LUME_LUNCH_NOT_SET").or("not set")    #=> not set
```

## 13.7 A small tool

All of this together: a to-do list kept in a file. Each run either adds a
line to it or shows it, numbered:

```lume-skip
# todo.lume — `lume run todo.lume add buy bread`, or `lume run todo.lume`
LIST = "todo.txt"

def main -> () or Error:
  args = Env.args
  if args.first == Some("add"):
    item = args.skip(1).join(" ")
    File.append(LIST, item + "\n")?
    puts "added: #{item}"
  elif not File.exists?(LIST):
    puts "nothing to do"
  else:
    for i, line in File.read(LIST)?.lines.enumerate:
      puts "#{i + 1}. #{line}"
```

```sh
lume run todo.lume add buy bread
lume run todo.lume add book a table
lume run todo.lume
```

```text
added: buy bread
added: book a table
1. buy bread
2. book a table
```

## Underneath, in Rust

`File`, `Dir` and `Path` are thin wrappers over Rust's `std::fs` and
`std::path`, with Rust's I/O errors turned into Lume `Error`s whose message
says what was being done and to which path. `File.read` is
`fs::read_to_string`, and `Dir.make` is `fs::create_dir_all`. `puts` and
`warn` are `println!` and `eprintln!`, and `Env.exit` is
`std::process::exit`.

## Try it

1. Write three lines to a file in a folder under `/tmp`, read it back, and
   print how many lines it has. Remove the folder at the end.
2. Append one more line, and print the file's last line. (`lines.last`
   gives a `Str?`.)
3. For the path `/srv/cafe/menu.v2.json`, print its folder, its name, its
   extension and its name without the extension.
4. Write a program that prints the longest of its arguments, or uses `warn`
   and exits with status 2 when it has none.

## Recap

- `File.read`, `File.write` and `File.append` read and write text; each may
  fail, so each is followed by `?` or taken apart with `match`.
  `File.exists?` asks first.
- `Dir.make`, `Dir.list`, `Dir.walk` and `Dir.remove` work on folders.
- `Path` takes paths apart and joins them, without touching the disk.
- `Env.args` gives the arguments, `Env.stdin` the input, `Env.get` an
  environment variable.
- `puts` writes results; `warn` writes messages to standard error;
  `Env.exit(code)` ends with an exit status.

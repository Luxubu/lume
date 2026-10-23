# Lunch 18: Programs alongside

> **A 0.5 preview.** This lunch teaches one of Lume's newest parts. It works
> as shown — every example is checked — but details may change before 1.0.

**In this lunch you will:**

- start a program and carry on while it runs;
- read what it prints as it prints it;
- write to it while it runs, and talk with it in turns;
- read output whose length is given in a header, as many protocols do.

## 18.1 Starting a program and carrying on

Lunch 15's `Process.run` waits for a program to finish and then gives you
everything at once. That is fine for `echo`; it is not for a program that
runs for an hour printing progress, or one you want to talk to. **`Process.spawn`
starts the program and gives you a `Child` at once**, a handle on the
running program:

```lume
async def main -> () or Error:
  c = Process.spawn("sh", ["-c", "echo one; sleep 0.05; echo two"])?
  for line in c.lines:
    puts "got #{line}"    #=> got one
                          #=> got two
  puts "exit #{await c.wait}"    #=> exit 0
```

**`c.lines` is the program's output as an async sequence** — lunch 17's
kind — so each line arrives as the program prints it: `got one` appears
before the program has even printed `two`. **`await c.wait` waits for the
program to end** and gives its exit code. Both wait, so a program that uses
them has `async def main`.

Starting can fail — a program that does not exist — so `Process.spawn`
gives `Child or Error`, with the same message as `Process.run`.
`Process.spawn_in(dir, program, args)` starts it in a folder.

## 18.2 Giving it input

A `Child` can be written to while it runs. **`c.write(text)` sends text to
its input, and `c.close_input` says there is no more** — which a program
that reads to the end, like `sort`, is waiting for:

```lume
async def main -> () or Error:
  s = Process.spawn("sort", [])?
  s.write("stew\nsoup\npie\n")?
  s.close_input
  puts s.lines.to_list    #=> ["pie", "soup", "stew"]
  puts await s.wait       #=> 0
```

`await c.wait` closes the input too, before it waits, so a program reading
to the end can finish.

## 18.3 Talking in turns

Some programs answer each line as it comes. **`await c.read_line` reads one
line of output** — the next answer — so you can send a question, read the
answer, and send the next:

```lume
async def ask(c: Child, question: Str) -> Str or Error:
  c.write(question + "\n")?
  (await c.read_line).or_error("no answer")

async def main -> () or Error:
  counter = Process.spawn("sh", ["-c", "n=0; while read dish; do n=$((n + 1)); echo \"order $n: $dish\"; done"])?
  puts await ask(counter, "soup")?    #=> order 1: soup
  puts await ask(counter, "pie")?     #=> order 2: pie
  puts await counter.wait             #=> 0
```

The little shell program counts the lines it is sent and answers each one.
`read_line` gives `None` once the program's output has ended, which
`or_error` (lunch 10) turns into a failure.

## 18.4 Its errors, its end, and stopping it

**`c.error_lines` is what the program writes to standard error**, as
`c.lines` is its standard output; the two are read apart, so a program that
writes both never gets stuck waiting for you to read one of them. A program
that ends badly gives its exit code as any other:

```lume
async def main -> () or Error:
  c = Process.spawn("sh", ["-c", "echo cooking; echo out of flour >&2; exit 3"])?
  puts c.lines.to_list          #=> ["cooking"]
  puts c.error_lines.to_list    #=> ["out of flour"]
  puts await c.wait             #=> 3
```

**`c.kill` stops a program** that need not run any longer. Its `wait` then
gives `-1`, the code for a program ended by a signal:

```lume
async def main -> () or Error:
  slow = Process.spawn("sleep", ["30"])?
  slow.kill
  puts await slow.wait    #=> -1
```

## 18.5 Reading by size

Many protocols send a message as a header that says how long the message
is, then the message itself, which may hold anything — even new lines.
Language servers, which editors talk to, work this way. **`await c.read(n)`
reads exactly the next `n` bytes** of output, so a header read with
`read_line` and a body read with `read(n)` take such messages apart:

```lume
async def main -> () or Error:
  m = Process.spawn("printf", ["Length: 4\n\nsoupLength: 3\n\npie"])?
  var bodies: [Str] = []
  while true:
    match await m.read_line:
      Some(header) ->
        size = header.split(": ").at(1).to_int.or(0)
        _ = await m.read_line
        bodies.push((await m.read(size)).or(""))
      None -> break
  puts bodies    #=> ["soup", "pie"]
```

Each message is a `Length:` line, an empty line, and the body with no line
ending after it — so the next header starts straight after. Lume's own
test suite drives `lume lsp`, its language server, exactly this way.

## 18.6 Several at once

A `Child` is a value like any other: it can go in a list, in a field, or
into a task. Here three programs run at once, each walked by a task of its
own:

```lume
async def main -> () or Error:
  var running: [Child] = []
  for n in 1..3:
    running.push(Process.spawn("sh", ["-c", "echo #{n * n}"])?)
  results = await running.map { |c| spawn: c.lines.first.or("") }.to_list
  puts results    #=> ["1", "4", "9"]
```

## Underneath, in Rust

`Process.spawn` is Rust's `std::process::Command` with `spawn()`, as
`tokio::process::Child` offers it. A thread of its own reads each of the
program's outputs as it comes, so the program is never held up because
nobody is reading — the classic way for a program to hang on its child. A
`Child` is cheap to copy and share; every copy is the same running program.

## Try it

1. Start `sh -c "echo a; echo b; echo c"`, and print its lines in capitals
   with `c.lines.map(_.upcase).to_list`.
2. Send `tr a-z A-Z` the text `"quiet please\n"`, close its input, and print
   what comes back.
3. Start a `sh` loop that answers each line with its length
   (`while read x; do echo ${#x}; done`), and ask it about `"soup"` and
   `"minestrone"`.
4. Start `sh -c "exit 5"` and print its exit code.

## Recap

- `Process.spawn(program, args)` starts a program and gives a `Child` at
  once; `Process.spawn_in` starts it in a folder.
- `c.lines` and `c.error_lines` are its outputs as async sequences;
  `await c.read_line` and `await c.read(n)` read a line or a number of
  bytes.
- `c.write(text)` sends input; `c.close_input` ends it.
- `await c.wait` gives its exit code; `c.kill` stops it.
- A `Child` can go in lists and fields and to tasks.

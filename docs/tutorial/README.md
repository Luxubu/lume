# Lume, a Lunch at a Time

A tutorial in twenty-six short chapters, each meant to be read — and its
examples typed and run — in about an hour: a lunch break. Each one teaches
one idea, with many small programs, a few things to try, and a recap at the
end.

**Every example in these chapters is run by Lume's test suite**, and its
output checked against what the page shows. If a release changes how
something works, the suite fails until the page is fixed, so what you read
here is what Lume does — for the version below.

Written for **Lume 1**. You need Lume installed; [install.md](../install.md)
says how. No Rust is needed to follow along, though each chapter ends with a
short *Underneath, in Rust* box for readers who know Rust or are curious about
it.

## Part 1: the language

| | lunch | what you will be able to do |
|---|---|---|
| 1 | [Hello, Lume](01-hello.md) | write a program, run it, and read what the compiler says when something is wrong |
| 2 | [Names and values](02-names-and-values.md) | name values, change the ones that change, and know what kind of value each is |
| 3 | [Numbers and text](03-numbers-and-text.md) | do arithmetic, and take text apart and put it together |
| 4 | [Functions](04-functions.md) | give a piece of work a name, and hand it what it needs |
| 5 | [Making decisions](05-making-decisions.md) | `if`, `elif`, `unless`, and loops with `for` and `while` |
| 6 | [Lists, maps and sets](06-lists-maps-and-sets.md) | keep many values, find them, and change them |
| 7 | [Blocks and chains](07-blocks-and-chains.md) | `map`, `filter` and their friends; hand a piece of code to another |
| 8 | [Structs](08-structs.md) | make types of your own, with fields and methods |
| 9 | [Enums and `match`](09-enums-and-match.md) | say "one of these", and handle each case |
| 10 | [Absence and failure](10-absence-and-failure.md) | `T?`, `T or Error`, and `?` — values that may not be there, and work that may fail |
| 11 | [Interfaces](11-interfaces.md) | say what a value can do, not what it is |
| 12 | [Generics](12-generics.md) | write a thing once, for values of many types |
| 13 | [Modules and packages](13-modules-and-packages.md) | split a program into files, and use a library of your own |
| 14 | [Tests](14-tests.md) | check that the code does what it should, with `lume test` |

## Part 2: the world

| | lunch | |
|---|---|---|
| 15 | [Files and folders](15-files-and-folders.md) | read, write and walk them |
| 16 | [Generators](16-generators.md) | sequences made a piece at a time, with `yield` |
| 17 | [Running other programs](17-running-other-programs.md) | `Process.run`, and what comes back |
| 18 | [Tasks](18-tasks.md) | many things at once, with `spawn:` and `await`; `shared var` |
| 19 | [Async sequences](19-async-sequences.md) | items that arrive later: async generators and their methods |
| 20 | [Programs alongside](20-programs-alongside.md) | `Process.spawn`: talk to a program while it runs |
| 21 | [HTTP](21-http.md) | requests, and bodies read as they arrive |

Every chapter teaches parts of Lume that are settled ([what is
settled](../stability.md) says what that promises). Lunch 26 prints what a
language server answers, whose words may change, as no message's do.

## Part 3: building things

Each lunch builds one small program from start to finish, using what the
first two parts taught, and a few methods met for the first time, each
marked *new here* where it is used.

| | lunch | |
|---|---|---|
| 22 | [A CSV query tool](22-a-csv-query-tool.md) | reading a table, and answering questions about it |
| 23 | [A log analyser](23-a-log-analyser.md) | a large file, read fast |
| 24 | [A job runner](24-a-job-runner.md) | programs run in tasks, and their results gathered |
| 25 | [A health checker](25-a-health-checker.md) | many web pages at once, with retries |
| 26 | [Talking to a language server](26-talking-to-a-language-server.md) | a program that drives another, message by message |

## How the chapters are written

- **One idea per lunch**, in sections short enough to read between bites.
- **Every program is complete and runs as shown.** A line ending in `#=>`
  shows what that line prints. Type the examples rather than pasting them;
  the typos are part of the lesson.
- **Mistakes on purpose.** Some examples are wrong, marked so, with the
  message Lume gives. Reading those messages is a skill the chapters teach.
- **Try it** at the end of each lunch: a few small tasks. The answers are in
  [answers.md](answers.md), and they are checked by the suite too.
- **Underneath, in Rust**: what Lume made of the chapter's ideas. Skip it if
  you like; nothing later depends on it.

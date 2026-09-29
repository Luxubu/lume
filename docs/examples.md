# The examples, in an order

There are about 5,000 lines of working Lume in `examples/`, and every one of
them is run by the test suite on every change, so none of it is stale. This is
the order to read it in rather than the order the directory lists it.

Run any of them:

```sh
lume run examples/fib.lume
```

## Start here

| | |
|---|---|
| `fib.lume` | the whole language in fifteen lines |
| `structs.lume` | fields, methods, constructors, `var self` |
| `enums.lume` | variants with data, `match`, `T?` |
| `loops.lume` | `for`, `while`, ranges, `where` |
| `printing.lume` | what every kind of value prints — the reference as a program |

## The everyday surface

| | |
|---|---|
| `stdlib.lume` | every built-in method, one line each |
| `collections.lume` | a word index in a map of sets, tuples, interface defaults |
| `sets.lume` | sets, set algebra, struct keys, character slicing |
| `chars.lume` | `Char`: roman numerals, an evaluator, character tables |
| `blocks.lume` | the three block forms and lazy chains |
| `pipes.lume` | `\|>` |
| `errors_and_results.lume` | `T or Error`, `?`, and what fails |
| `patterns.lume` | `\|` alternatives and what exhaustiveness catches |
| `shadow.lume` | rebinding a name to a new value of itself — `name = name.trim` — which is allowed at the same level, and is the only rebinding that is |
| `ownership.lume` | copy or move by analysis, recursive enums |

## Building your own abstractions

| | |
|---|---|
| `interfaces.lume` | interfaces, `extend`, operator methods |
| `generics.lume` | generic functions, structs and enums; bounds |
| `generic_interfaces.lume` | interfaces that take type parameters |
| `generic_extend.lume` | one interface over lists, sets, maps and your own type |
| `blocks_of_your_own.lume` | functions that take behaviour: `retry`, `time_it` |

## Programs, not snippets

These are whole programs written to find out what the language was missing.
Each one has a `FINDINGS.md` or a `README.md` saying what it exposed.

| | |
|---|---|
| `json/` | a JSON parser, printer and query tool (350 lines) with its own tests, plus the same algorithm in Rust and Python |
| `mini/` | a small language in four modules: lexer, parser, evaluator, driver |
| `site/` | a static site generator: Markdown in, an HTML tree out, incremental |
| `lib/` | a library written in Lume — `seq` and `table` — and a report program using both |
| `match/` | a concurrent record matcher: block, fan out across tasks, collect, report |
| `port/` | the design document's sample programs, plus `app_async.lume` on threads |
| `jobr/` | a job runner in two packages: waves of jobs run in tasks, retries, skips, a `shared var` tally, `Process.run` |
| `http_demo/` | the `http` package against a local server: JSON, a 404, a POST echoed back, no connection |
| `csvq/` | a CSV query tool in three packages — `csv`, `table` and the program — with a small query language, stdin, and each package's own tests |

## Modules

| | |
|---|---|
| `modules/` | a three-file program: `app.lume` imports `users/model` and `users/store` |
| `travel/` | a library that ships its conformances; the consumer writes no `extend` |
| `shipping/` | `pub import`: a library passes its own dependency on to its consumers |
| `packages/` | three packages: a program over two libraries, one of which uses the other |

## Async

| | |
|---|---|
| `async.lume` | `async def`, `spawn:`, `Task[T]`, `shared var` |
| `match/` | the same ideas in a real program |
| `port/app_async.lume` | one handler per request, sharing one store |

## The Rust bridge

| | |
|---|---|
| `crate.lume` | `regex` through the bridge: types, iterators, errors, a `rust:` block |
| `crates.lume` | `hex` and `urlencoding`, with nothing written for them |

## Tests and formatting

| | |
|---|---|
| `tests/` | files with `test` blocks, and the `lume test` output they must produce |
| `fmt/` | badly spaced input, and what `lume fmt` makes of it |
| `errors/` | 106 programs that must keep failing, each with the message it must give |

`examples/errors/` is worth a look even though nothing in it runs: it is the
catalogue of what Lume refuses and what it says when it does.

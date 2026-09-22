# Lume

Ruby's feel and Nim's terseness on the surface, Gleam's discipline in the rules,
Rust's memory model and speed underneath.

```ruby
def fib(n: Int) -> Int:
  if n < 2: n
  else: fib(n - 1) + fib(n - 2)

def main:
  for i in 0..10:
    puts "fib(#{i}) = #{fib(i)}"
```

Lume compiles to Rust, and `rustc` compiles that to a native binary. No garbage
collector, no interpreter, no runtime. `fib(35)` runs in about 30 ms; the same
program takes about 1 s in Python and 1.4 s in Ruby.

The language design lives in the [Lume Language Design](https://claude.ai/code/artifact/1872fe63-1816-4b1e-9983-dec2774382fb)
document. This repository is the compiler and the examples.

## Start here

```sh
./install.sh                    # builds the compiler, puts `lume` on your PATH
lume run examples/fib.lume
```

Then:

- **[A tour of Lume](docs/tour.md)** — the language in half an hour.
- **[The reference](docs/README.md)** — how a value prints, what `pad` fills,
  which methods a list has, the precedence table.
- **[The examples](docs/examples.md)** — 5,000 lines of working programs, in a
  reading order.
- **[Installing](docs/install.md)** — and what each command does.

Every example in the documentation is compiled, run, and checked against the
output it claims, by `tests/docs.sh`. An answer that stops being true fails
the build.

## What it is

Lume is a compiled language. Indentation makes blocks, types are inferred and
written where they help, and there is no garbage collector and no runtime —
ownership is worked out at compile time and the output is a native binary.

```ruby
struct Record:
  name: Str
  zip: Str

def main:
  rows = [Record(name: "Ada", zip: "90210"), Record(name: "Bo", zip: "10001")]
  by_zip = rows.group_by { |r| r.zip }
  for zip, people in by_zip:
    puts "#{zip}: #{people.map(_.name).join(", ")}"
```

It is sound in a way that is meant to be load-bearing: 283 programs written by
twenty-one independent reviewers who had never seen the compiler, and not one of
them produced a wrong answer or failed to compile after type-checking. What it
refuses, it refuses with a message that names the fix — `examples/errors/` is
115 programs kept around to prove it.

**Where it is.** Version 0.1. The language is settled enough to write real
programs in, and there is no editor support, no package manager, and no way to
depend on Lume code you did not write — imports resolve relative to your own
files. Rust crates are reachable today through `import rust.<crate>`, which is
most of what a young standard library would otherwise be for.

[HISTORY.md](HISTORY.md) is the record of how it got here, milestone by
milestone, and the [design document](https://claude.ai/code/artifact/1872fe63-1816-4b1e-9983-dec2774382fb)
is the reasoning behind the decisions.

## Speed

"As fast as Rust" is a claim, so it is measured. Each benchmark is a Lume
program and the Rust a person would have written for the same job, timing the
work itself rather than the process start or the file read.

| benchmark | Lume | Rust | |
| --- | --- | --- | --- |
| generic helpers taking blocks, over 2M ints | 19 ms | 19 ms | 100% |
| 4M interface values, dispatched in a loop | 22 ms | 21 ms | 104% |
| counting a million words into a `{Str: Int}` | 41 ms | 36 ms | 113% |
| building 200k lines with interpolation and `join` | 28 ms | 28 ms | 100% |
| printing 200k lines | 87 ms | 84 ms | 103% |

The one above parity is word count, and the reason is a deliberate trade:
Lume's map keeps insertion order, which Rust's `HashMap` does not, so a
lookup reads a key-to-position index and then the ordered entries. That is
one hash and one compare, the same as Rust's, plus one indirection.

```sh
tests/bench.sh            # build both sides, best of five, compare to bench/BASELINE
tests/bench.sh --update   # accept the current ratios as the new budget
```

The guard is the *ratio*, not the millisecond count, so it says the same
thing on a slower machine; a benchmark fails when it is more than 25% over
its number in `bench/BASELINE`. `bench/gen.py` makes the inputs, so nothing
large is checked in.

Two things worth knowing when you measure your own program:

- **Benchmark what `lume build` produces, not what `lume run` does.**
  `lume run` builds your program at `opt-level = 1` on top of fully
  optimised dependencies, which is what makes edit-and-run take 0.2 s.
  `lume build` is the real thing. On word count the difference is 11 ms
  against 9 ms.
- **Overflow still stops the program in a built binary.** That is milestone
  15's rule and it is not traded away for speed.


## Tests

```sh
tests/run.sh      # every example's output and every error message, diffed
tests/docs.sh     # every example in docs/, run and checked against its claims
tests/corpus.sh   # the 283 independently-written review programs
tests/bench.sh    # the benchmarks, against their budget
```

The suite also runs `lume test` on `examples/tests/`, `lume fmt` on
`examples/fmt/`, checks that formatting every example is idempotent and leaves
the generated Rust unchanged, and runs the review corpora (programs written by
independent reviewers who did not know the compiler).
`tests/corpus.sh` classifies the corpus (283 programs from seven review
rounds: `corpus/` and `corpus/edge/` after milestone 14, `corpus/m17/` on the
pattern and standard-method surface, `corpus/m18/` on sets and slicing,
`corpus/m26/` on generics, parameterised interfaces and blocks, `corpus/m28/`
on generic `extend`, the seams between features, and whole programs, and
`corpus/m30/` on modules — thirty multi-file programs, each a directory with
its own `main.lume` — and `corpus/m36/` on the documentation, thirty programs
written from `docs/` alone): 217
run, 66 stop with a Lume error (each a deliberate rule or a deliberate error
test: no shadowing, no `Float / Int`, an unbounded `T` as a map key, ...),
none leaks a rustc error. Each round has a `REPORT.md` beside its programs.

Tests in a Lume file:

```ruby
test "parse_user reads a valid line":
  u = parse_user("Ana, 31")!
  assert u.age == 31
```

```
$ lume test examples/tests/parse.lume
test parse_user reads a valid line ... ok
test this one fails on purpose ... FAILED
    parse.lume:37: assert u.age == 41
      left:  40
      right: 41

5 tests: 4 passed, 1 failed
```

Ownership: `Int`/`Float`/`Bool` pass by value; `Str`, lists, maps and structs
are passed by reference and copied only where an owned value is needed. Since
milestone 16 that decision is an analysis, not a guess: when a value is given
away (stored in a list, bound to another name, returned) the compiler copies
it if the name is used again later in the function, or inside a loop or block
that will run again, and moves it if not. `us = [u]; puts u` works, and `kept
= big` with no later use of `big` costs nothing. The programmer writes
mutability, never ownership: `var self` on a method that changes fields, `var
xs: [Int]` on a parameter changed in place, `for var a in xs` to change the
items of a list, and `xs[i].method(...)` / `xs[i].field = v` to reach one
item (a bare `xs[i]` as a value is a `T?`, since the position may be out of
range; the method and field forms reach the item directly and stop the
program if it is missing). A recursive `enum Tree: Leaf / Node(left: Tree, ...)` works as written;
the `Box` Rust needs is emitted for you (a nested pattern on such a field is
an error that says to `match` the field in the arm). Milestone 7
measured zero ownership syntax across 202 lines of ported programs
(`examples/port/`), so the memory policy is settled: Rust-faithful inferred
ownership, no reference-counting fallback. Milestone 12 measured the last
case, values shared between threads: 5 `shared` words in 97 lines of async
code, all of them at the declaration of the shared thing (`shared var
store`), none at its uses. That is the one ownership word in Lume.

Speed: computation runs at Rust speed (`fib(35)`: 56 ms with Lume's always-on
integer overflow checks, 27 ms without them; Python 1.04 s). Overflow, division
by zero and an out-of-range position stop the program with a one-line Lume
message (`error: Int overflow in `+``), never a silent wrap and never a Rust
trace unless `LUME_BACKTRACE=1` is set.
String-and-map code is within 20% of hand-written Rust after milestone 8
(word count over 360k words: hand-written Rust 19 ms, Lume 23 ms, Python
79 ms). What made the difference: `split`/`lines` are lazy and yield string
slices, `m[k] = m[k].or(0) + 1` compiles to one entry lookup, a key that is
not used afterwards is moved rather than cloned, and `{K: V}` is an
insertion-ordered hash map (Ruby's order, hash speed) written in the prelude.
Parsing, after milestone 19: `examples/json/json.lume` parses 867 KB of JSON
in 13 ms and prints it in 21 ms, against 10 ms and 16 ms for the same
algorithm written by hand in Rust, and 135 ms / 76 ms for the same algorithm
in Python. Before `Char` existed the Lume version took 100 ms, because
`s.chars` was a list of one-character heap strings — `examples/json/bench.sh`
reproduces all of it, including a Rust build with that handicap put back.


## Layout

```
compiler/src/lexer.rs    tokens, INDENT/DEDENT, string interpolation
compiler/src/parser.rs   hand-written recursive descent; all syntax errors
compiler/src/ast.rs      the tree
compiler/src/codegen.rs  Rust emission plus name/mutability checks
compiler/src/error.rs    error rendering (line, caret, help)
compiler/src/loader.rs   resolves imports to files, orders modules, rejects cycles
compiler/src/fmt.rs      lume fmt: prints the tree back out, with comments and blank lines
compiler/src/bridge.rs   crate signatures: rustdoc JSON -> Lume types and conversions
compiler/src/main.rs     the CLI
examples/                programs that must keep compiling
examples/port/           the design doc's sample programs and the graph program; app_async.lume is sample 3 on threads
examples/async.lume      async/await, spawn, Task[T], shared var
examples/ownership.lume  copy or move by analysis, a recursive enum, for var, xs[i].method
examples/patterns.lume   `|` alternatives and what the exhaustiveness check catches
examples/sets.lume       sets, set algebra, struct keys, character slicing
examples/collections.lume  a word index in a map of sets, tuple destructuring, interface defaults
examples/chars.lume      Char: roman numerals, an expression evaluator, character tables
examples/json/           a JSON parser, printer and query tool (350 lines) with its own tests,
                         plus the same algorithm in Rust and Python and a benchmark script
examples/mini/           a small language in four modules: lexer, parser, evaluator, driver,
                         with the same interpreter in Python beside it
examples/site/           a static site generator: Markdown in, an HTML tree out, incremental
examples/stdlib.lume     the built-in methods, one line each
examples/crate.lume      regex through the bridge: types, iterators, errors, a rust: block
examples/crates.lume     hex and urlencoding, with nothing written for them
examples/modules/        a three-file program: app.lume imports users/model and users/store
examples/interfaces.lume interfaces, extend, operator methods
examples/generics.lume   generic functions, structs and enums; Ordered, Hashable, interface bounds
examples/blocks_of_your_own.lume  functions that take behaviour: each, map, keep, fold, retry, time_it
examples/generic_interfaces.lume  interfaces with type parameters: bounds, defaults, values, extend
examples/generic_extend.lume      generic `extend`: lists, sets, maps and your own generic type under one interface
examples/travel/                  a library that ships its conformances, and a consumer that writes no `extend`
examples/shipping/                `pub import`: a library passes its own dependency on to its consumers
examples/printing.lume            how every kind of value prints, as a runnable table
examples/match/          a concurrent record matcher: block, fan out across tasks,
                         collect, report — and FINDINGS.md, what writing it exposed
examples/lib/            a library written in Lume: seq (generic helpers), table (typed CSV),
                         a report program that uses both, and what writing it exposed
bench/                   five benchmarks, each with the Rust a person would have written
                         beside it; gen.py makes the inputs, BASELINE holds the budget
docs/                    the manual: a tour, a reference, and the questions
                         six review rounds could not answer (QUESTIONS.md)
install.sh               builds the compiler and puts `lume` on your PATH
examples/tests/          files with `test` blocks; expected `lume test` output
examples/fmt/            badly spaced input; expected `lume fmt` output
examples/errors/         programs that must keep failing, with good messages
```

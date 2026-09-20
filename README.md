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

## Status: milestones 1–14 done

| # | Milestone | Status |
| --- | --- | --- |
| 1 | Lexer with indentation, parser, `fib` transpiles to Rust and runs | done |
| 2 | `struct` with methods, bare field access, keyword constructors | done |
| 3 | `_` shorthand, inline blocks, `do` blocks, `for ... where` | done |
| 4 | `enum`, `match` with exhaustiveness errors, tuples, `T?` | done |
| 5 | `T or E`, `Error`, `?` on Result, implicit `Ok`, `!`, `File`/`Env` | done |
| 6 | `\|>`, `import rust.<crate>` + `rust:` blocks | done |
| 7 | Port the sample programs plus a graph program; settle the memory policy | done: Rust-faithful ownership, no ORC |
| 8 | Stdlib speed: ordered hash map, entry updates, lazy `split`/`lines`, key liveness | done: 74 ms → 23 ms |
| 9 | Lume modules: `import users.model`, `model.User`, `import a.b.Name`, `pub`, cycle detection | done |
| 10 | `interface` with defaults, structural conformance, `extend T with I`, operator methods | done |
| 11 | `test "name":` blocks, `assert`, `lume test`, `lume fmt` | done |
| 12 | `async def`/`await`, `spawn:` tasks, `Task[T]`, `shared`/`shared var` | done: sample 3 runs on threads, 5 `shared` words in 97 lines |
| 13 | Rust bridge phase 2: crate signatures from rustdoc JSON; `regex.Regex.new(p)` with no bindings | done |
| 14 | Compile time: incremental rustc, one shared cargo cache per machine, skip when unchanged | done: edit-and-run 0.2 s plain, 0.7–0.9 s with crates |

What works today: functions (`def` block and one-liner forms, return type
inferred when omitted), `struct` with fields and methods (fields used bare
inside methods, `var self` for methods that change them, positional and
keyword constructors, `p.x = v`, `==` and printing derived), `Int`/`Float`/
`Bool`/`Str`/`[T]`, immutable bindings and `var` (with `x = x.trim`
self-transform rebinding), blocks in three forms (`xs.map(_.name)`,
`xs.map { |x| x * 2 }`, `xs.each do |x|` with an indented body) on `map`,
`filter`, `reject`, `each`, `sum`, `count`, `any?`, `all?`, `sort_by`, plus
`take`, `skip`, `to_list` — chains are lazy and compile to one fused Rust
iterator, `find`/`take_while`/`fold`/`min_by`/`max_by`/`enumerate`, `enum`
with data and methods (`Shape.Circle(1.0)`, bare `Circle(1.0)` where
unambiguous), `match` as an expression with variant, literal, range, tuple,
list (`[]`, `[x]`, `[first, ..rest]`) and string patterns, guards, and a
Lume-level error listing the cases a `match` misses, tuples (`(1, "a")`,
`t.0`, `for i, x in xs.enumerate`), optional values (`T?`, `Some`/`None`,
`.or(default)`, `?` early return in a function returning `T?`; `first`,
`last`, `find`, `max`, `min`, `pop` all return `T?`), errors as values
(`T or E`, the built-in `Error("message")` with `.message`, `?` passes the
error up, a bare value in a `T or E` function is `Ok` and an `Error(...)` is
`Err` automatically, `match` on `Ok(x)`/`Error(e)`, `.or(default)`, `.ok?`,
`.error?`, `.or_error("msg")` to turn a `T?` into a `T or Error`, `!` to
unwrap-or-stop with a warning, `Str.to_int`/`to_float` return `T or Error`),
typed bindings (`var xs: [User] = []`), `def main -> () or Error`,
`File.read`/`File.write`/`File.exists?`, `Env.args`/`Env.get`, the pipe
(`x |> .method`, `x |> f(y)` is `f(x, y)`, `x |> puts`; lines starting with
`|>` continue the expression), function names as blocks (`xs.map(parse)`),
the Rust bridge (`import rust.regex = "1"` adds a cargo dependency and reads
the crate's signatures, so `regex.Regex.new(p)`, `re.find_iter(text)`,
`hex.encode(s)` are typed Lume calls with nothing to declare: `&str`/`String`/
`Cow<str>` are `Str`, integer widths are `Int`, `Option` is `T?`, `Result<T,
E>` is `T or Error` with the crate's error shown as text, `Vec`/`&[T]` are
`[T]`, a crate struct is an opaque type usable in signatures and fields, a
crate iterator is a lazy chain, `T: AsRef<str>` parameters take a `Str`;
`lume crate file.lume regex` lists what the crate offers in Lume types and
says why the rest is not callable yet; a `rust:` block inside a Lume function
is Rust with the parameters in scope, for those corners), Lume modules (`import users.model`
loads `users/model.lume`; `model.User`, `model.parse(x)`, `model.Role.Guest(7)`
in expressions, types and patterns; `import users.model.User` for one name;
`as` to rename; only `pub` items cross a file boundary), interfaces
(`interface Shape:` lists required method signatures and default methods
with bodies; a type conforms by having the methods, with nothing to declare;
`def describe(s: Shape)` is a generic function, `[Shape]` holds mixed types
behind a pointer and says so once; `extend Str with Shape:` adds the methods
to a type you do not own; `pub interface` crosses modules), operator methods
(`def +(o: Point)`, `-`, `*`, `/`, `%`, `==`, `<`; `!=` follows from `==`,
and `<=`, `>`, `>=`, `sort`, `max`, `min` follow from `<`), tests in the
file they test (`test "name":` blocks with `assert`; `lume test` runs them
and a failing `assert` prints both sides; `lume run`/`build` strip them; `!`
is silent inside tests), `lume fmt` (one canonical layout, no options; keeps
comments, blank lines, `x |> y` pipes, one-liner/inline forms and literal
spelling; aligns `->` in a `match` and trailing comments), async
(`async def f` is called with `await f()`; `t = spawn:` runs an indented
block as its own task on a thread pool and gives a `Task[T]`; `await t`,
`await [t1, t2]`; `await Time.sleep(ms)`; `async def main`; every local a
task mentions is copied into it), sharing (`shared var x = v` puts one value
behind a lock that any task may change: `x.push(1)`, `x += 1`, `x.count`;
`shared x` is a read-only handle; a struct field can be `shared var Store`;
uses are wrapped in short locks, arguments are computed before the lock, and
a block that would take the lock twice is a compile error), `if`/`elif`/`else` as
expressions, trailing `if`/`unless`, `while`, `for x in range` with `where`,
lists, ranges (`1..10` inclusive, `1...10` exclusive), string interpolation,
`and`/`or`/`not`, `**`, calls, a small set of built-in methods (`len`, `sum`,
`sort`, `first`, `last`, `upcase`, `trim`, `sqrt`, ...).

Errors are Lume errors, not rustc errors: unknown names, fields and types (with
a "did you mean"), assignment to an immutable binding (pointing at where it was
declared), changing a field from a method without `var self`, calling a
mutating method on an immutable value, wrong or missing arguments and keywords,
`if` used as a value without `else`, non-exhaustive `match`, `?` in a function
that cannot return `None` or an error (with the right fix for each mismatch),
a method or arithmetic on a `T?` or `T or E` without unwrapping, an empty
`[]` binding with no type, a value used as an interface it does not satisfy
(naming the missing method or the signature that differs), an `extend` that
leaves a method out, an operator a type does not define, `sort` on a type
without `<`, `await` outside `async def`, an `async def` called
without `await`, a `var` changed inside a `spawn:` block (it is a copy), a
mutating call on a read-only `shared`, a `spawn:` inside a method that uses
`self`, a crate function or method that does not exist (or is not callable
from Lume yet, with the reason), a wrong argument type for a crate call, a
crate value with no text form printed, a borrowing crate type in a field,
`name (` with a space (ambiguous call), bad indentation, tabs. Warnings:
`return` inside a block.

## Tests

```sh
tests/run.sh            # every example's output and every error message, diffed
tests/run.sh --update   # accept current output as the new expectation
```

The suite also runs `lume test` on `examples/tests/`, `lume fmt` on
`examples/fmt/`, and checks that formatting every example is idempotent and
leaves the generated Rust unchanged.

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
are passed by reference and cloned only where an owned value is needed. The
programmer writes mutability, never ownership: `var self` on a method that
changes fields, `var xs: [Int]` on a parameter changed in place. Milestone 7
measured zero ownership syntax across 202 lines of ported programs
(`examples/port/`), so the memory policy is settled: Rust-faithful inferred
ownership, no reference-counting fallback. Milestone 12 measured the last
case, values shared between threads: 5 `shared` words in 97 lines of async
code, all of them at the declaration of the shared thing (`shared var
store`), none at its uses. That is the one ownership word in Lume.

Speed: computation runs at Rust speed (`fib(35)`: 30 ms vs Python 1.04 s).
String-and-map code is within 20% of hand-written Rust after milestone 8
(word count over 360k words: hand-written Rust 19 ms, Lume 23 ms, Python
79 ms). What made the difference: `split`/`lines` are lazy and yield string
slices, `m[k] = m[k].or(0) + 1` compiles to one entry lookup, a key that is
not used afterwards is moved rather than cloned, and `{K: V}` is an
insertion-ordered hash map (Ruby's order, hash speed) written in the prelude.

## Build

Requires a Rust toolchain (https://rustup.rs).

```sh
cd compiler
cargo build --release
# the binary is compiler/target/release/lume
```

## Use

```sh
lume run   examples/fib.lume        # compile and run (fast turnaround)
lume build examples/fib.lume        # compile to examples/.lume/fib, fully optimised
lume test  examples/tests/parse.lume  # build and run the file's `test` blocks
lume fmt   examples/fib.lume        # rewrite in the canonical layout (--check, --stdout)
lume crate examples/crate.lume regex  # what the crate offers, in Lume types
lume emit  examples/fib.lume        # print the generated Rust
lume check examples/fib.lume        # parse and check only (tests included)
lume clean examples/fib.lume        # remove examples/.lume (--cache: the shared cache too)
```

Generated Rust and binaries go in a `.lume/` directory next to the source
file. A program that imports a crate, or uses `async`, is built with cargo.

Compile time, after milestone 14:

| | first time | unchanged | after an edit |
| --- | --- | --- | --- |
| plain program (`fib`, `graph`) | 0.3–0.9 s | 0.01 s | 0.15–0.2 s |
| program using a crate (`regex`) | 1.3 s | 0.01 s | 0.7 s |
| async program (tokio) | 2 s | 0.07 s | 0.9 s |

How: `lume run` compiles with rustc's incremental cache (`.lume/inc-<name>/`),
so an edit recompiles only what changed; a program whose generated Rust is
identical to the last build is not compiled at all; every cargo build on the
machine shares one target directory (`~/.cache/lume/target`, or
`$LUME_CACHE_DIR/target`), so regex or tokio is compiled once per machine,
not once per program — the first ever use costs 20–30 s, every program after
that starts from the cache; through cargo, `lume run` builds the program
crate at opt-level 1 on top of crates at opt-level 3, and `lume build` uses
a separate `ship` profile with everything at opt-level 3. Reading a crate's
signatures (`cargo rustdoc --output-format json`, about 4 s for regex) is
cached next to the project; rustdoc JSON is still unstable in rustdoc, so the
compiler sets `RUSTC_BOOTSTRAP=1` for that one command, and the program
itself is built by the stable toolchain. The whole test suite (99 programs)
runs in 18 s.

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
examples/crate.lume      regex through the bridge: types, iterators, errors, a rust: block
examples/crates.lume     hex and urlencoding, with nothing written for them
examples/modules/        a three-file program: app.lume imports users/model and users/store
examples/interfaces.lume interfaces, extend, operator methods
examples/tests/          files with `test` blocks; expected `lume test` output
examples/fmt/            badly spaced input; expected `lume fmt` output
examples/errors/         programs that must keep failing, with good messages
```

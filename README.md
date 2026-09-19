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

## Status: milestone 5 of 7

| # | Milestone | Status |
| --- | --- | --- |
| 1 | Lexer with indentation, parser, `fib` transpiles to Rust and runs | done |
| 2 | `struct` with methods, bare field access, keyword constructors | done |
| 3 | `_` shorthand, inline blocks, `do` blocks, `for ... where` | done |
| 4 | `enum`, `match` with exhaustiveness errors, tuples, `T?` | done |
| 5 | `T or E`, `Error`, `?` on Result, implicit `Ok`, `!`, `File`/`Env` | done |
| 6 | `\|>`, `import rust.<crate>` with automatic type mapping | next |
| 7 | Port the sample programs plus a graph program; settle the memory policy | |

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
`File.read`/`File.write`/`File.exists?`, `Env.args`/`Env.get`, `if`/`elif`/`else` as
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
`[]` binding with no type,
`name (` with a space (ambiguous call), bad indentation, tabs. Warnings:
`return` inside a block.

## Tests

```sh
tests/run.sh            # every example's output and every error message, diffed
tests/run.sh --update   # accept current output as the new expectation
```

Ownership, for now: `Int`/`Float`/`Bool` pass by value; `Str`, lists and
structs are passed by reference and cloned only where an owned value is needed
(returned, stored in a struct or list, bound to a name). No ownership syntax
appears in Lume code. Milestone 7 measures how far this gets.

## Build

Requires a Rust toolchain (https://rustup.rs).

```sh
cd compiler
cargo build --release
# the binary is compiler/target/release/lume
```

## Use

```sh
lume run   examples/fib.lume        # compile and run
lume build examples/fib.lume        # compile to examples/.lume/fib
lume emit  examples/fib.lume        # print the generated Rust
lume check examples/fib.lume        # parse and check only
```

Generated Rust and binaries go in a `.lume/` directory next to the source file.

## Layout

```
compiler/src/lexer.rs    tokens, INDENT/DEDENT, string interpolation
compiler/src/parser.rs   hand-written recursive descent; all syntax errors
compiler/src/ast.rs      the tree
compiler/src/codegen.rs  Rust emission plus name/mutability checks
compiler/src/error.rs    error rendering (line, caret, help)
compiler/src/main.rs     the CLI
examples/                programs that must keep compiling
examples/errors/         programs that must keep failing, with good messages
```

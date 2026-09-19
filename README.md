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

## Status: milestone 2 of 7

| # | Milestone | Status |
| --- | --- | --- |
| 1 | Lexer with indentation, parser, `fib` transpiles to Rust and runs | done |
| 2 | `struct` with methods, bare field access, keyword constructors | done |
| 3 | `_` shorthand, inline blocks, `do` blocks, `for ... where` (`where` done) | next |
| 4 | `enum`, `match` with exhaustiveness errors | |
| 5 | `T or E`, `T?`, `?` propagation, implicit `Ok` | |
| 6 | `\|>`, `import rust.<crate>` with automatic type mapping | |
| 7 | Port the sample programs plus a graph program; settle the memory policy | |

What works today: functions (`def` block and one-liner forms, return type
inferred when omitted), `struct` with fields and methods (fields used bare
inside methods, `var self` for methods that change them, positional and
keyword constructors, `p.x = v`, `==` and printing derived), `Int`/`Float`/
`Bool`/`Str`/`[T]`, immutable bindings and `var` (with `x = x.trim`
self-transform rebinding), `if`/`elif`/`else` as
expressions, trailing `if`/`unless`, `while`, `for x in range` with `where`,
lists, ranges (`1..10` inclusive, `1...10` exclusive), string interpolation,
`and`/`or`/`not`, `**`, calls, a small set of built-in methods (`len`, `sum`,
`sort`, `first`, `last`, `upcase`, `trim`, `sqrt`, ...).

Errors are Lume errors, not rustc errors: unknown names, fields and types (with
a "did you mean"), assignment to an immutable binding (pointing at where it was
declared), changing a field from a method without `var self`, calling a
mutating method on an immutable value, wrong or missing arguments and keywords,
`if` used as a value without `else`, bad indentation, tabs.

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

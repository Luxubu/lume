# Milestones

How Lume got here, newest first. The language reference is in
[docs/](docs/README.md); this is the record of what was built when and
what each step found.

The design document behind it is
[Lume Language Design](https://claude.ai/code/artifact/1872fe63-1816-4b1e-9983-dec2774382fb).

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
| 15 | Soundness: nested rebinding is an error, argument/return/operand/branch type checks, `Str + Int` rejected, per-type method tables, overflow stops the program | done: review corpus leaks 14 → 6, all remaining are ownership (milestone 16) |
| 16 | Ownership by analysis: liveness-based copy or move, recursive enums boxed, `.or` on optional fields, string comparisons in blocks, `for var a in xs`, `xs[i].method` | done: review corpus leaks 6 → 0 |
| 17 | Pattern completeness: a real exhaustiveness checker with nested witnesses, `A \| B` alternatives; the everyday standard methods; statements in inline blocks | done: 37 new methods, corpus 40 ok / 26 Lume errors / 0 leaks |
| 18 | `{T}` sets with literals, algebra and iteration; `s[i]` / `s[a..b]` / `xs[a..b]` by character and position; structs and enums as map keys | done |
| 36 | Seventh review round (`corpus/m36/`, 30 programs by three reviewers) aimed at the documentation: written from `docs/` alone, counting what was still guessed | done: 98 guesses, none on the chronic questions; 27/27 print what their authors worked out by hand; 7 new reference pages; the pages found a compiler crash, 14 leaks, 2 silent wrong answers and `==` never type-checked |
| 35 | Documentation for users: a tour, a reference organised by question, an examples index, an installer — and `tests/docs.sh`, which runs every example in the docs and checks it against what the docs claim | done: 210 questions from six review rounds answered; writing it found 4 rustc leaks |
| 34 | Concurrency, dogfooded: a parallel record matcher written in Lume, and the rules writing it exposed — `await` takes the task, a failure nothing looks at is an error, a `shared var` collection can be walked, `split` settled | done: 2 rustc leaks closed as Lume errors, 1 soundness hole; `examples/match/` |
| 33 | Speed, measured: five benchmarks against the Rust a person would have written, with the ratio checked in and guarded by `tests/bench.sh` | done: four of five at parity, word count at 113%; interpolation no longer builds a string to copy and throw away |
| 32 | How a value prints: a value on its own prints as itself, a value inside another as you would write it — plus `pad`/`pad_right` on every scalar and the rules written down | done: the question every one of six review rounds asked first; `examples/printing.lume` is the reference |
| 31 | Re-export: `pub import seq` passes a module on to your own importers, so a library's internal layout stops being part of its public surface | done: the last of the milestone-24 findings; `report.lume` imports `table` alone and still calls `seq` |
| 30 | Sixth review round (`corpus/m30/`, 30 multi-file programs by three reviewers): module mechanics, extends across files, and real programs laid out the way a person would | done: 9 ranked problems fixed, 20 ok / 10 Lume errors / 0 leaks; an `extend` now travels as far as the program goes |
| 29 | An `extend` travels with the import: a library ships its conformances, and two claiming the same type and interface is one error naming both files | done: `seq` ships `Walkable[T]`, and its consumer extends nothing; no `pub extend` — there is no name to export |
| 28 | Fifth review round (`corpus/m28/`, 36 programs by three reviewers): generic `extend`, the seams where features meet, and whole programs | done: 12 ranked problems fixed, 27 ok / 9 Lume errors / 0 leaks; `Result` is a name you can use again |
| 27 | Generic `extend`: an `extend` introduces a type parameter by using one, so `[T]`, `{T}`, `{K: V}` and a user's own `Stack[T]` all conform to the same interface | done: the second half of the gap milestone 24 found; one bounded helper now reaches every container |
| 26 | Fourth review round (`corpus/m26/`, 36 programs by a fourth reviewer): how a block travels — reused, forwarded, handed to a built-in, carrying a failure — plus conformance at an argument other than `Self` and a generic enum's boxed field | done: 11 ranked problems fixed, 30 ok / 6 Lume errors / 0 leaks; `Box` is a name you can use again |
| 25 | Interfaces that take type parameters: `interface Comparable[T]:`, bounds that carry arguments (`[T: Comparable[T]]`), `extend Int with Measures[Str]`, conformance that works the arguments out | done: the gap the library hit; structural conformance unchanged |
| 24 | Dogfood a library: `seq` (24 generic helpers), `table` (a typed CSV table) and a program using both, then fix the six things writing it exposed | done: generics and blocks needed no workaround; `pub def`, keywords as method names, `at`, `trim_left`/`trim_right`, `decimals` |
| 23 | Blocks of your own: a parameter typed `(A) -> B` takes a `{ \|x\| ... }` block, a `do \|x\|` body, the `_` shorthand, or a function's name | done: `map`, `retry`, `time_it` and friends are writeable in Lume; a block is inlined at the call, not boxed |
| 22 | User-defined generics: `def first[T]`, `struct Stack[T]`, `enum Tree[T]`, two parameters, interface bounds and the built-in `Ordered` and `Hashable` | done: real Rust generics, no boxing; type arguments read off the call |
| 21 | Dogfood a program that talks to the world: a static site generator, and the I/O it needed — `warn`, `Env.exit`/`stdin`, `Dir`, `Path`, more `File`, raw strings | done: builds a 3-page site, skips unchanged pages, exits 2 when the input is missing |
| 20 | Dogfood again, across modules: a lexer/parser/evaluator for a small language, and what it exposed — nested patterns through recursive enums, top-level constants, `Variant(..)`, method-before-function resolution, `return` as a tail | done: Mini's `fib(21)` in 34 ms, 7× the same interpreter in Python |
| 19 | Dogfood: a real JSON parser/printer/query tool written in Lume, and everything it exposed — `Char`, triple-quoted strings, `\r` and `1e15` literals, `Str +=`, `Time.now_ms`, six formatter defects | done: 13 ms vs hand-written Rust's 10 ms on 867 KB (was 100 ms) |
| 18r | Third review round (`corpus/m18/`, 30 programs by a third reviewer): two block `if`s in a row, `{}` outside a typed binding, block parameters bound one reference too deep, mutations landing on temporaries, unchecked built-in arguments | done: 10 ranked problems fixed, 0 leaks |

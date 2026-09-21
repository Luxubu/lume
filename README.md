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

## Status: milestones 1–24 done

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
| 24 | Dogfood a library: `seq` (24 generic helpers), `table` (a typed CSV table) and a program using both, then fix the six things writing it exposed | done: generics and blocks needed no workaround; `pub def`, keywords as method names, `at`, `trim_left`/`trim_right`, `decimals` |
| 23 | Blocks of your own: a parameter typed `(A) -> B` takes a `{ \|x\| ... }` block, a `do \|x\|` body, the `_` shorthand, or a function's name | done: `map`, `retry`, `time_it` and friends are writeable in Lume; a block is inlined at the call, not boxed |
| 22 | User-defined generics: `def first[T]`, `struct Stack[T]`, `enum Tree[T]`, two parameters, interface bounds and the built-in `Ordered` and `Hashable` | done: real Rust generics, no boxing; type arguments read off the call |
| 21 | Dogfood a program that talks to the world: a static site generator, and the I/O it needed — `warn`, `Env.exit`/`stdin`, `Dir`, `Path`, more `File`, raw strings | done: builds a 3-page site, skips unchanged pages, exits 2 when the input is missing |
| 20 | Dogfood again, across modules: a lexer/parser/evaluator for a small language, and what it exposed — nested patterns through recursive enums, top-level constants, `Variant(..)`, method-before-function resolution, `return` as a tail | done: Mini's `fib(21)` in 34 ms, 7× the same interpreter in Python |
| 19 | Dogfood: a real JSON parser/printer/query tool written in Lume, and everything it exposed — `Char`, triple-quoted strings, `\r` and `1e15` literals, `Str +=`, `Time.now_ms`, six formatter defects | done: 13 ms vs hand-written Rust's 10 ms on 867 KB (was 100 ms) |
| 18r | Third review round (`corpus/m18/`, 30 programs by a third reviewer): two block `if`s in a row, `{}` outside a typed binding, block parameters bound one reference too deep, mutations landing on temporaries, unchecked built-in arguments | done: 10 ranked problems fixed, 0 leaks |

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
list (`[]`, `[x]`, `[first, ..rest]`) and string patterns, nested patterns
through a recursive enum's fields (`Binary("+", Num(0.0), r, _)` reaches
inside the pointer a recursive field lives behind), `Variant(..)` to ignore a
variant's fields, `|` alternatives
(`1 | 2`, `"a" | "e"`, `Some(0) | None`, `Wrap(B) | Empty`; every
alternative binds the same names), guards, and an exhaustiveness check that
works through nested patterns, tuples, lists and `T?`/`T or E` and names a
missing value exactly (`Wrap(A(_))`, `(Green, Tick)`, `Ok(false)`,
`[false, _, ..rest]`), tuples (`(1, "a")`,
`t.0`, `(name, count) = pair` to take one apart, `_` for a part you do not
need, `for i, x in xs.enumerate`), optional values (`T?`, `Some`/`None`,
`.or(default)`, `?` early return in a function returning `T?`; `first`,
`last`, `find`, `max`, `min`, `pop` all return `T?`), errors as values
(`T or E`, the built-in `Error("message")` with `.message`, `?` passes the
error up, a bare value in a `T or E` function is `Ok` and an `Error(...)` is
`Err` automatically, `match` on `Ok(x)`/`Error(e)`, `.or(default)`, `.ok?`,
`.error?`, `.or_error("msg")` to turn a `T?` into a `T or Error`, `!` to
unwrap-or-stop with a warning, `Str.to_int`/`to_float` return `T or Error`),
typed bindings (`var xs: [User] = []`), `def main -> () or Error`,
talking to the world (`puts` and its twin `warn`, which writes to the error
stream; `File.read`/`write`/`append`/`exists?`/`remove`/`size`/`modified`;
`Dir.exists?`/`make`/`list`/`walk`/`remove`; `Path.join`/`dir`/`base`/`ext`/
`stem`; `Env.args`/`get`/`stdin`/`exit(code)`; a user type of the same name
wins over any of these namespaces), the pipe
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
with bodies; a type conforms by having the methods, with nothing to declare,
and then has the defaults as its own methods (`q.describe`);
`def describe(s: Shape)` is a generic function, `[Shape]` holds mixed types
behind a pointer and says so once; `extend Str with Shape:` adds the methods
to a type you do not own; `pub interface` crosses modules), blocks of
your own (a parameter typed `(A) -> B` takes behaviour: `def each[T](xs: [T],
f: (T) -> ())` is called `each(xs) { |x| puts x }`, or with `do |x|` and an
indented body, or `each(xs, _ * 2)`, or `each(xs, double)` naming a function;
`() -> ()` takes a block with no arguments, and when the block is the only
argument the parentheses go (`repeat do`); a block closes over what is around
it and may change it; a method of your own wins over the built-in of the same
name; a block parameter can be handed on to another function; the block is
compiled into the call, so there is no boxing and no lookup at run time;
behaviour cannot be stored in a field or returned yet), generics
(`def first[T](xs: [T]) -> T?`, `struct Stack[T]:`, `enum Tree[T]:`,
`struct Pair[A, B]:` — the type arguments are read off the call, or off the
type the result is going into (`var s: Stack[Int] = Stack(items: [])`), and
never written at the call site; a bound says what the parameter can do:
`[T: Ordered]` for `<`, `[T: Hashable]` to be a map key or set item, and any
interface name for its methods; a generic type is a real Rust generic, so
there is no boxing and no lookup at run time), operator methods
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
`Char` (one character: `s.chars` is a `[Char]`, `s[i]` is a `Char?`; a plain
copied value, so character-at-a-time code runs at Rust speed; it compares
with and matches one-character string literals (`c == "a"`, `"a" | "e" ->`),
has `digit?`, `alpha?`, `space?`, `alnum?`, `upper?`, `lower?`, `upcase`,
`downcase`, `code`, goes in sets and map keys, and becomes a `Str` with
`.to_s` or wherever a `Str` is wanted; `Int.to_char` goes back),
multi-line strings (a `"""` block, with the leading line break and the common
indentation removed; escapes and `#{}` work as usual), `1e15` and `2.5e-3`,
`xs.at(i)` (the item when the position is already known to be good — the
read generic code needs, since it has no default to fall back on),
`trim_left`/`trim_right`, `x.decimals(n)` (the number as text to that
many places, halves away from zero, for money and reports),
top-level constants (`MAX = 100`, `KEYWORDS = {"let", "if"}` — computed once,
`pub` to share them with other files), raw strings (`r"#{not interpolated}"`
and `r"""blocks"""`, for templates, regexes and code),
`and`/`or`/`not`, `**`, calls, `()` for an arm or block that does nothing,
`xs + ys` to join two lists, `\u{1F600}` and `\r` escapes, one statement in an
inline block (`xs.each { |x| total += x }`), and the everyday built-in
methods: on `Str` `len`, `empty?`, `upcase`, `downcase`, `capitalize`, `trim`,
`lines`, `split` (with or without a separator), `chars`, `contains?`,
`starts_with?`, `ends_with?`, `index_of`, `digit?`, `alpha?`, `space?`, `pad`,
`pad_right`, `reverse`, `slice`, `replace`, `repeat`, `to_int`, `to_float`; on
`Int`/`Float` `abs`, `max(x)`, `min(x)`, `clamp(lo, hi)`, `pow`, `even?`,
`odd?`, `sqrt`, `floor`, `ceil`, `round`, `to_float`, `to_int`, `pad`; on lists
`len`, `empty?`, `first`, `last`, `max`, `min`, `sum`, `avg` (a `Float`; `0.0`
for an empty list), `sort`,
`sort_by`, `reverse`, `uniq`, `flatten`, `zip`, `index_of`, `contains?`,
`join`, `push`, `pop`, `insert`, `remove_at`, `take`, `skip`, `enumerate`,
`to_list`, and the block methods `map`, `filter`, `reject`, `each`, `sum`,
`count`, `any?`, `all?`, `find`, `take_while`, `fold`, `min_by`, `max_by`,
`group_by`, `partition`, `flat_map` (a function name works as the block for
all of these: `xs.map(parse)`, `xs.group_by(kind)`); on maps `len`, `keys`, `values`, `get`,
`remove`, `contains?`, `merge`, `to_list`, `each`, `filter`, `reject`,
`map_values`; a collection stored in a `var` map or list is changed where it
is stored (`idx[w].add(x)`, `grid[i].push(x)`, `grid[i][j] = v`; a missing map
key starts from an empty value), and a `for` loop whose body changes the
collection walks a snapshot; on `T?` `or`, `some?`, `none?`, `or_error`, `map`; on `T or E`
`or`, `ok?`, `error?`, `error`, `ok`, `map`; sets (`{1, 2, 3}` is a `{Int}`,
each value once, insertion order kept; `var seen: {Str} = {}`; `add` returns
whether the value was new, `remove`, `contains?`, `len`, `union`,
`intersect`, `diff`, `subset?`, `superset?`, `to_list`, `sort`, `sum`, `max`,
`min`, `join`, `first`, `xs.to_set`, `==` ignores order, `for x in s`, and
the block methods with `filter`/`reject` giving a set back; items and map
keys may be `Int`, `Str`, `Bool`, tuples of those, or a struct/enum made of
them — never a `Float`), positions (`xs[i]` and `s[i]` are a `T?` / `Str?`,
`s[i]` counting characters not bytes; `s[a..b]`, `s[a...b]`, `xs[a..b]` are
slices clamped to the value, so `s[2..99]` is the tail and `s[3..2]` is
empty).

Errors are Lume errors, not rustc errors: unknown names, fields and types (with
a "did you mean"), assignment to an immutable binding (pointing at where it was
declared), rebinding an outer name inside a loop or branch (`total = total + i`
would silently make a new `total`; the error says to declare it `var`), an
argument, field, return value or typed binding of the wrong type (`add(1,
"2")`, with the conversion to use), operands that do not go together (`"n=" +
5`, `7.0 / 2`, with the interpolation or `.to_float` to write), branches of an
`if`/`match` used as a value that give different types, a method a type does
not have (`"abc".reverse` is fine; `"abc".skip(1)` lists what `Str` has),
`9223372036854775807 + 1` and `10 / 0` on literals, changing a field from a method without `var self`, calling a
mutating method on an immutable value, wrong or missing arguments and keywords,
`if` used as a value without `else`, non-exhaustive `match`, `clamp` with
its bounds reversed (at run time, in Lume's words), `?` in a function
that cannot return `None` or an error (with the right fix for each mismatch),
a method or arithmetic on a `T?` or `T or E` without unwrapping, an empty
`[]` binding with no type, a built-in method given
the wrong argument type (`s.add("x")` on a `{Int}`), a list or set literal
whose items disagree, a change that would land on a temporary copy
(`m[k].or({}).add(x)`, `for var p in xs[0..1]`), a value used as an interface
it does not satisfy (naming the missing method or the signature that differs), an `extend` that
leaves a method out, an operator a type does not define, `sort` on a type
without `<`, `await` outside `async def`, an `async def` called
without `await`, a `var` changed inside a `spawn:` block (it is a copy), a
mutating call on a read-only `shared`, a `spawn:` inside a method that uses
`self`, a crate function or method that does not exist (or is not callable
from Lume yet, with the reason), a wrong argument type for a crate call, a
crate value with no text form printed, a borrowing crate type in a field,
a `Float` (or a type holding one) as a set item or map key, indexing a set,
`name (` with a space (ambiguous call), a user type named after a built-in
(`struct Option`), `|` alternatives that bind different names, bad
indentation, tabs, and the spellings other languages use (`continue`, `'single
quotes'`, `null`, `print`, `;`, each with the Lume form). Warnings:
`return` inside a block, a `match` arm that can never run because the arms
above it already cover its values.

## Tests

```sh
tests/run.sh            # every example's output and every error message, diffed
tests/run.sh --update   # accept current output as the new expectation
```

The suite also runs `lume test` on `examples/tests/`, `lume fmt` on
`examples/fmt/`, checks that formatting every example is idempotent and leaves
the generated Rust unchanged, and runs the review corpora (programs written by
independent reviewers who did not know the compiler).
`tests/corpus.sh` classifies the corpus (151 programs from three review
rounds: `corpus/` and `corpus/edge/` after milestone 14, `corpus/m17/` on the
pattern and standard-method surface, `corpus/m18/` on sets and slicing): 111
run, 40 stop with a Lume error (each a deliberate rule or a deliberate error
test: no first-class closures, no shadowing, no `Float / Int`, ...), none
leaks a rustc error.

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
examples/lib/            a library written in Lume: seq (generic helpers), table (typed CSV),
                         a report program that uses both, and what writing it exposed
examples/tests/          files with `test` blocks; expected `lume test` output
examples/fmt/            badly spaced input; expected `lume fmt` output
examples/errors/         programs that must keep failing, with good messages
```

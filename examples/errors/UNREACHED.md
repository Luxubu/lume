# Messages no example shows

`tests/message_coverage.py` lists these as not shown by any test. Each one was
tried; this is why there is no `examples/errors/*.lume` for it.

## Not reachable from `lume check`

- `lexer.rs`: bad number literal `{}` — only a float literal can reach it, and Rust's `f64` parse accepts every digit string the lexer builds (`1e99999` becomes infinity), so it never fails.
- `parser.rs`: `match` has no arms — `match x:` must be followed by an indented line, and an indented line always holds an arm, so the arm list is never empty.
- `parser.rs`: expected `}` to close the block, found {} — an inline block's statement always ends at `}`, a new line (which gives "an inline block holds a single statement" instead) or an error of its own ("expected the end of the line"); no input tried got past those.
- `codegen.rs`: no `main` function — `lume check` compiles in test mode, which allows a file without `main`; only `lume run` and `lume build` give it.
- `codegen.rs`: a block cannot be {} — dead code: `no_fn_outside_params` returns `Ok` before the check (blocks may be kept anywhere since milestone 40).
- `codegen.rs`: a block that produces a value cannot be empty — the parser never builds an empty block.
- `codegen.rs`: `{}` is already shared — dead code: the value's type goes through `materialized()`, which turns a `shared` type into the type it holds, so the `Type::Shared` test never matches.
- `codegen.rs`: `{}` keeps the block it is given, and `{}` is a block this function was only handed for the call — Lume works out which parameters a function keeps, so handing a block parameter on to a function that keeps it (or storing it in a struct) makes this function keep it too; nothing tried was refused.
- `codegen.rs`: `{}` is a block this function was handed, so a kept block cannot hold on to it — same reason: a kept block that uses a block parameter makes the parameter kept.
- `codegen.rs`: a sequence is wanted here, and this is {} — a guard: an argument for a parameter `Iterator[T]` is checked to be a sequence before it is written, so nothing else reaches it.
- `codegen.rs`: `return` inside a block ends this item's block, not the function — a warning, not an error: `lume check` succeeds, and when the same file also has an error only the error is printed.

## Need a crate or packages

- "`{}` uses up `{}`, and `{}` is still needed after it" (milestone 65): needs a crate type that cannot be cloned, used again after a method that consumes it. The crates the suite already fetches (`regex`, `hex`, `urlencoding`, `ureq`) have `Clone` on every type with such a method, so Lume copies instead of refusing.

- `codegen.rs`: `{}` cannot be copied, so it cannot be a struct field — needs a crate type without `Clone`; every public type of the crates available offline (`regex`, `hex`) is `Clone`, and fetching another crate needs the network.
- `main.rs`: `{}` asks for crate `{}` at "{}", and `{}` at "{}" — needs two packages whose `lume.toml` ask for the same crate at different versions (a packages setup).

## Reworded, milestone 60

The examples above found six messages that read badly. Five are fixed:

- **"a `Int`"**: the article now follows the name ("an `Int`", "an `Error`",
  "a `User`"), for every message, in one place (`error.rs`).
- **"as a enum"**: now "as an enum".
- **`mods.shapes.Align`**: an enum or a constant reached through a module is
  now named as the program wrote it, `shapes.Align`, and the suggestion
  compiles.
- **`x.y = 1` with no `x`**: now "unknown name `x`", as reading `x.y` gives.
- **`LIMIT: Int` with no value**: now "`LIMIT` at the top level needs a
  value", with the two ways to write one. `constant_two_at_once` still shows
  the older message for two constants on one line.

Still open: "cannot tell what `x` is" for `[].first!(3)` names an `x` the
program does not have.

## Crashes fixed, milestone 60

A variant pattern with arguments that does not fit the value (`Some(n)` on an
`Int`) and a tuple pattern with fewer parts than the tuple both crashed the
compiler in its check that a `match` covers every case. Both now give the
error the pattern check gives: `pattern_some_on_int`, `pattern_tuple_short`.

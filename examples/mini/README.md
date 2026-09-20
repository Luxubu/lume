# Mini: a small language, interpreted in Lume

Four files, about 700 lines: `lexer.lume` (text to tokens), `parser.lume`
(tokens to a tree), `eval.lume` (walks the tree), `main.lume` (the driver).
Each keeps its own tests beside the code it tests.

```
lume run  examples/mini/main.lume              the built-in samples
lume run  examples/mini/main.lume -- fib.mini  a Mini program of your own
lume run  examples/mini/main.lume -- time      the samples, plus a timing
lume test examples/mini/main.lume              15 tests, all four files
```

Mini has numbers, strings, booleans, `let` and assignment, `if`/`else`,
`while`, `fn` with `return`, `print`, `len`, and the usual operators with
precedence. Every error names the line it happened on, at every stage:

```
print 1 +          -> line 1: expected a value, found the end of the program
print nope         -> line 1: `nope` is not defined
print "a" * 2      -> line 1: `*` does not work on a string and a number
```

This was written during milestone 20 to find out what Lume is missing when a
program is split across modules and built around a recursive tree. Seven
things turned up; all seven are fixed, and the code here is the version
written *after* the fixes — the way it wanted to be written in the first
place. The most visible is `eval.fold`, which matches tree shapes whole:

```ruby
match e:
  Binary(op, Num(a), Num(b), line) -> ...
  Binary("+", Num(0.0), r, _)      -> fold(r)
  Unary("-", Unary("-", x, _), _)  -> fold(x)
```

Before milestone 20 every one of those had to be a cascade of separate
`match` statements, because the fields of a recursive enum sit behind a
pointer.

## Speed

`mini.py.reference` is the same interpreter in Python — same token list, same
tree, same recursive evaluation. Running Mini's `fib(21)`:

| | time |
| --- | --- |
| Lume | 34 ms |
| Python | 240 ms |

Both are tree-walking interpreters doing the same work; the gap is what the
host language costs.

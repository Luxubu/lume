# Lume documentation

**New here?** [Install it](install.md), then take [the tour](tour.md).

Every example in these pages is compiled, run, and checked against the output
it claims by `tests/docs.sh`. If an answer here is wrong, the build fails.

## Reference

| | |
|---|---|
| [Printing and laying things out](reference/printing.md) | how every value prints, `pad`, `join`, blank lines |
| [Lists, maps and sets](reference/collections.md) | lazy chains, `.to_list`, ordering, every method |
| [Strings](reference/strings.md) | characters vs bytes, `Char`, slicing, `split` |
| [Operators and precedence](reference/operators.md) | the precedence table, `and`/`or`, arithmetic, `\|>` |
| [Control flow](reference/control-flow.md) | `if` as an expression, `for`, `while`, `next`, `break` |
| [Functions and blocks](reference/functions-and-blocks.md) | the three block forms, `_`, blocks of your own |
| [Structs and enums](reference/structs-and-enums.md) | fields, `var self`, variants, `match` |
| [Absence and failure](reference/errors.md) | `T?`, `T or Error`, `?`, and the failures you cannot drop |
| [Interfaces](reference/interfaces.md) | structural conformance, defaults, `extend` |
| [Generics](reference/generics.md) | type parameters, bounds, generic `extend` |

## Guides

- [The tour](tour.md) — the language in half an hour.
- [The examples](examples.md) — 5,000 lines of working programs, in a reading
  order.
- [Installing](install.md) — and the commands.

## About these docs

[QUESTIONS.md](QUESTIONS.md) is where this came from: eighteen independent
reviewers wrote Lume programs across six rounds, from the README alone, and
recorded every question they had to guess the answer to. There were 210 of
them. That list is the table of contents, and the reason each page leads with
the question rather than the feature.

Writing these pages found four places where the compiler accepted a program
and then produced Rust that did not compile. Documentation you run is a test
suite.

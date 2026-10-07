# deps — what writing it found

`deps` answers questions about a dependency graph: what a package uses, as a
tree; an order to build in; every path by which one package comes to use
another; the cycles; and what uses a package. Every answer is a generator
method of `Graph` (milestone 75): `walk`, `order`, `paths`, `cycles` and
`users_of`, and the commands only walk them.

    cd examples/deps
    lume run main.lume -- tree shop data/shop.deps
    lume run main.lume -- cycles data/loop.deps

`check.sh` runs every command on a graph with no cycles, one with three, and
one with mistakes. The answers for `order` and `users` were checked against a
Python script over the same file.

## What went well

Generator methods held the whole program, and the first program written with
them needed nothing new from them:

- `walk` keeps its own stack and set as it goes, and the caller stops when it
  likes: `users_of` asks `walk(n).any? { .. }`, which stops at the first hit;
- `paths` calls itself through a second generator method, and `cycles` is
  built from `paths`, three generators deep;
- each method walked a copy of the graph taken at the call, so nothing had
  to be thought about there.

## What was wrong, and is fixed

1. **A block could not take a triple apart.** `walk` yields
   `(depth, name, again)`, and `for depth, name, again in g.walk(root)`
   worked, but `walk(n).any? { |depth, m, again| m == name }` was refused:
   "this block takes one argument, but 3 were named". A block now names one
   argument for each part of a tuple, as `for` does, and naming the wrong
   number says how many parts there are (`examples/errors/block_tuple_arity`).
2. **`line.split("#").at(0).trim` reached rustc** (a leak from milestone 71).
   Reading a list's item in place was meant for a list, and the check let a
   lazy chain through too, because the chain's type was looked at after it
   had been turned into a list's. The same check in `Time.parse`'s arguments
   had the same mistake; both look at the type as it is now.

## Left as it is

- **Recursion with no end stops with Rust's own words.** The first `paths`
  followed a cycle for ever, and the program ended with
  ``thread 'main' has overflowed its stack``. The bug was the program's;
  the message is Rust's, printed by its runtime where Lume cannot put its own.
- **`puts (a + b).join(..)` is refused** as ambiguous, which is the
  documented rule: the message says what to write.

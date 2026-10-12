# Reviewer C — round sixteen notes

## What I read

- `corpus/m78/BRIEF.md`.
- `docs/tour.md`, and `docs/install.md` (the part on warnings and commands).
- `docs/reference/`: `collections.md` (all, especially "Generators: `yield`" and
  "Taking a sequence"), `interfaces.md` (all, especially "An interface's own
  methods may use interfaces"), `methods.md`, `structs-and-enums.md`,
  `control-flow.md`, `patterns.md`, `errors.md`, `functions-and-blocks.md`,
  `strings.md`, `numbers.md`, `printing.md`, `operators.md`, `names.md`,
  `concurrency.md`, and the first half of `generics.md`.
- `examples/`: `generators.lume`, `iterators.lume`, `interface_signatures.lume`,
  `interfaces.lume`, `interface_methods.lume`, `generic_interfaces.lume`,
  `enums.lume`, `iterable.lume`, and their `.expected` files (for the warning
  format); `examples/errors/generator_*.lume`, `yield_in_block`,
  `iface_mismatch`, `for_three_names`, `block_tuple_arity`,
  `iface_pass_held_value`, `interface_param_untyped`, `iface_unused_param` and
  their `.expected`; `examples/errors/UNREACHED.md`.
- Nothing under `packages/` was needed.

## The programs

| file | what it is | leans on |
|---|---|---|
| `c01_file_tree.lume` | A file system as a recursive enum (`Item`/`Folder`) walked by generator methods on the enum: `paths(prefix)`, `walk(depth)` giving triples, `files` giving pairs; a lookup failing as `Entry or Error` | generator methods on an enum with `yield` inside `match self`, recursion into children's generators, `for a, b, c in` a generator, three-name blocks, map tally, `to_set` on a chain, `max_by` on a map, `?` in `main -> () or Error`, a generator keeping its copy after the `var` is replaced, `-> Iterator[T]` with no `yield` |
| `c02_board_pieces.lume` | Chess-like pieces on a 5x5 board as `Piece` interface values; a generator default `moves`, a type that replaces it with its own generator, `attacks?(o: Piece)`, `moved(..) -> Piece` | generator default through a pointer, interface param and result, `[Piece]` warning, tuple map keys, set of tuples, an endless chain from a generator into a generator (`turns(naturals().map ..)`, `.skip(1)`), `match` on a pair of options |
| `c03_document.lume` | A document (enum blocks in nested section structs) rendered and outlined by generator methods; a line parser giving `Block or Error`; an interface `Style` with a generator default taking a list | enum generator walked by another generator of the same enum by bare name, struct generators recursing with a number argument, triples through a three-name block, word tally, `sort_by` on a map, set, interface default taking `Style`, required method giving `Style`, `[Style]` warning |
| `c04_scheduler.lume` | An event scheduler: sources behind `Source` with an endless generator default `times`, a second generator default `events` over it, a plain default handing back a chain; a free generator merging two endless sequences with `next` | generator defaults built from required methods, `take_while` on endless sequences, `next` on `Iterator` parameters inside a generator, an endless chain whose block copies a local, map of lists via missing keys, `pad`, `?` inside constructor arguments, tasks in a list awaited together, `[Source]` warning |
| `c05_type_checker.lume` | An expression language: generator methods `nodes` (yields `self`) and `tree` (triples), a checker giving `Ty or Error` with `?` through recursion, lints behind `Lint` with a generator default `report(e: Expr)` | or-patterns over variants, nested literal patterns in a recursive enum, enum `==`, interface default taking an interface, required method giving one, `-> Iterator[T]` chain with no `yield`, map tally, `flat_map` to a set, `[Lint]` warning |
| `c06_catalogue_fail.lume` | Meant to FAIL: a library catalogue whose search is a generator method taking a `Query` interface value | the documented "not yet" for a generator taking an interface value, applied to a method, and the error format of `examples/errors/generator_block_param` |

## Guesses

1. **c01, c04** — a block inside a chain that is handed back as `-> Iterator[T]`
   may read a parameter of the function or method (`over` in `big_files`,
   `limit` in the interface default `upto`). Looked in collections.md ("Taking
   a sequence": blocks of a chain going into a generator take copies, and
   `def evens(xs: [Int]) -> Iterator[Int] = xs.filter {..}` reads nothing from
   outside) and interfaces.md (`firsts(n)` passes `n` to `take`, no block).
   Assumed it compiles and behaves like any chain.
2. **c01, c05** — a plain `-> Iterator[T]` function may hand back a chain that
   starts from a generator method called on a borrowed parameter
   (`root.files....`, `e.nodes....`). Looked in collections.md and
   `examples/interface_signatures.lume` (`from_list` starts from a list
   parameter). Assumed fine because the generator method walks its own copy.
3. **c02** — a type's own generator method (`Knight.moves`) replaces an
   interface's generator default, and is the one used through a pointer and
   by another default (`attacks?`). Looked in interfaces.md (only a plain
   default, `name`, is shown replaced) and the generators section of
   collections.md (nothing on overriding a generator default).
4. **c02** — trailing `unless` on a compound assignment (`safe += 1 unless
   ..`). control-flow.md shows a trailing `if` on an assignment and `unless`
   only on `puts`.
5. **c02, c03, c04, c05** — the warning's `-->` path is the path as given on
   the command line from the repository root (`corpus/m78/<file>`), as the
   `.expected` files show `examples/<file>`. The format, column 3 and the
   three-digit gutter were copied from `examples/generic_interfaces.expected`.
6. **c03, c05** — a generator *default* may take an argument
   (`restyle(xs: [Str])`, `report(e: Expr)`), and be called through a pointer
   (and, in c03, on a `Style` just returned by `narrower`). interfaces.md shows
   only `numbered`, which takes none; collections.md says a generator takes
   copies of its arguments.
7. **c04** — a generator may call `.next` on its own `Iterator[T]` parameters
   (not declared `var`) and keep going from there (`merge`). collections.md
   says `xs.next` works on such a parameter; the only example
   (`first_and_rest` in `examples/generators.lume`) is a plain function.
8. **c04** — a one-line `spawn:` body may walk a generator default of a
   struct copied into the task (`spawn: tea.upto(100).len`). concurrency.md
   says nothing about generators inside tasks.
9. **c05** — `yield self` inside a generator method yields the method's copy
   of the receiver, and `match`/`kids` on `self` still work after it.
   collections.md's `Node.names` yields a field, never `self`.
10. **c05** — inside an interface default, a block handed to `count` may call
    the interface's own required method by bare name (`flags?(n)`) as well as
    one on another interface value. interfaces.md's `Walk.total` block only
    changes a local.
11. **c01, c02, c05** (and others) — block or loop names that are never used
    (`{ |d, t, s| t }`, `Item(n, _)` style arms aside) give no warning. No
    page says whether unused names warn; my EXPECTED STDERR blocks hold only
    the interface-list warnings.
12. **c06** — a generator *method* with an interface parameter is refused with
    the same message, help and position (the parameter's name) as
    `examples/errors/generator_block_param` gives for a free function with a
    block parameter. collections.md only says "Not yet: ... a generator that
    takes a block or an interface value".
13. **c06** — with an error in the file, `lume check` prints only the error and
    not the `queries: [Query]` warning. The only hint is
    `examples/errors/UNREACHED.md` ("when the same file also has an error only
    the error is printed", said of a different warning).

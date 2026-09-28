# Reviewer C — whole programs where moves meet the rest of the language

## What I read

- `corpus/m53/BRIEF.md`.
- `docs/README.md`, `docs/tour.md`.
- `docs/reference/`: `collections.md` (all of it, most closely "When a list is
  copied, and when it moves"), `structs-and-enums.md`, `generics.md`,
  `interfaces.md`, `functions-and-blocks.md`, `concurrency.md`, `modules.md`,
  `packages.md`, `control-flow.md`, `patterns.md`, `printing.md`,
  `methods.md`, `names.md`, the first half of `strings.md` and `numbers.md`,
  the top of `operators.md`, and the `!` section of `errors.md`.
- `examples/`: `moves.lume`, `moves_values.lume` (and their `.expected`),
  `async.lume`, `kept_blocks.lume`, `generics.lume` (first half),
  `packages/` (all three packages and `shelf/main.expected`), `csvq/`
  (layout, `lume.toml`, `queries.sh`, `FINDINGS.md`), `match/main.lume` and
  `match/records.lume`, `modules/app.lume` and `modules/users/`.

I did not read `docs/QUESTIONS.md`, the compiler, `tests/` or other corpus
folders, and I did not run anything. Every expected output was worked out by
hand.

## The programs

| file | what it is | leans on |
|---|---|---|
| `c01_stackvm.lume` | A stack machine that computes a factorial. Its `Vm` state struct (list, map, text, ints) is replaced on every instruction; the program is an enum with payloads, rewritten by `patch` and run again. | `vm = step(vm, op)` in a `while`; a `var` struct rebuilt from its own fields, each read once; `var last = second` at `second`'s last use; `zip` and `count` over enums; a literal inside a variant pattern |
| `c02_shelves/` (modules `kit/stack`, `kit/cell`) | Generic `Stack[T]` holding `Book` records, then `Str`, then `cell.Box[Str]`. Each stack is drained and rebuilt; a generic `Box[T].apply[U]` is chained twice. | `Stack.empty` from another module; `drain` on a `var self`; a list walked by `for` and read again after; records rebuilt field by field; `acc = acc + ...` fed by a drained stack |
| `c03_resize/` (modules `geo/api`, `geo/shapes`) | An interface `Resizable[T]` whose implementations (`Rect`, `Word`) build a new value of their own type. Generic `grow_all` and `report` live in another module. | A `var` list replaced by a generic call on itself, then by `filter ... .to_list`, then by `+`; a default returning `T`; defaults called in `_` chains |
| `c04_orders/` (package; library `deps/money`; modules `model`, `flow`) | An order's `Status` enum (payloads carry `Line` lists and `money.Money`) moves from state to state on events parsed from text. Refunds are summed with `Money`'s own `+`. | `s = match e.split(":"):` where one arm hands back `s` itself; variants built bare across modules; `t = t + x` on a struct from a dependency; finished states moved into a list literal |
| `c05_workers/` (modules `work/job`, `work/text`) | One task per `Job`. Each task builds an `Outcome` in another module, passes it through an `async def` that builds a new one, and tallies into a `shared var` map and list. | A loop-variable struct copied into `spawn:`; `seen[key] += 1` on a map whose keys were put in first; awaited results walked twice; the jobs still whole afterwards; `shared` read-only text in a second round of tasks |
| `c06_pipeline.lume` | A `Pipeline` of `Stage`s, each holding a kept block. It is extended by `then`, which returns a new pipeline, and applied to text. Blocks are composed in a loop, looked up in a map, and used to rebuild a `Cell` record each step. | `p = p.then(...)`; `cur = s.run(cur)`; `whole = compose(whole, f)`; `n = match rules[key]:`; `c = step(c, s)`; a block that copied a local in; a struct holding blocks, printed |
| `c07_report/` (package; libraries `deps/grid`, `deps/counts`) | Sales records are parsed from text and laid out by `grid`, which builds column text in loops. They are tallied by `counts.tally` and collected in a generic `counts.Bucket[Sale]` that grows by returning a new one. | Structural `grid.Row` across packages; `line = line + ...` inside a dependency; a `var` map replaced by its own `filter`; `b = b.adding(s)` in a `for ... where`; a grid changed after it was built from a bucket's items |
| `c08_lexers/` (modules `lex/token`, `lex/scan`) | A character scanner as a state machine: `Scan` holds a `Mode` enum with payloads and a token list, and is replaced per character. There is one async task per input line, and a `shared var` tally. | `s = step(s, c)`, `s = flush(s)`; payloads rebuilt from old ones (`InWord(w + c)`); tokens walked twice in a task, then handed on; tuples returned from tasks; `kinds[tk.kind] += 1` |

Every program is Meant to RUN and is deterministic. Where tasks change a
`shared var` map, its keys are put in before any task starts, so the map's
order does not depend on which task runs first. The one `shared var` list
whose order would vary is printed sorted. The `shared var` warnings go to
standard error and are not in any `EXPECTED OUTPUT`.

## Guesses

1. **Where the library packages sit** (c04, c07). I put each dependency
   inside the program's own folder (`c04_orders/deps/money`, with
   `money = { path = "deps/money" }`), so each program is one folder. Both
   `examples/packages/` and `examples/csvq/` put the libraries in *sibling*
   folders instead. `packages.md` says a dependency is "found by `path`
   from this file's folder", and its `lume fmt` row mentions "a package kept
   in a folder inside it", so nesting seems to be allowed. I assumed that
   works, and that the nested package's `lib.lume` is not also read as a
   module of the outer package. I kept each folder name the same as its
   package name, so I did not have to guess whether they must match.
2. **A multi-line, typed constant** (c01, `PROGRAM: [Op] = [ ... ]` over
   several lines with a trailing comma). `modules.md` shows typed constants
   (`NOTHING: Int? = None`), and `kept_blocks.lume` shows a multi-line list
   literal with a trailing comma inside `main`. Neither shows the two
   together at the top level of a file.
3. **Taking a three-part tuple apart** (c01, `(rest, a, b) = pop2(...)`).
   `structs-and-enums.md` only shows two-part destructuring. Three-part
   tuples appear only as values (`(1, "two", true)`).
4. **Where an interface's defaults can be called** (c03).
   `rects.at(0).twice` and `.line` are called in `main.lume`, which imports
   `geo.api` (where the interface is). `geo/shapes.lume` (where `Rect` is)
   never imports it. `packages.md` says a type fits an interface it never
   imports, "and the interface's defaults are then its own methods". I
   assumed that means "in any file that can see the interface".
5. **A default that returns `T` itself** (c03, `def twice -> T =
   resized(2)` in `Resizable[T]`). `generics.md` shows defaults that return
   `T?` and `[T]` (`Ranked`), and `Bool` (`Comparable.before?`). It does
   not show a default that builds a `T` by calling a required method.
6. **A generic method that builds its own struct with another type
   parameter** (c02, `Box[T].apply[U] -> Box[U] = Box(item: f(item), ...)`).
   `interfaces.md` shows `map_all[V]` returning `[V]` on a `Box[T]`, and
   `generics.md` shows `Pair.swapped -> Pair[B, A]`. Neither shows both at
   once.
7. **Calling a block bound by a pattern** (c06, `Some(f) -> f(n)` on
   `rules[key]`, a map of kept blocks). `functions-and-blocks.md` shows
   `ops["mul"]!(6, 7)` and calling a kept block from a binding, a field, a
   list and a tuple, but not from a `match` binding.
8. **`[]` as a variant's payload argument** (c04, `var s: model.Status =
   Draft([])`). `collections.md` lists where an empty literal gets its type:
   a typed binding, a field, a function's declared parameter. A variant's
   payload field is not on the list. I assumed it counts, as a struct field
   does.

Things I checked rather than guessed, for the record:

- `decimals(2)` on `1449/100`, `500/100`, `1949/100` and `0.0`.
- `pad_right` never cuts a value short.
- `split(",")` on a line with no comma gives one piece.
- `s[0...4]` is exclusive.
- `Str + Char` works.
- A map keeps insertion order through `filter`, and a set keeps it too.
- `await` on a list of tasks keeps start order.
- `shared var` warnings go to standard error.
- `Stack.empty` works on a type imported by name, with `T` read from the
  binding.
- Bare variants work across modules when the type is known.
- A struct prints by its bare type name across packages.
- A block prints as `<block>`.

## Verdict

The docs were enough for eight programs that reach across modules, packages,
generics, interfaces, kept blocks and tasks, with eight guesses between
them. None of the guesses is about moves. The moves page answers the
question it sets itself: "moved or copied" never changes the output, so
every expected output came from ordinary semantics. The line between moved
and copied is listed plainly: last use, a replacement, named twice, a
parameter, a field, used by a block.

What stays open is what the brief says the moves round is about: where the
compiler has to decide on its own. These programs put these cases in front
of it:

- a struct replaced by a call on itself inside `while` and `for`
  (c01, c08);
- a `var` replaced by a `match` whose arm hands the old value back (c04);
- a block value replaced by a composition of itself (c06);
- a bucket replaced by a method of itself inside a `for ... where` (c07);
- a struct copied into several tasks from a loop variable, and then read
  again after the tasks (c05);
- tokens walked twice inside a task body, then moved into a call (c08).

If any of these fails to build, that is the finding. None of them should
change what is printed.

The docs are thinnest on packages laid out differently from
`examples/packages` (guess 1), and on the edges of generic interfaces
(guesses 5 and 6).

# Reviewer C — round fifteen (m73)

## What I read

- `corpus/m73/BRIEF.md`.
- `docs/tour.md`, and in full: `docs/reference/collections.md`,
  `strings.md`, `printing.md`, `control-flow.md`, `errors.md`,
  `structs-and-enums.md`, `numbers.md`, `methods.md`,
  `functions-and-blocks.md`, `patterns.md`, `names.md`, `operators.md`,
  `interfaces.md`, `io.md` (the `Time` part most closely), and the first
  half of `concurrency.md` (`spawn:`, `await` on a list, what a body may
  hold).
- `examples/generators.lume` + `.expected`, `examples/split_slices.lume` +
  `.expected`, `examples/iterators.lume` + `.expected`,
  `examples/pipes.lume` + `.expected`, `examples/dates.lume` + `.expected`,
  `examples/logq/main.lume` and `examples/logq/FINDINGS.md`.
- `examples/errors/`: every `generator_*`, `yield_*` and
  `iterator_param_var` case, and `block_wrong_result` for the layout of a
  two-digit line number.

Not read: compiler source, `tests/`, `bench/`, other corpus folders,
`README.md`, `HISTORY.md`, `CHANGELOG.md`, `docs/QUESTIONS.md`. Nothing was
run.

## The programs

All six are meant to RUN.

| file | what it is | leans on |
|---|---|---|
| `c01_ini_config.lume` | INI reader: a generator lexer → a generator parser → a map of maps, then checks through an interface | generator yielding enum values from `lines`; `xs: Iterator[Token]` given a list; `[k, ..rest] if` on `split("=")`; `.or({})`; set `add`/`sort`/`diff`; `?` on `T?` in a method; `or_error`, `map_error`; interface default calling a required method; `warn` from a generator |
| `c02_stack_machine.lume` | stack-machine interpreter with labels and registers, run on five small programs | generator over `lines.enumerate` yielding `(Int, Instr or Error)`; `for n, r in` over an `Iterator` parameter; list patterns with literals and a guard on bare `split`; `?` inside constructor arguments and arms; `var self` methods giving `T or Error`; `return` from arms inside `while`; raw `r"""` |
| `c03_inventory.lume` | reconciles a CSV ledger against a whitespace stock count | one generator for both tables; `rows.next` for the header, then `for` over the rest; generator taking two maps and yielding enum values with guarded `Some(n)` arms; generator into generator; `\|` alternatives binding one name; `keys.to_set.diff`; `m[k].push` through a missing key; `pad_right`; whole cents |
| `c04_schedule.lume` | two-week agenda of recurring meetings over working days | endless generator; generators chained lazily and stopped by `take` and `take_while { \|t, title\| .. }`; `next` on a `var` generator; `Time.date/format/parse/weekday/day/day_of_year`; enum with a method; `group_by` with two names, then a map's `map` |
| `c05_adventure.lume` | text adventure: the world is read from text, then a script is replayed | generator of `Fact or Error`; `match f?`; generator ending itself with bare `return` after `quit`; `["inventory"] \| ["i"]`; `var self` methods changing `Str`, map and set fields and calling each other; map `filter` with two names; `set.add` as `Bool` |
| `c06_log_tasks.lume` | log report: parsed by a generator pipeline, summed per service in one task each | `split(" ")` read by position plus `slice`; `match` as a value with `return Error` in an arm; `warn` for bad lines; a list into an `Iterator[Rec]` parameter inside a task; `spawn:` per group, `await` on `[Task[Summary]]`; enum as a map key; `max_by` tie on a map; call across lines with a tuple field |

Hand-worked facts worth checking against the run: 2026-10-01 is a Thursday
and day 274 (from the doc's 2024-09-29 Sunday: 730 days on is Tuesday
2026-09-29); the factorial program executes 81 instructions; the ledger is
worth 15125 cents and the count changes it by -4075.

## Guesses

1. **c01** — `Section(name, _) -> named.add(name)` next to `_ -> ()` in a
   statement `match`: I assumed an arm may be a call whose `Bool` result is
   dropped while another arm is `()`. `collections.md` shows `seen.add(..)`
   as a statement and `patterns.md` ("What an arm can do") lists what a
   one-line arm may be, but neither shows a value-giving call beside `()`.
2. **c01** — `values[e.section].or({})` on a `{Str: {Str: Str}}`: I assumed
   `{}` takes the inner map type from the map, as `.or([])` does.
   `collections.md` ("Where `[]` and `{}` need a type") only shows `.or([])`.
3. **c01** — an interface method with a struct-typed parameter
   (`def run(c: Config) -> Str or Error`) and a default that calls it as
   `run(c)`, through a `[Check]` list. `interfaces.md` shows defaults calling
   required methods with no arguments, and block parameters, never a plain
   value parameter.
4. **c01, c02, c03, c05** — `next if ..` inside a generator's `for`. I
   assumed `next` works there as in any loop. `collections.md` and
   `examples/generators.lume` show `break` and `yield .. if`, never `next`,
   in a generator.
5. **c02** — `def pop(var self) -> Int or Error = stack.pop.or_error(..)`:
   a one-line `= expr` body on a `var self` method that changes a field.
   `structs-and-enums.md` shows `var self` only with indented bodies.
6. **c03** — `rows.next` on an `Iterator[[Str]]` parameter, then
   `for r in rows`, expecting the loop to start at the second row.
   `collections.md` says `xs.next` "works on it as it is", and
   `examples/generators.lume` (`first_and_rest`) shows `next` then
   `to_list`, not `next` then `for`.
7. **c04** — the message of `Time.date(2026, 9, 31)`: I wrote
   `Error("September 2026 has days 1 to 30, not 31")`, following the one
   example in `io.md` and `examples/dates.lume` (February 2023).
8. **c04, c05** — a one-item set literal, `{Time.date(2026, 10, 12)?}`
   (with `?` inside) and `{"hall"}` as a constructor argument. `printing.md`
   says `{` can only start a map or a set, but every set literal in the docs
   has two or more items.
9. **c05** — an enum variant whose payload field is a map,
   `Room(name: Str, exits: {Str: Str})`, built positionally and taken apart
   by `match`. `structs-and-enums.md` shows payloads of scalars and of the
   enum itself only.
10. **c05** — calling another method of the same type by bare name *with
    arguments* (`go(w, dir)`, `pick_up(t)`, `describe(w)` from inside
    `step`). `structs-and-enums.md` ("Calling another method of the same
    type") shows only methods that take no arguments (`bump`).
11. **all** — no warnings: I predicted empty standard error (apart from
    `warn` in c01 and c06) although some payload fields (`line` in c01's
    `Token`), struct fields (`name` in c03's `Item`) and block parameters
    (`title` in c04) are never read. The docs mention warnings only for `!`
    and for unreachable `match` arms.

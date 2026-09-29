# Reviewer C — whole programs where JSON, dates and the rest meet

## What I read

- `corpus/m63/BRIEF.md`.
- `docs/`: `README.md`, `tour.md`, `examples.md`, and in `docs/reference/`:
  `json.md`, `io.md` (all of it, "Dates" and `Process` especially),
  `methods.md`, `collections.md`, `errors.md`, `printing.md`, `numbers.md`,
  `interfaces.md`, `generics.md`, `packages.md`, `modules.md`,
  `structs-and-enums.md`, `patterns.md`, `control-flow.md`,
  `functions-and-blocks.md`, `strings.md`, `operators.md`, `names.md`,
  `concurrency.md`. Not `docs/QUESTIONS.md`.
- `packages/json/`: `lib.lume` (read whole, to know every error message
  `need_…` and the parser give, and exactly how `show`/`pretty` lay out),
  `lume.toml`, `test.expected`.
- `examples/json_config/` (all), `examples/jobr/` (all, including
  `FINDINGS.md`), `examples/csvq/` (all), `examples/packages/` (all),
  `examples/dates.lume` and `.expected`, `examples/json/README.md`,
  `examples/check_json.expected`, and the three `examples/errors/time_*`
  cases.
- Not read: the generated `.lume/main.rs` files that sit inside
  `examples/match/`, `examples/shipping/` and `examples/travel/` (a grep
  showed they hold the `Time` runtime; they are build output, and reading
  them would be reading the compiler's). Nor the other reviewers' folders in
  `corpus/m63/`.
- Real-world weekdays (not Lume behaviour) were checked with the system
  `date` tool after working them out from the documented anchor
  "2024-09-29 is a Sunday". Nothing was compiled or run.

## The programs

| file | what it is | leans on |
|---|---|---|
| `c01_logs/main.lume` | log analyzer: 13 JSON lines, 4 refused with the line and reason, requests bucketed by hour, busiest hour, slowest request, levels, mean, and a JSON summary | `json.parse` error position, `need_str`/`need_int`, `Time.parse` with `%H:%M:%S` and its position error, `Time.format` with `%A %d %B`, `{Int: Int}` tally, `max_by` on a map, `avg`, `decimals`, `json.obj` |
| `c02_shop/main.lume` + `c02_orders/lib.lume` | a store: orders with nested items and dates decoded by a library package (which depends on json); invalid orders reported by index, item and field; totals by month; one order and the month totals written back | two packages over json, `def self.from_json` through `import c02_orders.Order`, `map_error` at two levels, `Time.parse` month error, `group_by`, `%a %b %B`, `ToJson` on lists of my own structs, `json.show`, `json.pretty` |
| `c03_schedule/main.lume` | scheduler: jobs with a start and `{"days": n}`/`{"hours": n}`; next five runs of each after a fixed now, across 29 February and month ends; the next five of all; the leap-day run; the plan as JSON | `need`, `get(..).some?`, date arithmetic in seconds, `Time.month`/`day`/`day_of_year`, sorting `(Int, Str)`, a two-name block over a chain, `main -> () or Error` |
| `c04_runner/main.lume` | runs due commands with `sh -c` in tasks, skips the ones not due yet, reports and writes each result as JSON, then a pretty summary | `async def main -> () or Error`, `Process.run` in an `async def` called from `spawn:`, `await` on a list of tasks, comparing times, `json.show`, `json.pretty` of nested object/array |
| `c05_jdiff/main.lume` | a diff of two JSON documents: added, removed and changed keys and positions, with the kind change noted; days between the two release dates | bare `Obj`/`Arr` patterns from another package, `match` on a pair of `Json`, `==` on `Json` (8.0 vs 8), a `var` list down a recursion, `kind`, `path`, `json.show` of each kind |
| `c06_timeline/main.lume` + `c06_feed/lib.lume` | posts and comments decoded through a library's generic `decode_all[T]`; reported through the library's `Dated` interface and my own `[T: c06_feed.Dated]` generic; a mixed timeline; the library's `envelope[T: ToJson]` over my structs and over `Int` | generic function taking a type's `from_json` as a block, structural conformance to another package's interface (field `at`, method `stamp`), interface defaults through a bound, `[Interface]` list of two types, `Json.Arr(...)` built outside the json package, tuple destructuring |

## Guesses

1. **A bare `"` inside a `"""` string** (c05). `strings.md` "Triple-quoted
   strings" says escapes and `#{}` work as in a one-line string, and says
   nothing about an unescaped `"`. I assumed only `"""` closes the string,
   so JSON can be pasted in as it is. The other five programs escape every
   quote (`\"`), which the same paragraph does cover. For JSON in a program
   this is the question that matters most, and no page answers it.
2. **Pushing a struct into a `var xs: [Interface]` list** (c06,
   `timeline.push(p)`). `interfaces.md` "Where an interface can be used"
   shows an annotated list literal (`mixed: [Shape] = [Circle(1.0), Sq(5.0)]`)
   and a function returning one. It does not show a value becoming the
   interface when passed to `push`. I assumed it converts as the literal does.
3. **An interface's default method, called on a value held as the
   interface** (c06, `d.when` and `d.stamp` on items of `[c06_feed.Dated]`).
   `interfaces.md` says a held value's methods can be called, except generic
   and `async` ones. Required methods are clearly included. Defaults called
   through the pointer are only implied.
4. **A generic `T` read off a function handed over as a block** (c06,
   `c06_feed.decode_all(items, "post", Post.from_json)` where the parameter
   is `decode: (Json) -> T or Error`). `generics.md` says `T` is read off the
   arguments. `structs-and-enums.md` shows `[50.0].map(Temp.from_f)`, a
   type's function passed as a block. No page combines the two, a
   `def self.` function whose result type fixes a caller's `T`.
5. **The wording and position of `Time.parse` errors other than the one
   shown.** `io.md` and `examples/dates.lume` show
   ``expected `-` at position 2`` for `29/09/2024` against `%Y-%m-%d`, and
   ``month 13 is not 1 to 12``. I extended this in two ways. First, to
   another literal: c06 expects ``expected `T` at position 10`` for
   `2024-10-14 08:20` against `%Y-%m-%dT%H:%M`. Second, to a four-digit
   year: c03 expects position 4 for `2024/02/27 10:00`, on the reading that
   `%Y` takes the digits up to the first non-digit, as `29` was taken in the
   documented example. c01's line 9 is the documented case exactly. I
   avoided every date error with no documented form: Feb 30, hour 25, text
   that ends early.
6. **English names for `%a`, `%b`, `%A` and `%B` beyond the ones shown.**
   The directive table shows only `Sun`, `Sunday`, `Sep` and `September`,
   and `dates.lume` adds `Mon`–`Sat`. I assumed the rest are strftime's
   (`Jan`, `Feb`, `Mar`, `Apr`, `Jun`, `Oct`, `January`, `February`,
   `March`, `Monday`, `Tuesday`), with `%d`/`%H`/`%M` zero-padded as `%m`
   and `Tue 01` show.

Things I could check, so they are not guesses. Each `need_…` message and
parser message comes from `packages/json/lib.lume` (`a_kind`: "a number",
"an object"). `json.show` writes 8.0 as `8`, and `pretty` indents two
spaces per level, from the source. `Process.run` in an `async def` without
`await` is how `examples/jobr/` calls it. Package names may hold digits and
`_` (`packages.md`); I kept each folder name equal to its package name so as
not to lean on anything else. Only one field of each bad record is wrong, so
nothing depends on the order a constructor's arguments are evaluated in. The
shell commands' output does not depend on the platform once trimmed
(`wc -w` pads differently on macOS and Linux).

## Verdict

These programs were easy to write from the docs. Six whole programs, two of
them with a library package of their own, came to six guesses. None is about
the `json` package. Its page plus its source answered every question about
parsing, typed reads, decoding errors and writing, and `json_config`, `jobr`
and `csvq` gave the patterns: `map_error` per item, `Ok((code, out, err))`,
tasks over a list. Dates were almost as good. The one example and the
directive table covered format, parse, `date` and the parts, and the
"a day is 86400 seconds" rule made scheduling plain arithmetic.

Three gaps did cost guesses:

- **JSON inside a program.** Can a `"""` string hold bare quotes? Every
  JSON-heavy program either escapes every quote or bets on it. One sentence
  in `strings.md` would settle it.
- **`Time.parse` failures.** Only two are shown, so any other one in an
  expected output is extrapolated. A short table of the kinds of message
  would help: wrong literal, bad month, bad day, text ending early, text
  left over.
- **Values held as an interface.** Pushing onto an `[Interface]` list and
  calling defaults through it are both left to inference.

The best design point: decoding across packages needed nothing written.
A struct in my program is a `ToJson` for the json package's list `extend`,
and a `Dated` for my library's generics, with no `extend` and no import of
the interface in the type's file. The one trap was naming a field `stamp`
when the interface wants a `stamp` method. `structs-and-enums.md` warns
about exactly this, and I worked around it with a field called `at`.

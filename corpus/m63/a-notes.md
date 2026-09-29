# Reviewer A — everyday programs with JSON and dates

## What I read

- `corpus/m63/BRIEF.md`.
- `docs/reference/json.md` and `docs/reference/io.md` (all of it, "Dates" closely).
- `packages/json/lib.lume` and its `lume.toml`, line by line: every error
  message and every printed form in the expected outputs below was worked
  out from this source (`show_num`, `quote`, `pretty_at`, `a_kind`, the
  parser's `fail` positions).
- `examples/json_config/` (all files), `examples/dates.lume` and
  `dates.expected`, `examples/json/json.lume` (its `"""` sample and
  `json.expected`).
- `docs/tour.md`, and from `docs/reference/`: `errors.md`, `strings.md`,
  `printing.md`, `collections.md`, `numbers.md`, `methods.md`,
  `structs-and-enums.md`, `control-flow.md`, `functions-and-blocks.md`
  (to "`?` inside a block"), `packages.md`, the first half of `modules.md`,
  and the table at the top of `patterns.md`.
- Weekdays and day counts were worked by hand from the anchors in the docs
  (2024-09-29 is a Sunday, day 273), then checked against a calendar. No
  Lume program was compiled or run.

## The programs

| file | what it is | leans on |
|---|---|---|
| `a01_settings/main.lume` | settings decoded into a struct; missing or `null` optional fields take defaults; wrong kinds and broken JSON are refused | `need_str`, `get`, `int`/`bool`/`num`/`items`, `null?`, `kind`, `or_error`, `return` from a `match` arm, parser error position |
| `a02_address_book/main.lume` | address book from JSON, sorted by surname ignoring case, printed as a table, with a summary and one refused document | `need_items` + `map_error`, a `Str?` field, `split.last`, `sort_by`, `pad`/`pad_right`, `avg`, `max_by`, `count` |
| `a03_to_json/main.lume` | a list of `Book` structs written with `to_json`, `json.show` and `json.pretty`, then read back and compared with `==` | `ToJson` through `[T: ToJson]`, `json.obj`, `Json.Null` for `None`, `show_num` (whole floats, `1e20`), `quote` escaping, empty `[]` in `pretty` |
| `a04_event_log/main.lume` | events with ISO timestamps, parsed, sorted, grouped by day; one bad timestamp is reported and skipped | `Time.parse` with a `T` in the pattern, `Time.format` (`%A %d %B`, `%H:%M`), `group_by` order, `max_by` on a map |
| `a05_birthdays.lume` | days until each birthday from a fixed today; 29 February falls on 1 March in a common year | `Time.date` and its `Error`, `Time.year`, `Time.format` (`%a %d %b %Y`), `sort_by`, `count`, a constant list of structs |
| `a06_invoice.lume` | invoice in cents, three payments 30/60/90 days after issue; a weekend due date moves to Monday | `Time.date` + days × 86400, `Time.weekday` (6, 7), `Time.format`, `pad`, `if`/`elif` as a value |
| `a07_edit_json/main.lume` | reads a document, bumps its version, changes a nested field, adds a key, writes it with `json.show` to a file and reads it back | `pairs`, `json.obj`, `need`, `need_str`, `keys`, `path`, `File.write`/`read`/`remove`, `.map(json.show)` |
| `a08_durations.lume` | seconds shown as "2h 05m" and as a clock | `Int` `/` and `%`, a zero-pad helper, `Time.format(secs, "%H:%M:%S")`, `for a, b in` a list of pairs, `max_by` |
| `a09_json_explorer/main.lume` | walks a nested document by dotted paths, printing kind, keys and item counts; path queries that hit and miss; a total over nested arrays | `path`, `items`, `keys`, `kind`, `null?`, `Json.Null` as a default, `match` on a `Str` with a binding arm |

## Guesses

1. **A letter in a `Time` pattern is matched as itself, and a mismatch there
   reads ``expected `T` at position 10``** (a04). `io.md` and
   `examples/dates.lume` show only `-`, `:`, `,` and spaces between
   directives, and the one error example is ``expected `-` at position 2``.
   I assumed every character that is not a `%` directive behaves like `-`,
   letters included, and that the error message has the same shape.
2. **A struct with a `Float` field still gets `==`** (a03,
   `again == books`). `structs-and-enums.md` says equality, printing and
   hashing come free; `numbers.md` says a `Float` cannot be a set item or
   map key. Nothing says whether a struct holding a `Float` loses `==`
   along with hashing. I assumed it keeps `==` (the `Json` enum itself
   holds a `Float`, and `packages/json` compares `Json` values with `==` in
   its tests, which supports this).
3. **`Json.Null` works as an expression after `import json.Json`** (a03,
   a09). `modules.md` shows `Kind.Tool(grams: 450)` after importing a
   type by name, and `packages/json` tests write `Json.Num(1.0)` inside the
   package. I found no example of a variant with no payload, from another
   package, written this way. I assumed it works.
4. **Whether `import json.ToJson` is needed to call `.to_json` on `Str`,
   `Int`, `Float`, `Bool` and lists, and whether an unneeded import is
   complained about.** `json.md` does not say; `packages.md` says an
   `extend` "travels with the import", which suggests `import json` alone
   is enough. I copied `examples/json_config/`, which imports all three
   (`json`, `json.Json`, `json.ToJson`), in a03 and a07, and left
   `ToJson` out of the programs that never call `to_json`. The docs
   mention no unused-import warning, so I assumed there is none.
5. **A pattern that is not written out in the call** (a04's
   `Time.format(first.stamp, ISO)`, with `ISO` a constant) is used as it is
   when the program runs. `io.md` says only that a pattern *written out* is
   checked at compile time. I assumed a constant works and is simply not
   checked early. My pattern is valid, so what a bad pattern would do at
   run time did not matter here.

## Verdict

The JSON package and the date functions were enough for every one of the
nine everyday programs, and nearly everything I needed was either on a
docs page or readable in `packages/json/lib.lume`. Having the package's
source was what made exact expected output possible: `json.md` does not
give the parser's error texts or the rule that a whole number prints
without `.0` in `pretty` as well as `show`, but the source does. Three
things would have saved me looking:

- `json.md` could show one optional field with a default. It is the most
  common thing a settings file needs. The package has `get` + `int`, but
  no `Json?` → `Int?` step (an optional has no `flat_map`), so each
  program writes its own `present`/`int_or` helper, as a01 does.
- `io.md` could say what a non-directive character in a pattern does (my
  guess 1), and whether directives read exactly two digits.
- `json.md` could say whether `import json.ToJson` is needed (guess 4).

Five guesses in total. None were about the language's core rules.

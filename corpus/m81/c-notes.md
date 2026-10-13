# Reviewer C — notes

## What I read

- `corpus/m81/BRIEF.md`
- `docs/README.md`, `docs/tour.md`
- `docs/reference/`: `strings.md`, `collections.md` (all of it, including
  "When a list is copied, and when it moves"), `methods.md`, `printing.md`,
  `numbers.md`, `control-flow.md`, `patterns.md`, `errors.md`,
  `structs-and-enums.md`, `functions-and-blocks.md`, `interfaces.md`,
  `names.md`, `operators.md`, the first half of `generics.md`, the
  "Constants" section of `modules.md`, and the `File` part of `io.md`
  (for what `lines` does with a trailing newline).
- `examples/`: `split_slices.lume` and `.expected`, `moves.lume`,
  `moves_values.lume`, `generators.lume`, `stdlib.lume` (first 150 lines),
  `deps/main.lume`, `logq/main.lume`.

Nothing else: no compiler source, no `tests/`, no other corpus folder, and
nothing was run.

## The programs

| file | what it is | leans on |
|---|---|---|
| `c01_markdown.lume` | Markdown to plain text: headings underlined, bullets and numbered items re-marked, quotes, emphasis stripped; a table of contents and a map of headings by level | generator over `lines`; list patterns with literals, guards and `..rest` on `split(" ")`; enum payloads built from pieces of a line; one bound name kept in a list and a map of lists; a `var` line replaced while a word split from it is held |
| `c02_assembler.lume` | two-pass assembler (labels, symbol table in a map, operands as numbers or labels) and the accumulator machine that runs the result; five broken programs | comments cut with `split(";").at(0)`; bare `split` read by position; a `.slice` of an item kept as a map key that outlives the text; `match` on text with `\|`; `T or Error` through both passes; `.or_error` on a lookup; `var self` writing into a list field |
| `c03_reconcile.lume` | bank statement against ledger, two CSV tables joined on `ref`: matched, differing, only on one side, totals, and four rejected files | header names kept as every row's map keys; fields copied into maps then structs; cents read with `split(".")`; `?`, `.or_error`, `map_error`; a generator joining two maps; `group_by`; a `var` CSV replaced after the table is built |
| `c04_logs.lume` | access-log analyser: per-minute buckets with gaps filled, top-N paths with averages, status classes, slowest request; bad lines warned about | position-only `split(" ")` with an item named and `.slice`d into `to_int`; a clock item split again; an interface with a default held in `[Report]`; `group_by` + `sort_by` on a tuple key + `take`; `match` on `Int` ranges; `warn`; a `var` line replaced while its pieces are read |
| `c05_sheet.lume` | mini spreadsheet: `A1=3`, `B1=A1*2`, precedence of `*`/`/` over `+`/`-`, cycles, empty cells, division by zero, bad tokens; a grid | `split("=")` items trimmed and kept twice; a `chars` tokenizer that hands a `var` word to an enum payload and then resets it; `Char` matched against literals; mutually recursive methods with `T or Error`; 3-tuple result taken apart; `pad` |
| `c06_make.lume` | Makefile-like resolver from `target: deps` lines: build order, sources, what a changed file rebuilds; cycle, missing rule, bad line, duplicate target | comment cut with `split("#")`; `split(":")` taken apart by a list pattern, the target kept as map key and list item; depth-first walk via a method with a `var` list parameter; generator methods with a `var` set; a `var` spec replaced after parsing |

## Guesses

1. **c01_markdown** — that `lines` keeps an empty line between two line
   breaks: `"a\n\nb".lines` is `["a", "", "b"]` (three lines, as Rust's
   `str::lines`). The two blank lines in the rendered output depend on it.
   Looked in `strings.md` ("`split`, and getting the string back": only
   `"a\nb\nc"`, `"a"` and `""` are shown), `methods.md` (`lines`: "the
   lines"), and `io.md` (only that a trailing newline gives no extra line).
2. **c04_logs** (and c03_reconcile) — that `to_int` reads a leading zero:
   `"09".to_int` is `Ok(9)`, `"01"` is `Ok(1)`, `"00"` is `Ok(0)`. c04's
   minute numbers and c03's `"1200.00"`, `"-60.00"`, `"-800.00"` cents
   depend on it. Looked in `numbers.md` ("Converting": sign and spaces
   allowed, `_` and `2.5` refused; nothing about leading zeros) and
   `errors.md`.
3. **c02_assembler** — that inside a `var self` method a list field can be
   assigned by position, `mem[x] = acc`. Looked in `collections.md` (`ys[0]
   = 9` on a `var` local), `structs-and-enums.md` ("`var self`": a field
   replaced whole, `log.push`, `for var` over a field) — position
   assignment on a field is not shown.
4. **c06_make** — that a *method* may have a `var` parameter
   (`def visit(name: Str, path: [Str], var done: [Str])`) and pass it on to
   itself recursively, `visit(d, path + [name], done)?`, with the changes
   reaching the caller's `var done`. Looked in `functions-and-blocks.md`
   ("Changing an argument: `var` parameters": shown only on a free
   function called once from `main`) and `interfaces.md` (`var x: T` on a
   generic function).
5. **c05_sheet** — that a three-part tuple is taken apart by assignment,
   `(sheet, order, problems) = read_sheet(TEXT)`. Looked in
   `structs-and-enums.md` ("Tuple destructuring": only two parts shown,
   "on anything that gives a tuple") and `methods.md` (three-part tuples
   are taken apart by blocks and `for`, not shown for `=`).

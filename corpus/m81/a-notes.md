# Reviewer A — notes

## What I read

- `corpus/m81/BRIEF.md`
- `docs/README.md`, `docs/tour.md`, `docs/examples.md`
- `docs/reference/`: `strings.md`, `collections.md`, `printing.md`,
  `methods.md`, `errors.md`, `control-flow.md`, `patterns.md`,
  `structs-and-enums.md`, `names.md`, `functions-and-blocks.md`,
  `numbers.md`, `operators.md`, the first half of `concurrency.md`
- `examples/`: `split_slices.lume`, `generators.lume`, `moves.lume`,
  `moves_values.lume`, `stdlib.lume`, `sets.lume`, `collections.lume`
  (each with its `.expected`), `logq/main.lume`, `csvq/table/lib.lume`
- `examples/errors/`: the file listing, and `temp_mutation`,
  `push_immutable`, `reassign`, `var_hides_outer`, `index_assign_str`,
  `list_pattern_on_str`, `for_var_string`, `map_error_gives_str`,
  `sum_of_text` and the `generator_*` cases, for the exact shape of an error
- `grep` over `examples/` for a `push` in a one-line `match` arm (found in
  `examples/match/main.lume`) and for "unused" warnings in the docs (none)

Nothing was compiled or run. I used a small shell `python3` one-liner as a
calculator to lay out padded table columns and to count the caret column
in a09. It never touched Lume.

## The programs

| file | what it is | leans on |
|---|---|---|
| `a01_csv_report.lume` | sales CSV into structs, revenue by region, bad rows reported with line numbers | `split(",")` read by position, header names as map keys, `map_error` block reading a field, `?`, `group_by`, `max_by`, `pad`/`pad_right`, `var` text replaced after load |
| `a02_log_summary.lume` | log summariser: a generator yields `Event`s from lines; tallies, error minutes, repeats, busiest minute, logins | generator over `lines`, split items in struct fields, `.slice` as a `group_by` key, tally map, set `add`, `max_by` on a map, list patterns with text literals |
| `a03_ini_config.lume` | INI parser into `[Section]`, lookups, a broken file | `var` name set from `.slice` of the line, `s[a...b]`, `.trim`, `index_of`, map in a struct field then replaced, `?` on `T?` and `T or Error`, `.or_error` |
| `a04_word_index.lume` | word -> line numbers index with cleaning, counts, multi-word queries | bare `split`, cleaned word used as a key twice, map of sets through missing keys, `intersect`, `sort`, map `filter`/`keys`/`max_by`, tuples of split items |
| `a05_fixed_width.lume` | fixed-width records by column offset, re-rendered as an aligned table with a total | `.slice` offsets, `.trim.to_int`, `map_error`, price split on ".", `decimals`, widths over `[[Str]]`, `var` line changed after pieces were cut |
| `a06_template.lume` | `{{name}}` template filler with values from `key: value` lines | `var rest` replaced by a slice of a slice of itself, `index_of`, `.or_error` + `?`, `while true` / `break` in a `match` arm, `Ok`/`Error` printing |
| `a07_word_diff.lume` | diff of two word lists, then a line-by-line diff | bare `split` of two `var` texts replaced while in use, an item given to `spawn:`, sets `diff`/`intersect`, `zip` + `enumerate`, tuple of optionals matched with a guard |
| `a08_passwd.lume` | `/etc/passwd`-style parser joined with a group file | seven-item list pattern over `split(":")`, catch-all binding the split, GECOS re-split, `split("/").last`, one split item as map key and value, `partition`, `group_by`, `for ... where` |
| `a09_generator_field.lume` | **FAIL**: a line generator stored in a struct field `Iterator[Str]` | the rule that a generator cannot be held in a field |

## Guesses

1. **`?` straight after a call that ends in a `{ }` block** (a01
   `parse_sale`, a03 `get`, a05 `parse_row`, a08 `parse_user`), as in
   `x.to_int.map_error { |e| Error(..) }?` and `secs.find { |x| .. }?`. I
   assumed it parses and applies to the whole call. Looked at:
   `operators.md` (postfix `?` is at level 1) and the `map_error` examples
   in `errors.md`/`methods.md`. None of them puts a `?` after the `}`.
2. **Assigning by index as a one-line `match` arm** (a06 `parse_vars`):
   `Some(i) -> vars[k] = v`. `patterns.md` ("What an arm can do") says an
   arm can be "an assignment", but its only example is `done += 1`.
3. **`pairs = {}` re-assigning an empty literal to an existing typed `var`**
   (a03). `collections.md` ("Where `[]` and `{}` need a type") lists a typed
   binding, a field, an argument and `.or([])`. It does not mention
   re-assigning to a `var` whose type is already known. I assumed that is
   accepted too.
4. **A `T?` value as the last line of a `-> T?` function passes through
   unwrapped** (a03 `get`, which ends `found.pairs[key]`) rather than being
   wrapped in another `Some`. `errors.md` says this for `T or Error` only
   ("a value that is already a `T or Error` passes through"). Its `T?`
   examples return a plain `T`.
5. **What `lume check` prints for a09.** I assumed it prints only the error
   block, in exactly the shape of `examples/errors/generator_as_interface.expected`,
   with `Iterator[Int]` replaced by `Iterator[Str]` (the field's type), with
   the path as given on the command line, with the caret on the generator
   call inside the constructor, and with no summary line after it. I also
   assumed this error comes first and no other error is reported, for
   example about the struct with an `Iterator[Str]` field getting printing
   or `==`. Looked at the `generator_*` files in `examples/errors/` and
   `collections.md` ("Generators: `yield`").

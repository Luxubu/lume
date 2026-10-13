# Reviewer B — notes

## What I read

- `corpus/m81/BRIEF.md`
- `docs/reference/collections.md` (all of it, and closely "When a list is copied, and when it moves" and "Generators: `yield`")
- `docs/reference/strings.md` (all of it: `slice`, `split`, `lines`, ordering, `to_int`)
- `docs/reference/printing.md`, `docs/tour.md`
- `docs/reference/names.md` (binding again, self-transforms), `docs/reference/patterns.md` (list patterns on `split`, `Str` literals)
- `docs/reference/functions-and-blocks.md` (kept blocks copy what they use)
- `docs/reference/concurrency.md` up to `shared var` (what a `spawn:` copies, `await` on a list of tasks)
- `docs/reference/methods.md` (every list, map, set, optional and result method)
- `docs/reference/control-flow.md`, the first half of `docs/reference/errors.md`, the struct, `==` and printing parts of `docs/reference/structs-and-enums.md`, the `to_int` part of `docs/reference/numbers.md`, a grep of `docs/reference/operators.md` for `==`
- `examples/split_slices.lume` and `.expected`, `examples/moves.lume`, `examples/moves_values.lume` and `.expected`, `examples/generators.lume` and `.expected`
- `examples/errors/`: `reassign`, `immutable`, `push_immutable`, `generator_used_twice`, `typed_rebinding`, `var_hides_outer`, `nonexhaustive_nested` (for the gutter width at line 10 and up), `spawn_changes_copy`, `loop_no_items`, plus a few others to learn the error format
- A grep of `examples/` for list `==` (`generic_interfaces.lume`, `sets.lume`)

## The programs

| file | probes |
|---|---|
| `b01_var_source_grows.lume` | `var` line grown in place by `line = line + parts.at(1)` while the list and a bound item are still read, then replaced; a `var` replaced each time round a `while` by a slice of itself, with that round's list read after the replacement |
| `b02_source_kept.lume` | source pushed into a list before its items are read; source given to a function that keeps it in a struct; source and an item in one constructor, keywords out of order; a local line returned in a tuple beside its last item |
| `b03_list_escapes.lume` | the list returned after being read by position; pushed into a list of lists; stored as a map value under its own first item; `==` against a literal; item `==` item; `for`; a block; `sort`; printed whole |
| `b04_item_kept.lume` | item of a `var` line reassigned straight after; item added to a set (twice, via two routes), used as map keys, put in a field, given to a function that keeps it, pushed; item returned from a function |
| `b05_item_uses.lume` | items self-transformed (`a = a.trim`), interpolated, matched (alternatives, `""`, binding catch-all), given to a `Str` function, `==` and `<`, used as map lookups, the same item twice in `+` and in a tuple |
| `b06_item_block_spawn.lume` | item captured by a kept `do` block, the list read in a kept `{ }` block, item and list read by one-line `spawn:`, the `var` line reassigned before any runs; list mapped into tasks |
| `b07_slice_reads.lume` | `.slice` on items read by `.to_int`, `.len`, `.upcase`, `==`, interpolation, kept in a list; past-the-end offset and count, zero count, negative offset, negative count; accented text counted in characters; failing `.to_int` message with an accented slice |
| `b08_nested_split.lume` | an item split again and read by position, `.to_int` on an inner item, an inner empty item; inner item as a map key while the outer list is indexed in a loop; inner split taken apart by list patterns with `..rest` interpolated |
| `b09_generator_lines.lume` | generators that `lines` then `split` and yield an item or a tuple; a `var` text reassigned after the generator call but before the walk; a generator method over a field reassigned after the call; yielded items pushed into a list |
| `b10_result_items.lume` | items in a `T or Error` function: trimmed into a struct, `?` on `.to_int` inside a constructor, line in an `Error` message; an item handed back through another function with `?`; a struct parsed from a `var` line reassigned afterwards |
| `b11_tally_items.lume` | items into a set and as map keys/values (a key set twice keeps its place); `group_by` on an item of a split in the block; items `map`ped to a list, then `uniq`, `sort`, `to_set`, `max_by` |
| `b12_field_source.lume` | a struct field split (not a local), an item named, then the field of the `var` struct reassigned while list and item are read; a method splitting its own field; a copy of the struct changed |
| `b13_buffer_rebuilt.lume` | a `var` buffer outside a loop set from each chunk, split, an item named, then the buffer replaced by `buf.upcase` before the item and list are read; the item as a map key; `lines` read by position with a computed last index |
| `b14_item_rebind_fail.lume` | FAIL: a name bound to an item bound again by `w = "<" + w` (not a self-transform) |
| `b15_generator_twice_fail.lume` | FAIL: a generator that splits lines, used up by `.to_list` and then asked for `.len` |

## Guesses

1. `b07_slice_reads.lume`: `date.slice(0 - 3, 2)` gives `"20"`. I assumed a negative offset is clamped to 0, as "Both arguments are clamped, so nothing is ever out of range" suggests, and does not count from the end. I looked in `docs/reference/strings.md` ("`slice(a, b)` is offset and *length*"), `docs/reference/methods.md` (the `Str` table) and `docs/reference/numbers.md`. The examples there only show positions past the end and a zero count, never a negative one.
2. `b07_slice_reads.lume`: `date.slice(2, 0 - 1)` gives `""`. I assumed a negative count is clamped to 0, so the program neither stops nor reads backwards. I looked in the same places as guess 1, and none of them shows a negative count.

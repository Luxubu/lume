# Benchmarks

Five programs, each with the Rust a person would have written sitting beside
it. Run them with `tests/bench.sh`, which builds both sides, takes the best of
five, and compares the ratio against `BASELINE`.

| | Lume | Rust | measures |
|---|---|---|---|
| `b1_generic_blocks` | `.lume` | `b1.rs` | generic helpers taking blocks, over 2M ints |
| `b2_interface_values` | `.lume` | `b2.rs` | 4M interface values dispatched in a loop |
| `b3_wordcount` | `.lume` | `b3b.rs` | a million words into a `{Str: Int}` |
| `b4_text` | `.lume` | `b4.rs` | 200k lines built with interpolation and `join` |
| `b5_printing` | `.lume` | `b5.rs` | 200k lines printed |

`b3c_split_only.lume` is `b3` without the map, so the map's share can be told
apart. `b3.rs` is word count with borrowed keys — not the comparison, since
Lume's `{Str: Int}` owns its keys, but a useful floor.

Each program times the work itself and prints `<n> ms` as its last line, so
process start and the file read are outside the measurement.

`python3 gen.py` writes `words.txt` (6 MB), which is not checked in.
`tests/bench.sh` runs it when the file is missing.

Two things to know before reading a number:

- **Build with `lume build`, not `lume run`.** `lume run` compiles your
  program at `opt-level = 1` on top of optimised dependencies, which is what
  makes edit-and-run take 0.2 s. On word count that is 11 ms against 9 ms.
- **Overflow checks stay on** in a built binary, as milestone 15 requires.

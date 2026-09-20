# The JSON program

`json.lume` is a JSON parser, printer and query tool written in Lume as a
real program, not a demo: it handles every escape including surrogate pairs,
reports the position of an error, prints compactly and indented, and keeps
its own tests beside the code. It was written during milestone 19 to find
out what the language is missing — the workarounds it forced became the
milestone's fix list.

```
lume run  examples/json/json.lume     # parse, print, query, stats
lume test examples/json/json.lume     # the tests at the bottom of the file
```

## Speed

`bench.sh` times the same algorithm — a cursor over the characters, an enum,
recursive descent — in three languages: `json.lume`, `json.rs.reference`
(hand-written Rust) and `json.py.reference` (pure Python, no `json` module).
On 867 KB of generated JSON (11,774 objects, best of ten runs):

| | parse | print |
| --- | --- | --- |
| Lume | 13 ms | 21 ms |
| Rust (the same algorithm by hand) | 10 ms | 16 ms |
| Rust, but with `[String]` instead of `[char]` | 68 ms | 16 ms |
| Python | 135 ms | 76 ms |

The third row is what Lume compiled to before this milestone: without a
`Char` type, `s.chars` was a list of one-character heap strings. That one
missing type cost 5×, and no amount of code generation would have recovered
it — which is why `Char` exists now.

To run it yourself: `cargo build --release` in a copy of `json.rs.reference`,
then `bench.sh` beside the binaries.

# Reviewer B — the edges of `json` and of dates, and their messages

## What I read

- `corpus/m63/BRIEF.md`.
- `docs/reference/json.md`, all of `packages/json/lib.lume` (the whole
  package, including its tests), `packages/json/lume.toml`.
- `docs/reference/io.md`, the `Time` section and "Dates"; `examples/dates.lume`
  and `.expected`.
- `docs/reference/packages.md` (the orphan rule, the naming of package items),
  `docs/reference/printing.md`, `docs/reference/errors.md` (first half),
  `docs/reference/numbers.md`, the parts of `strings.md` and `collections.md`
  on indexing, slicing, `to_char`, raw strings; `docs/install.md` (commands,
  `lume check`); `docs/reference/control-flow.md` on `for (a, b) in`.
- `examples/json_config/` (main, expected, team.json), `examples/check_json.expected`.
- In `examples/errors/`: `UNREACHED.md`, every `time_*` example,
  `iface_missing`, `iface_mismatch`, `generic_bound_unmet`,
  `iface_generic_unmet`, `extend_*` (interface, unknown, bound, generic
  missing), `option_method`, `method_on_result`, `result_method`,
  `missing_arm`, `partial_arm`, `nonexhaustive_nested`, `none_arith`,
  `wrong_enum`, `pattern_no_variant`, `builtin_arg_type`, `sort_without_lt`,
  `for_three_names`, `process_no_try`, `map_error_on_option`, and
  `packages/orphan/` (all its files).
- A search of all of `docs/` and `examples/` for other uses of
  `Time.parse`, `Time.date`, `Time.format` (there are none beyond the above).

## The programs

| file | what it probes | |
|---|---|---|
| `b01_need/main.lume` | `need_int` on 2.5, `"7"`, 7.0, -3, 1e20, null, array, object, bool, missing key; `need`/`need_int` on an array, a string, `Json.Null`; `need_items`, `need_bool`, `need_str` on the wrong kind | RUN |
| `b02_int_path/main.lume` | `int` on 1e20, 999999999999999, 1e15 and how `show` writes them; `path` out of range, `-1`, key on an array, `" 1"` and `"+1"` as indices, step past a number; `at` on an object; a key written twice | RUN |
| `b03_parse_errors/main.lume` | 18 texts: trailing commas in array and object, bad escape, lone high and low surrogates, high surrogate + plain `A`, unterminated string, empty and blank text, `-`, `1e`, `tru`, missing `,`/`:`, trailing characters, `01`, a good pair, short `\u` — each with its exact message and position | RUN |
| `b04_to_json/main.lume` | a struct's `to_json` inside a list and a list of lists; `[[Str]]`; floats; an object with a quote and a tab; round trip; `pretty` of a nested list | RUN |
| `b05_orphan/main.lume` | `extend Str with json.ToJson` in your own package: the orphan rule | FAIL |
| `b06_not_tojson/main.lume` | a struct without `to_json` passed as a `ToJson` | FAIL |
| `b07_format_directive.lume` | `Time.format` with `%y` (strftime's two-digit year) | FAIL |
| `b08_get_then_int/main.lume` | `doc.get("age").int`: a `Json?` used as a `Json` | FAIL |
| `b09_date_is_result.lume` | `Time.date(...) + 3600` without unwrapping | FAIL |
| `b10_match_json/main.lume` | a `match` on `Json` missing `Obj` | FAIL |
| `b11_dates.lume` | `Time.date` on 29 Feb in 2024, 2023, 1900, 2000, 2100; 30 Feb, 31 Apr; times before 1970 formatted and split into parts; `Time.weekday` on the Moon landing, 1 Jan 2000, either side of 1970 | RUN |
| `b12_parse.lume` | `Time.parse` with `%j`, month names in lower/upper case, a weekday name, trailing text, `%s` below zero, 1900, 29 Feb 2023, `%%` | RUN |

## Guesses

The json package's behaviour could nearly always be worked out, because its
source is under `packages/`; what it rests on in the language was sometimes not
documented. Dates were the opposite: `io.md` gives the directives and one
example of each message, and nothing about the edges.

1. **Negative list index** (b02, `xs.-1`). `"-1".to_int` is `Ok(-1)`, so
   `path` does `items[-1]`. `collections.md` says an out-of-range position gives
   `None`, but not whether `-1` counts as out of range or means the last item
   (as in Ruby). I assumed `None`.
2. **`"1e".to_float`** (b03). `numbers.md` says `to_float` "takes anything that
   looks like a number, including ... an exponent"; not whether a bare `e` with
   no digits is refused. I assumed an `Error`, so `bad number `1e` at position 2`.
3. **`"01".to_float`** (b03). Not documented. I assumed `Ok(1.0)` (Rust's
   parse accepts it), so the package accepts `01` as JSON.
4. **`56320.to_char`** (b03, `"\udc00"`). `strings.md` says "not every number
   is a character" but does not say which ones. I assumed a surrogate gives
   `None`, so the result is `bad code point 56320`.
5. **List ranges clamp** (b03, `"\ud800"` and `"\u12"`). `strings.md` says
   string ranges are clamped; for lists, `chars[a...b]`, nothing says so. The
   package's `short \u escape` check only works if they clamp, so I assumed
   they do.
6. **`Json.Null` from outside the package** (b01). `json.md` never builds a
   `Json` by its variant. The package's own test writes `Json.Num(1.0)`, so I
   assumed `Json.Null` works after `import json.Json`.
7. **The json package gives no warnings under `lume check`** (every FAIL
   package). `install.md` says `lume check` always shows warnings. No
   `lume check` output for a program using `json` is shown anywhere, so I
   assumed the error is the only thing printed.
8. **The orphan rule is reported, not an overlap** (b05). The json package
   already has `extend Str with ToJson`, so my `extend` both breaks the orphan
   rule and overlaps. Nothing says which one is reported. I assumed the
   orphan rule, with the wording of `packages.md` and the names changed
   (`Str` is "built in").
9. **How an imported interface is named** (b06). With `import json.ToJson`
   and `x: ToJson`, I assumed the `iface_missing` message says `ToJson`, not
   `json.ToJson`, and that the help shows the signature as
   `def to_json -> Json`.
10. **How an imported type is named in messages** (b08 `Json?`, b10 `Json`).
    `packages.md` says a *value prints* by its bare type name. I assumed
    messages name it the same way.
11. **Bare variant patterns across a package** (b10). `json.md` says "match on
    it", but shows no match. I assumed that `Null`, `Num(n)`, `Str(s)`, ... work
    as patterns after `import json.Json`, and that the pattern `Str(s)` is not
    confused with the type `Str`. If they don't, b10 fails with a different
    error.
12. **Times before 1970** (b11, b12). `io.md` says "seconds since 1970"
    and nothing about negative values. I assumed `format`, `year`...`second`,
    `weekday`, `day_of_year`, `Time.date` and `%s` all round towards the past
    (floor), giving 1969-12-31 23:59:59 for -1 and a Wednesday.
13. **The message for other months and days** (b11). The only example is
    ``February 2023 has days 1 to 28, not 29``. I extended it to
    ``February 2024 has days 1 to 29, not 30`` and
    ``April 2023 has days 1 to 30, not 31``.
14. **`%j` in `Time.parse`** (b12). The help lists `%j` among what
    `Time.parse` knows, but nothing says what it does there. I assumed it sets
    the date from the year (`"2024 060"` gives 29 February 2024) and that it
    reads three digits with the zeros in front.
15. **Month names in any case** (b12). I assumed `dec`, `DEC` and `december`
    are read like `Dec` and `December`.
16. **Trailing text** (b12). I assumed `"2024-09-29 extra"` against
    `%Y-%m-%d` is an error, not a success that ignores the rest. The reason
    after the colon is a pure guess:
    ``unexpected ` extra` at position 10``.
17. **`%s` below zero** (b12). I assumed `"-1"` is read as -1.
18. **A date that does not exist, in `Time.parse`** (b12). The example shows
    only the month-13 reason. I assumed the day reason is the same as
    `Time.date`'s: ``... does not match `%Y-%m-%d`: February 2023 has days 1 to 28, not 29``.
19. **What a pattern leaves out** (b12). I assumed missing fields default to
    1 January at 00:00, for `"%Y %j"` and `"100%% %Y"`.
20. **`%A` in `Time.parse`** (b12). I assumed a weekday name is read. The one I
    wrote is right for the date, so whether a wrong one is refused was not tested.

**20 guesses.** Not guesses: every JSON error text and position in b01–b03
(worked out from `lib.lume`), the message shapes in b06–b10 (copied from
`examples/errors/` with the names changed), and the float printing in b02/b04
(`numbers.md`).

## Verdict

The error examples now cover almost everything I needed. Every FAIL program
except the naming questions (8–11) was copied from an example. The date docs
are where the guesses pile up. Nine of the twenty guesses are about
`Time.parse` and times before 1970, and `io.md` could answer all of them in
one paragraph: case, trailing text, defaults, `%j`, negative times.

Things I found in the json package while working out its outputs. Each is
shown by one of the programs if my reading is right:

- **`need_int` on `1e20` says "`e` is a number, not a whole number".** That
  is false: 1e20 is whole, just too big. The same happens at exactly `1e15`
  (b02: `int` is `None`). The limit (`abs < 1e15`) is not in `json.md`, and
  the message should say "too large".
- **`show` writes `1e15` as `1000000000000000.0`** (b02). `json.md` says "A
  number with no fraction is written without one". That is true only below
  1e15. Above 1e16 it writes `1e20`, which is valid JSON but not what the doc
  promises.
- **A high surrogate followed by any `\u` is accepted.** `"\ud800A"`
  parses to U+2441 `⑁` instead of giving `lone surrogate` (b03). The low half
  is never checked to be 0xDC00–0xDFFF. This is a real bug.
- **A lone low surrogate's error has no position.** It says `bad code point 56320`.
  `hex_value`'s `bad hex digit` is the same, and so is a high surrogate
  followed by a short `\u`. Every other parse error says "at position N".
  They skip `fail`.
- **`01` is accepted** (b03), though `lib.lume` says RFC 8259, which forbids
  leading zeros.
- **`quote` escapes only `"`, `\`, `\n`, `\t`, `\r`.** A string holding
  U+0001, `\b` or `\f` is written raw, which is not valid JSON. I read this in
  the source and did not write a program for it.
- **`path` takes `" 1"`, `"+1"` (and maybe `"-1"`) as positions** (b02),
  because `Str.to_int` trims and takes a sign.
- **"`f` is a null"** reads oddly. "`f` is null" would be better.
- A key written twice is kept twice. `get` gives the first, and `show`
  writes both.

# csvq — what writing it found

`csvq` is a CSV query tool in three packages: `csv` reads the text,
`table` lays rows out in columns, and `csvq` is the program and its
query language. It was the first program written after packages existed
(milestones 47–49), to see what building from packages is like in daily
use.

    cd examples/csvq/csvq
    lume run -- data/people.csv where age ">=" 70 sort age desc show name,age
    cat data/people.csv | lume run -- - where city = Boston count
    lume test

## Packages

Nothing got in the way. Each library was written and tested on its own
(`lume test` in its folder), then used from the program by name. `lume fmt`
and `lume build` with no file named did what they should.

## What was wrong, and is fixed

1. **`xs.enumerate.map { |i, x| ... }` over a list of text did not compile**
   (a rustc leak). A two-name block over `enumerate` assumed the pairs were
   borrowed, but they are values holding a borrow. `enumerate`'s pairs are
   now plain values, copied out of a borrowed list as `.to_list` would copy
   them, so every block, loop and chain step reads them the same way.
2. **`sort_by` worked out the key twice per comparison.** Sorting 300,000
   rows by a parsed number made about eleven million `String` copies and
   float parses. Each key is now worked out once per item, as Rust's
   `sort_by_cached_key` does, and the sort stays stable: that sort went
   from 0.78 s to 0.31 s.
3. **`s += c` with a `Char` made a new string for every character.** It is
   now a `push`: reading the file went from 0.36 s to 0.21 s.
4. **`line.trim == ""` made a trimmed copy just to compare it.** A
   comparison now reads the trimmed slice.

## What was in the way, and now says so

5. **`for (i, w) in xs.enumerate`**, as Rust writes it, was "expected a
   loop variable". It is now accepted, and `lume fmt` writes it
   `for i, w in`.
6. **`"-" * w`**, as Ruby and Python write it, said only that `*` cannot
   combine a `Str` and an `Int`. The help now says `.repeat(w)`.
7. **`table.Right` for a variant of `table.Align`** listed the module's
   items. The help now says `write table.Align.Right`.

## What was in the way, and stays

- **`'"'` is not a character literal.** A one-character string such as
  `"\""` is a `Char` wherever one is wanted. The error says so.
- **A `match` spread over lines cannot go inside a call's brackets.** Bind
  it to a name first. The error says the arms go on the following lines.
- **`xs[0]` is a `T?`.** `.first.or_error("...")?` says what to do when the
  file is empty, which is what a CSV reader should say anyway.

## Speed

On a 300,000-row, 12 MB file, a query that filters twice, sorts by a number
and takes three rows runs in 0.23 s. Python's `csv` module, written in C,
does the same in 0.23 s. Before fixes 2–4 csvq took 0.69 s. What remains is
Lume's value semantics: `var rows = sheet.rows` and each `filter ...
.to_list` copy the rows. A Rust programmer would borrow them there.

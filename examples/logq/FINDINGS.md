# logq — what writing it found

`logq` answers questions about an access log: a summary with failure rate,
percentiles and the busiest minute, the busiest paths, the failed requests,
the slow ones, and everything since a time. It reads lines lazily with
generators (milestone 70) and was written to use them the way a real
program would.

    cd examples/logq
    lume run main.lume -- summary data/access.log
    grep /api data/access.log | lume run main.lume -- top 2 -

`check.sh` runs every command, a pipe and the errors on `data/access.log`, a
40-line sample with two lines that cannot be read. `gen_log.py N` writes a
log of N lines; the benchmark uses 500,000.

## What went well

Once it compiled it gave the right answers on the first run, checked against
a Python script over the same file. These carried it:

- a generator that yields `(Int, Entry or Error)` pairs, taken apart by
  `for n, read in` and a `match` on the result inside a second generator;
- `group_by`, `max_by` and `sort_by` on tuples for the reports;
- `Time.parse` and `Time.format` on the log's timestamps, and a bad `since`
  argument reported with where it stopped matching;
- laziness that shows: `logq top many FILE` fails on its argument before a
  line of the file is read.

## What was wrong, and is fixed

1. **A generator could not be given to a function** — not even to another
   generator. The program wants `entries(lines_of(text))` and
   `summary(entries)`, a pipeline, and the first version had to collect a
   list in `main` instead. Now **a parameter `xs: Iterator[T]` takes any
   sequence by move**, as Rust's `impl Iterator<Item = T>` does: a
   generator, a lazy chain, a list, a set, or a type with `next`. Inside, it
   is walked like a generator, and `xs.next` works on it. A chain into a
   generator is worked out first, since the generator outlives the call;
   into a plain function it stays lazy. See
   [Collections](../../docs/reference/collections.md#taking-a-sequence-xs-iteratort).
2. **A function ending in `Env.exit` had no return type** ("cannot work
   out what `usage` returns"). A function whose body ends by leaving the
   program now gives nothing, as Rust's `-> !` does.
3. **`at: Time` said "unknown type `Time`"**, with a list of built-in types.
   It now says `Time` is not a type and that a time is an `Int`, seconds
   since 1970 (`examples/errors/time_not_a_type`).
4. **It ran at 4.7 times the hand-written Rust.** Three things were cheap
   to fix:
   - `Time.parse` built a `String` for every number it read and copied the
     text to the heap; it now reads the digits as it goes, with short text on
     the stack: 221 ms to 82 ms for 500,000 timestamps;
   - `parts.at(4).to_int` copied the item out of the list before reading
     it; a text method on a list's item now reads it in place;
   - `Time.parse(parts.at(0), "...")` built the pattern as a new `String`
     on every call, and copied the item; both are passed as they are now.

   `bench/b8_logq` is now 3.3 times the Rust a person would write
   (310 ms against 94).

Milestone 72 took the next step: `parts = line.split(" ")` now holds slices
of `line` when `parts` is only read by position and `line` does not change,
so the six strings per line are gone. `bench/b8_logq` went from 3.0 to 2.2
times the Rust (205 ms against 93).

## Left as it is

- **What is left of the gap** is copies the program asks for: each entry's
  `path`, the `took` field copied out to be sliced, and `group_by` below.
- **`group_by` copies every entry into its group**, where the Rust counts.
  A Lume program that wants counts can count; this one, like most, groups.
- **A call needs `()`, even with no arguments** (`usage()`): the bare name
  was refused, with a message that says exactly that. This is the
  documented rule, so the program changed, not the compiler.

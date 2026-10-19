# Reviewer C, round nineteen

## What I read

- `corpus/m90/BRIEF.md`
- `docs/tour.md`
- `docs/reference/collections.md` (all of it, most closely "Generators",
  "Async generators", "Chains over an async sequence", "Taking a sequence")
- `docs/reference/concurrency.md` (all of it, most closely "Whichever
  finishes first: `next_done`", "A task that waits without `await`",
  "Where the warnings go")
- `docs/reference/interfaces.md` (all of it, most closely "An interface's
  own methods may use interfaces" and "Methods that take a block")
- `docs/reference/functions-and-blocks.md` (most closely "Blocks as values")
- `docs/reference/io.md`, `printing.md`, `strings.md`, `errors.md`,
  `structs-and-enums.md`, `methods.md`, `control-flow.md`
- parts of `patterns.md`, `numbers.md`, `names.md` (the keyword list),
  `operators.md` (`|>`)
- `examples/async_generators.lume` and `.expected`,
  `examples/generators.lume` and `.expected`, `examples/async.lume` and
  `.expected` (for the warning layout), `examples/generic_extend.lume` and
  `.expected`, `examples/kept_blocks.lume`

I read nothing else: no compiler source, no `tests/`, no `tools/`, no other
corpus folder (the a- and b- files in this folder were not opened), no
`.lume` folder. I did not run the compiler or any program.

## The programs

All are `Meant to RUN.`; each writes only under `/tmp/lume-review-c/`.

| file | what it is | leans on |
|---|---|---|
| `c01_scheduler.lume` | a job scheduler: shell, compute and nap jobs run by a pool of `at_once` tasks | `await running.next_done` in an async generator; `reject`/`map`/`take`/`enumerate` over it; a struct's async generator method walking another async generator; enum + `match`; `Process.run` from tasks; a map tally; a 250 ms vs 1 ms ordering |
| `c02_logpipe.lume` | a log pipeline: a log file written, read back, and its entries handed on one at a time | a struct's async generator methods; `filter`, `map`, `take_while`, `skip`, `take`, `enumerate`, `reject` chained over them with copied locals; an enum's async generator method; `T or Error` with `?`; `grep -c` through `Process.run`; `File`/`Dir`/`Path` |
| `c03_plugins.lume` | a plugin system: plugins are interface values (three structs, and `Str` by an `extend`) | generators that keep one interface value, a list of them and a block; an async generator that keeps one; a struct field of interface values read by an async generator method; an interface default that is a generator; the warning for a binding of interface values (predicted on stderr) |
| `c04_textkit.lume` | a text tool: tokens, word counts, runs of characters, digits | generators in `extend Str`, `extend [Str]`, `extend Int` and `extend Doc`; a generator in an `extend Str` that takes a block and keeps it (copied `var` changed afterwards); an interface default generator with an argument; `tr` through `Process.run`; a file read back |
| `c05_sim.lume` | a simulation of two particles bouncing between walls | generator methods of a struct (frames, enum events, a block-picked watch); an async generator method with `map`/`take_while`/`take`/`enumerate`/`skip` chains; a generator in an `extend` of the struct drawing frames; a generator method walking a copy taken at the call |
| `c06_index.lume` | a word index over files, one missing and one empty | a two-task read pool with `next_done` giving `Doc or Error` items; a map of lists and a map of counts; generators in `extend {Str: Int}` and `extend {Str}`; an async generator keeping a query's block, with `filter`/`map` over it; `warn` to stderr; `Process.run_in` with a pipe through `sh -c` |

## Guesses

1. **`.to_list` on an async sequence, followed straight away by a list
   method in the same expression** — `pool(jobs, 3).to_list.sort_by { .. }`
   (c01), `...map { .. }.to_list.to_set` (c02). The docs and
   `examples/async_generators.lume` only show `puts x.to_list` alone, and
   say the waiting methods go in async code. I took the result to be an
   ordinary `[T]`. Looked: collections.md "Async generators",
   examples/async_generators.lume.

2. **An async generator with no `await` in its body** — `LogFile.problems`
   (c02). "its body may `await`" reads as optional; no example has none.
   Looked: collections.md "Async generators".

3. **An async generator method on an enum, called on a bare variant** —
   `Warn.and_above` (c02), whose body calls the enum's own `rank` by its
   bare name. The brief says a method of a struct or enum; the docs'
   examples are structs only. Looked: collections.md "Async generators",
   BRIEF.md.

4. **The interface-binding warning, exactly** (c03): that `ps: [Plugin] =`
   gives the one warning, that `for p in ps`, the generator parameters and
   `Host(plugins: ps)` give none, that the path is printed as
   `corpus/m90/c03_plugins.lume` (how the runner names the file), and that
   a three-digit line number makes the gutter four spaces wide. The docs
   show one- and two-digit layouts only. Looked: interfaces.md (the
   `mixed` warning), concurrency.md "Where the warnings go",
   examples/async.expected.

5. **A generator default with a parameter of its own, called on an
   interface value taken out of a list** — `ps.at(0).outputs(lines)` (c03),
   and `ps.at(1)` passed to a parameter typed `Plugin` that a generator
   keeps. The docs' generator default (`numbered`) takes no argument and is
   called on a concrete value; they say a sequence-giving method stays
   usable through a pointer. Looked: interfaces.md "An interface's own
   methods may use interfaces", collections.md "Generators".

6. **A `Str` literal handed as an interface value to a generator that keeps
   it** — `tagged("dogs", lines)` with `extend Str with Plugin` (c03).
   Documented in pieces (extend on built-ins; generators copy interface
   values through `lume_box`), not together. Looked: interfaces.md,
   generators.lume (`loads`).

7. **A generator in an `extend` that takes a block** —
   `extend Str with Scan: def runs(keep: (Char) -> Bool) -> Iterator[Str]`
   (c04). interfaces.md says an interface method's block is lent as a
   `&mut dyn FnMut`, while collections.md says a generator keeps the block
   it is given. Which wins is not said; I assumed it works and keeps a copy
   of what the block uses (so `["ueue"]`, not `["q"]`). Looked:
   interfaces.md "Methods that take a block", collections.md "Generators",
   functions-and-blocks.md "Blocks as values".

8. **An interface default generator with an argument** —
   `longer_than(n: Int)` in `Tokens` (c04). Looked: interfaces.md.

9. **A struct's generator method that takes a block, with the block written
   after the call's parentheses and a method chained after the closing
   brace** — `world.watch(8) { |p| p.pos <= edge }.join(", ")` (c05). The
   docs show a generator keeping a block passed inside the parentheses
   (`keep_if([..], { .. })`), and methods taking trailing blocks; not the
   two together. Looked: collections.md "Generators",
   functions-and-blocks.md "Blocks of your own", examples/generators.lume.

10. **`extend` with concrete map and set targets** — `extend {Str: Int} with
    Ranked` and `extend {Str} with Listing` (c06). The docs show `[Int]`
    concrete, and `{T}` / `{K: V}` only as generic targets. Looked:
    interfaces.md, examples/generic_extend.lume.

11. **`yield Error(e.message)` in an async generator giving
    `Iterator[Doc or Error]`** (c06). Documented for generators ("Items
    that may fail"), not said for async ones. Looked: collections.md
    "Generators", "Async generators".

12. **The machine's tools** (c01, c04, c06): that `/bin/sh`'s `printf`
    turns `\n` into a newline, that `uniq -c` pads its count with spaces
    (trimmed away), that `tr a-z A-Z` and `sort` treat plain lower-case
    ASCII the same in every locale, and that `grep -c` prints the bare
    count. Not Lume; nothing in the docs can settle it.

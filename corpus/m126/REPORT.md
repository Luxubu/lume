# Milestone 126 review corpus — REPORT (twenty-fifth review round)

This round tested what came after 1.0, in milestones 122–125: serving
HTTP, a preview; the URL and form helpers, settled; lunches 12, 13 and 14,
split from one; a `for` over a list of tasks; what an optional or a result
says of a method it lacks; and, through whole programs, the chains
milestone 125 made faster, whose output must not change. Three reviewers
wrote thirty programs from `docs/`, `examples/` and `packages/` alone, with
nothing run. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday code: a JSON API, a search page on `params`, a form on `form`, the four helpers on awkward text, a library package and its tests, tasks awaited in a `for`, a request answered twice, two refusals |
| `b01`–`b15` | the edges: `decode` and `parse_form` on every odd input, a query on a query, a form after a body, a reply's type, `accept` after `close`, a taken address, `incoming` answered in tasks, header case, a nested list pattern, alternatives of two shapes, a `() or Error` match, a stray `Error`, a misspelt method, `lume fmt` |
| `c01`–`c06` | whole programs: a key-value store, a URL shortener, a to-do service, a webhook receiver, a log report, a library with modules and tests |

## The numbers

| | round twenty-four | round twenty-five |
|---|---|---|
| guesses | 42 | **35** (A 8, B 19, C 8) |
| meant to run, test or format, right on the first run | 22 of 24 | **22 of 24** |
| refused as predicted | 3 of 6 | **4 of 6**, word for word |
| rustc leak | 1 | **1** (c06) |
| wrong output | 0 | 0 |

Every program that uses the server — fourteen, each a server and its own
client — printed what its author predicted, as did every one on the
helpers. Of the two that did not run as predicted, one was refused when it
should not have been (a06) and one leaked (c06); of the refusals, one was
not refused at all (b13).

## What the programs found

1. **`xs[i % xs.len] = xs.at(i) + 1` reached rustc** (c06): the list was
   read, for the value and the position, while Rust held it for writing.
   Both are worked out first now.
2. **`await t` in a loop was refused, as `t` "is used again below"**
   (a06), when what was below was another `for t in ...`, which names a
   `t` of its own. A loop that names the name uses the outer one only in
   what it walks.
3. **A stray `Error(...)` at the end of an `if` in the loop that ends a
   `() or Error` function was taken as the function's value** (b13), and
   left the function at the first bad seat, unseen. Milestone 120 made
   such a line refused, and an `Error` that ends a branch at the function's
   end its value; a loop's body is not the function's end. It is refused.
4. **What a header named twice sends depended on its name** (B, from
   `packages/http` against `http.md`'s "one more header"): ureq sends the
   later of two in the same case, both of two in different cases, and both
   of any beginning `X-`. One rule now: of two with one name, in any case,
   only the later is sent, from a `Request` and a `Reply` alike. A
   request's `headers` keeps both, as it always has — a first try made the
   list drop the earlier one, and `corpus/m96/b14`, which reads the list,
   said that broke a settled promise.
5. **`.with_query` put the query inside a `#fragment`, and after a bare
   `?` added `&`** (B, b03): it goes before the fragment now, and after
   `?` or `&` adds nothing.
6. **Alternatives of two shapes were pointed at by their first**, with
   help about `Some("a")` whatever the shape (b11); the caret is on the one
   that differs, and the help is to give each an arm of its own.

b03 prints what the package does now, not what its author read in its
source; it was right about what it did.

## What the docs now say

- The tutorial's README numbered Part 2 and Part 3 as before the split,
  and lunch 21 sent tasks to lunch 16 across a line break, where the
  renumbering of milestone 122 did not look (A, C).
- Lunch 21, where `stability.md` sends readers for the helpers, teaches
  `.with_form`, `.with_query`, `http.encode` and `http.form`, in its
  checked program too (A).
- `methods.md`: the error for a method an optional or a result lacks,
  shown, and its opening says when each error comes (A, B).
- `concurrency.md`: the error for a list of tasks used after its loop,
  shown (A).
- `http.md`: the words for a status out of range, and that a refused answer
  sends nothing; that redirects are followed, up to five, and a `201`'s
  `Location` is not; that `path` comes as it was sent; that a `for` over
  `requests` ends at a `break`; what a decoding error names; a header named
  again; a query and a fragment (A, B, C).
- `testing.md`: a library's tests need no `main`; what `lume test` writes
  where (C).

Some of these changed while B and C were still writing: A finished first,
and its findings were fixed at once. B and C read the newer pages where
they read them after, and say so in their notes.

## Left

- `stability.md` opens with "Lume is at 1.0" beside parts of 1.1 (C);
  release 1.1.0, milestone 127, makes that true.
- `lume test` on a library package's `lib.lume` is how `a05` and `c06` are
  run; the suite finds corpus programs by their `main.lume`, so each has a
  check of its own in `tools/lumecheck/plan.lume`.

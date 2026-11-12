# Milestone 131 review corpus — REPORT (twenty-sixth review round)

This round tested milestones 128–130 on the way to 1.2: serving HTTP,
settled, and lunch 27 that teaches it; `examples/notes/`; `http.value`;
generators that are a loop around one `yield`, which run as plain
iterators now and must still do their work when the pages say; an `extend
[T: I]` on the wrong items; and a `() or Error` call ending a branch at a
function's end. Three reviewers wrote thirty programs from `docs/`,
`examples/` and `packages/` alone, with nothing run. The brief is in
`BRIEF.md`. This time no page changed until all three were done.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday code after lunch 27: a pantry service and its tests, a query, a JSON body and its 400s, two generators that print as they work, two refusals, a `match` of `() or Error` calls |
| `b01`–`b15` | the edges: statements after a `yield`, a set-up call put off, a `while` that fails at once, `next` past the end, two `yield`s, `take_while`, a generator over a `for`, generators used up twice, `http.value` and `close` and `204`, an `extend` bound on three kinds of list, branch calls kept and dropped |
| `c01`–`c06` | whole programs: a library-booking service, an event feed a generator pages, a stock service saved to a file, a static site from four generators, a rate limiter with an upstream of its own, a library package with modules and tests |

## The numbers

| | round twenty-five | round twenty-six |
|---|---|---|
| guesses | 35 | **34** (A 7, B 15, C 12) |
| meant to run or test, right on the first run | 22 of 24 | **22 of 23** |
| refused as predicted | 4 of 6 | **5 of 7**, word for word |
| rustc leak | 1 | **1** (c03) |
| wrong output | 0 | 0 |

Every generator printed its work where its author said it would — the set
up put off to the first item, what follows a `yield` run at the next, the
end found once — so the iterators of milestone 128 kept the coroutines'
order everywhere this round looked. Every server, the rate limiter's two
among them, answered as predicted.

## What the programs found

1. **A guard reading a name a list pattern inside another binds reached
   rustc** (c03): `("POST", ["items", sku, verb]) if verb == "take"`. Such
   names are bound at the top of the arm, after the guard has run; the
   guard binds them for itself now.
2. **A dropped failure in a `match` was pointed at the `match`, as "this
   can fail"** (b05, a09), when what is dropped is an arm's call: it is
   named, at the call, `` `save` can fail ``.

## What the docs now say

- `generics.md`: an `extend [T: I]`'s methods are not there for items that
  are not `I`, with the error (A).
- `errors.md`: at the end of a `() or Error` function a branch's last call
  is its value, and with a line after it is a dropped failure (A, B).
- `json.md`: a `Json` document is `ToJson` itself (C).
- `http.md`: what `req.json` gives for a body that is not JSON (A).
- `testing.md`: modules' tests run in the order the entry file imports
  them (C).
- Lunch 16: its "Underneath" named `naturals`, a generator of the reference
  page, not of the lunch, and said less than `collections.md` about which
  generators are plain iterators (B).

## Left

- `stability.md` opens with "Lume is at 1.1" beside parts settled in 1.2
  (B, C); release 1.2.0, milestone 132, makes that true.

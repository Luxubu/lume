# Milestone 90 review corpus — REPORT (nineteenth review round)

This round tested what milestones 87–89 added: chains over async
sequences, async generator methods, `await tasks.next_done`, generators in
an `extend`, and generators that keep a block or an interface value.

Three reviewers wrote thirty programs from `docs/` and `examples/` alone,
with nothing run. The brief is in `BRIEF.md`.

| set | reviewer aimed at |
|---|---|
| `a01`–`a09` | everyday code: a paged fetcher with chains, a worker pool with `next_done`, a feed reader as methods, a report from `extend` generators, a retry policy as a kept block, policies as interface values, endless sensor readings, two refusals |
| `b01`–`b15` | the edges: every chain, chains of chains, `take(0)`, `.next` on a chain, captures, methods on structs, enums and generics, `next_done` on an empty list, `extend` generators of a struct, `Str`, `[T]` and a generic, kept blocks and interface values, six refusals |
| `c01`–`c06` | whole programs: a job scheduler, a log pipeline, a plugin system, a text toolkit, a particle simulation, a word index |

## The numbers

| | round eighteen | round nineteen |
|---|---|---|
| guesses | 34 | **51** (A 6, B 33, C 12) |
| meant to run, right on the first run | 16 of 19 | **20 of 22** |
| refused as predicted | 9 of 11 | **6 of 8**, word for word |
| rustc leak | 1 | **1** (c04) |
| wrong output | 0 | 0 |

No program printed a wrong answer. All thirty now print what their authors
predicted, word for word — the first round where every prediction held after
the fixes, without a prediction having to be set aside as wrong.

## What the programs found

1. **An interface method that gives a sequence lent its block, and a
   generator implementing it keeps one** (c04: `def runs(keep: (Char) ->
   Bool) -> Iterator[Str]` in an interface, a generator in `extend Str`,
   reached rustc). The sequence is walked after the call, so the block must
   be kept, as Rust's trait method handing back a lazy iterator takes its
   closure owned. Such an interface method now keeps its block: the trait
   takes a kept block, every call through it hands one over, and a type's
   method that only uses the block during the call is refused with its own
   message. Checking this found two older gaps of the same kind, both
   reaching rustc before: a non-generator method building a chain over the
   block (now refused, `iface_seq_lends_block`), and a method keeping the
   block an interface lends it, in a field (now refused,
   `iface_block_kept`).
2. **A block of two names after `enumerate` on an async chain was refused**
   (b01: `naturals().enumerate.map { |i, n| i * n }`). A list's chain takes
   the pair apart; an async chain's block now does too.
3. **`.join` (and every list method) on an async sequence said only "has no
   method"** (b13), where a block-taking one said "has no `count` yet" and
   named the chains it has. Both now say the second.
4. **A `for` in a plain `def` over an async generator method** (b14) said a
   `for` over it waits, where the same loop over a function said the call
   starts a task. The call's message now comes first, as for a function.

## What the docs now say

- `collections.md`: a chain may be kept in a name and walked later, its
  block keeping the copies from when it was made, and a block may name the
  two parts after `enumerate` (A).
- `interfaces.md`: a method that keeps a block an interface lends is
  refused; an interface method that gives a sequence keeps its block, with
  an example (C).

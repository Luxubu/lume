# Writing a concurrent program in Lume — what it exposed

`examples/match/` is a parallel record matcher: parse records, block them so
only plausible pairs meet, fan the blocks out across tasks, gather what came
back, and report — including what failed. It is the shape every concurrent
program has, and it was written the way a person would write it, with every
workaround recorded here as it happened.

## Fixed in this milestone

1. **`await` on a task that is used again leaked to rustc.** `results = await
   tasks` followed by `tasks.len` emitted a defensive `.clone()`, and a task
   handle is the one thing that cannot be cloned. Awaiting the same task twice
   gave Rust's "use of moved value". Both are one missing rule: **a task is the
   work itself, so `await` takes it.** Now a Lume error that names the fix.

2. **A `T or Error` in statement position was silently discarded.** Not a
   concurrency bug, found while writing one, and the worst thing on this list:

   ```
   def risky(n: Int) -> Int or Error:
     if n == 3:
       Error("three is bad")    # evaporated — no `return`
     n * 10
   ```

   `risky(3)` gave `30`. The same for a call whose failure was ignored:
   `may_fail(-1)` on its own line kept going. You wrote the failure path and
   it vanished. Now an error.

3. **`shared var` on a struct field worked through a method but not through an
   assignment.** `s.hits += 1` where `hits: shared var Int` was refused with
   "`s` is immutable, so its field `hits` cannot be changed" — blaming the
   struct for the mutability that `shared var` exists to provide.

4. **A `shared var` map or list could not be looped over.** That is the report
   step of fan out → collect → report. It worked if you copied it out first,
   and the error did not say so.

5. **`pub X = 1` was rejected as `pub const X = 1`** with a message saying
   "`pub` goes before `def`, `struct`, `enum`, `interface`, `import` or a
   constant" — which is what had just been written.

## Noted, not fixed

- **No character literal.** `'?'` is a syntax error; a `Char` is a
  one-character double-quoted string, and only where one is wanted. So a
  `match` whose other arm gives a `Char` cannot answer `"?"` — it needs
  `f[0].or("?").to_s` and a `Str` on both sides. Deliberate (Lume has one
  string syntax), but it costs a line in the first program that indexes text.

- **`[.., last]` is not a pattern.** A list can be taken apart at the head
  (`[first, ..rest]`) and not at the tail, so "the last word" reverses the
  list. Worth having; the exhaustiveness checker is the reason it is not free.

- **A `for` cannot be an inline `match` arm body.** `Ok(ps) -> for p in ps:
  pairs.push(p)` needs the indented form. Consistent with the rest of the
  language; still the first thing a person reaches for.

- **`puts (await t)?`** is rejected for the space before `(`. The rule is
  sound and the message is good, but this is a natural line to write.

- **List indexing gives `T?`.** Correct, and it means arithmetic over a table
  — the edit-distance matrix in `scoring.lume` — is written with `.at(i)`
  throughout. The alternative is worse; noting it because it is the one place
  Lume reads heavier than the language it is imitating.

- **`Error(e) -> Error(e)`** says "`Error` field `message` is `Str`, but this
  is a `Error`". The answer is `Error(e) -> e`, and the message does not say
  so.

## What held up

Everything concurrency was supposed to get wrong, it got right. Across the
probes and this program: `await [tasks]` gives results in spawn order, not
completion order; a task spawned inside a task works; 200 tasks incrementing
10 keys of one shared map lost no updates; two tasks locking two `shared var`s
in opposite orders did not deadlock, because each use takes and releases
rather than holding; a worker pool pulling from one shared queue processed
every item exactly once; and a task that returns `T or Error` carries its
failure home intact.

The design is sound. What it was missing was the rule that a task is used up
by waiting for it, and — everywhere, not just here — that an error you do not
look at is a bug.

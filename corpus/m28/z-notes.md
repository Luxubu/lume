# corpus/m28 — twelve whole programs, written blind from the README

Twelve programs of 50–130 lines each, written from `README.md` and the
top-level `examples/*.lume` only. I did not read the compiler, the tests, the
other corpora, or any example under a subdirectory, and I did not run
anything. Every expected output below was worked out by hand and checked
twice.

The brief was *whole realistic programs*, so each one has a job to do and
reaches for whatever the job needs. Three (z09, z10, z11) deliberately aim at
the corners I found least comfortable while reading. One (z12) is meant to be
rejected.

## 1. What each program probes

| # | Program | Runs? | What it leans on |
| --- | --- | --- | --- |
| z01 | `z01_ledger.lume` | runs | Enum with data driving a report; `match self` one-liner methods; `Variant(..)`; `enumerate` into a struct; `{Str: Int}` tally read back in insertion order; `sort_by(…).last` matched as a `T?`; `sum` with a block over a lazy `filter`; four-column `pad`/`pad_right` alignment; integer money |
| z02 | `z02_inventory.lume` | runs | One interface with a default satisfied both by a struct (structurally) and by `Str` (via `extend`); a generic function bounded by that interface; `{Str: {Str}}` grown by the missing-key-starts-empty rule; `sort_by(…).reverse`; three different aligned layouts |
| z03 | `z03_tournament.lume` | runs | `interface Comparable[T]` with a default; a generic insertion sort bounded `[T: Comparable[T]]`; `at(i)` / `insert(at, x)`; `return` as an early exit inside a method; `Str` `<` as a tie-break; `for … where`; a ten-column table |
| z04 | `z04_log_analyser.lume` | runs | `T or Error` parsing with list patterns over `split` (no separator); collecting successes and failures side by side with `match Ok/Error`; four `{Str: Int}` tallies; a struct derived from map keys; `sort_by(…).reverse.take(3)`; `count` with a block; `s[0..1]` slicing to bucket by hour |
| z05 | `z05_ini_parser.lume` | runs | Enum of line shapes as `Line or Error`; nested patterns through the result (`Ok(Section(name))`, `Ok(Pair(k, v))`); `match` on the `Int?` from `index_of`; exclusive slicing; `Char` predicates (`alnum?`, `c == "_"`); `{Str: [(Str, Str)]}` and `m[k].push((a, b))`; `for k, v in …`; errors that carry a line number |
| z06 | `z06_scheduler.lume` | runs | Blocks of my own with two parameters, one a list (`(Int, [Slot]) -> ()`), called with `do \|i, r\|` and a nested `for` in the body; a generic helper handing its block to a built-in (`xs.map(show)`); a function returning a tuple of two lists, destructured; `[[Slot]]`; `Int.to_char`; a `while` driven by a rebound `var` |
| z07 | `z07_diff.lume` | runs | A real LCS diff: `[[Int]]` grid written with `grid[i][j] = v` and read with `at(i).at(j)`; descending `while` loops (no descending range exists); enum of changes with `\|` alternatives; `count` with predicate methods; integer percentages |
| z08 | `z08_route_planner.lume` | runs | Dijkstra: `{Str: [(Str, Int)]}` adjacency grown with `graph[a].push((b, w))`; `{Str}` visited set; `{Str: Str}` predecessors; a function returning a tuple of two *maps*, destructured; `for n, w in …`; `insert(0, x)`; `for … where` with a set test; a top-level constant as infinity |
| z09 | `z09_survey_sources.lume` | runs — **awkward 1** | Generic `extend`: `[T]`, `{T}`, `{K: V}` and my own `Panel[T]` under one parameterised interface; `report[T, S: Samples[T]]` where `T` lives only inside another parameter's bound; the same `[T]` and `{T}` conforming to a *second* interface (`Scored[T: Ordered]`) at the same time; a default written in terms of another default |
| z10 | `z10_marks_table.lume` | runs — **awkward 2** | `read_column[T](…, parse: (Str) -> T or Error)` — a block parameter that is both generic and fallible — fed four ways (bare value, already-a-result, named function, `do \|c\|` body using `?` partway through); `?` written straight after a `{ … }` block call; a block handed on unchanged to another function returning `([T], [T]) or Error`, destructured through `?`; a struct method that returns `T or Error` and uses `?`; `summarise[T, A]` with a two-argument block |
| z11 | `z11_doc_toc.lume` | runs — **awkward 3** | A `"""` block whose common indentation must be stripped; raw strings as both template and `replace` needle (`r"#{title}"` must not interpolate while `"#{h.level}"` beside it must); a multi-line `\|>` chain and `x \|> wrap(30)`; `Str +=` a `Char` at a time; `c == "#"`; `"  ".repeat(0)`; trailing `unless`; `T?.map { }.or(default)` |
| z12 | `z12_err_budget.lume` | **must be rejected** | One mistake in ~90 otherwise-valid lines: a subtotal declared without `var` and accumulated inside a nested `for` (line 84, declared line 82). Checks that the error is a Lume error, names `spent`, says to declare it `var` (or points at the declaration), and reports nothing else |

Deliberately **not** probed: `async`/`spawn`/`shared` (task completion order
is not pinned down, so no honest hand-written expected output is possible for
a realistic concurrent program), the Rust bridge (needs network), and `File`
/`Dir`/`Env` I/O (needs a filesystem state I cannot predict).

Across all eleven running programs I avoided printing a `Float`, a list, a
map, a set, an optional or a struct directly — see point 3 below for why.

## 2. Ambiguous, surprising or under-specified in the README

Numbered roughly by how much it cost me.

1. **How anything other than a `Str`, `Int` or `Bool` prints is nowhere
   stated.** The examples lean on it constantly — `puts nums.take_while(_ < 10)`,
   `puts counts`, `puts quarter(8)`, `puts m` (a map), `puts pts.len`,
   `puts t.to_list`, `puts p` (a struct: "`==` and printing derived") — but
   the README never shows the rendering for a list, a map, a set, a tuple, a
   struct, an enum, a `T?` or a `T or E`. Is `Some(4)` printed as `Some(4)`,
   `4`, or something else? Is `[1, 2]` printed as `[1, 2]`? Is a `{Str: Int}`
   printed Ruby-style or JSON-style? For a reviewer writing expected output
   by hand this is the single biggest gap — it made a whole class of natural
   program lines unusable, and every one of my twelve programs is written
   around it (`match` on every optional, `join` on every list, hand-built
   money strings instead of floats).

2. **`pad` / `pad_right`: which side fills, and what happens on overflow.**
   The README lists both on `Str` and `pad` on `Int`/`Float` and says nothing
   else. `examples/stdlib.lume` prints `"[#{"ab".pad_right(5)}]"` and
   `"[#{"ab".pad(5)}]"` but the file gives no expected output, so it does not
   settle it either. I inferred *`pad` right-aligns, `pad_right` left-aligns*
   from `examples/collections.lume` line 38 (`"#{w.pad_right(6)} #{…}"`,
   which only makes sense as left-align). Every column in every program here
   depends on that inference. Also unstated: what `"abcdef".pad(3)` does —
   truncate, or overflow the column? I kept every width ≥ its content so it
   never comes up, but a real report generator cannot.

   Related: **`Int.pad(n)`** is listed among the `Int`/`Float` methods. Does
   it pad with spaces (like `Str.pad`) or with zeros (which is what you'd
   actually want on a number)? I never used it — every numeric column here is
   `"#{n}".pad(w)` — because I could not tell.

3. **No zero-padding and no `format`.** With no printf, no `%`, and no
   documented zero-pad, formatting `7` as `07` needs
   `if frac < 10: "0#{frac}" else: "#{frac}"`. I wrote that same three-line
   idiom in four separate programs (z01, z02, z06, z12). For a language whose
   pitch includes "for money and reports" (the `decimals` blurb), this is the
   most conspicuous missing everyday method.

4. **A `{ |x| … }` block inside a `#{}` interpolation is never attested.**
   `_`-shorthand blocks inside interpolation are
   (`"#{users.filter(_.adult?).sum(_.age)}"`, blocks.lume:29), and so are
   nested string literals (`"[#{"ab".pad(5)}]"`) and `{}` literals
   (`"#{idx[w].or({}).to_list…}"`, collections.lume:38). But a brace-block
   inside `#{}` — where the lexer must decide whether `}` closes the block or
   the interpolation — appears in no example. I wrote five of them by
   accident and then moved every one out to a preceding binding. Whichever
   way it behaves, it should be documented, because it is the natural thing
   to write.

   The same question, one level deeper: **`"#{"#{n}".pad(2)}"`** — an
   interpolation inside a string inside an interpolation. This is the
   *only* way I found to right-align an integer in a column, given that
   `Int.pad`'s fill character is unspecified (point 2), so I reached for it
   in seven of the twelve programs before noticing it is unattested.
   `examples/stdlib.lume` shows one level only. I pulled it back to a
   three-line `def num(n: Int, w: Int) -> Str` helper everywhere except
   z01, which keeps it deliberately so the construct is still tested
   without taking six other programs down with it if it fails.

5. **`"##{x}"`: a literal `#` immediately before an interpolation.** I wanted
   a Markdown anchor (`#slug`) in z11 and could not convince myself this
   lexes. The README only promises that `#{}` interpolates; it never says a
   bare `#` in a literal is fine (I assume it is, and z05/z11 depend on it
   via `"# demo config"` and `c == "#"`), and it says nothing at all about
   the adjacent case. I rewrote z11's output to avoid it.

6. **There is no documented way to *read* a nested container by index.**
   `grid[i][j] = v` is documented for assignment. But reading: `xs[i]` is a
   `T?`, so `grid[i][j]` is a `[T]?` indexed again, which the README lists
   under errors ("a method or arithmetic on a `T?` … without unwrapping").
   So the read counterpart of `grid[i][j] = v` does not exist as such; z07
   uses `dp.at(i).at(j)`. That asymmetry deserves a sentence.

7. **`at` is under-documented.** It is introduced only in the milestone-24
   line ("`xs.at(i)` (the item when the position is already known to be
   good — the read generic code needs, since it has no default to fall back
   on)") and never appears in the per-type method lists. What happens when
   the position is *not* good — stop the program like `xs[i].method`, or
   something else — is not said. z03, z07 and z10 all rely on it.

8. **Map-of-lists mutation.** The README gives `idx[w].add(x)` (map of sets)
   and `grid[i].push(x)` (list of lists) but never `m[k].push(x)` on a
   `{K: [V]}`. z05 and z08 both need it. It follows from "a collection stored
   in a `var` map or list is changed where it is stored" plus "a missing map
   key starts from an empty value", but it is exactly the kind of thing the
   "change that would land on a temporary copy" error is about, so a worked
   example would help.

9. **Self-transform rebinding and the loop/branch rebinding rule collide.**
   `examples/shadow.lume` shows `name = name.trim` is fine. The README also
   says rebinding an outer name *inside a loop or branch* is an error. So
   what is `if s.ends_with?("-"): s = s[0...s.len - 1]` — a permitted
   self-transform, or a forbidden branch rebinding? Both descriptions apply
   word for word. I restructured z11 to sidestep it.

10. **`not` precedence against `and`/`or` is never stated.**
    `not a and not b` could be `(not a) and (not b)` or
    `not (a and (not b))`; the two differ. I parenthesised everywhere
    (z05, z08, z11) once I noticed.

11. **`sort_by`: direction, stability, and result type.** Never stated to be
    ascending (it clearly is, from `enums.lume`'s `sort_by(_ * -1).first`),
    never stated to be stable, and never stated whether the result is a list
    or a lazy chain — `blocks.lume` chains `.reverse.map(…)` off it and
    `enums.lume` chains `.first`. I kept every sort key distinct in every
    program so stability cannot bite, and I sprinkled defensive `.to_list`
    calls. `sort_by` with a direction, or `sort_by_desc`, does not exist;
    `.reverse` after the sort is the only route.

12. **`min_by`/`max_by` are listed but their return type is not.** `max`/`min`
    are documented as giving `T?`; `max_by` is only named. I used
    `sort_by(…).last` and a `match` in four programs rather than guess.

13. **No descending range and no `step`.** `1..10` / `1...10` count up only.
    Every backwards traversal has to become a hand-written `while` with
    `i -= 1` (z07's two DP loops). A `10..1` or `(1..10).step(2)` is the
    obvious thing to reach for and is not there.

14. **No `break`; `next` is undocumented.** The error list says the spelling
    `continue` is rejected "with the Lume form", and `examples/chars.lume`
    line 29 uses a bare `next` — but `next` appears **nowhere** in the
    README's feature prose. There is no `break` at all, so early exit from a
    `while` is faked by assigning the loop variable past the end
    (`i = out.len`, as `generic_interfaces.lume` does and as z03 copies).
    Worth documenting both.

15. **`Time` is undocumented too.** `Time.now_ms` (blocks_of_your_own.lume)
    and `Time.sleep` (async.lume) are used in examples but `Time` is not in
    the README's list of namespaces (`File`, `Dir`, `Path`, `Env`).

16. **Built-in error message text is unspecified.** `"x".to_int` gives a
    `T or Error`, but the message is never shown, so no program that prints
    `e.message` from a built-in is portable. Every error message printed in
    z04, z05 and z10 is one my own code wrote; z10's header says so
    explicitly. This also means a user cannot write a test asserting on a
    conversion failure's text.

17. **`decimals(n)` is described but never shown.** "the number as text to
    that many places, halves away from zero" — but does `7.decimals(1)` give
    `7.0`? Does `-0.004.decimals(2)` give `-0.00` or `0.00`? Because of
    point 1 I could not verify against any example output, so I used no
    floats anywhere in these twelve programs. Combined with
    `Float / Int` being rejected and `avg` returning a `Float`, *every*
    average in a report has to be either `.decimals(…)` on `avg` or
    hand-rolled `Int` division. I chose Int division in z04, z07 and z10.

18. **`repeat(0)`.** `"-".repeat(35)` is obviously fine; `"  ".repeat(0)`
    (z11, for a top-level heading's indent) is not stated to give `""`, and a
    negative count is not stated at all.

19. **The shadowing trap in `extend {K: V}`.** The README says "a method of
    your own wins over the built-in of the same name". Combine that with the
    generic-extend example and you get a live trap: an interface whose
    required method is called `values`, extended onto `{K: V}`, cannot be
    implemented — `def values -> [V] = self.values` is unbounded recursion,
    and there is no `super`, no qualified call, no documented way to reach
    the shadowed built-in. The README's own example dodges it by choosing the
    name `items`; a reader will not know they had to. z09's header documents
    my workaround (`readings`, `total_count`, `blank?`). The same applies to
    `len`, `count`, `keys`, `to_list`, `map`, `first`…

20. **Two interfaces over the same target with same-named defaults.**
    `generic_extend.lume` puts `[T]` under both `Bag[T]` and `Ranked[T]`, and
    z09 puts `[T]` under both `Samples[T]` and `Scored[T]`. Neither the
    README nor the example says what happens if the two interfaces declare
    *defaults with the same name* — ambiguity error, or last-one-wins? I
    renamed to avoid the collision; a rule should be stated.

21. **A type parameter that lives only in a bound, applied to a container.**
    The README shows `def keys_of[K, V, T: Keyed[K, V]]` where the parameters
    come out of a *struct's* conformance. z09 does the same thing where the
    conforming type is `{Str: Int}` conforming through
    `extend {K: V} with Samples[V]` — so `T` has to be worked out through two
    levels of parameter substitution. That combination appears nowhere and is
    the thing in z09 I would least bet on.

22. **`Ok(…)` as a constructor.** The examples `match` on `Ok(x)`, but the
    README only ever says the Ok is *implied* ("a bare value in a `T or E`
    function is `Ok`"). Can you write `Ok(v)` as an expression? I never did.
    Nested patterns *inside* `Ok(…)` (`Ok(Section(name))`, z05) are likewise
    only implied by "works through … `T?`/`T or E`" and never shown.

23. **`?` after a block call, and the `do` asymmetry.** z10 writes
    `read_column(t, "name") { |c| c.trim }?`. Not attested. And there is no
    way at all to write `?` after a `do |x|` block, because the block's body
    is the rest of the indented region — so the `{ … }` and `do` forms are
    not interchangeable in a `T or Error` function. z10 has to bind and then
    `?` on the following line for the `do` case, right next to three `{ … }`
    calls that do it inline. That asymmetry is real and worth a note in the
    docs.

24. **A tuple as the `Ok` type, and destructuring through `?`.**
    `-> ([T], [T]) or Error` and `(a, b) = f(…)?` (z10), and
    `(dist, prev) = route(…)` returning two maps (z08). Tuples are documented
    and `T or E` is documented, but their composition is not shown anywhere,
    and `(a, b) = expr` is documented only as taking apart an existing
    `pair`, never as the result of a call, never inside a loop, and never
    interacting with the no-shadowing rule (does `(picked, rest) = …` inside
    a `while` body *declare* two names each time round? z06 does exactly
    this).

25. **Three-name tuple destructuring.** Only the two-name forms appear
    (`for i, x in xs.enumerate`, `for v, sign in values`,
    `{ |a, b| … }` over a `{(Int, Int)}`). I wanted `for a, b, w in edges` in
    z08 and could not tell if three works, so I introduced an `Edge` struct
    instead.

26. **Triple-quoted string edges.** "with the leading line break and the
    common indentation removed" leaves two things open: does the line holding
    the closing `"""` contribute a trailing newline, and does the closing
    delimiter's own indentation count towards "the common indentation"? z11
    uses a `"""` block but pipes it through
    `.lines.reject(_.trim.empty?)` so the answer cannot affect its output —
    which is itself a sign the rule needs pinning down.

27. **Unary minus on a value.** Negative *literals* appear (`-1`, `-5..100`,
    `_ * -1`) but `-x` for a binding never does. I wrote `0 - c` in z01 and
    `other.points - points` in z03 rather than risk it.

28. **Insertion order of `keys` / `values` / `to_list`.** The README says
    `{K: V}` is "an insertion-ordered hash map" and a set keeps "insertion
    order", but never says that `keys`, `values` and `to_list` *yield* in
    that order. Six of my programs (z01, z02, z04, z05, z09, z10, z11) print
    a tally in map-insertion order, so this is load-bearing for them.

29. **`for … where` over a set.** `where` is documented for
    `for x in range`, and `structs.lume` uses it over a list. z08 uses it
    over a list with a set-membership test (fine), but iterating a set with
    `where` is not shown; nor is `for k, v in <map>` (the README gives maps
    an `each` block but no `for` form — I used `.keys` everywhere instead,
    which may be the wrong idiom).

30. **`puts ""`.** Every one of my reports wants a blank line between
    sections. `puts` with an empty string appears in no example. I assume it
    prints an empty line.

31. **`join` on a non-`Str` list.** `join` is listed as a list method with no
    statement about the item type. `chars.lume` does
    `"mississippi".chars.uniq.sort.join("")` on a `[Char]`, so at least
    `Char` works. For `[Int]` I always went through
    `.map { |v| "#{v}" }.to_list.join(…)` — which is verbose enough that
    `join` accepting anything with a text form would be a real win.

32. **`Str + Char`.** "a `Char` … becomes a `Str` with `.to_s` or wherever a
    `Str` is wanted" — so does `out += c` work directly? I wrote
    `out += c.downcase.to_s` in z11 to be safe. Similarly `Int.to_char` for a
    value outside Unicode is unspecified (z06 uses `(65 + i).to_char`).

33. **Behaviour cannot be stored or returned**, which the README states
    plainly. Worth flagging anyway as the thing I missed most while writing
    these: a strategy table (category → handler) has to become a `match`, and
    a "build a formatter once, apply it to every row" pattern has to become a
    function taking the same block at every call site. z01, z02 and z12 all
    work around it.

34. **Method names that silently shadow built-ins.** Because a user method
    wins, `struct Slot: def length` (z06), `def size`, `def count`,
    `def first` on your own type are all silent shadowings of names that mean
    something else on other types. In a 100-line program this is easy to do
    by accident and nothing warns. A note in the errors/warnings list would
    help.

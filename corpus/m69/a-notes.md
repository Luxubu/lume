# Reviewer A — round fourteen (m69) notes

## What I read

- `corpus/m69/BRIEF.md`.
- `docs/README.md`, `docs/tour.md`, `docs/install.md`, `docs/examples.md`.
- `docs/reference/`: `collections.md` (twice, especially "Types of your own:
  `items`" and "Types of your own: `next`"), `interfaces.md`, `methods.md`,
  `generics.md`, `structs-and-enums.md`, `control-flow.md`, `printing.md`,
  `strings.md`, `functions-and-blocks.md`, `numbers.md`, `errors.md`,
  `patterns.md`, `operators.md`, `names.md`.
- `examples/iterable.lume` + `.expected`, `examples/iterators.lume` +
  `.expected`, `examples/interface_methods.lume` + `.expected`,
  `examples/generic_interfaces.lume` + `.expected`, a grep through
  `examples/enumerate_blocks.lume` and `examples/stdlib.lume` for `zip` and
  `enumerate`.
- `examples/errors/`: `items_and_next`, `items_not_a_list`,
  `iterable_bound_unmet`, `iterator_bound_var_self`, `next_wrong_shape`,
  `for_var_items`, `for_var_next`, `interface_var_self`, `extend_var_self`,
  `iface_mismatch`, `iface_missing`, `readonly_self`, `immutable_receiver`,
  `loop_no_items`, `chain_no_items`, `generic_bound_unmet` (each `.lume` and
  `.expected`), plus the directory listing.
- Not read: `docs/QUESTIONS.md`, `concurrency.md`, `modules.md`,
  `packages.md`, `io.md`, `http.md`, `json.md`, `crates.md`, `packages/`.
  None of my programs use what they cover.

## The programs

| file | what it is | leans on |
|---|---|---|
| `a01_pager.lume` | a paginated API reader that fetches the next page only when its rows run out | Iterator[Str], `while`/`return None` inside `next`, take/count/enumerate/filter+join, `[S: Iterator[Str]]` + `find`, walking a copy, a `var` reader moved by hand, `while true` + `match` + `break` |
| `a02_tokenizer.lume` | a lexer that yields `Tok` enum values from a line of code | Iterator[Tok], `?` on an index to end the sequence, Char methods, Str + Char, `len`, `map(fn)`, `enumerate.find` with two names, `[S: Iterator[Tok]]` with `for` + `match` |
| `a03_ring_buffer.lume` | a ring buffer of the last N readings, looped over oldest first | Iterable[Int], an own `push` beating the list's, field index assignment, list `+`, avg/max/min/zip/sort/last through `items`, `[C: Iterable[Int]]` with a ring, a list and a set, a mixed `[Iterable[Int]]` |
| `a04_id_minting.lume` | invoice/order ID generators | an interface with a `var self` required method and a default, a `[M: Minter]` function with a `var` parameter, a `var held: Minter`, a struct that is both a Minter and an endless Iterator[Str], zip/find/skip on it |
| `a05_playlist.lume` | a playlist (Iterable[Song]) and a radio that repeats it for ever (Iterator[Song]) | sum with a block, max_by + `.map`, group_by, `[C: Iterable[Song]]` and the tie rule, an endless sequence stopped by take/break/enumerate.skip.first, a field named `at` |
| `a06_running_average.lume` | raw readings, a running mean, and an endless pseudo-random sensor | Iterator[Float] and Iterator[Int], bounds on both, how floats print, `last`, `zip` with another sequence of your own, take/find/take_while on an endless one, a held `Iterator[Int]` |
| `a07_disk_tree.lume` | a folder tree walked depth-first into `(path, size)` pairs | recursive enum through a list, `items -> [(Str, Int)]`, a recursive helper with a `var` list parameter, two-name blocks everywhere, `Some((p, s))`, `[C: Iterable[(Str, Int)]]` |
| `a08_word_stream.lume` | a cleaned word stream from text, with a tally | Iterator[Str] with skipping inside `next`, len/take/uniq/any?/all?/enumerate/group_by by Char, a tally map, sort_by on a map then take, `[S: Iterator[Str]]` + max_by tie |
| `a09_queue_var_self.lume` | **FAIL**: an interface asks for `take_one(var self)`, and the struct's `take_one` only reads | the rule "a type fits only if its own method takes `var self` too", for a bound that is the user's own interface |

## Guesses

Everything below was something I could not settle from the docs or
examples. Where I say "by analogy", the docs show something close but not
this exact case.

1. **All programs: where warnings go.** I took "EXPECTED OUTPUT" to be stdout
   only. The one warning I predict (a03) is in a separate block of the
   header. I looked at `examples/interface_methods.expected`, where warnings
   come first in the same file, and at `docs/install.md` ("Warnings are shown
   when a program is compiled"). Neither says which stream they go to under
   `lume run`.
2. **a01: `return None` from inside a `while` in a `next` method.** The docs
   show `return None` only inside an `if` (`Countdown`). I assumed it leaves
   the method, as in any function (`functions-and-blocks.md`, `errors.md`
   `first_even`).
3. **a01: a method's bare last value gets the implied `Some`.**
   `rows.remove_at(0)` is a `Str` in a `-> Str?` method. `errors.md` says
   this for *functions*, and `examples/iterators.lume` writes `Some(...)`
   explicitly in every `next`. I assumed methods work the same way. The same
   guess comes up in a02 (`Sym(c.to_s)`, `return Num(n)`), a05 (`s`), a06
   (`x`, the Float division) and a08 (`return w`).
4. **a01: `count { }` on an Iterator.** `collections.md` says "every chain
   method". `count` is in the list table but no example uses it on a `next`
   type.
5. **a01, a02, a03, a05: `for i, x in seq.enumerate` with two names when
   `seq` is your own type.** `iterators.lume` only does
   `enumerate.take(2).to_list`. `control-flow.md` shows two names over a
   list's `enumerate`.
6. **a01: a generic parameter walks a copy too.** `r.fetched` is still `0`
   after `first_with(r, ...)` runs `find` on the parameter. The docs say "a
   loop or chain walks a copy" and that a function borrows its parameters,
   but they never show a reader whose state you could see afterwards. I
   assumed the caller's value does not move.
7. **a01: a chain on a `var` binding of a concrete Iterator type starts from
   where it is and leaves it there.** The rule is shown only for an
   interface-typed `var s: Iterator[Int]` (`iterators.lume`: `s.next`, then
   `s.take(5)` gives `[1]`). I assumed `live.skip(2).take(2)` starts from
   `live`'s current position and does not move `live`.
8. **a01: `find` on a generic `S` parameter gives `Str?`.** This is by
   analogy with `fib.find`. I assumed it gives an optional and not the bare
   item.
9. **a02: `cs.at(pos).code - 48` to read a digit.** No docs say how to turn
   a digit `Char` into an `Int`. I used `code` (`methods.md`) and assumed
   ASCII '0' is 48.
10. **a02: `filter(_.word?)`, the `_` shorthand with a method whose name ends
    in `?`.** `functions-and-blocks.md` shows `_.upcase` and `_.len == 2`.
    I assumed a `?`-name parses the same way.
11. **a02: `map(describe)` (a function's name as a block) on an Iterator
    chain.** This is shown only on lists.
12. **a02: an enum method with `match self:` and `Word(_) -> true` on a
    variant with one field.** I assumed `_` stands for that one field.
13. **a02: `!=` between enum values built on the spot.** `Sym(";")` is
    built positionally, with a `Str` literal. `structs-and-enums.md` gives
    `==` for free, and `interfaces.md` says `!=` follows from `==`.
14. **a02: `lx.len` on a `next` type.** `collections.md` names `len` only as
    a method that "never returns" on an endless sequence. I assumed it works
    and counts the items of a finite one.
15. **a02: `enumerate.find { |i, t| ... }` gives `Some((7, Sym(op: "*")))`,
    and how a tuple holding an enum prints inside `Some`.** This is by
    analogy with `Some(("ann", 3))` in `methods.md`.
16. **a03: a `var self` method of your own whose name the list also has
    (`push`).** It is called on an Iterable type and should beat the list's
    `push`. The docs' "its own methods come first" is shown only with `len`
    and `first`, which are read-only.
17. **a03: `buf[head] = x`, assigning by position into a *field* inside a
    `var self` method.** The docs show it only on a local `var` list.
18. **a03: `avg` through `items` on a bound parameter.** I assumed the
    `[C: Iterable[Int]]` bound gives `c.avg` as well as `for`, giving
    `5.0`.
19. **a03: `c.max` and `c.min` on an Iterable bound parameter.**
    `iterable_bound_unmet` uses `c.sum`, so I assumed every list method is
    there.
20. **a03: what `zip` gives when it comes through `items`.** I added
    `.to_list` so the printed form does not depend on whether it is a list
    or a chain.
21. **a03: a mixed `[Iterable[Int]]` of a struct and a list literal.** The
    docs say "a value can be held as an `[Iterable[Int]]`" and show nothing
    more. Several guesses here: that a list literal can sit beside a struct
    in it, that `c.sum` works on a held item, and that the compiler gives the
    "holds values of different types" warning. I also guessed its exact text
    ("behind `Iterable`", without `[Int]`, copying "behind `Mappable`" in
    `interface_methods.expected`), its column (3) and its gutter.
22. **a03: `Ring.with_cap(3)`.** A type function's parameter is named like a
    field (`cap`), and the result goes into a `var`. I assumed there is no
    clash.
23. **a04: an interface with a `var self` required method *and* a reading
    default.** `interfaces.md` shows each on its own, never both in one
    interface.
24. **a04: a `var` parameter of generic type moves the caller's value.**
    After `batch(inv, 3)`, `inv.n` is 3. `interfaces.md`'s `twice(c)` never
    prints `c` afterwards. `functions-and-blocks.md` says a `var` parameter
    "is the caller's own value", but shows this only for a list.
25. **a04: `Some(mint)`, calling the type's own `var self` method by bare
    name *inside an expression* in another `var self` method.** The docs'
    `twice` calls `bump` only as a statement.
26. **a04: one struct is both a user-interface conformer (`Minter`) and an
    `Iterator[Str]`.** Nothing says this is forbidden. Only `items` + `next`
    together is.
27. **a04: a `var` binding of a user interface type with a `var self`
    method.** `var held: Minter = Letters(i: 0)` and then `held.mint` is
    shown only for the built-in `Iterator[Int]`.
28. **a04: `held.kind` reaches the type's own *override* of a default
    through the interface binding.** I assumed it prints `letters`.
    `interfaces.md` says a held value "answers the interface's defaults",
    and says nothing about overrides reached through a pointer.
29. **a04: no warning for a single-type `var held: Minter`.** I assumed
    none, by analogy with `var s: Iterator[Int]` in `iterators.expected`,
    which shows none.
30. **a04: `for id, item in ids.zip(orders)`, two names over an endless
    sequence zipped with a list.** The docs only show
    `colors.zip(1..5).to_list`.
31. **a04: `.or("?")` gives a `Char` after `to_char`, and interpolating a
    `Char` prints it bare.** This is by analogy with `strings.md`.
32. **a05: a field named like a list method (`at`) on a `next` type.**
    "Own methods and fields come first" is written only in the `items`
    section. I assumed `radio.at` reads the field (0), and that `at` inside
    `next` is the field.
33. **a05: `max_by(_.len)` on the result of `c.map(...)` on an Iterable
    bound parameter, keeping the first on a tie (`Blue`).** The tie rule is
    documented for lists. I assumed it holds through `items`.
34. **a05: `p.max_by { }.map(_.title)`, the `_` shorthand inside an
    optional's `map`.** `methods.md` shows `price.map(_ * 2)`. A field
    access through `_` on an optional is by analogy.
35. **a05: `group_by` through `items`, then `for artist, tracks in by`.** I
    assumed it gives an ordinary `{Str: [Song]}` map in first-seen order.
36. **a05: `songs[at % songs.len]?`, computing the index before the `?`.**
    `Cycle` in `iterators.lume` does the same. I assumed an empty list would
    stop the program (`%` by zero), and avoided it.
37. **a06: `Iterator[Float]` as a bound, and a `next` giving `Float?`.**
    Only Int and Str sequences appear in the docs.
38. **a06: `last` on a finite Iterator.** It is in the list table, and
    "every chain method" covers it, but no example uses it.
39. **a06: `feed.zip(r)`, zip with another *sequence of your own* as its
    argument.** `methods.md` says `zip` takes `[U]`, and the only non-list
    argument shown is a range (`colors.zip(1..5)`). I guessed it takes
    anything with `next` and stays lazy, giving pairs that a two-name block
    takes apart. This is the riskiest line in the set.
40. **a06: the shortest round-trip text of 38/3 and 85/6.** I worked out
    `12.666666666666666` and `14.166666666666666` from what I know of f64
    and the "shortest text that reads back" rule in `numbers.md`. I could
    not check them.
41. **a06: `decimals(2)` of 76/5 is `15.20`, and of 12.666... is `12.67`.**
    I assumed the stored value of 15.2 rounds to `15.20`.
42. **a06: a chain on a held `var s: Iterator[Int]` after `s.next`
    (`s.skip(1).take(2)`).** By analogy with `iterators.lume`. I also
    assumed `skip` on a held interface value works like `take`.
43. **a07: a recursive enum through a list field (`kids: [Node]`).**
    `structs-and-enums.md` shows only a variant holding the enum directly.
44. **a07: a function's `var` list parameter handed on to a recursive call
    of itself (`walk(k, ..., out)`).** I also assumed it is called from a
    read-only method (`items`) with a local `var`.
45. **a07: `out.push(("...", size))`, a tuple as the single argument with
    doubled parentheses.**
46. **a07: an `Iterable` bound whose type argument is a tuple:
    `[C: Iterable[(Str, Int)]]`.** This is not shown anywhere.
47. **a07: `for path, size in c` on a *generic parameter* whose items are
    pairs.** The docs show pair items only on a concrete value.
48. **a07: `t += size if cond`, a trailing `if` on a compound assignment.**
    `control-flow.md` shows it on a plain `=` assignment.
49. **a07: `match disk.find { |p, s| s > 100 }:`, a block inside the
    subject of a `match` followed by `:`.** I assumed it parses.
50. **a07: `sum { |p, s| s }`, a two-name block on `sum` through
    `items`.** `methods.md` says "on every method that takes a block".
51. **a07: variant names.** I avoided `File` and `Dir`. `File` is refused
    as a *type* name, and `names.md` does not say whether a *variant* may use
    it.
52. **a07: `group_by` with pairs, then `map_values`.** I assumed the map
    values are lists of pairs whose `len` is taken.
53. **a08: a `return` inside an `if` inside a `while`, then a final bare
    `None` as the method's last line.**
54. **a08: `c == "'"`, comparing a `Char` with a one-character literal that
    is an apostrophe.** `strings.md` shows only `c == "a"`.
55. **a08: `uniq` on a `next` type.** It is in the list table, but no
    example uses it on a sequence or chain.
56. **a08: a map with `Char` keys prints the keys quoted (`{"t": 4, ...}`).**
    This is by analogy with `["a", "b", "c"]` for a `[Char]`.
57. **a08: `for w, n in counts.sort_by { |w, n| 0 - n }.take(2):`.** The
    block's names are the same as the loop's, on the same line. I assumed no
    clash, since block parameters are new names (`names.md`).
58. **a08: `ws.group_by { |w| w.chars.at(0) }`, a `Char` as a `group_by`
    key.** `strings.md` says a Char works as a map key.
59. **a08: `all` as a variable name, next to the method `all?`.** The same
    name is used in a03.
60. **a09: the whole error text.** I copied the header line from
    `generic_bound_unmet` / `iterator_bound_var_self` ("`drain` needs
    `Q: Queue`, and `Fixed` is not Queue"), with no `[...]` after a
    non-generic interface. I copied the help line from the built-in case
    ("its `take_one` is `-> Int?`, and `Queue` asks for
    `(var self) -> Int?`"). The docs only say "a type fits only if its own
    method takes `var self` too". They never show the message for a user
    interface.
61. **a09: where the error is reported.** I assumed it is at the call (col 8,
    the function name), as for every bound error shown. It could also be at
    `Fixed`'s `take_one`, or it could be a "used as a `Queue`" message like
    `iface_mismatch`.
62. **a09: no earlier error.** I assumed the `var q: Q` parameter and the
    `while true` / `match` / `break` body check cleanly, so this is the
    first and only error. I also assumed `lume check` prints nothing after
    it.
63. **a09: the `|` gutter width for a two-digit line number** (three spaces
    before `|`). I copied this from `iterator_bound_var_self.expected`.

**Summary of the gaps.** The two "Types of your own" sections in
`collections.md` say "every chain method works" but show only a handful of
methods. `count`, `last`, `uniq`, `avg`, `group_by` and `enumerate` in a
two-name `for` are never shown on a type of your own. Most guesses were
filling that gap. The other big gaps:

- **`zip`.** The docs say it takes `[U]` but show it taking a range. They
  never say whether it takes another sequence.
- **Copies versus the caller's value.** "A loop or chain walks a copy" is
  clear for a local. The docs do not cover a sequence handed to a generic
  parameter, or to a `var` generic parameter, where the caller's value
  should move.
- **Interfaces with `var self` methods.** These get one example: a bound
  and a `var` parameter, with nothing printed afterwards. There is nothing
  on holding one in an interface-typed binding, on mixing such a method
  with defaults, on how an override is reached through a held value, or on
  the message when a read-only method fails the requirement.
- **The "own methods come first" rule.** It is stated only for `items`
  types and read-only names. It is not stated for `next` types or for
  `var self` names like `push`.
- **Held `[Iterable[T]]` values.** The docs give them one sentence. They do
  not say what a held item can do, or whether the pointer warning applies.

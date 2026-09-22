# m30 review — multi-file programs and the rule that an `extend` travels with the import

Ten programs, `q01`–`q10`, each in its own directory with `main.lume` as the
entry. Eight are meant to run; **q07** and **q10** are deliberate-error
programs. Everything was written from `README.md`, `examples/travel/`,
`examples/modules/` and the top-level `examples/*.lume` only. No compiler was
run; every expected line was worked out by hand.

## 1. What each program probes

| # | Directory | Files | Verdict | What it probes |
| --- | --- | --- | --- | --- |
| q01 | `q01_ship_builtins` | `main.lume`, `tally.lume` | runs | A library ships one interface and five `extend`s, on `[T]`, `{T}`, `{K: V}`, `(A, B)` and `T?`. The consumer extends nothing. Sharpest sub-probe: `found.bulk` on a `Str?`, where "a method on a `T?` without unwrapping is an error" meets `extend T? with ...`. |
| q02 | `q02_two_owners` | `main.lume`, `forge.lume` | runs | An `extend` on a type the *library* owns (`Ingot`), and an `extend` on a type the *consumer* owns (`Crate`) against the library's imported interface. One bounded helper (`cheapest[P: Priced]`) crosses the boundary in both directions. Defaults called straight on values. |
| q03 | `q03_two_levels` | `main.lume`, `report.lume`, `stats/core.lume` | runs | The `extend` is two levels down, in a file the entry never imports. Tests whether the rule is transitive or one hop. Also: a module-qualified interface used as a generic bound (`[A: core.Spread[Int]]`). |
| q04 | `q04_alias_extend` | `main.lume`, `text/metrics.lume` | runs | The module carrying the `extend` is imported under an alias (`as tm`). The module's names are qualified; the extend's methods must still be bare. Also the same interface on two shapes (`Str`, `Headline`) inside one file. |
| q05 | `q05_diamond` | `main.lume`, `base.lume`, `left.lume`, `right.lume` | runs | A diamond: `left` and `right` both import `base`, and the entry imports `base` too. One `extend` arriving along three paths must not trip the clash rule — i.e. the rule counts definitions, not arrivals. |
| q06 | `q06_no_clash` | `main.lume`, `counted.lume`, `labelled.lume` | runs | Both permitted near-clashes at once: two different interfaces on one shape (`[T]` is `Counted` from one file and `Labelled` from another), and one interface on two shapes across two files (`Counted` on `[T]` here, on `{T}` there). Plus `extend [T] with Labelled` where the interface takes no parameter and `T` lives only in the target. |
| q07 | `q07_clash` | `main.lume`, `store/vault.lume`, `store/annex.lume` | **refused** | Two files claim `{T}` for `Bagged` (`vault.lume:10` and `annex.lume:10`). Checks that the error fires, names **both** files with line numbers, names the type and the interface, and does not blame the innocent entry. |
| q08 | `q08_iface_value_field` | `main.lume`, `gallery/frame.lume`, `gallery/wall.lume` | runs | The interface as a struct field type (`hero: Panel`), as a value type in a field (`rest: [Panel]` holding a `Str` and an `(Int, Int)`), and as a bound. The entry never writes the word `Panel` yet reaches `plate` and `big?`. |
| q09 | `q09_bounded_extend` | `main.lume`, `seqx/util.lume`, `seqx/rank.lume` | runs | `extend`s whose methods and defaults call the library's own generic functions (`util.top`, `util.dedup`) across a boundary the consumer never names. Three kinds of bound: `Ordered`, `Hashable`, and the library's own `Named` interface. `[Int]` answers to two of the extends at once. |
| q10 | `q10_same_method_name` | `main.lume`, `megaphone.lume`, `poster.lume` | **refused** | The consumer writes `extend Str with Polite: def shout`, colliding by *name* with `shout` that `extend Str with Loud` already supplied. Different interfaces, so the documented clash rule does not cover it — this is the gap. |

## 2. Ambiguous, surprising or under-specified in the README

Numbered roughly by how much a real user would trip on it.

1. **Does an `extend` travel one hop or all the way?** The whole rule is one
   sentence: *"An `extend` travels with the import, so a library ships its
   conformances and a consumer that imports it writes none of its own."* Every
   worked example (`examples/travel/`) is a single hop — `main.lume` imports
   `shapes.lume` directly. Nothing says what happens when the extend is two
   files away. **q03** stakes a whole program on it being transitive. If it is
   one hop only, q03's last three lines become unknown-method errors and the
   feature is far less useful than the prose suggests.

2. **"Travels with the import" and "anywhere in one program" cannot both be
   literally true.** The clash rule is scoped to the *program*: *"two `extend`s
   claiming the same type and the same interface anywhere in one program is an
   error."* If extends are collected program-wide for the purpose of the error,
   the natural implementation collects them program-wide for the purpose of
   *method lookup* too — in which case a file that imports nothing at all still
   gets the methods, and "travels with the import" is the wrong mental model.
   Conversely, if lookup really is import-scoped, the clash rule is stricter
   than it needs to be: two unrelated subtrees of a program that never see each
   other are still forbidden from making the same claim. The README needs to
   pick one and say it. I could not construct a program that distinguishes them
   without a file that imports nothing and is still compiled, and I could not
   tell from the docs whether such a file is even reachable.

3. **`extend T?` versus "a method on a `T?` without unwrapping is an error".**
   The error list includes *"a method or arithmetic on a `T?` or `T or E`
   without unwrapping"*, and the interface paragraph offers
   `extend T? with Holder[T]:` as a legal target. Those two rules meet head-on
   the moment you call the extend's method: `found.bulk` where `found: Str?`.
   I assumed the extend wins (otherwise `extend T?` is unusable), but the
   README never says, and the phrasing of the error suggests the check happens
   before method lookup. **q01** is the test.

4. **No way to opt out of, or shadow, an imported `extend`.** Because an
   `extend` has no name, there is no `pub extend`, and equally no way to import
   a module *without* its conformances, no `hiding`, and no way to say "use
   mine here". Combined with (2) this means importing a library can add methods
   to `Str`, `[T]`, `{T}` and `{K: V}` throughout your program with no local
   syntax marking it and no escape hatch. **q10** is what happens when you want
   your own `shout` anyway.

5. **Two extends with the same type and interface but different *bounds*.** Is
   `extend [T: Ordered] with X[T]` plus `extend [T: Hashable] with X[T]` one
   clash or two legitimate claims? Read literally ("the same type and the same
   interface") it is a clash, which would be surprising — the two apply to
   overlapping but different sets of types. I did not write a program for this
   (it would have needed an eleventh), but it is the first thing I would test
   next. Related: is `extend [T] with X[T]` the same claim as
   `extend [T: Ordered] with X[T]`?

6. **Method-name collisions between two extends of *different* interfaces on
   one type are entirely unspecified.** The clash rule is about the
   (type, interface) pair, so it says nothing here. The only tie-break in the
   README is *"a method of your own wins over the built-in of the same name"*,
   which is about built-ins and blocks, not about two extends. **q10** asks for
   an error; silently picking one would change what the *library's* own
   `announce[L: Loud]` does, because it calls `x.shout` through the bound.

7. **Colliding with a built-in method name is a minefield with no stated
   rule.** An interface default becomes "the type's own method"
   (`[3, 1, 2].size` in `examples/generic_extend.lume`). So what happens if a
   library's default is called `len`, `count`, `first`, `sum` or `map` on
   `[T]`? Lists already have all of those, and `count`, `sum`, `map`, `filter`,
   `find`, `any?` and `all?` are *block* methods, so the signatures differ too.
   I had to deliberately avoid every built-in name across all ten programs
   (`bulk`, `shown`, `blank?`, `how_many`, `roster`, `held_count`, `once`,
   `plate`) — the fact that I had to is itself the finding. A library author
   has no list of reserved names to check against.

8. **Is an interface allowed as a struct field type?** The README shows an
   interface as a parameter type (`def describe(s: Shape)`), as a value type in
   a list (`[Shape]`, `[Renders[Str]]`) and in a binding — never as a field. It
   does say, of blocks, that *"behaviour cannot be stored in a field or
   returned yet"*, which I read as about blocks only. **q08**'s
   `gallery/wall.lume:10` writes `hero: Panel`. If that is refused, the
   sentence about fields should be widened and say so.

9. **Can an interface value be *returned*?** Same sentence, same doubt. `[T]`
   returns are everywhere, but `-> Shape` appears nowhere. q08's
   `pub def hang(...) -> Wall` dodges it by returning the struct. Worth a line
   in the README either way.

10. **Where does the `[Iface]` "different types behind a pointer" warning point,
    and does it fire on a field or a parameter?** The only instance I have seen
    is in `examples/interfaces.expected`, pointing at a *binding*
    (`shapes: [Shape] = [c, r, "abcd"]`). q08 has `rest: [Panel]` as a struct
    field and as a parameter, filled with two different types at the call. My
    `# EXPECTED OUTPUT:` for q08 is stdout only and excludes warnings; if the
    suite diffs warnings too, q08 needs one more line and I cannot say which.
    The README says the warning is emitted "once" — once per program, per
    binding, or per type?

11. **A module-qualified name in a type position is documented only for
    values.** *"`model.User`, `model.parse(x)`, `model.Role.Guest(7)` in
    expressions, types and patterns"* covers `model.User` as a field type
    (`users: [model.User]` in `examples/modules/users/store.lume`). It does not
    cover a qualified *interface* in a **bound** — `[A: core.Spread[Int]]`, in
    `q03/report.lume:12` — nor a qualified interface as the target of an
    `extend` (`extend Crate with forge.Priced`). I probed the first and dodged
    the second by importing the name (`import forge.Priced`), because I had no
    evidence either way and did not want q02 to fail for the wrong reason.

12. **Does `import a.b.Name` alone bring `a/b.lume`'s extends?** The forms are
    listed (`import users.model`, `import users.model.User`, `as`), but the
    extend rule is stated in terms of "a consumer that imports it". If you
    import only one name, have you imported the module? q02 hedges by writing
    both `import forge` and `import forge.Priced`. q06's `labelled.lume` does
    the same. A one-line answer in the README would remove the hedge.

13. **Does an aliased import still carry the extends?** **q04** assumes yes —
    the extend has no name, so an alias has nothing to rename. But the
    alias *is* the mechanism for "I want this module's names under a different
    prefix", and a reader could reasonably expect the conformances to follow
    the prefix too (they cannot, since there is no name; that is worth saying
    out loud in the README rather than leaving to inference).

14. **Imports resolve from the program root, not relative to the importing
    file.** `examples/modules/users/store.lume` writes `import users.model`,
    not `import model`, even though the two files are siblings. This is load
    bearing for every multi-file program and appears *only* in that example —
    the README prose says *"`import users.model` loads `users/model.lume`"*
    without ever saying relative to what. q03, q07, q08 and q09 all depend on
    reading it off that one example.

15. **Does a bare struct field satisfy a required interface method?**
    Conformance is *"by having the methods"*, and fields are read bare inside
    methods and as `p.x` outside, so `struct Player: name: Str` reads exactly
    like a `def name -> Str` at every call site. I could not tell, so
    **q09** gives `Player` an explicit `def name -> Str = handle` and I noted
    it in the program header. This is a very likely user mistake either way:
    if a field does *not* conform, the error needs to say "`Player` has a field
    `name` but the interface wants a method".

16. **Tuple element access inside an `extend`.** The README shows `t.0` on a
    binding and `extend (A, B) with Pairish[A, B]:` as a target, but never the
    two together. q01 and q08 write `self.0` / `self.1`. If the right spelling
    is a bare `0` / `1` (the way fields are bare inside methods), both are
    wrong and I would like the error to say so.

17. **`{K: V}` can only be claimed once for a given interface, so a map cannot
    be both "a bag of its values" and "a bag of its keys".** Both would be
    `extend {K: V} with Bag[...]`, which is the same type and the same
    interface — a clash by the rule, even though the two are obviously
    different intents. Every example (`travel/shapes.lume`,
    `generic_extend.lume`, and my q01/q05) picks the values. This is a real
    expressiveness limit that the README does not mention.

18. **An `extend` introducing a parameter used only in the target.**
    *"An `extend` introduces a type parameter by using one"* — q06 writes
    `extend [T] with Labelled` where `Labelled` has no parameters at all, so
    `T` appears once, in `[T]`, and nowhere else. I believe that is exactly
    what the rule licenses, but every example uses `T` on both sides.

19. **A user interface as a bound on an interface's own type parameter.**
    `interface Ranked[T: Ordered]` is shown; `interface Roster[T: Named]`,
    where `Named` is another interface of the same library, is not. Nor is
    `extend [T: Named] with Roster[T]`. q09 assumes "any interface name for its
    methods" (stated for *function* generics) applies in both places.

20. **Where does an interface default resolve the module names it uses?**
    q09's `def best -> T? = util.top(all)` lives in `seqx/rank.lume`, which
    imports `seqx.util`; it is *called* from `main.lume`, which does not. This
    has to resolve at the definition site, but the README never frames defaults
    as carrying their file's imports with them — and it is the same question as
    (1), one level in.

21. **Can an `extend` override one of its interface's defaults?** Nothing says.
    A library shipping `extend [T] with Bag[T]` that wants a faster `size` than
    `items.len` would want to. Untested here; I'd add it next to (5).

22. **What about an `extend` that supplies a method the interface never asked
    for?** The error list covers *"an `extend` that leaves a method out"* but
    not the other direction. Is the extra method added to the type too?

23. **`puts` followed by a `{` needs parentheses.** `examples/generic_extend.lume`
    writes `puts({"a", "b"}.listed)` and `examples/stdlib.lume` writes
    `puts((1..10).to_list.group_by { ... })`. The README never mentions it; the
    only related note is that *"`name (` with a space"* is the ambiguous-call
    error. So `puts {…}` is wrong, `puts(…)` is right, and `puts (…)` with a
    space is an error — three spellings, one documented. I rewrote q08's
    `puts (3, 5).big?` as a binding plus `puts pair.big?` for exactly this.

24. **`join` on a lazy chain.** Every example that joins writes
    `.map { ... }.to_list.join(...)`, but `examples/modules/users/store.lume`
    returns `users.filter(_.admin?).map(_.name)` and it prints as a list. I
    could not tell whether `.to_list` before `.join` is required or merely
    conventional, so I wrote it everywhere. If it is required, the README's
    method list should say which methods end a lazy chain.

25. **Tie-breaking in `max_by` / `min_by` is unspecified.** q04 uses
    `words.max_by { |w| w.len }` and I had to choose a sentence with a unique
    longest word ("the sleepy brown fox") to have a pinnable expected line.
    First-wins or last-wins should be documented.

26. **Rendering I had to take on faith.** `Some(x)` prints a `Str` unquoted
    (`Some(ada)` in `examples/generics.expected`), lists print `[a, b]` with no
    quotes, sets print in insertion order, maps as `{k: v}`. q01, q04 and q09
    each have at least one line that depends on this. It is all inferable from
    the `.expected` files and nowhere in the README.

27. **A duplicate value in a set literal.** `{"red", "red", "blue"}` — the
    README says a set holds *"each value once"*, which suggests silent
    deduplication, but it also has an error for *"a list or set literal whose
    items disagree"*, which shows literals are checked. I avoided the case in
    q01 rather than guess.

28. **The clash error's own format.** *"an error naming both files"* — but
    which file gets the caret, are the paths relative to the entry file or the
    cwd, and does it print the type and interface? q07's header states what I
    hope for and what I would consider a bad message; that program is really a
    test of the message, not of the rule.

29. **README inconsistency: the milestone-29 row describes a library that is
    not the one shipped.** Row 29 reads *"done: `seq` ships `Walkable[T]`, and
    its consumer extends nothing"*, but the Layout section lists
    `examples/travel/` as *"a library that ships its conformances, and a
    consumer that writes no `extend`"*, and `examples/travel/shapes.lume`
    ships `Countable[T]`, not `Walkable[T]`. Two names for the feature's
    worked example, in one file.

30. **Small README oddities.** The status table is headed "milestones 1–29
    done" but jumps from row 18 to row 29 and then counts *down* (29, 28, 27,
    … 19, 18r), so 18 appears twice under two spellings and the reader has to
    scan for the newest row. Also the interfaces paragraph is one sentence
    roughly 90 lines long (README lines 58–228 are a single "What works today"
    period); the `extend`-travels rule — the newest thing in the language — is
    three clauses buried at lines 131–141 of it, which is why almost everything
    in this list is a guess.

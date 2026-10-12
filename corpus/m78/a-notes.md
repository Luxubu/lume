# Reviewer A — notes

## What I read

- `corpus/m78/BRIEF.md`.
- `docs/README.md`, `docs/tour.md`, `docs/install.md` (the `lume check` commands and warnings).
- `docs/reference/collections.md` in full, especially "Types of your own: `next`", "Generators: `yield`" and "Taking a sequence".
- `docs/reference/interfaces.md` in full, especially "An interface's own methods may use interfaces".
- `docs/reference/methods.md` in full (blocks over pairs and triples, chains, maps, results).
- `docs/reference/structs-and-enums.md`, `functions-and-blocks.md`, `control-flow.md`, `errors.md`, `printing.md`, the first part of `patterns.md`, `strings.md` (the first 200 lines), `numbers.md` (the first 140 lines), and the keyword and reserved-name tables in `names.md`.
- `examples/generators.lume`, `examples/interface_signatures.lume` and their `.expected` files, `examples/iterators.lume`, the start of `examples/interfaces.lume`, and the warnings in `examples/interface_methods.expected` and `examples/generic_interfaces.expected`.
- `examples/errors/`: every `generator_*` and `yield_*` file, plus `iface_pass_held_value`, `iface_mismatch`, `iface_missing`, `iface_unused_param`, `interface_*`, `block_tuple_arity`, `for_three_names` and `generic_unused`. I used these for the exact error layout (the gutter width, `-->`, `help:`).

## The programs

| file | what it is | leans on |
|---|---|---|
| `a01_disk_usage.lume` | a folder tree that reports sizes by path, depth and extension | a generator method on a recursive **enum**, with `yield` inside `match self` arms and calling itself on the payload; `for a, b, c in`; three-name blocks on filter, map, max_by, sort_by and sum; a map tally |
| `a02_shelf_pages.lume` | a bookshelf shown a page at a time, filtered by author | a generator method giving `Iterator[[Book]]` with a `var` page reset after each `yield`; a plain method `-> Iterator[Book]` handing back a chain over its own field; a function `-> Iterator[Str]` handing back a list; a generator walking its own copy after the value changes |
| `a03_seat_map.lume` | a cinema seat map read from text, walked by rows and by cells | two generator methods on one struct (rows as lists, and `(row, col, seat)` triples); a generator calling another method by its bare name; triple blocks; `Grid or Error` with `?` in `main -> () or Error` |
| `a04_playlist_shuffle.lume` | a seeded deterministic shuffle of a playlist, plus dice | an endless sequence of your own (`next`) turned into an endless `.map` chain and handed to a generator function that ends itself with `return`; a generator method building that chain; a `var` sequence copied into a chain |
| `a05_shipping_quotes.lume` | choosing the cheapest carrier for a parcel | an interface method giving back the interface (`-> Carrier`) and a default taking one (`o: Carrier`); a `[Carrier]` list and the pointer warning; `sort_by` over interface values; `Carrier or Error` from a list `match` with `?` in it |
| `a06_news_feeds.lume` | a reader following a blog and an endless price ticker | an interface with a required `-> Iterator[Str]` (a list from one type, an endless generator from another), a generator default, and a plain default; functions taking `[Feed]` and giving `Iterator[Str]`; `zip` of a finite sequence with an endless one; a generator method on a struct with a `[Feed]` field |
| `a07_stock_manifest.lume` | parsing and summarising a warehouse manifest | a generator method over `text.lines.enumerate` with `next`; `(Str, Int, Str) or Error` with `map_error` then `?`; `?` inside a loop over a generator; triples through `for` and group_by, sum, filter, map and max_by; `pad` columns |
| `a08_free_slots.lume` (FAIL) | a room's free hours checked against a person's calendar | a generator method taking an interface value, which the docs list as "Not yet" |
| `a09_sensor_twice.lume` (FAIL) | sensor readings averaged and then asked for their peak | a generator method's result moved into an `Iterator[Int]` parameter, then used again |

## Guesses

1. **a01: `yield` as the single expression of a `match` arm** (`Doc(n, s) -> yield (...)`). The docs say `yield` may sit in a generator's `match`es, and show arms like `2 -> puts "two"`, but no example puts `yield` right after `->`. Looked in: collections.md "Generators", control-flow.md "`match`", generators.lume.
2. **a01: a generator method on a recursive enum, calling itself on a payload item** (`for t in k.walk(...)`). The docs say "a method of a struct or enum", but the only recursive example (`Node.names`) is a struct holding `[Node]`. I assumed the enum's boxed payload works the same way. Looked in: collections.md, structs-and-enums.md "Recursive enums".
3. **a02: a plain method `-> Iterator[Book]` that hands back a chain over its own field, with a block that reads a parameter** (`books.filter { |b| b.author == author }`). The docs show this for a free function over its parameter (`from_list`) and for a method over a range (`Squares.each_item`), never over `self`'s field. Looked in: collections.md "Taking a sequence", interfaces.md, interface_signatures.lume.
4. **a02: what a generator hands out when it yields a `var` list and then rebinds it** (`yield page` then `page = []`). I assumed each item is the list as it was at the `yield`. No example yields a `var` that changes afterwards. Looked in: collections.md "Generators".
5. **a03: a block, with a string literal in it, inside `#{...}`** (`"free per row: #{g.rows.map { |row| row.count { |s| s == "." } }.to_list}"`). The docs show nested quotes in interpolation only around a method call (`"#{"  ".repeat(depth)}"`), never a block. Looked in: printing.md, strings.md, generators.lume.
6. **a05: a concrete struct as the implied `Ok` of a `Carrier or Error`.** An arm gives `Flat(...)` where `Carrier or Error` is wanted, so the value has to become a `Carrier` and be wrapped in `Ok` in one step. Each conversion is documented on its own, never both together. Looked in: errors.md "A bare value is the implied ... `Ok`", interfaces.md "Where an interface can be used".
7. **a05: `sort_by` on a `[Carrier]`.** It builds a new list of interface values (copies through `lume_box`), and no example sorts interface values. Looked in: methods.md, interfaces.md.
8. **a05 and a06: the "holds values of different types behind `X`" warning.** No page under `docs/` mentions it. I took its text, its column (3, the binding's name) and its `help:` line from `examples/*.expected`. I guessed:
   - that it fires for `options: [Carrier] = [post, courier, post.with_surcharge(1)]`, where one item is already a `Carrier`;
   - that it fires for `feeds: [Feed] = [blog, tick]`;
   - that the interface name shows without type arguments, as `Source[Int]` showed as `Source`.

   Looked in: interface_signatures.expected, interface_methods.expected, generic_interfaces.expected.
9. **a06: no such warning for a mixed list given to a struct field** (`Reader(who: "kim", follows: [blog, tick])`). The examples only ever warn about local bindings. Looked in: the same `.expected` files, and structs-and-enums.md "A field may be an interface type", whose output does not show stderr.
10. **a06: a method `-> Iterator[Str]` that hands back a list** (`posts.reverse`). This is documented for a function ("a generator, a chain or a list"), and I assumed it holds for a method too, including one that fills a required interface method. Looked in: collections.md "Taking a sequence".
11. **a06: a generator method on a struct whose field holds interface values** (`Reader.follows: [Feed]`). "Not yet" names only a generator that *takes* an interface value. I assumed copying `self`, boxed items included, is fine, as it is for a generator default. Looked in: collections.md "Not yet", interfaces.md "A generator default walks a copy ... through the same `lume_box`".
12. **a07: `?` straight after a call that takes a block** (`parts.at(1).to_int.map_error { |e| ... }?`). `map_error` is shown without `?`, and no example puts `?` after a closing `}`. Looked in: errors.md "`Error(e) -> e`", methods.md.
13. **a08: the error text for an interface-value parameter on a generator method.** I reused `generator_block_param`'s message, which already says "a block or an interface value", and its column (the parameter's name). That example is a function with a block parameter, not a method with an interface parameter. Looked in: examples/errors/generator_block_param.*.
14. **a08: that this error comes first,** before anything else in the file (for example the trailing `yield ... if not ...`) could be reported. Looked in: examples/errors/UNREACHED.md (only one error is printed).
15. **a09: that moving a generator into an `Iterator[T]` parameter counts as "used up"**, giving the same message as `generator_used_twice` at the argument's column, and not something about the later `rs.max`. The docs say a parameter "takes ... by move" and that "using it again is refused", but no example shows the error for that case. Looked in: collections.md "Generators" and "Taking a sequence", examples/errors/generator_used_twice.*.

**15 guesses.**

## Summary of the gaps

The generator and interface-signature pages are clear on the main cases, and every program here could be written mostly from them. The gaps are where features meet:

- **Recursion:** a generator method on a recursive *enum*, as opposed to a struct, has no example.
- **Methods giving `Iterator[T]`:** nothing shows a plain method handing back a chain over `self`'s own fields, or a list.
- **Implicit conversions combined:** a concrete value that has to become an interface value and an `Ok` at the same time.
- **Error text:** the "Not yet" list (a generator taking an interface value) has no error text in the docs, only a sibling example for blocks, and handing a generator to a parameter and then using it again is described without an error example.
- **The pointer warning:** the biggest single gap is the "holds values of different types behind `X`" warning, which every program with a `[Interface]` binding produces and which no page under `docs/` mentions. Its wording, and when it fires (bindings only? fields? a list whose items are all the same type?), can only be learned from `examples/*.expected`.

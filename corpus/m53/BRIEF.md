# Round eleven — the brief every reviewer was given

This round tests milestones 50–52. Values now **move** instead of being
copied where they are not needed again: a list, text, a map, a set or a
struct at its last use; a local replaced by a value built from it
(`rows = rows.filter { .. }.to_list`, `acc = acc + w`); a struct's fields
one by one when the struct is finished with. The rules are in
`docs/reference/collections.md`, "When a list is copied, and when it
moves". Milestone 50 also added `for (i, x) in`, `enumerate` pairs as
values, and `sort_by` working each key out once.

**A move never changes what a program prints.** When the compiler gets a
move wrong, the program fails to build ("the generated Rust did not
compile") or uses a value it gave away. So every program here is **meant
to RUN**, and its value is in how close it walks to the edge: a value
moved in one place and read in another, in loops, branches, blocks, tasks,
`shared var`, interfaces, generics, across modules and packages.

**Read only:** everything under `docs/` except `docs/QUESTIONS.md`, and
everything under `examples/`. Nothing else: not the compiler source, not
`tests/`, not `corpus/` (except this brief), not `README.md`, not
`HISTORY.md`.

**Do not run the compiler.** Every expected output is worked out by hand from
what the docs say. A guess only counts as a guess if you could not check it.

**The number that matters** is how many things you had to guess. Record every
one, with where you looked.

A program is one file `corpus/m53/<id>_<name>.lume`, or a folder
`corpus/m53/<id>_<name>/` with `main.lume` (and a `lume.toml` if it is a
package; its dependencies beside it). The file that holds `main` starts
with a header: what it probes, `Meant to RUN.`, and `EXPECTED OUTPUT:` as
comment lines (a line that is just `#` is a blank output line).

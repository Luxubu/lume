# Reviewer B — the rules at the edges of packages

## What I read

- `corpus/m49/BRIEF.md`
- `docs/reference/packages.md` (all of it, several times)
- `docs/reference/modules.md` (all of it; `pub import`, `extend` travel, error forms)
- `docs/reference/modules/basics/` and `docs/reference/modules/reexport/` (the programs it quotes)
- `docs/reference/interfaces.md` (`extend`, fields bare inside an extend, `pub` and methods)
- `docs/reference/generics.md` (generic `extend`, "Two `extend`s of one interface")
- `docs/install.md`, `docs/README.md`, `docs/examples.md`
- `examples/packages/` (shelf, tally, labels)
- `examples/errors/packages/` (all eleven: crate_unlisted, crate_version, cycle, no_lib, not_listed, orphan, private, shadow, two_copies, two_sources, wrong_name)
- `examples/errors/`: private_access, missing_item, missing_module, import_twice, extend_overlap_modules, extend_concrete_overlap, extend_bounds_overlap, reexport/, travel/, typo, unknown_type, iface_missing, iface_extend_incomplete, crate_no_function
- `examples/crates.lume` (hex, urlencoding), `examples/crate.lume`, `examples/extend_coherence.lume`, `examples/travel/`, `examples/shipping/`

## The programs

| file | what it is | RUN/FAIL | leans on |
|---|---|---|---|
| `b01_passed_on` | `import shapes`, `import shapes.geo` (passed on), `import shapes.code` (a `pub` item by name); the library uses `hex` from its own `[rust]` and the app lists no crate | RUN | packages.md "Importing a package"; shelf; packages.md "Rust crates" |
| `b02_arrives_by_name` | only `import shapes`; uses `geo`, `Point` and `tx`, which lib.lume passes on as a module, an item and an alias | RUN | modules.md "`pub import`" carried across a package boundary |
| `b03_item_not_module` | lib.lume passes on `geo.Point` only; the app imports `shapes.geo` | FAIL | errors/packages/private |
| `b04_crate_version_in_dep` | a dependency writes `import rust.hex = "0.4"` in its lib.lume; the app's `[rust]` lists hex | FAIL | errors/packages/crate_version; "`lume check`: the package and every package it uses" |
| `b05_orphan_allowed` | the app extends its own `Jar` with `tally.Sized`, `labels.Label` with its own `Loud`, and `Int` with `Loud`; tally's `[T]` extend arrives | RUN | packages.md orphan rule; interfaces.md `extend` |
| `b06_orphan_other_lib` | the app extends `labels.Label` with `tally.Shouty` | FAIL | errors/packages/orphan |
| `b07_extend_through_dep` | the app depends on `labels` only, which imports `tally` privately; `xs.size` still works; the app has its own `tally.lume` | RUN | modules.md "`extend` travels"; packages.md "across packages"; errors/packages/shadow (by contrast) |
| `b08_list_of_own_type` | `bag` extends `[Bean]` (a list of its own type) with `tally.Tagged`, while tally extends `[T]` with it | FAIL | packages.md "Built-in types … belong to no package"; Rust's orphan rule |
| `b09_wrong_name` | the dependency key is the folder name `counter`; the package there is `tally` | FAIL | errors/packages/wrong_name |
| `b10_import_lib` | a package with main.lume and lib.lume; main.lume writes `import lib` | FAIL | packages.md "`import shelf`, not `import lib`" |

5 RUN, 5 FAIL.

## Guesses

1. **A dependency's crates need nothing in the app (b01).** Looked: packages.md "Rust crates", errors/packages/crate_unlisted. The page says a program is one Rust crate and two packages asking for one crate must agree, but never shows a crate that only a dependency uses. Assumed: the dependency's own `[rust]` is enough, and the app lists nothing.
2. **Passed-on modules arrive across a package boundary with just `import shapes` (b02).** Looked: modules.md "`pub import`" (they arrive under their own name) and packages.md, whose example writes `import tally.counts` as well as `import tally`, with the comment "passed on: `import tally.counts` works". Nothing says whether `counts` alone would also have worked in shelf. Assumed: yes, as between modules.
3. **How a passed-on item arrives (b02).** Looked: modules.md "One item can be passed on the same way: `pub import stats.mean`". It doesn't show the importer's side. Assumed: `pub import geo.Point` gives the importer a bare `Point`.
4. **A passed-on `Point` and `geo.Point` are one type (b02).** Not stated anywhere. Assumed: yes, so `q: geo.Point = p` type-checks.
5. **Passing on an item does not open its module (b03).** Looked: packages.md "only what its `lib.lume` makes public", errors/packages/private. Assumed: the module is still private, and the error is the private-module one word for word, with `pub import geo` in the help.
6. **An error inside a dependency's file (b04, b08).** Looked: modules.md "An error is reported against the file it is in"; packages.md says `lume check` covers every package used. There is no example with the error in a dependency. Assumed: it is reported at `codec/lib.lume` / `bag/lib.lume`, with the repo-relative path, and the crate help names the dependency's `lume.toml`, not the app's.
7. **A `[rust]` entry for a crate the package never imports (b04).** Not covered. Assumed: accepted silently, with no "unused" error, and it does not cover the dependency.
8. **A qualified extend target, `extend labels.Label with Loud` (b05, b06).** Looked: packages.md and errors/packages/orphan qualify only the interface (`with tally.Sized`); labels/lib.lume extends its own unqualified `Label`. Assumed: a prefixed type is accepted as a target.
9. **A library struct's fields are bare inside the app's extend (b05, b06).** interfaces.md says fields are bare in an extend of a struct, but shows only a struct from the same file. Assumed: this holds across packages.
10. **Orphan help wording when the type comes from a third package (b06).** Only the built-in form is shown ("`[T]` is built in"). Assumed: "`labels.Label` is from `labels`", parallel to "`tally.Sized` is from `tally`".
11. **An extend reaches the app through a dependency's private import of a package the app cannot import (b07).** modules.md says an extend goes up the chain of imports, and packages.md says it crosses packages, but the example has shelf import `tally` directly. Assumed: `xs.size` and `xs.empty?` work in an app that cannot name `tally`.
12. **A local module named like a package that is in the program but is not the app's dependency (b07).** packages.md lists "a dependency and a module of your own with the same name" as an error, and says two packages' `util`s never meet. It doesn't say which rule applies here. Assumed: allowed, and `import tally` means `app/tally.lume`.
13. **`[Bean]` is built in, not `bag`'s own (b08).** packages.md: "Built-in types (`Int`, `Str`, `[T]`, …) belong to no package." It doesn't say whether a list of your own type is yours. Assumed Rust's answer: an orphan. I also assumed the orphan check fires before the overlap check against tally's `[T]`, and that the type prints as written, `[Bean]`. If `[Bean]` counted as bag's own, the error would instead be the cross-package overlap, reported at an import.
14. **Manifest errors come before errors in main.lume (b09).** errors/packages/wrong_name's main.lume imports nothing. Assumed: the `lume.toml` error is the one printed, not "`counter` is not a dependency" or a missing module.
15. **`import lib` inside a package (b10).** packages.md says only "`import shelf`, not `import lib`", with no error shown, and outside a package `lib.lume` *is* the module `lib`. Assumed: it is an error. The whole message is invented: "`lib.lume` is the root of package `app`, so it is imported as `app`, not `lib`", with help "write `import app`".

**15 guesses.**

## Verdict

Once stated, the rules are clear and they fit Rust's model: a package is reached through `lib.lume`, only listed dependencies can be imported, and an extend needs to own one side. Where packages.md gives an error, the matching `examples/errors/packages/` file gives the exact text, including repo-relative paths, so every rule with an example cost nothing. The guesses cluster where two pages meet and neither says which rule wins:

- modules.md's `pub import` arrival rules against packages.md's `import tally.counts` example (2–4)
- "built-in" against "a list of my own type" in the orphan rule (13)
- extends and errors that come from a dependency the app doesn't import (6, 11)
- the one thing the page explicitly rules out with no error shown, `import lib` (15)

So mostly they are stated, but the boundary cases are stated only for the single-package situation, and the reader has to carry them across.

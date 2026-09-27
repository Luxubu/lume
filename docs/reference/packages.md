# Packages

> A folder of modules becomes something another program can use once it
> has a name and a front door. This page is Cargo's model, in Lume: a
> `lume.toml` names the package, `lib.lume` is what it offers, and the
> rules for `pub`, `pub import` and `extend` are the ones
> [Modules](modules.md) already has.

The program this page quotes is real and is run by `tests/run.sh`:
[`examples/packages/`](../../examples/packages/). It has three packages.
`shelf` is a program. It uses `tally` and `labels`, which are libraries, and
`labels` uses `tally` as well.

A single `.lume` file, or a folder of modules with no `lume.toml`, works
exactly as before. Nothing on this page applies to it.

## `lume.toml`

**A package is a folder with a `lume.toml` at its root.**

```toml
[package]
name = "shelf"
version = "0.1.0"

[dependencies]
tally  = { path = "../tally" }
labels = { path = "../labels" }

[rust]
regex = "1"
```

- `name` follows the rules for a Lume name: lower case letters, digits and
  `_`. The name `rust` is taken, because `import rust.…` means a crate.
- `version` is informational for now.
- A dependency is another package, found by `path` from this file's folder.
  It is named by its own name: `counter = { path = "../tally" }` is an error
  that says the package there is `tally`.
- `[rust]` gives the version of each Rust crate the package imports; see
  [Rust crates](#rust-crates).

`lume new shelf` makes the folder with a `lume.toml` and a `main.lume`.
`lume new tally --lib` makes a `lib.lume` instead.

## `main.lume` and `lib.lume`

**`main.lume` is where a program starts; `lib.lume` is the root of a
library,** as `main.rs` and `lib.rs` are in Rust. Both sit next to
`lume.toml`, and a package may have both. Then `main.lume` imports the
library by the package's own name, `import shelf`, not `import lib`.

**Module paths start at the package root,** the folder that holds
`lume.toml`, whichever file you run. For a program whose `main.lume` sits
there, that is where they started anyway.

## Importing a package

**A dependency is imported by its name, and the name means its
`lib.lume`:**

```lume-skip
# tally/lib.lume
pub import counts        # passed on: `import tally.counts` works
import util              # not passed on: private to tally

pub interface Sized:
  def size -> Int

  def empty? -> Bool = size == 0

extend [T] with Sized:
  def size -> Int = self.len

pub def total(xs: [Int]) -> Int:
  var t = 0
  for x in xs:
    t += x
  t
```

```lume-skip
# shelf/main.lume
import tally
import tally.counts
import labels
import util              # shelf's own util.lume, not tally's

def main:
  xs = [4, 8, 9]
  puts tally.total(xs)                   # 21
  puts xs.size                           # 3: the extend came with the import
  puts counts.count("xs", xs).line       # xs: 3
```

**From outside a package, only what its `lib.lume` makes public can be
reached.** That means its `pub` items (`import tally.total` works too) and
the modules it passes on with `pub import`. A module it does not pass on
is private to the package, as a module without `pub mod` is private to its
crate in Rust:

```
error: module `util` of package `tally` is private
  --> app/main.lume:1:1
  |
1 | import tally.util
  | ^
  help: only what `tally/lib.lume` makes public can be reached from outside the package; it would need `pub import util`
```

Inside a package nothing changes: every module may import every other.
Two packages may each have a module called `util`, and the two never meet.

**A package can use only the packages its own `lume.toml` lists.** A
dependency of a dependency is not enough:

```
error: `tally` is not a dependency of `app`
  --> app/main.lume:2:1
  |
2 | import tally
  | ^
  help: `tally` is a dependency of `mid`; to use it here, add `tally = { path = "…" }` under `[dependencies]` in `app/lume.toml` (it is at `tally`)
```

The following are all errors, and each message names what to change:

- A dependency and a module of your own with the same name (`tally.lume`
  next to a `tally` dependency).
- Two packages that depend on each other.
- One package name that leads to two different folders. A program has one
  copy of each package.
- Importing a package that has no `lib.lume`.

## `extend` between packages: the orphan rule

**An `extend T with I` must be written in the package that defines `I`, or
in the package that defines `T`.** This is Rust's orphan rule, and it is why
adding a dependency can never break a build. Without it, two packages that
know nothing of each other could both make `[T]` a `tally.Sized`, and a
program that used both would have two answers.

In the example, `tally` extends `[T]` with its own `Sized`, and `labels`
makes its own `Label` a `tally.Sized`. Both are allowed. A package that
owns neither side is refused:

```
error: package `app` cannot extend `[T]` with `tally.Sized`: neither is its own
  --> app/main.lume:4:1
  |
4 | extend [T] with tally.Sized:
  | ^
  help: as in Rust, an `extend` goes in the package that defines the interface or the type; `tally.Sized` is from `tally` and `[T]` is built in. Wrap the type in a struct of your own and extend that
```

Built-in types (`Int`, `Str`, `[T]`, maps, sets, tuples) belong to no
package. Only the package that defines the interface may extend them with
it.

The rule is about packages, not modules. Inside one package, every
`extend` that [Generics](generics.md#two-extends-of-one-interface-the-rust-rule)
allows is still allowed. An `extend` still travels with the import, across
packages as across modules: `xs.size` works in `shelf` because it imports
`tally`.

## Rust crates

A single file names a crate's version where it imports it:
`import rust.regex = "1"`. **Inside a package, the version goes in
`[rust]` of `lume.toml`**, as in `Cargo.toml`, and the file says only
`import rust.regex`. Writing the version in the file, or importing a crate
that `[rust]` does not list, is an error that says which line to add.

A whole program is compiled into one Rust crate, so it has one version of
each crate. Two packages that ask for the same crate must ask for versions
Cargo can meet together. `"1"` and `"1.5"` are fine, and the program gets
1.5 or later. `"0.3"` and `"0.4"` are not, and the error names both
packages.

A crate import still cannot be `pub`. A package that wants to expose a
crate's type wraps it.

## Commands

With no file named, a command uses the package the current folder is in.
It looks upward for a `lume.toml`, as Cargo does.

| | |
|---|---|
| `lume new <name> [--lib]` | makes a package |
| `lume run [-- args]` | runs the package's `main.lume`; a library has nothing to run |
| `lume build`, `lume check` | the package and every package it uses |
| `lume test` | this package's tests, not its dependencies' tests, as `cargo test` does; `main.lume`, or `lib.lume` in a library |
| `lume fmt [--check]` | every `.lume` file in the package; a package kept in a folder inside it is left alone |

Naming a file still works in a package: `lume run tools/report.lume`.

## Not yet

Dependencies come only from a `path` for now. Git dependencies and a
`lume.lock` that records exactly what was fetched are the next milestone,
and a registry comes after that.

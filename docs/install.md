# Installing Lume

Lume compiles to Rust and `rustc` compiles that to a native binary, so you
need a Rust toolchain — [rustup.rs](https://rustup.rs) is the usual way.

```sh
./install.sh
```

That builds the compiler (a minute or two the first time) and puts `lume` in
`~/.local/bin`. Pass a different directory if you want one:

```sh
./install.sh /usr/local/bin
```

If `~/.local/bin` is not on your `PATH`, the script says so and gives you the
line to add to your shell profile.

Check it:

```sh
lume --version
lume run examples/fib.lume
```

## Without installing

If you would rather not copy anything:

```sh
cargo build --release --manifest-path compiler/Cargo.toml
compiler/target/release/lume run examples/fib.lume
```

## The commands

```sh
lume run   file.lume [-- args...]   compile and run it
lume build file.lume [-o binary]    compile it, fully optimised
lume test  file.lume                run the file's `test` blocks
lume check file.lume                parse and type-check only
lume fmt   file.lume                rewrite in the canonical layout
lume emit  file.lume                print the generated Rust
lume crate file.lume <crate>        what a Rust crate offers, in Lume types
lume clean file.lume                remove build output
lume new   name [--lib]             make a package: a folder with lume.toml
lume update [package]               move git dependencies on, rewrite lume.lock
lume check file.lume --json         errors and warnings as JSON, for tools
lume lsp                            a language server, for editors
```

Inside a package the file can be left out: `lume run`, `lume test` and the
rest use the package the current folder is in. See
[Packages](reference/packages.md).

`lume run` trades some of the program's optimisation for compile speed, so
edit-and-run takes about 0.2 s. `lume build` is the one to measure.

**Rebuilding after an edit** takes about 0.75 s for a 9,000-line program and
1.6 s for 27,000 lines: Lume's own work is a small part of that (0.15 s at
27,000 lines, which is all `lume check` does), and rustc's incremental cache
does the rest. An unchanged program is not rebuilt at all.

Generated Rust and binaries go in a `.lume/` directory next to the source
file. Programs that use a crate or `async` are built with cargo, which shares
one build cache per machine (`~/.cache/lume/target`), so a crate is compiled
once rather than once per program.

## Editors

`editors/vscode/` is an extension for VS Code: highlighting, and Lume's
errors and warnings as you type. Its README says how to install it.

Other editors can use the same server. `lume lsp` speaks the Language Server
Protocol over stdin and stdout. It checks the program a file belongs to on
every change, using the text not yet saved, and reports what `lume check`
would. **Hover** shows what a name is: a local's type, a function's
signature, a struct's fields, a field's or method's type, or what a built-in
method gives back there. **Go to definition** jumps to where it was
written, in this file, another module or another package. For tools that want data rather than text, `lume check --json`
prints every error and warning with its file, line, column, message and
help:

```sh
lume check --json main.lume
```

```json
{
  "diagnostics": [
    {
      "col": 8,
      "file": "main.lume",
      "help": null,
      "line": 4,
      "message": "`add` is missing an argument: b",
      "severity": "error"
    }
  ],
  "ok": false
}
```

The exit status is 1 when there is an error, as with `lume check`.

## What to read next

- [A tour of Lume](tour.md) — half an hour, start to finish.
- [The reference](README.md) — the specific answers.

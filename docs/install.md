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
```

`lume run` trades some of the program's optimisation for compile speed, so
edit-and-run takes about 0.2 s. `lume build` is the one to measure.

Generated Rust and binaries go in a `.lume/` directory next to the source
file. Programs that use a crate or `async` are built with cargo, which shares
one build cache per machine (`~/.cache/lume/target`), so a crate is compiled
once rather than once per program.

## What to read next

- [A tour of Lume](tour.md) — half an hour, start to finish.
- [The reference](README.md) — the specific answers.

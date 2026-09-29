# Rust crates

> Any crate on crates.io can be used from Lume. `examples/crate.lume` and
> `examples/crates.lume` show it working, and `examples/crate_builder.lume`
> shows a builder.

## Importing one

```lume-skip
import rust.regex = "1"          # a single file names the version here
```

In a package the version goes in `[rust]` of `lume.toml` instead, and the
file says only `import rust.regex`; see [Packages](packages.md#rust-crates).

Lume reads the crate's signatures, so its functions and types are ordinary
Lume, with no bindings to write:

- `regex.Regex.new(p)` is a function of a type;
- `re.is_match(s)` is a method;
- a Rust `Result` is a `T or Error`, and `?` works on it;
- an `Option` is a `T?`;
- a crate iterator is a lazy chain.

**`lume crate <file> <crate>` lists what the crate offers, in Lume types**,
and says why for anything Lume cannot call yet: a parameter of a type Lume
has no word for, a generic bound it cannot meet, an `unsafe` or `async`
function.

## Builders: methods that use their value up

Most Rust builders take `self` by value. Each step uses the value up and
gives back a new one: `ureq.get(url).set("Accept", "application/json")`.
`lume crate` marks such a method `(uses the value up)`, and Lume calls it as
Rust would:

- **In a chain**, the value each step makes goes straight into the next
  step.
- **A local that nothing needs afterwards** moves into the call, by the same
  rule every Lume value moves by (see
  [Collections](collections.md#when-a-list-is-copied-and-when-it-moves)).
- **A local that is needed again** is copied into the call, when the crate's
  type can be cloned. `base.set(..)` twice works on two copies, and `base`
  is left as it was.
- Otherwise the call is refused: ``… uses up `x`, and `x` is still needed
  after it``. Make it the last use, or make a new value where one is needed
  again.

## When the bridge cannot

A `rust:` block holds Rust that Lume puts into the program as it is. Its
value takes the type written where it goes, and Lume's names are visible
inside it:

```lume-skip
def word_count(s: Str) -> Int:
  rust("s.split_whitespace().count() as i64")

  raw: (Int, Str) or Error = rust:
    { Ok((1, String::from("one"))) }
```

`packages/http/` uses one for the few lines where it needs a crate's error
type itself. See [HTTP](http.md).

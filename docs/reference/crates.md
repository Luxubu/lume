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

## How a crate's types are read

| Rust | Lume |
|---|---|
| `&str`, `String`, `Cow<str>` | `Str` |
| `bool` | `Bool` |
| `i64`; `i8`–`i32`, `u8`–`u64`, `isize`, `usize` | `Int`, converted with a check: an `Int` the crate's type cannot hold stops the program — ``-1 does not fit in the Rust `usize` the crate asks for`` — as Rust's `TryFrom` would, and so does a `u64` past the largest `Int` coming back |
| `f64`; `f32` | `Float` |
| `()` | nothing |
| `Vec<T>`, `&[T]` | `[T]` |
| `Option<T>` | `T?` |
| `Result<T, E>` | `T or Error`, with the crate's error as the message |
| a tuple | a tuple |
| a type of the crate | that type, by the crate's name, as `regex.Regex`; one that iterates is a lazy chain of its items |
| `Box<T>` | `T` |

A borrowed parameter, `&T`, is lent; `&mut T` is lent to change, for a
crate's type, a `String` or a `Vec`. A function whose parameter or result
is none of these — a generic Lume cannot meet, a closure, `dyn Trait`, a
fixed-size array, an `unsafe` or `async` function — is not offered, and
`lume crate` says which and why.

## What is promised

`import rust.<crate>`, the `[rust]` table of `lume.toml`, the table above
and the rules for builders below are settled in 0.8 ([what is
settled](../stability.md)): a crate function Lume offers keeps being
offered, in the same Lume types, for the same crate version. What a crate
itself does, and which of its functions a newer version of it has, are the
crate's. **A `rust:` block is outside the promise, and will stay so**: it is
Rust that Lume puts into the program as it is, next to the Rust Lume writes,
whose names and shape change freely — as Rust keeps `unsafe` code and its
nightly features apart from what it promises.

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

## When the bridge cannot: `rust:`

A `rust:` block — outside the promise, as above — holds Rust that Lume puts
into the program as it is. Its
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

**A stream of bytes from Rust becomes an async sequence** through a number
it is parked under: `lume_park(Box::new(reader))` takes anything that is
`std::io::Read + Send` and gives an `Int`, and `lume_parked_lines(id)` or
`lume_parked_chunks(id)` give it back as an `async Iterator[Str]`, read by a
thread of its own as it arrives. That is how `http`'s `Opened.lines` reads
a body:

```lume-skip
  def lines -> async Iterator[Str]:
    id = body_id
    s: async Iterator[Str] = rust:
      lume_parked_lines(id)
    s
```

Inside a method, the Rust names a field as `self.field`; binding it to a
name first, as here, keeps the Rust short.

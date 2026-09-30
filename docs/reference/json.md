# JSON

> A first-party package, not part of the language: `packages/json/` in this
> repository. It began as the milestone 19 dogfood program, `examples/json/`.

## Using it

In a package's `lume.toml`, by path from a checkout of this repository, or
by git from anywhere:

```toml
[dependencies]
json = { path = "../lume/packages/json" }
# or
json = { git = "https://github.com/Luxubu/lume", tag = "…" }
```

The repository holds more than one package. A git dependency finds the one
named `json` by the name in its `lume.toml`, as Cargo does; see
[Packages](packages.md#packages-from-git-and-lumelock).

`examples/json_config/` is a whole program built on it, and its output is
checked by `tests/run.sh`.

`import json` is all a file needs: the package's `ToJson` extends of text,
numbers, `Bool` and lists come with it, as any `extend` does. Import
`json.Json` or `json.ToJson` by name only to write them without the prefix.
Messages still name them `json.Json` and `json.ToJson`.

## Reading

```lume-skip
import json
import json.Json

doc = json.parse(text)?          # Json or Error; the error gives the position
puts json.show(doc)              # one line, no spaces
puts json.pretty(doc)            # two spaces per level
```

A `Json` is one of `Null`, `Bool(b)`, `Num(n)` (a `Float`), `Str(s)`,
`Arr(items)` and `Obj(pairs)`. An object keeps its keys in the order they
were written. Match on it, or ask for what you expect:

| method | gives | |
|---|---|---|
| `get(key)` | `Json?` | the value under a key of an object |
| `at(i)` | `Json?` | an item of an array |
| `path("users.0.name")` | `Json?` | keys and positions separated by dots |
| `str`, `num`, `int`, `bool` | `Str?`, `Float?`, `Int?`, `Bool?` | the value, when it is that kind (`int` wants a number with no fraction, up to 2^53 either way, which a `Float` holds exactly) |
| `null?` | `Bool` | |
| `items` | `[Json]` | an array's items, or none; so `for x in doc` walks an array, and `doc.filter { .. }` works on it |
| `pairs`, `keys` | `[(Str, Json)]`, `[Str]` | an object's entries, or none |
| `kind` | `Str` | `"null"`, `"bool"`, `"number"`, `"string"`, `"array"` or `"object"` |

## Decoding into your own types

Lume has no derive, so a type decodes itself: `need_…` gives a
`T or Error` whose message names the field and what was wrong with it, and
`?` passes the first failure up:

```lume-skip
struct Member:
  name: Str
  years: Int

  def self.from_json(j: Json) -> Member or Error:
    Member(name: j.need_str("name")?, years: j.need_int("years")?)
```

| method | gives | the error, when it fails |
|---|---|---|
| `need(key)` | `Json or Error` | ``` `key` is missing ``` |
| `need_str(key)`, `need_num(key)`, `need_int(key)`, `need_bool(key)` | `T or Error` | ``` `years` is a string, not a whole number ``` |
| `need_items(key)` | `[Json] or Error` | ``` `members` is an object, not an array ``` |
| `opt_str(key)`, `opt_int(key)`, `opt_num(key)`, `opt_bool(key)` | `T? or Error` | missing or `null` is `None`; only the wrong kind is an error |

An optional field with a default is `opt_…` and `.or`, as serde reads an
`Option` field:

```lume-skip
port = j.opt_int("port")?.or(8080)     # absent or null: 8080; "80": an error
```

A number too large to be an `Int` exactly says so:
``` `size` is too large a number to be an `Int` exactly ```.

Add where you are with `map_error`, as `examples/json_config/` does for each
member of a list: ``Error("member #{i + 1}: #{e.message}")``.

## What `parse` refuses

It is strict RFC 8259, and every error ends with where it stopped:

| text | error |
|---|---|
| `[1, 2` | ``expected `,` or `]` at position 5`` |
| `{"a" 1}` | ``expected `:`, found `1` at position 5`` |
| `[1,]` | ``unexpected character `]` at position 3`` |
| `tru` | ``expected `true` at position 3`` |
| `"a\x"` | ``bad escape `\x` at position 4`` |
| `"\ud800"`, `"\udc00"`, `"\ud800A"` | ``lone surrogate at position …`` |
| `"abc` | ``unterminated string at position 4`` |
| `01` | ``a number cannot start with `0` at position 0`` |
| `1 2` | ``trailing characters at position 2`` |
| (nothing) | ``unexpected end of input at position 0`` |

Printing is strict too: a character below a space other than `\n`, `\t` and
`\r` is written as `\u00XX`, so what `show` writes always parses back.

## Writing

A type with `def to_json -> Json` fits the package's `ToJson` interface.
`Str`, `Int`, `Float`, `Bool` and lists of anything with `to_json` have it
already, so a list of your own values writes itself:

```lume-skip
struct Member:
  name: Str
  years: Int

  def to_json -> Json = json.obj([("name", name.to_json), ("years", years.to_json)])

puts json.show(members.to_json)   # [{"name":"Ada","years":12},...]
```

`json.obj(pairs)` makes an object with its keys in that order. A number with
no fraction, up to 2^53, is written without one: `12`, not `12.0`.

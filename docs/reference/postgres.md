# PostgreSQL

> A first-party package, not part of the language: `packages/postgres/` in
> this repository, built on the Rust crate postgres. **A preview**, new in
> 1.3: its shape may change in a 1.x release, each change listed in
> `CHANGELOG.md` under **Preview:** ([what is settled](../stability.md#previews)).
> `examples/postgres_demo/` tries every part of it against a real server.

## Using it

```toml
[dependencies]
postgres = { git = "https://github.com/Luxubu/lume", tag = "…" }
json = { git = "https://github.com/Luxubu/lume", tag = "…" }   # for JSON columns
```

```lume-skip
import postgres

def main -> () or Error:
  db = postgres.connect("postgresql://app:secret@localhost:5432/app")?
  db.execute("INSERT INTO people (name, age) VALUES ($1, $2)", ["Ada".sql, 36.sql])?
  for row in db.query("SELECT name, age FROM people WHERE age > $1", [30.sql])?:
    puts "#{row.str("name")?} is #{row.opt_int("age")?.or(0)}"
  ()
```

**`postgres.connect(url)` gives a `Db`**, or an `Error` that names the URL
with its password shown as `***`. The URL is
`postgresql://user:password@host:port/dbname`, or the `key=value` form
`host=localhost user=app dbname=app`.

## Statements and queries

| on a `Db` | gives | |
|---|---|---|
| `execute(sql, params)` | `Int or Error` | runs a statement: how many rows it changed |
| `query(sql, params)` | `[Row] or Error` | runs a query: its rows |
| `query_one(sql, params)` | `Row or Error` | the one row: an `Error` for none, or more |
| `query_opt(sql, params)` | `Row? or Error` | the row or `None`: an `Error` for more than one |
| `batch(sql)` | `() or Error` | statements with no parameters, any number of them: a schema, say |
| `close` | `()` | ends the connection; using it after is an `Error` |

**Parameters are `$1`, `$2` and so on in the SQL, and a list of values
beside it.** A value goes to the server apart from the query's text, so what
a user typed — `O'Neil; DROP TABLE people` — is a value and nothing more,
and is never written into the SQL by hand.

| parameter | is | for a parameter of type |
|---|---|---|
| `"Ada".sql` | `Text` | `text`, `varchar`, `char`, `name` |
| `36.sql` | `Int` | `smallint`, `int`, `bigint` — an `Error` if it does not fit — or `real`, `double precision` |
| `9.5.sql` | `Float` | `real`, `double precision` |
| `true.sql` | `Bool` | `boolean` |
| `postgres.data(doc)` | `Data` | `json`, `jsonb`: a `json.Json` document |
| `postgres.maybe(v)` | the value, or `Null` | anything: an optional value, `None` as SQL's `NULL` |

The statement is prepared first, so the server says what type each `$n`
is, and each value is sent as that type: `36.sql` goes to an `int` column
and a `bigint` alike. A value of another kind is an `Error` naming the
parameter and the type: ``$1: text cannot go to a parameter of type `int4` ``.

## Rows

**A `Row` is read by column name.** Each read gives the value or an
`Error`; the `opt_` form gives `None` for SQL's `NULL`, and the plain form
an `Error`, for a column that is never `NULL`:

| on a `Row` | gives |
|---|---|
| `int(name)`, `opt_int(name)` | `Int`, from `smallint`, `int` or `bigint` |
| `float(name)`, `opt_float(name)` | `Float`, from `real` or `double precision`, or an integer column |
| `str(name)`, `opt_str(name)` | `Str`, from `text`, `varchar`, `char` or `name` |
| `bool(name)`, `opt_bool(name)` | `Bool` |
| `json(name)`, `opt_json(name)` | `json.Json`, from `json` or `jsonb` |
| `get(name)` | the `Value` as it came |
| `columns`, `values` | the columns' names, and their values, in order |

A column not there, or of another kind, is an `Error` that says so:
``no column `nmae`: the row has `name`, `age` `` and ``column `age` is
NULL``.

**Other column types** — `uuid`, `timestamp`, `date`, `numeric` and the
rest — are not read yet: the query says so, and how to read one as text
meanwhile: ``column `id` is a `uuid`, which this package does not read yet:
write `id::text` in the query``.

## When it fails

Every `Error` starts with the statement, cut short, then the server's own
words:

```text
`SELEC name FROM people`: syntax error at or near "SELEC"
`SELECT * FROM nobody`: relation "nobody" does not exist
`INSERT INTO people (name) VALUES ($1)`: null value in column "name" of relation "people" violates not-null constraint
`SELECT name FROM people WHERE age > $1 AND score > $2`: it has 2 parameters, and 1 was given
```

## Many tasks, one connection

**Copies of a `Db` share its one connection, and take turns with it**: a
task given `db` waits while another task's statement runs, so twenty tasks
inserting at once insert twenty rows. The connection lives on a thread of
its own, and each call hands its statement to that thread and waits for the
answer, as `http`'s `send` waits for its server. That works the same from
plain code and from async code; to let other tasks go on meanwhile, run a
call in a task of its own, `spawn: db.query(...)`.

## Trying it on your machine

A server in Docker is the quickest, on a port of its own:

```sh
docker run -d --name lume-postgres -e POSTGRES_USER=lume -e POSTGRES_PASSWORD=lume -e POSTGRES_DB=lume -p 127.0.0.1:5440:5432 postgres:16-alpine
```

`postgresql://lume:lume@127.0.0.1:5440/lume` is then the URL. The suite's
check, `examples/postgres_demo/check.sh`, uses that server, or the one
`LUME_PG_URL` names, and says it skipped where there is none.

## Not in it yet

Each of these could come in a later release without changing what is here:

- **TLS**, for a server on another machine: until then, a tunnel.
- **A pool** of connections, so tasks run statements at the same time.
- **Transactions**: meanwhile, `batch` with `BEGIN` and `COMMIT` in it,
  for statements that need no parameters.
- **`COPY`**, for loading many rows at once.
- **The types above** that are read as text for now.

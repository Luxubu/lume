# HTTP

> A first-party package, not part of the language: `packages/http/` in this
> repository, built on the Rust crate ureq. `examples/http_demo/` uses it,
> and `tests/http_local.sh` runs that against a server on this machine.

## Using it

```toml
[dependencies]
http = { git = "https://github.com/Luxubu/lume", tag = "…" }
json = { git = "https://github.com/Luxubu/lume", tag = "…" }   # for .json
```

```lume-skip
import http
import json

def main -> () or Error:
  r = http.get("https://api.example.com/items")?
  if not r.ok?:
    warn "the server said #{r.status}"
    Env.exit(1)
  items = r.json?                    # the body, parsed
  puts items.path("0.name").map(json.show).or("none")
  ()
```

## Requests

| call | sends |
|---|---|
| `http.get(url)`, `http.delete(url)` | no body |
| `http.post(url, body, content_type)`, `http.put(…)` | a body, with its `Content-Type` |
| `http.post_json(url, doc)` | a `Json` document, as `application/json` |
| `http.send(method, url, headers, body)` | any method, headers of your own as `[(Str, Str)]`, a body (`""` for none) |

Each gives `Response or Error`. **Only a request that could not be made is
an `Error`**: no connection, a URL that does not parse, no answer in time
(30 seconds, unless the request says otherwise). The message names the method and the URL. **A 404 or a 500 is a
`Response`** like any other, as Python's `requests` and Rust's `reqwest`
have it: look at `ok?` or `status`. **A redirect is followed**, up to
five, as ureq does: a `301`, `302`, `303`, `307` or `308` with a
`Location` gives the answer from there. Any other status, a `201` with a
`Location` among them, is the answer.

### Built a step at a time

**`http.request(method, url)` gives a `Request`**, and each step gives a
new one, as Rust's `reqwest::RequestBuilder` does:

| step | |
|---|---|
| `.header(name, value)` | one more header; of two with one name, in any case, only the later is sent |
| `.with_body(text, content_type)` | a body, with its `Content-Type` |
| `.with_file(path, content_type)` | a file as the body, read as it is sent, never held whole; its length goes first, as `Content-Length` |
| `.with_form(pairs)` | a form as the body, `[(Str, Str)]` written as `http.form` writes it, as `application/x-www-form-urlencoded` |
| `.with_query(pairs)` | names and values added to the URL's query — after a `?`, or the query it has, and before a `#fragment` |
| `.timeout(ms)` | how long to wait for an answer, in milliseconds |

A body step sets the request's `Content-Type`, taking the place of one an
earlier step gave; a later body step takes the place of an earlier body.

and then one of:

| call | gives | |
|---|---|---|
| `req.send` | `Response or Error` | waits for the whole answer |
| `await req.fetch` | `Response or Error` | the same, in async code, as other tasks go on |
| `req.open` | `Opened or Error` | the status and headers as soon as they arrive; the body after, as it arrives |
| `req.send_parts(parts)` | `Response or Error` | a body of the parts of an `Iterator[T]` of `Str`, each sent as the sequence makes it, then waits for the whole answer |

```lume-skip
r = http.request("GET", url).header("Accept", "text/plain").timeout(5000).send?
```

## Responses

| field or method | is |
|---|---|
| `status` | the status code, an `Int` |
| `ok?` | a 2xx status |
| `body` | the body, as text |
| `header(name)` | `Str?`, the name in any case |
| `headers` | `[(Str, Str)]`, names in lower case, in the order sent |
| `json` | `Json or Error`: the body, parsed with the `json` package |

## A body as it arrives: `open`

**`req.open` gives an `Opened`**: `status`, `headers`, `ok?` and
`header(name)` at once, and the body after, through one of

| method | gives |
|---|---|
| `o.lines` | `async Iterator[Str]`: a line at a time, without line endings, as the server writes them |
| `o.chunks` | `async Iterator[Str]`: the pieces as they arrive, each whole characters |

so a large body, or one the server writes slowly — a log, a feed — is
handled as it comes, never held whole. The body can be walked once. These
are async sequences, with their chains and methods
([Async generators](collections.md#async-generators-async-def-with-yield)),
so they are walked in async code:

```lume-skip
async def main -> () or Error:
  o = http.request("GET", "https://example.com/events").open?
  puts o.status
  for line in o.lines.filter { |l| l.starts_with?("data:") }.take(10):
    puts line
  ()
```

## A body sent as it is made

**`.with_file(path, content_type)` sends a file as it is read**, so a
large one is never held whole; a file that is not there is an `Error`,
named with its method and URL, as every request's `Error` is: ``PUT
http://example.com/up: cannot read `/no/such/file`: No such file or
directory (os error 2)``.
**`req.send_parts(parts)` sends a body made of a sequence's parts**, each
as the sequence makes it — a generator's, say — in chunks, as HTTP/1.1
sends a body whose length is not known yet, the way Rust's `reqwest` sends
a body from a stream:

```lume-skip
def rows(n: Int) -> Iterator[Str]:
  for i in 1..n:
    yield "row #{i}\n"

def main -> () or Error:
  up = http.request("PUT", "https://example.com/upload").with_file("big.csv", "text/csv").send?
  puts up.status
  made = http.request("POST", "https://example.com/rows").header("Content-Type", "text/plain").send_parts(rows(10000))?
  puts made.status
  ()
```

`send_parts` takes the parts by move, as any `Iterator[T]` parameter does,
and sets aside a body `with_body` or `with_file` gave.

## URLs and forms

A URL carries only some characters as they are, and a form — what a
browser sends from a `<form>`, and what a query is — writes its names and
values a particular way. **These write and read them**, as Python's
`urllib.parse` does:

| call | gives | |
|---|---|---|
| `http.encode(text)` | `Str` | a piece of a URL — a path segment, a value — with every byte but letters, digits and `-._~` as `%XX` |
| `http.decode(text)` | `Str or Error` | the other way |
| `http.form(pairs)` | `Str` | names and values as `a=1&b=two+words`: each encoded, a space as `+` |
| `http.parse_form(text)` | `[(Str, Str)] or Error` | the other way, in order; a name with no `=` has the value `""` |
| `http.value(pairs, name)` | `Str?` | the value of `name` in such a list, the first if it comes twice: `http.value(req.params?, "q")` |

```lume-skip
import http

def main:
  puts http.encode("a café")                    #=> a%20caf%C3%A9
  puts http.decode("a%20caf%C3%A9")             #=> Ok("a café")
  puts http.form([("q", "pie & tea"), ("n", "2")])    #=> q=pie+%26+tea&n=2
  puts http.parse_form("q=pie+%26+tea&n=2")     #=> Ok([("q", "pie & tea"), ("n", "2")])
  puts http.decode("100%")
  #=> Error("`%` in `100%` is not a byte written as `%` and two hex digits")
```

**Decoding is an `Error`** for a `%` without two hex digits after it, and
for bytes that are not UTF-8 text: ``"`%FF` does not decode to UTF-8
text"``. The message names the text given to `decode`; for `parse_form`, the
name or value that would not decode. A query or a form is a list of pairs, not a map, because a name
may come more than once, and the order can matter; to look one up, use
`find`.

## Many requests at once

A request waits for its answer. To make many at once, `await req.fetch`
in `spawn:` bodies; or make each with `send` in a `spawn:` body that
awaits nothing, and it runs on a pool of its own (see
[Concurrency](concurrency.md#a-task-that-waits-without-await)).
`examples/healthcheck/` does this, and `examples/http_demo/` does both.

## Serving: a preview

> **A preview**, new in 1.1: its shape may change in a 1.x release, with
> each change listed in `CHANGELOG.md` under **Preview:**
> ([what is settled](../stability.md#previews)). Requests, above, are
> settled. It is built on the Rust crate tiny_http.

**`http.listen(address)` gives a `Server`**, or an `Error` when the
address cannot be listened on — one taken already, or one that does not
parse. Port 0 asks the system for a free one, which `server.port` then
says:

```lume-skip
import http

def main -> () or Error:
  server = http.listen("127.0.0.1:8080")?
  puts "listening on http://#{server.address}"
  for req in server.requests:
    match (req.method, req.path):
      ("GET", "/") -> req.respond(http.reply(200, "hello"))?
      _ -> req.respond(http.reply(404, "no #{req.path} here"))?
  ()
```

| on a `Server` | |
|---|---|
| `address`, `port` | where it listens: `127.0.0.1:8080` and `8080` |
| `accept` | waits for the next request: `Incoming or Error`, an `Error` once the server is closed |
| `requests` | every request as it comes, for a `for`, until the server is closed |
| `incoming` | the same as an async sequence: in async code, other tasks go on while it waits |
| `close` | stops listening; an `accept` waiting then gives an `Error` |

**Each request is an `Incoming`**, its body read whole before it is given:

| on an `Incoming` | |
|---|---|
| `method`, `path` | `"GET"`, `"/menu"`: the path without its query, as it was sent — `%20` stays `%20`; `http.decode` reads it |
| `query` | what follows `?` in the URL, as it was sent; `""` for none |
| `headers`, `header(name)` | as a `Response`'s: names in lower case, found in any case |
| `body`, `json` | the body as text, or parsed |
| `params` | the query's names and values, decoded as `http.parse_form` decodes them |
| `form` | a form sent as the body, decoded the same way |
| `respond(reply)` | sends the answer: `() or Error` |

**A request is answered once**: a second `respond` is an `Error` — ``GET
/menu has been answered already`` — and so is a status outside 100 to 599:
``600 is not an HTTP status: one from 100 to 599``. A refused answer sends
nothing, so the request can still be answered.
A request never answered keeps its connection open until the program
ends.

**An answer is a `Reply`**, built as a request is:

| | |
|---|---|
| `http.reply(status, text)` | text, as `text/plain; charset=utf-8` |
| `http.reply_json(status, doc)` | a `Json` document, as `application/json` |
| `.header(name, value)` | one more header; of two with one name, in any case, only the later is sent |
| `.with_body(text, content_type)` | a body, in place of the one before |

`requests` handles one request at a time: the next waits until this one is
answered. A `for` over it ends when the server is closed, or at a `break`,
or after `.take(n)`. To answer many at once, walk `incoming` in async code and give
each request to a task, as `spawn: answer(req)`.

`examples/http_server/` is a server and its own client in one program: it
listens on a port the system picks, and a task makes the requests it
answers. `examples/notes/` is a whole service: notes kept in a JSON file,
a task per request on one `shared var` store, and every method a REST
service needs.

## Not in it yet

Each of these could come later without changing what is here:

- **Cookies:** there is no cookie jar. Send and read the `Cookie` and
  `Set-Cookie` headers yourself.
- **For serving**: TLS (`https://`), a body read as it arrives, and
  answers sent as they are made.

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
have it: look at `ok?` or `status`.

### Built a step at a time

**`http.request(method, url)` gives a `Request`**, and each step gives a
new one, as Rust's `reqwest::RequestBuilder` does:

| step | |
|---|---|
| `.header(name, value)` | one more header |
| `.with_body(text, content_type)` | a body, with its `Content-Type` |
| `.timeout(ms)` | how long to wait for an answer, in milliseconds |

and then one of:

| call | gives | |
|---|---|---|
| `req.send` | `Response or Error` | waits for the whole answer |
| `await req.fetch` | `Response or Error` | the same, in async code, as other tasks go on |
| `req.open` | `Opened or Error` | the status and headers as soon as they arrive; the body after, as it arrives |

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

## Many requests at once

A request waits for its answer. To make many at once, `await req.fetch`
in `spawn:` bodies; or make each with `send` in a `spawn:` body that
awaits nothing, and it runs on a pool of its own (see
[Concurrency](concurrency.md#a-task-that-waits-without-await)).
`examples/healthcheck/` does this, and `examples/http_demo/` does both.

## Not yet

- **A body sent as it is made:** a request's own body is text, sent whole.
- **Encoding and cookies:** there are no form or URL helpers and no
  cookies. Build the body as text, and send headers with `send`.

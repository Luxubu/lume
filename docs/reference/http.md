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
an `Error`**: no connection, a URL that does not parse, no answer within 30
seconds. The message names the method and the URL. **A 404 or a 500 is a
`Response`** like any other, as Python's `requests` and Rust's `reqwest`
have it: look at `ok?` or `status`.

## Responses

| field or method | is |
|---|---|
| `status` | the status code, an `Int` |
| `ok?` | a 2xx status |
| `body` | the body, as text |
| `header(name)` | `Str?`, the name in any case |
| `headers` | `[(Str, Str)]`, names in lower case, in the order sent |
| `json` | `Json or Error`: the body, parsed with the `json` package |

## Not yet

- **Streaming:** a request is made and waited for in full, so there is no
  streaming of large bodies, and no `async` call.
- **Fixed timeout:** it is 30 seconds, with no way to change it.
- **Encoding and cookies:** there are no form or URL helpers and no
  cookies. Build the body as text, and send headers with `send`.

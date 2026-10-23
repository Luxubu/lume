# Lunch 19: HTTP

> **A 0.5 preview.** This lunch teaches one of Lume's newest parts. It works
> as shown, but details may change before 1.0. Most of its examples talk to
> a real web server, so the test suite cannot run them on every change; they
> were run by hand, against `example.com`, a site kept for examples like
> these. The ones that need no network are in `docs/tutorial/http-requests/`,
> which the suite does run.

**In this lunch you will:**

- use a package: `http`, which comes with Lume;
- fetch a web page, and read its status, headers and body;
- build a request a step at a time, with a timeout;
- read a large body as it arrives, and make many requests at once.

## 19.1 Using a package

HTTP is not built into the language; it comes in a **package**, `http`,
which lives in Lume's repository next to `json`. A program that uses a
package is a package itself — lunch 12's folder with a `lume.toml` — and
says what it depends on:

```sh
lume new pages
cd pages
```

Then add the dependency to `pages/lume.toml`:

```toml
[package]
name = "pages"
version = "0.1.0"

[dependencies]
http = { git = "https://github.com/Luxubu/lume", tag = "v0.5.0" }
```

Lume fetches the package the first time you run the program. From then on
`import http` brings it in, and `lume run`, in the folder, runs `main.lume`.

## 19.2 Fetching a page

**`http.get(url)` fetches a page** and gives a `Response`, or an `Error` if
the request could not be made:

```lume-skip
import http

def main -> () or Error:
  r = http.get("https://example.com/")?
  puts r.status                                  # 200
  puts r.ok?                                     # true
  puts r.header("content-type").or("none")       # text/html; charset=utf-8
  puts r.body.contains?("Example Domain")        # true
```

A `Response` has the **status** — `200` for "here it is", `404` for "not
found", `500` for "the server broke" — and `ok?`, which is `true` for any
status in the 200s. `header(name)` reads a header, whatever its case, and
`body` is the page itself, as text.

**Only a request that could not be made is an `Error`**: no connection, a
URL that does not parse, no answer in time. A page that is not there is a
`Response` with the status `404`, because the server did answer:

```lume-skip
import http

def main -> () or Error:
  r = http.get("https://example.com/no-such-page")?
  puts r.status    # 404
  puts r.ok?       # false
  match http.get("not a url"):
    Ok(_) -> puts "fetched?"
    Error(e) -> puts "could not make it"    # could not make it
```

It is the same rule as lunch 15's programs: a program that ran and failed
gave an exit code, not an `Error`. Look at `ok?` or `status` to decide
what an answer means.

## 19.3 Building a request

`get` is the common case. For the rest, **`http.request(method, url)` builds
a request a step at a time**, each step giving a new one, and `send` makes
it:

```lume-skip
import http

def main -> () or Error:
  req = http.request("GET", "https://example.com/").header("Accept", "text/html").timeout(5000)
  r = req.send?
  puts r.status    # 200
```

| step | |
|---|---|
| `.header(name, value)` | a header to send |
| `.with_body(text, content_type)` | a body, and what kind of text it is |
| `.timeout(ms)` | how long to wait for an answer, in milliseconds |

Without `.timeout`, a request waits up to 30 seconds. A server slower than
that gives an `Error` saying it timed out. `http.post(url, body,
content_type)` and `http.post_json(url, doc)` are shortcuts for sending a
body; the second sends a JSON document made with the `json` package.

## 19.4 A body as it arrives

`body` holds the whole page, which is fine for a page. For something large
— a big file, a feed that keeps coming — **`req.open` gives the status and
headers at once, and the body afterwards, as it arrives**, as an async
sequence of lines (lunch 17):

```lume-skip
import http

async def main -> () or Error:
  page = http.request("GET", "https://example.com/").open?
  puts page.status    # 200
  line = page.lines.find { |l| l.contains?("<title>") }.or("")
  title = line.split("<title>").at(1).split("</title>").at(0)
  puts title    # Example Domain
```

`find` stops reading at the first line holding the title, so on a long
page the rest of the body is never read. (`example.com` sends its whole
page as one line, so `split` takes the title out of that line.) `page.chunks` gives the body in the pieces it arrives in,
rather than by line. A body can be walked once.

## 19.5 Many requests at once

A request waits for its answer. **`await req.fetch` does the same as `send`,
but in async code**, where other tasks go on while it waits — so a few
lines fetch several pages at the same time:

```lume-skip
import http

async def main:
  urls = ["https://example.com/", "https://example.com/missing", "not a url"]
  statuses = await urls.map { |u| spawn: await http.request("GET", u).fetch }.to_list
  for u, r in urls.zip(statuses):
    match r:
      Ok(resp) -> puts "#{resp.status} #{u}"
      Error(_) -> puts "--- #{u}"
  # 200 https://example.com/
  # 404 https://example.com/missing
  # --- not a url
```

The results come back in the order the requests were started, as lunch 16
said of all tasks, so `zip` pairs each with its URL.

## 19.6 Checked without a network

`docs/tutorial/http-requests/` is a package that builds requests and makes
ones that cannot succeed — to a port where nothing listens — so it needs no
network, and the test suite runs it. Its `main.lume`:

```lume-skip
import http

def main:
  base = http.request("GET", "http://127.0.0.1:9/menu")
  careful = base.header("Accept", "text/plain").timeout(500)
  puts base.headers        # []
  puts careful.headers     # [("Accept", "text/plain")]
  puts careful.timeout_ms  # 500
  match careful.send:
    Ok(_) -> puts "answered?"
    Error(e) -> puts e.message.starts_with?("GET http://127.0.0.1:9/menu")    # true
```

Each step gave a new request and left `base` as it was. The failure's
message starts with the method and the URL, so you can tell which request
failed; the rest of it is the system's reason.

## Underneath, in Rust

The `http` package is written in Lume, with a few lines of Rust where it
calls the Rust crate **ureq**, which makes the requests. `send` blocks the
thread it runs on, as ureq does; `fetch` runs it in a task of its own (lunch
16), so other tasks go on. `open` hands the body to a thread that reads it
as it arrives, as lunch 18's programs' outputs are read. A Rust program
would more often use the `reqwest` crate, which is async throughout; the
result is much the same.

## Try it

1. Make a package that depends on `http`, fetch `https://example.com/`,
   and print its status and how many lines its body has.
2. Fetch a page that does not exist and print a message that says so, by
   looking at `ok?`.
3. Build a request with a timeout of 1 millisecond to `https://example.com/`
   and print whether it failed.
4. Fetch `https://example.com/` three times at once with `fetch` in tasks,
   and print the three statuses.

## Recap

- `http` is a package: depend on it in `lume.toml`, and `import http`.
- `http.get(url)` gives a `Response` — `status`, `ok?`, `header`, `body` —
  or an `Error` when the request could not be made at all. A `404` is a
  `Response`.
- `http.request(method, url)` builds a request with `.header`,
  `.with_body` and `.timeout(ms)`; `send` makes it.
- `open` gives the body as it arrives, as `lines` or `chunks`.
- `await req.fetch` makes a request in async code, so many can run at once.

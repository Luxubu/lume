# Lunch 21: A log analyser

**In this lunch you will build `loglens`**, which reads a web server's
log — one line per request — and reports what happened: how many
requests, how many failed, which pages were slowest. Logs grow large, so
this lunch is also about reading a big file quickly.

Each line of the log says when, what, which page, the status (lunch 19),
and how many milliseconds the answer took:

```text
12:00:01 GET /menu 200 35
12:00:01 POST /order 201 120
12:00:02 GET /menu 200 41
12:00:03 GET /order/9 500 900
```

## 21.1 One line, one entry

A line becomes a struct, and reading one can fail — a line cut short, a
status that is not a number — so `parse_line` gives an `Entry or Error`.
A list pattern takes the five fields apart in one step:

```lume-skip
struct Entry:
  time: Str
  method: Str
  path: Str
  status: Int
  ms: Int

def parse_line(line: Str) -> Entry or Error:
  match line.split(" "):
    [time, method, path, status, ms] ->
      Entry(time: time, method: method, path: path, status: status.to_int?, ms: ms.to_int?)
    _ -> Error("expected 5 fields")
```

`status.to_int?` passes a bad number on as the line's failure. A pattern
`[time, method, path, status, ms]` fits only a list of exactly five, so a
line with four fields, or six, falls to `_`.

## 21.2 Entries as they are read

A log may hold millions of lines. Rather than building a list of every
entry, **a generator (lunch 14) hands them out one at a time**, along with
the line number of each bad line, so a report can say where it found
trouble. Its items are `Entry or Error`, so a bad line is an item too, not
the end of the walk:

```lume-skip
def entries(text: Str) -> Iterator[Entry or Error]:
  for i, line in text.lines.enumerate:
    next if line.trim.empty?
    match parse_line(line):
      Ok(e) -> yield e
      Error(err) -> yield Error("line #{i + 1}: #{err.message}")
```

A generator item that is a `T or Error` is written as a function giving one
would be: `yield e` is a good item, `yield Error(...)` a failed one.

## 21.3 The report

The report walks the entries once, keeping running totals — nothing holds
every entry. (The log's text itself is read whole, by `File.read`; for a
log too big for memory, it would be read in pieces, which is beyond this
lunch.)

```lume-skip
struct Report:
  requests: Int
  failed: Int
  bad_lines: [Str]
  by_status: {Int: Int}
  total_ms: {Str: Int}
  hits: {Str: Int}
  slowest: Entry?

def analyse(text: Str) -> Report:
  var r = Report(requests: 0, failed: 0, bad_lines: [], by_status: {}, total_ms: {}, hits: {}, slowest: None)
  for item in entries(text):
    match item:
      Error(e) -> r.bad_lines.push(e.message)
      Ok(e) ->
        r.requests += 1
        r.failed += 1 if e.status >= 500
        r.by_status[e.status] = r.by_status[e.status].or(0) + 1
        r.total_ms[e.path] = r.total_ms[e.path].or(0) + e.ms
        r.hits[e.path] = r.hits[e.path].or(0) + 1
        if e.ms > r.slowest.map { |s| s.ms }.or(0 - 1):
          r.slowest = Some(e)
  r
```

Four running totals are maps keyed by status or by page (lunch 6's
counting pattern). `slowest` is an `Entry?`: before the first entry there
is none.

## 21.4 Showing it

Averages and the slowest pages come from the totals. **New here:**
`text.pad_right(n)` adds spaces to the end of text until it is `n` long,
which lines up a column, as a person likes to read a table; `map` on a
`T?` changes the value inside, if there is one; and a map's `to_list`
gives its pairs, `(key, value)`, which `sort` orders by key:

```lume-skip
def show(r: Report) -> Str:
  var out: [Str] = []
  out.push("#{r.requests} requests, #{r.failed} failed")
  out.push("status: " + r.by_status.to_list.sort.map { |s, n| "#{s}x#{n}" }.join(" "))
  averages = r.hits.to_list.map { |p, n| (p, r.total_ms[p].or(0) / n) }
  for p, avg in averages.sort_by { |p, avg| 0 - avg }.take(3):
    out.push("  #{p.pad_right(10)} #{avg} ms on average")
  match r.slowest:
    Some(e) -> out.push("slowest: #{e.method} #{e.path} at #{e.time}, #{e.ms} ms")
    None -> out.push("no requests")
  for b in r.bad_lines.take(3):
    out.push("skipped #{b}")
  out.join("\n")
```

`sort_by { |p, avg| 0 - avg }` sorts slowest first: sorting by the
negative of a number is sorting by the number backwards.

## 21.5 The whole program, tried out

Everything so far, with a `main` that analyses a small log written into
the program:

```lume
struct Entry:
  time: Str
  method: Str
  path: Str
  status: Int
  ms: Int

def parse_line(line: Str) -> Entry or Error:
  match line.split(" "):
    [time, method, path, status, ms] ->
      Entry(time: time, method: method, path: path, status: status.to_int?, ms: ms.to_int?)
    _ -> Error("expected 5 fields")

def entries(text: Str) -> Iterator[Entry or Error]:
  for i, line in text.lines.enumerate:
    next if line.trim.empty?
    match parse_line(line):
      Ok(e) -> yield e
      Error(err) -> yield Error("line #{i + 1}: #{err.message}")

struct Report:
  requests: Int
  failed: Int
  bad_lines: [Str]
  by_status: {Int: Int}
  total_ms: {Str: Int}
  hits: {Str: Int}
  slowest: Entry?

def analyse(text: Str) -> Report:
  var r = Report(requests: 0, failed: 0, bad_lines: [], by_status: {}, total_ms: {}, hits: {}, slowest: None)
  for item in entries(text):
    match item:
      Error(e) -> r.bad_lines.push(e.message)
      Ok(e) ->
        r.requests += 1
        r.failed += 1 if e.status >= 500
        r.by_status[e.status] = r.by_status[e.status].or(0) + 1
        r.total_ms[e.path] = r.total_ms[e.path].or(0) + e.ms
        r.hits[e.path] = r.hits[e.path].or(0) + 1
        if e.ms > r.slowest.map { |s| s.ms }.or(0 - 1):
          r.slowest = Some(e)
  r

def show(r: Report) -> Str:
  var out: [Str] = []
  out.push("#{r.requests} requests, #{r.failed} failed")
  out.push("status: " + r.by_status.to_list.sort.map { |s, n| "#{s}x#{n}" }.join(" "))
  averages = r.hits.to_list.map { |p, n| (p, r.total_ms[p].or(0) / n) }
  for p, avg in averages.sort_by { |p, avg| 0 - avg }.take(3):
    out.push("  #{p.pad_right(10)} #{avg} ms on average")
  match r.slowest:
    Some(e) -> out.push("slowest: #{e.method} #{e.path} at #{e.time}, #{e.ms} ms")
    None -> out.push("no requests")
  for b in r.bad_lines.take(3):
    out.push("skipped #{b}")
  out.join("\n")

LOG = """
  12:00:01 GET /menu 200 35
  12:00:01 POST /order 201 120
  12:00:02 GET /menu 200 41
  12:00:03 GET /order/9 500 900
  12:00:03 GET /menu
  12:00:04 GET /tables 200 12
  12:00:05 POST /order 201 140
  12:00:05 GET /tables 200 lots
  """

def main:
  puts show(analyse(LOG))
  #=> 6 requests, 1 failed
  #=> status: 200x3 201x2 500x1
  #=>   /order/9   900 ms on average
  #=>   /order     130 ms on average
  #=>   /menu      38 ms on average
  #=> slowest: GET /order/9 at 12:00:03, 900 ms
  #=> skipped line 5: expected 5 fields
  #=> skipped line 8: `lots` is not an integer
```

Two lines were bad, and each is reported with its line number and why; the
other six were counted.

## 21.6 Reading a real log, fast

The tool's `main` reads the file named in its arguments. To try it on a
large log, it can also make one — a million made-up lines, so the
measurements below can be repeated anywhere:

```lume-skip
PATHS = ["/menu", "/order", "/tables", "/order/9"]

def two(n: Int) -> Str = if n < 10: "0#{n}" else: n.to_s

# A log of `n` made-up requests, the same every time.
def make_log(n: Int) -> Str:
  var out: [Str] = []
  for i in 0...n:
    path = PATHS.at(i % 4)
    status = if i % 97 == 0: 500 else: 200
    out.push("12:#{two((i / 60) % 60)}:#{two(i % 60)} GET #{path} #{status} #{(i * 37) % 300}")
  out.join("\n") + "\n"

def main -> () or Error:
  args = Env.args
  match args:
    ["make", n, file] -> File.write(file, make_log(n.to_int?))?
    [file] -> puts show(analyse(File.read(file)?))
    _ ->
      warn "usage: loglens FILE | loglens make COUNT FILE"
      Env.exit(2)
```

```sh
lume build loglens.lume -o loglens
./loglens make 1000000 big.log
time ./loglens big.log
```

```text
1000000 requests, 10310 failed
status: 200x989690 500x10310
  /order/9   151 ms on average
  /tables    149 ms on average
  /menu      148 ms on average
slowest: GET /order/9 at 12:03:47, 299 ms
```

On the machine these lunches were written on, the 28 MB log takes about
half a second to analyse. Three things make it fast:

- **`lume build` optimises fully**; `lume run` builds for a quick start,
  and is slower on work this size — here, about 1.4 seconds.
- **Only what is kept is copied.** The pieces of `line.split(" ")` are
  views into the line until something keeps them: the three text fields an
  `Entry` holds are copied, but `status` and `ms` are read as numbers
  straight from the line.
- **The log is walked once**, by a generator, into running totals; nothing
  holds every entry.

## Underneath, in Rust

The generator compiles to a Rust iterator, so `for item in entries(text)`
is one loop over the text with no list in between. The maps are hash maps
that keep their keys in order. Lume's own benchmark of this kind of work,
`bench/b8_logq.lume`, runs at about twice the time of the same program
written in Rust by hand — the price of the text copies Lume still makes.

## Try it

1. Add `slow COUNT`: the `COUNT` slowest requests, slowest first. (Collect
   the good entries in a list with a `for`, then `sort_by` and `take`.)
2. Count failures per minute: the first five characters of `time`, such as
   `12:00`, are the minute. Which minute had the most?
3. Make bad lines a failure when there are more than ten of them, with a
   message saying how many.
4. Add `--status 500` to report only the requests with that status.

## Recap

- One line becomes one struct; parsing can fail, so it gives `T or Error`,
  and the failure says which line.
- A generator of `Entry or Error` hands out entries as they are read, and
  bad lines as failed items, without stopping the walk.
- Running totals in maps make the report in one pass.
- For speed: `lume build`, read fields by position, walk once.

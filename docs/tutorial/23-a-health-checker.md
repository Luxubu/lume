# Lunch 23: A health checker

> **A preview.** This lunch is built on the `http` package (lunch 19), one of
> Lume's newest parts. It works as shown — every example is checked — but details may
> change before 1.0. See [what is settled](../stability.md).

**In this lunch you will build `pulse`**, which checks a list of web pages
at once — is each one up, does it say what it should, is it quick enough —
trying again before it calls a page down. Every team that runs a website
has a tool like this somewhere.

It brings together the `http` package (lunch 19), tasks (lunch 16) and
failures that are results (lunch 22) — and one new technique: **keeping
the part that talks to the world in a block, so the rest can be tried out
without it.**

## 23.1 What to check

A check is a page and what is expected of it:

```lume-skip
struct Check:
  name: Str
  url: Str
  # text the page must contain, if any
  contains: Str?
  # the slowest answer still counted as up
  within_ms: Int
```

And its outcome — up or not, why not, and how many tries it took:

```lume-skip
struct Outcome:
  name: Str
  up: Bool
  why: Str
  tries: Int
```

## 23.2 Fetching, kept apart

Checking needs to fetch a page. If `run_check` called `http.get` itself, the
only way to try it would be with real pages — slow, and never the broken
page you want to test. Instead, **the fetching is a block, kept in a field**
of a `Checker`:

```lume-skip
struct Checker:
  # fetches a page: its status and its body, or why it could not
  fetch: (Str) -> (Int, Str) or Error
  tries: Int
```

The real `pulse` gives it a block that uses `http`; a test gives it one
that answers from a list written in the program. The checking code cannot
tell the difference — which is exactly what makes it testable. (A block
type, from lunch 7, may be a field's type; `(Str) -> (Int, Str) or Error`
reads "given a `Str`, gives a status and a body, or an `Error`".)

## 23.3 One check, with retries

A page that fails once may be fine a moment later — a server restarting,
a dropped connection. So a check tries up to `tries` times, pausing a
little longer each time, before it calls a page down:

```lume-skip
def judge(c: Check, status: Int, body: Str, ms: Int) -> Str?:
  if status != 200:
    return Some("status #{status}")
  match c.contains:
    Some(text) if not body.contains?(text) -> return Some("no \"#{text}\" on the page")
    _ -> ()
  if ms > c.within_ms:
    return Some("slow: over #{c.within_ms} ms")
  None

def run_check(ch: Checker, c: Check) -> Outcome:
  var why = ""
  for attempt in 1..ch.tries:
    start = Time.now_ms
    problem = match ch.fetch(c.url):
      Ok(answer) -> judge(c, answer.0, answer.1, Time.now_ms - start)
      Error(e) -> Some(e.message)
    match problem:
      None -> return Outcome(name: c.name, up: true, why: "", tries: attempt)
      Some(p) -> why = p
    Time.sleep(attempt * 20) if attempt < ch.tries
  Outcome(name: c.name, up: false, why: why, tries: ch.tries)
```

`judge` gives `None` when all is well and `Some(reason)` when not — an
answer that may be absent, lunch 10's `T?`. `run_check` returns as soon as
a try succeeds, and remembers the last reason otherwise.

`Time.sleep(ms)` outside async code simply pauses the task it is in; the
pause grows with each try, so a struggling server gets room to recover.

## 23.4 All at once

Every check is a task of its own — `run_check` waits for pages and awaits
nothing, so each runs on the pool for blocking work (lunch 16) — and a
slow page holds up no other:

```lume-skip
async def check_all(ch: Checker, checks: [Check]) -> [Outcome] =
  await checks.map { |c| spawn: run_check(ch, c) }.to_list
```

The results come back in the checks' order, so the report reads like the
list.

## 23.5 The whole program, with pretend pages

Here is everything, with a fetching block that answers from a few pretend
pages — one up, one down, one with the wrong text, one slow, one whose
server cannot be reached:

```lume
struct Check:
  name: Str
  url: Str
  contains: Str?
  within_ms: Int

struct Outcome:
  name: Str
  up: Bool
  why: Str
  tries: Int

struct Checker:
  fetch: (Str) -> (Int, Str) or Error
  tries: Int

def judge(c: Check, status: Int, body: Str, ms: Int) -> Str?:
  if status != 200:
    return Some("status #{status}")
  match c.contains:
    Some(text) if not body.contains?(text) -> return Some("no \"#{text}\" on the page")
    _ -> ()
  if ms > c.within_ms:
    return Some("slow: over #{c.within_ms} ms")
  None

def run_check(ch: Checker, c: Check) -> Outcome:
  var why = ""
  for attempt in 1..ch.tries:
    start = Time.now_ms
    problem = match ch.fetch(c.url):
      Ok(answer) -> judge(c, answer.0, answer.1, Time.now_ms - start)
      Error(e) -> Some(e.message)
    match problem:
      None -> return Outcome(name: c.name, up: true, why: "", tries: attempt)
      Some(p) -> why = p
    Time.sleep(attempt * 20) if attempt < ch.tries
  Outcome(name: c.name, up: false, why: why, tries: ch.tries)

async def check_all(ch: Checker, checks: [Check]) -> [Outcome] =
  await checks.map { |c| spawn: run_check(ch, c) }.to_list

def line_for(o: Outcome) -> Str:
  retried = if o.tries > 1: " (#{o.tries} tries)" else: ""
  if o.up: "up    #{o.name}#{retried}" else: "DOWN  #{o.name}: #{o.why}#{retried}"

# Pretend pages: what each URL answers, with no network.
def pretend(url: Str) -> (Int, Str) or Error:
  match url:
    "https://cafe.test/" -> Ok((200, "Welcome to the cafe"))
    "https://cafe.test/menu" -> Ok((500, "oops"))
    "https://cafe.test/about" -> Ok((200, "We sell soup"))
    "https://cafe.test/slow" ->
      Time.sleep(300)
      Ok((200, "eventually"))
    _ -> Error("could not connect to #{url}")

async def main:
  ch = Checker(fetch: pretend, tries: 2)
  checks = [
    Check(name: "home", url: "https://cafe.test/", contains: Some("Welcome"), within_ms: 1000),
    Check(name: "menu", url: "https://cafe.test/menu", contains: None, within_ms: 1000),
    Check(name: "about", url: "https://cafe.test/about", contains: Some("pie"), within_ms: 1000),
    Check(name: "slow", url: "https://cafe.test/slow", contains: None, within_ms: 100),
    Check(name: "shop", url: "https://shop.test/", contains: None, within_ms: 1000),
  ]
  outcomes = await check_all(ch, checks)
  for o in outcomes:
    puts line_for(o)
  down = outcomes.count { |o| not o.up }
  puts "#{outcomes.len - down} up, #{down} down"
  #=> up    home
  #=> DOWN  menu: status 500 (2 tries)
  #=> DOWN  about: no "pie" on the page (2 tries)
  #=> DOWN  slow: slow: over 100 ms (2 tries)
  #=> DOWN  shop: could not connect to https://shop.test/ (2 tries)
  #=> 1 up, 4 down
```

`Checker(fetch: pretend, ...)` hands over the function `pretend` as the
block, by its name (lunch 7). The slow page sleeps 300 ms against a limit
of 100, so it fails however busy the machine is. The run takes about as
long as the slowest page's two tries, not as long as all the checks one
after another.

## 23.6 Real pages

The real `pulse` gives the `Checker` a block that fetches with `http`, as
a package (lunch 19) that depends on it. Only this part is new:

```lume-skip
import http

# A page for real: its status and body, or why it could not be fetched.
def fetch_page(url: Str) -> (Int, Str) or Error:
  r = http.request("GET", url).timeout(5000).send?
  Ok((r.status, r.body))

async def main:
  ch = Checker(fetch: fetch_page, tries: 3)
  checks = [
    Check(name: "example", url: "https://example.com/", contains: Some("Example Domain"), within_ms: 2000),
    Check(name: "missing", url: "https://example.com/no-such-page", contains: None, within_ms: 2000),
  ]
  outcomes = await check_all(ch, checks)
  for o in outcomes:
    puts line_for(o)
  Env.exit(1) if outcomes.any? { |o| not o.up }
```

```text
up    example
DOWN  missing: status 404 (3 tries)
```

It exits with status 1 when a page is down, so a scheduled job or a build
can raise the alarm. A real `pulse` would read its checks from a file, as
lunch 22's runner reads its jobs — the
[`examples/healthcheck/`](../../examples/healthcheck/main.lume) program in
Lume's repository does, from JSON.

## Underneath, in Rust

A block kept in a field is an `Arc<dyn Fn>` in Rust: a pointer to a
function value that any number of tasks can share. Calling it costs one
call through that pointer — nothing next to a web request. Each check runs
as a `spawn_blocking` task, and `Time.sleep` in it is
`std::thread::sleep`, which pauses only that task's thread.

## Try it

1. Add a pretend page that answers `200` with the text `"Closed today"`,
   and a check that expects `"Open"` on it.
2. Make a check pass when the status is any of `200`, `201` or `204`. (Add
   a field `ok_statuses: [Int]`.)
3. Make the pause between tries double each time: 20, 40, 80 ms.
4. Count how many times `pretend` was called for each URL, with a `shared
   var` map, and print it after the checks. Which pages were fetched twice?
   (A function can take a `shared var` as a parameter, written
   `calls: shared var {Str: Int}`, and change it.)

## Recap

- Keep the part that talks to the world — here, fetching — behind a block
  in a field, so the rest can be tried out and tested with a pretend one.
- A check is a struct, its result is a struct, and the judgement in between
  gives a reason when something is wrong (`Str?`).
- Retry with a growing pause before calling something down.
- One task per check, so a slow page holds up nothing, and results in the
  checks' order.

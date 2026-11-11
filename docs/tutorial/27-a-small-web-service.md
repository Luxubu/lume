# Lunch 27: A small web service

**In this lunch you will build a café's orders service**: a program that
answers HTTP requests — take an order, list the orders, serve one — and
keeps the orders that many requests share. Lunch 21 made requests; this one
answers them, with the same `http` package.

It brings together `match` on a route (lunch 9), a struct with `var self`
methods (lunch 8), tasks and `shared var` (lunch 18), and forms and JSON
(lunch 21). The whole program is in `docs/tutorial/orders-service/` in
Lume's repository, and the suite runs it.

## 27.1 Listening

**`http.listen(address)` gives a `Server`**, and `server.requests` gives
each request that comes to it, for a `for`. Each is an `Incoming`, with a
`method`, a `path` and a body; `respond` answers it, once:

```lume-skip
import http

def main -> () or Error:
  server = http.listen("127.0.0.1:8080")?
  puts "listening on http://#{server.address}"
  for req in server.requests:
    req.respond(http.reply(200, "hello from #{req.path}"))?
  ()
```

Run it, and open `http://127.0.0.1:8080/menu` in a browser: it says
`hello from /menu`. It answers until you stop it. Port `0` asks the system
for any free port, and `server.port` says which it chose, which is how a
program that tries itself out, below, never picks one already taken.

## 27.2 Routes are a `match`

What a request asks for is its method and its path. **A `match` on both
turns them into a route**, an enum (lunch 9), so the rest of the program
works with names rather than strings:

```lume-skip
enum Route:
  List
  Take
  Serve(id: Int)
  Nowhere

def route(method: Str, path: Str) -> Route:
  match (method, path.split("/").reject(_.empty?).to_list):
    ("GET", ["orders"])        -> List
    ("POST", ["orders"])       -> Take
    ("DELETE", ["orders", id]) -> id.to_int.map { |n| Serve(n) }.or(Nowhere)
    _                          -> Nowhere
```

`/orders/` and `/orders` are the same route, as the empty pieces are
dropped; `/orders/seven` is no route at all, as `seven` is not a number. A
route is a plain function of two strings, so it is tested with no network
at all — 27.6.

## 27.3 What many requests share

The orders are one list that every request reads or changes. **A struct
with `var self` methods holds it**, and `shared var` shares it between the
tasks that answer requests (lunch 18):

```lume-skip
struct Kitchen:
  orders: [Order]
  next_id: Int

  def add(var self, dish: Str, table: Int) -> Order:
    o = Order(id: next_id, dish: dish, table: table)
    orders.push(o)
    next_id += 1
    o
```

Each call of a method of a `shared var` runs under its lock, start to
finish, so two orders taken at once never get the same number: one `add`
waits for the other. Lume warns that `kitchen` is a `shared var`, as it
does for each one; here, sharing it is the point.

**`server.incoming` is `requests` for async code**: while it waits for the
next request, other tasks go on. Give each request a task of its own, and a
slow one holds up no other:

```lume-skip
async def listen(server: http.Server) -> () or Error:
  shared var kitchen = Kitchen(orders: [], next_id: 1)
  var answering: [Task[() or Error]] = []
  for req in server.incoming:
    answering.push(spawn: req.respond(answer(kitchen, req)))
  for t in answering:
    await t?
  ()
```

The loop ends when the server is closed; then each answer is awaited, and
a failure to send one is passed on.

## 27.4 Answers: text, JSON and forms

`answer` turns a request into a `Reply`. **`http.reply(status, text)` is an
answer of text, `http.reply_json(status, doc)` one of JSON** (lunch 21's
`json` package). An order arrives as a form, as a browser sends one;
`req.form` decodes it into names and values, and `http.value` finds one:

```lume-skip
def answer(kitchen: shared var Kitchen, req: http.Incoming) -> http.Reply:
  match route(req.method, req.path):
    List -> http.reply_json(200, kitchen.orders.map { |o| o.to_json }.to_list.to_json)
    Take ->
      form = req.form.or([])
      dish = http.value(form, "dish").or("")
      table = http.value(form, "table").or("").to_int.or(0)
      if dish.empty? or table < 1:
        http.reply(400, "an order needs a dish and a table")
      else:
        o = kitchen.add(dish, table)
        http.reply_json(201, o.to_json)
    Serve(id) -> if kitchen.serve(id): http.reply(200, "served #{id}") else: http.reply(404, "no order #{id}")
    Nowhere   -> http.reply(404, "no #{req.method} #{req.path} here")
```

The statuses say what happened, as HTTP has them: `201` made something,
`400` the request was wrong, `404` there is no such thing. `req.json`
reads a body sent as JSON, and `req.params` the query after a `?`, the same
way `req.form` reads a form.

## 27.5 Trying it, from inside

A program can be its own client: **listen on port 0, and make the requests
from a task** with lunch 21's `http.request`. The client closes the server
when it is done, which ends `listen`'s loop:

```lume-skip
async def main -> () or Error:
  server = http.listen("127.0.0.1:0")?
  client = spawn: try_it("http://#{server.address}", server)
  await listen(server)?
  for line in await client:
    puts line
  ()
```

`try_it` takes three orders, serves one twice, lists them, asks for a page
there is none of, and sends an order with no table:

```text
201 {"id":1,"dish":"soup","table":4}
201 {"id":2,"dish":"pie","table":4}
201 {"id":3,"dish":"tea","table":9}
DELETE /orders/2: 200 served 2
DELETE /orders/2: 404 no order 2
GET /orders: 200 [{"id":1,"dish":"soup","table":4},{"id":3,"dish":"tea","table":9}]
GET /menu: 404 no GET /menu here
400 an order needs a dish and a table
```

The client makes one request at a time, so what it prints never depends on
which task ran first, and never shows the port.

## 27.6 Testing without a network

The parts that decide things are plain functions and methods, so **tests
try them with no server at all** (lunch 14):

```lume-skip
test "routes":
  assert route("GET", "/orders") == List
  assert route("POST", "/orders/") == Take
  assert route("DELETE", "/orders/7") == Serve(7)
  assert route("DELETE", "/orders/seven") == Nowhere

test "a kitchen numbers its orders":
  var k = Kitchen(orders: [], next_id: 1)
  assert k.add("soup", 4).id == 1
  assert k.add("pie", 4).id == 2
  assert k.serve(1)
  assert not k.serve(1)
```

Keep `answer` thin and the decisions in functions like these, and most of a
service is tested in a second. `examples/notes/` in Lume's repository is the
same shape grown up: notes kept in a file, every method a service needs,
and its own tests and client.

## Underneath, in Rust

`http.listen` is a `tiny_http::Server`; `incoming` waits for each request
in a task of its own, so the runtime's other tasks go on. A `shared var
Kitchen` is an `Arc<Mutex<Kitchen>>`, and each call of a `var self` method
locks it once, for the whole call. A `Reply` becomes a `tiny_http::Response`
when `respond` sends it.

## Try it

1. Add a route `GET /orders/<id>` that answers with one order as JSON, or
   `404`.
2. Make `GET /orders?table=4` list only that table's orders, with
   `req.params` and `http.value`.
3. Write a test for 1, on `route` alone.

## Recap

- `http.listen(address)` gives a `Server`; `requests` — or `incoming` in
  async code — gives each request; `req.respond(reply)` answers it once.
- A `match` on `(method, path pieces)` turns a request into a route.
- What every request shares is a `shared var`, best a struct whose `var
  self` methods each run under its lock.
- `http.reply`, `http.reply_json` and `.header` make answers; `req.form`,
  `req.json` and `req.params` read what was sent.
- A program can be its own client on port 0; routes and methods are tested
  without a network.

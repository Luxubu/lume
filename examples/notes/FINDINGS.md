# What building `notes` found

Milestone 129 wrote this service to try the server preview of 1.1 on a
whole program before it is settled: a store kept in a JSON file, a request
per task working on one `shared var`, JSON and form bodies, a query, every
method a REST service needs, and its own client.

## In the server: nothing in the way

`http.listen`, `incoming` in async code, a task per request with
`spawn: req.respond(...)`, and `server.close` called by the client's task
to end the loop all did what `http.md` says, the first time. A `Reply` with
`.header("Allow", ...)` and a `204` with no body were each one line. No
part of the preview needed to change shape.

## Added

- **`http.value(pairs, name)`**, the value of a name in a query or a form.
  This program needed it three times and `examples/http_server/` twice, and
  in review round twenty-five five programs wrote it for themselves.

## Fixed on the way

- **A method of `extend [T: ToJson]` was accepted on a list whose items are
  not `ToJson`**, and the program reached rustc: `[Json].to_json`, here.
  A `Json` document is `ToJson` itself now, so a list of them is JSON too.
  And where the items are built-in values and the bound is an interface
  this file knows, the bound is checked, with an error that says which is
  not met: `[1, 2].loud`. For a type of a module's own, extended perhaps
  from another, the check is left to Rust still — a gap to close.
- **A `() or Error` call at the end of a `match` arm that ends a `() or
  Error` function was refused** as a failure nothing looks at, though it is
  the function's value, as it is on the function's own last line.

## Left for a decision

- **Every `shared var` warns**, and a service whose store is one warns on
  every build, though sharing it is the point. `concurrency.md` says the
  warning is meant; whether a `shared var` of a type with `var self`
  methods — one that is built to be shared — should go without it is a
  question for the language, not for this program.
- **Answering with a file**, and **reading a body as it arrives**, were not
  needed here; `http.md` lists them as not in it yet.

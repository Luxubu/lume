# healthcheck — what writing it found

`healthcheck` checks a list of services at once and says which are down. It
reads the checks from JSON with the `json` package, makes each request with
the `http` package in a task of its own, retries a failure once, and reports
as a table and, with `--json`, as JSON. It was written to use milestone 65's
`http` package the way a real program would.

    cd examples/healthcheck
    lume run -- checks.json --json

`check.sh` runs it against a server on this machine, with a fast page, four
slow ones, one that fails, one that fails once and then works, and one with
the wrong body.

## What went well

The program compiled and ran correctly on the first try, which no dogfood
program had before. These carried it with nothing in the way:

- decoding the checks with `need_…` and `opt_…`, with errors that name the
  check and the field;
- `?` and `map_error`, and `Env.exit` ending a `match` arm;
- tasks awaited in a list, and a list of results written with `to_json`.

## What was wrong, and is fixed

1. **A request held a thread of the task runtime.** The requests in sixteen
   tasks ran twelve at a time on this twelve-core machine, then four: 0.62 s
   for what should take 0.3 s. A request waits without `await` (the `http`
   package is built on a blocking client), and so does `Process.run`, or any
   computation. Now **a `spawn:` body that awaits nothing runs on Tokio's
   pool for blocking work**, as Rust's `spawn_blocking` does. Sixteen checks
   take 0.42 s, and forty take 0.69 s. The rule is in
   [Concurrency](../../docs/reference/concurrency.md#a-task-that-waits-without-await),
   and `examples/tasks_blocking.lume` keeps it: thirty 100 ms tasks finish
   together.

## Left as it is

- **The `http` package has no async call**: a request waits in full. A task
  with no `await`, as above, is how to wait for many at once, and that is
  what Rust programs using a blocking client do too.

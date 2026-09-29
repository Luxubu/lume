# jobr — what writing it found

`jobr` runs the jobs in a jobs file, as many at once as their order allows.
It is two packages: `jobfile` parses the file and puts the jobs in waves,
and `jobr` runs each wave's jobs in tasks, retries failures with a growing
wait, skips jobs that wait for a failed one, counts tries in a `shared var`,
and reports. It was written to use the concurrency features in a real
program, with the language server's checking as it went.

    cd examples/jobr/jobr
    lume run -- jobs.txt --dry
    lume run -- jobs.txt

## Before any code: Lume could not run a program

A job runner starts programs, and Lume had no way to. **`Process.run(program,
args)`** now gives `(Int, Str, Str) or Error`: the exit code, then standard
output and standard error. Only failing to start is an `Error`; a non-zero
exit has still run. This follows Rust's `std::process::Command`. Inside a task
it lets the other tasks go on while it waits, so the two 0.3 s fetches in
`jobs.txt` take 0.3 s together.

## What was wrong, and is fixed

1. **A value in an `assert` could be moved** (milestone 51). An `assert`
   writes its expression twice, once to test it and once to show the values
   when it fails. The move rules counted one use, so
   `assert waves.map { .. }.to_list == [..]` gave away `waves` in the first
   copy and read it in the second: a rustc error. A `while` condition, which
   runs every time round, had the same hole. Neither moves anything now.
   `examples/tests/asserts_move.lume` keeps it fixed.

## What was in the way, and now is not

2. **A branch could not end in `Env.exit`.** A `match` arm that reports an
   error and exits still had to give a value of the other arms' type, so
   every such arm ended in a dummy `""` or `[]`. `Env.exit` never comes back,
   as Rust's `exit` gives `!`, so such a branch now has no value to agree on.
3. **No way to change a failure's message.** `.or_error` is for optional
   values. For a result, the docs showed a four-line `match`, and
   `.ok.or_error(..)` loses the original message. **`map_error`** is Rust's
   `map_err`: `v.to_int.map_error { |e| Error("retries: #{e.message}") }`.
4. **A warning on every run.** The `shared var` warning was printed each time
   the program ran, although nothing had changed. Warnings are now shown when
   the program is compiled, as cargo shows them.
5. **`shared var n: Int` as a parameter** said only that `shared` is a
   keyword. The message now says `shared` goes in the type:
   `n: shared var Int`.

## What was in the way, and stays

- **`Time.sleep` inside an `async def` needs `await`.** The error says so,
  and it is right: the sleep is a future there.
- **`slice(a, b)` is an offset and a length**, not a range. I wrote it as a
  range from Rust habit; the docs give this its own heading.
- **`jobs.pop!` warns**, and was worth avoiding: `jobs[i].run = value`
  changes the last job in place.

## What went well

Tasks, `await` on a list of them, a `shared var` map changed by tasks and a
parameter typed `shared var`, retries with a growing sleep, the orphan-free
two-package layout, and `lume test` in the library. The whole runner is 90
lines, and each fix above was a change to the compiler or the docs, not to
the program's design.

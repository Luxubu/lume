# Lunch 22: A job runner

**In this lunch you will build `jobrun`**, which runs a list of shell
commands — jobs — several at a time, and reports which worked. Build
scripts, nightly tasks and test suites all have this shape. Lume's own test
suite is one, written in Lume.

The jobs are kept in a file, one to a line, a name and a command:

```text
# jobs.txt
greet: echo hello
sort: printf 'pear\napple\n' | sort | head -1
broken: echo starting; echo out of tea >&2; exit 3
nap: sleep 0.2; echo rested
```

## 22.1 Reading the jobs

A job is a name and a command. The line is cut at its first `: ` — the
command may well contain more colons:

```lume-skip
struct Job:
  name: Str
  command: Str

def parse_jobs(text: Str) -> [Job] or Error:
  var jobs: [Job] = []
  for i, line in text.lines.enumerate:
    next if line.trim.empty? or line.starts_with?("#")
    cut = line.index_of(": ").or_error("line #{i + 1}: expected `name: command`")?
    jobs.push(Job(name: line[0...cut], command: line[cut + 2..line.len]))
  jobs
```

`line[0...cut]` is the text before the colon, three dots stopping just
before it; `line[cut + 2..line.len]` is everything after `": "` (lunch 3's
ranges). Lines that start with `#` are comments, so a jobs file can explain
itself.

## 22.2 Running one job

A job runs with `sh -c` (lunch 15), so pipes and `;` work in a command. The
outcome keeps what a report needs: the exit code, the first line of each
output, and how long it took:

```lume-skip
struct Outcome:
  name: Str
  code: Int
  out: Str
  err: Str
  ms: Int

  def ok? -> Bool = code == 0

def run_job(j: Job) -> Outcome:
  start = Time.now_ms
  (code, out, err) = Process.run("sh", ["-c", j.command]).or((127, "", "could not start sh"))
  Outcome(name: j.name, code: code, out: out.trim, err: err.trim, ms: Time.now_ms - start)
```

`run_job` gives an `Outcome`, never an `Error`: a job that fails is a
result to report, not a reason to stop the runner. If `sh` itself could not
be started, the outcome says so, with `127`, the code shells use for "not
found".

`Time.now_ms` is the time now, in milliseconds; the difference between two
is how long something took.

## 22.3 Running several at once

Running jobs one after another wastes time when each mostly waits. Running
all at once can swamp a machine. The usual answer is **a pool**: at most
`at_once` jobs running, and a new one started each time one finishes —
lunch 16's `next_done`:

```lume-skip
async def run_all(jobs: [Job], at_once: Int) -> [Outcome]:
  var running: [Task[(Int, Outcome)]] = []
  var done: [(Int, Outcome)] = []
  var started = 0
  while started < jobs.len or not running.empty?:
    while started < jobs.len and running.len < at_once:
      i = started
      job = jobs.at(i)
      running.push(spawn: (i, run_job(job)))
      started += 1
    match await running.next_done:
      Some(result) -> done.push(result)
      None -> ()
  done.sort_by { |i, o| i }.map { |i, o| o }.to_list
```

Three things to notice:

- **Each task gives back its job's position** with its outcome, `(i, ...)`,
  because jobs finish in any order. The last line puts the outcomes back in
  the jobs' order, so the report reads like the file.
- **`run_job` awaits nothing**, so each task runs on the pool for blocking
  work (lunch 16): a job waiting on its program holds up no other.
- **`i` and `job` are named before the `spawn:`**, so the task copies those
  values, not the changing `started`.

## 22.4 The report

```lume-skip
def line_for(o: Outcome) -> Str:
  if o.ok?:
    "ok    #{o.name.pad_right(8)} #{o.out.lines.last.or("")}"
  else:
    "FAIL  #{o.name.pad_right(8)} exit #{o.code}: #{o.err.lines.first.or("")}"
```

A passing job shows the last line it printed — usually its result; a
failing one shows its exit code and the first line of its error output —
usually the reason.

## 22.5 The whole program, tried out

Everything so far, with a `main` that runs four jobs written into the
program, two at a time:

```lume
struct Job:
  name: Str
  command: Str

def parse_jobs(text: Str) -> [Job] or Error:
  var jobs: [Job] = []
  for i, line in text.lines.enumerate:
    next if line.trim.empty? or line.starts_with?("#")
    cut = line.index_of(": ").or_error("line #{i + 1}: expected `name: command`")?
    jobs.push(Job(name: line[0...cut], command: line[cut + 2..line.len]))
  jobs

struct Outcome:
  name: Str
  code: Int
  out: Str
  err: Str
  ms: Int

  def ok? -> Bool = code == 0

def run_job(j: Job) -> Outcome:
  start = Time.now_ms
  (code, out, err) = Process.run("sh", ["-c", j.command]).or((127, "", "could not start sh"))
  Outcome(name: j.name, code: code, out: out.trim, err: err.trim, ms: Time.now_ms - start)

async def run_all(jobs: [Job], at_once: Int) -> [Outcome]:
  var running: [Task[(Int, Outcome)]] = []
  var done: [(Int, Outcome)] = []
  var started = 0
  while started < jobs.len or not running.empty?:
    while started < jobs.len and running.len < at_once:
      i = started
      job = jobs.at(i)
      running.push(spawn: (i, run_job(job)))
      started += 1
    match await running.next_done:
      Some(result) -> done.push(result)
      None -> ()
  done.sort_by { |i, o| i }.map { |i, o| o }.to_list

def line_for(o: Outcome) -> Str:
  if o.ok?:
    "ok    #{o.name.pad_right(8)} #{o.out.lines.last.or("")}"
  else:
    "FAIL  #{o.name.pad_right(8)} exit #{o.code}: #{o.err.lines.first.or("")}"

JOBS = r"""
  # four jobs, one of them broken
  greet: echo hello
  sort: printf 'pear\napple\n' | sort | head -1
  broken: echo starting; echo out of tea >&2; exit 3
  nap: sleep 0.2; echo rested
  """

async def main:
  match parse_jobs(JOBS):
    Error(e) -> puts e.message
    Ok(jobs) ->
      outcomes = await run_all(jobs, 2)
      for o in outcomes:
        puts line_for(o)
      passed = outcomes.count { |o| o.ok? }
      puts "#{passed} of #{outcomes.len} jobs passed"
  #=> ok    greet    hello
  #=> ok    sort     apple
  #=> FAIL  broken   exit 3: out of tea
  #=> ok    nap      rested
  #=> 3 of 4 jobs passed
```

`JOBS` is a **raw string**: `r"""` is lunch 3's triple-quoted text with no
escapes and no `#{ }`, so its `\n` reach `printf` as the two characters
written, for the shell to turn into new lines. (In the jobs file, which is
not Lume, they are just text anyway.)

## 22.6 The tool

The real `main` reads the jobs file named on the command line, takes `-j N`
for how many to run at once, shows each job's time, and — like any tool a
script might run — **exits with status 1 when any job failed**:

```lume-skip
async def main -> () or Error:
  var args = Env.args
  var at_once = 4
  if args.len >= 2 and args.at(0) == "-j":
    at_once = args.at(1).to_int.or(4).max(1)
    args = args.skip(2).to_list
  if args.len != 1:
    warn "usage: jobrun [-j N] JOBS_FILE"
    Env.exit(2)
  jobs = parse_jobs(File.read(args.at(0))?)?
  outcomes = await run_all(jobs, at_once)
  for o in outcomes:
    puts "#{line_for(o)}  (#{o.ms} ms)"
  failed = outcomes.reject { |o| o.ok? }.map(_.name).to_list
  if not failed.empty?:
    warn "failed: #{failed.join(", ")}"
    Env.exit(1)
```

```sh
lume build jobrun.lume -o jobrun
./jobrun -j 2 jobs.txt
```

```text
ok    greet    hello  (4 ms)
ok    sort     apple  (7 ms)
FAIL  broken   exit 3: out of tea  (4 ms)
ok    nap      rested  (217 ms)
failed: broken
```

The times will differ on your machine, and from run to run.

## Underneath, in Rust

The pool is a tokio `JoinSet` in all but name: a set of running tasks, and
`join_next` to take whichever finishes first. Each job is a
`spawn_blocking` task running a `std::process::Command`. The whole program
builds to one native file with no runtime to install — which is why tools
like this are so often written in Rust, and why Lume makes them easy to
write.

## Try it

1. Make a job fail by giving it a command that does not exist, and look at
   what the report says.
2. Add a `--fail-fast` flag: once a job fails, start no new ones. (The pool
   loop can stop starting jobs when a finished one failed.)
3. Show each failing job's whole error output, indented, under its line.
4. Run your jobs with `-j 1`, then `-j 4`, and compare the times of `nap`
   when there are two `nap` jobs.

## Recap

- A job runner is jobs read from a file, a pool that runs a few at a time,
  and a report in the jobs' order.
- A job's failure is an `Outcome`, not an `Error`: the runner reports it
  and goes on.
- Each task carries its job's position, so results can be put back in
  order however they finish.
- A tool other programs run says how it went with its exit status.

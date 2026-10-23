# Reviewer C — lunches 17 to 24

Who I was: a Python programmer, a little JavaScript, no Rust. I skimmed
lunches 1–16 (without their exercises) to know what had been taught, then
did 17–24 properly: every example typed into `/tmp/lume-dryrun-c/` and run
with the repository's `compiler/target/release/lume` (it reports
`lume 0.5.0`), every *Try it* done before I opened `answers.md`.

For lunches 19, 23 and 24 I used the **git form as the page writes it**
(`{ git = "https://github.com/Luxubu/lume", tag = "v0.5.0" }`), with
`LUME_HOME` pointed into my scratch folder so nothing went into `~/.lume`.
**It works**: the first `lume run` printed ``fetching `http` from
https://github.com/Luxubu/lume``, took about 16 s, wrote a `lume.lock`
(commit `7cab1b8`), and later runs were quick. `packages/http` and
`packages/json` have not changed between `v0.5.0` and `HEAD`, so the tag
gives the same packages the lunches were written against. (The compiler on
`PATH` is newer than the `v0.5.0` tag, though — `compiler/src/codegen.rs`
has changed since, in the "unawaited async calls refused" commit — so a
reader who installed the released 0.5.0 might see slightly different
messages around unawaited `async` calls. I could not check that.)

Every example in 17–24 that I typed printed what the page says, including
the hand-run network ones (example.com) and `lspeek`, which I ran five
times with the same output each time. Two exercise answers do not work or
were not doable from the lunches (24's completion, 23's `shared var`
parameter), and a few sentences are false or contradict earlier lunches.
Details below.

---

## Lunch 17: Async sequences

**Pace:** about 50 minutes, exercises included. A good size.

**Examples:** all six printed what the page shows. The `lume-bad` example
in 17.1 gives exactly the message shown, plus a useful help line
(``call it in an `async def` or a `spawn:` body``).

**What was wrong:**

- **Recap, last-but-one bullet:** "`first`, `count`, `sum`, `any?`, `all?`,
  `find`, `each` and `fold` wait for the values; **the first four stop at
  their answer.**" The first four in that list are `first`, `count`, `sum`
  and `any?`. `count` and `sum` cannot stop early; 17.4 itself says the
  four are "`find`, `any?`, `all?` and `first`". The recap needs to name
  them, not say "the first four".
- **17.2, "makes at most one value ahead of whoever walks it".** The
  two-dish example is too short to show what really happens. I gave the
  kitchen four dishes:

  ```text
    kitchen: soup ready
    kitchen: pie ready
    kitchen: tea ready
  waiter: served soup
    kitchen: stew ready
  ...
  ```

  While the waiter is still serving the soup, the kitchen has the pie on
  the pass *and* the tea made, waiting at its `yield`. That is two ahead
  of the value being used, not one. A reader who adds a dish, which is the
  natural experiment, will think they have misunderstood. Either say
  "one value waits on the pass, and the kitchen may have the next one made
  and be waiting to hand it over", or use an example that shows it.

**What confused me:**

- **When do I write `await`?** Lunch 16.2 says of an `async def` that
  "every call to it is awaited". Here `orders()` is an `async def`, and
  `orders().count { ... }`, `.find`, `.to_list` and `for` all go without
  `await`, but 17.5's `await count_all(k.serve)` needs one. As a Python
  programmer I expected `async for`, and `await` before `count`. I tried
  `n = await orders().count` and got ``error: `await` needs an async call
  or a task, but this is an `Int` ``. That message got me out, but the page
  should say it in one sentence: *calling an async generator starts it,
  and needs no `await`; its questions (`count`, `find`, …) wait for you, so
  they need none either; an `async def` that is not a generator still
  does.*
- **`-> Iterator[Str]` versus `async Iterator[Str]`.** 17.1 declares an
  async generator as `async def orders -> Iterator[Str]`, then 17.5 says
  "the type of an async sequence can be written: **`async Iterator[T]`**".
  So I wrote `async def orders -> async Iterator[Str]:` with a `yield`,
  which seemed the obvious thing. That compiles and then goes wrong in
  baffling ways:
  - `puts orders().to_list` gives ``error: `to_list` is an `async def`: its
    result arrives later``, although 17.3 used `.to_list` with no `await`;
  - `for o in orders():` gives ``error: cannot loop over an `async async
    Iterator[Str]` ``.

  "`async async`" is where I gave up and went back to the page. The page
  should say that a generator's own result type is written `Iterator[T]`
  and the `async def` supplies the `async`, and that `async Iterator[T]`
  is for functions *without* `yield` and for parameters. The compiler
  would ideally refuse `async def … -> async Iterator[T]` with a `yield` in
  it, and say exactly that.
- 17.3: "It cannot change them, nor use `self` in a method; bind what it
  needs to a name first." I read this twice. "Bind" is a word the
  tutorial has not used before (it says "name"). An example would help.

**Exercises:** about 15 minutes, all doable from the lunch. For 4 I used
`fold(None) { |best, x| if x.len > best.or("").len: Some(x) else: best }`,
and it worked. The answer uses `to_list` and then `max_by`, saying "An
async sequence has no `max_by` of its own". The lunch never says that,
though it does list what it has. Also, all three `bells` values are the
same length, so exercise 4 cannot tell a right answer from one that
returns the first value. Use values of different lengths.

**Worked well:** 17.2's kitchen and waiter; the `break` on an endless
generator; the copies rule for chain blocks, set beside the
during-the-call rule for `each`.

## Lunch 18: Programs alongside

**Pace:** about 40 minutes. Comfortably short.

**Examples:** all seven printed what the page shows. `sleep 30` killed
gives `-1` as stated.

**What confused me:**

- **`read(n)` reads bytes, but Lume counts characters** (lunch 3,
  *Underneath*). 18.5 says "reads exactly the next `n` bytes" and leaves it
  there. I tried `printf 'héllo'` with `read(2)` and got `Some("h�")`, then
  `read(5)` gave `Some("�llo")` (only four characters, with no error).
  Lunch 24 does explain this, but 18.5 is where the method is taught. Say
  here that a body cut in the middle of a character comes out with `�`.
- The page never says what `read_line` and `read(n)` give back. I found
  out from an error that `read(n)` is a `Str?` (``x is declared Int, but
  this is a Str?``). The `.or("")` in 18.5 is the only clue. Say it: "both
  give `None` once the output has ended".
- `puts await ask(counter, "soup")?`: does the `?` apply to the `await`
  or to the call? It works, but precedence of `await` and `?` is never
  said. Since 18.3 also writes `(await c.read_line).or_error(...)` *with*
  parentheses, I could not tell when the parentheses are needed.
- What happens to a `Child` that is never `wait`ed? The answer to exercise
  3 never closes or waits for its `sh` loop. It exits fine, but a reader
  of 18.2 ("`await c.wait` closes the input too") will wonder whether they
  must always call it.

**Error messages:** good. Leaving `?` off `c.write` says ``write can fail,
and nothing here looks at whether it did`` with all three ways out.
Leaving `await` off `c.wait` says ``wait waits for the program to end: its
result arrives later``.

**Exercises:** about 10 minutes. All four were doable from the lunch, and
my answers matched the given ones.

**Worked well:** the shell one-liners as stand-in servers (the order
counter in 18.3 is lovely); 18.5 preparing for lunch 24.

## Lunch 19: HTTP

**Pace:** about 45 minutes, plus a 16-second first fetch of the package.

**Setup:** `lume new pages` made `lume.toml` (with an empty
`[dependencies]`) and a hello-world `main.lume`. The git dependency worked
as written (see the top). The page does not mention that a `lume.lock`
appears, or that the first run is slow while it fetches. One sentence
would stop a reader wondering whether it has hung.

**Examples:** every example printed what its `#` comments say, against
the real example.com: 200 / true / `text/html; charset=utf-8` / true; 404 /
false / "could not make it"; the 19.4 title; the three statuses of 19.5;
and 19.6.

**What was wrong or untaught:**

- **19.5 uses `urls.zip(statuses)`.** `zip` appears nowhere in lunches
  1–18. It reads fine, but it is new, and the page says nothing about it.
- The bullet "use a package: `http`, **which comes with Lume**" made me try
  `import http` without a dependency first. It does not come with Lume: it
  has to be fetched. The error then was ``no module `http`: `http.lume`
  does not exist``, with help about module paths. The message never says
  "if `http` is a package, add it to `[dependencies]` in `lume.toml`",
  which is what I needed to hear.
- The error messages repeat the URL:
  `GET http://127.0.0.1:9/menu: http://127.0.0.1:9/menu: Connection Failed:
  Connect error: Connection refused (os error 61)`. That is cosmetic, but
  19.6 points readers at this message.

**Exercises:** about 15 minutes. 1, 2 and 4 were easy. For 3 I used a
`match`; the answer uses `r.error?`, which the tutorial has never taught
(it first appears in lunch 20's tests). Answer 4 uses `r.map { |x|
x.status }` on a *result*, also never taught, and `{ |_| ... }`. All of
them ran as shown.

**Worked well:** the 404-is-a-Response rule tied back to lunch 15's exit
codes; 19.6's offline package is a nice idea.

## Lunch 20: A CSV query tool

**Pace:** about 75 minutes. Typing the 120-line program in 20.6 took 20 of
them. A little long for a lunch, but satisfying.

**Examples:** 20.6 printed exactly the fifteen lines shown. The command-line
tool in 20.7 gave the shown output for `where dish soup` and `sum cost`
(status 1), usage with status 2, and ``error: cannot read `nofile.csv`: No
such file or directory`` for a missing file. The three tests in 20.8 pass.
`lume build` made a 521 KiB program ("a few hundred kilobytes" is true).

**What was wrong:**

- **"`map_error` (lunch 10's table)"** (20.4, first bullet). Lunch 10's
  table (10.8) does not have `map_error`, and no lunch so far mentions it.
- **"with list patterns (lunch 9)"** (20.5). Lunch 9 teaches variant,
  value, range, `|` and `_` patterns, and guards. It has no list patterns.
  `["sum", c] -> Sum(c)` is new here and is then used again in 21. It is
  intuitive, but the cross-reference is false, and a reader who goes back
  to lunch 9 to look will lose time.
- The tutorial README says Part 3 builds programs "using what the first two
  parts taught", but this lunch introduces, without a word: `[header] +
  rows` (joining lists with `+`), `.error?` (in 20.8's tests),
  `map.to_list` giving pairs, and a block `{ |k, n| ... }` taking a pair
  apart. Each is small; together they break the promise. A line each
  would do: "`+` joins two lists", "`.error?` asks whether a result
  failed", "`to_list` on a map gives its `(key, value)` pairs, and a block
  with two names takes each pair apart".

**What confused me:**

- 20.3 and 20.4 show methods indented by two spaces, with no `struct`
  around them. It took me a moment to see that they go inside `Table`. 20.6
  saves the reader, but a line ("these go inside `struct Table`") would
  help sooner.
- 20.8 `assert false` inside a `match` arm: fine, but `Ok(_) -> assert
  false` is a new idiom.

**Exercises:** about 20 minutes. Adding `Max` first and letting `answer`
fail was great: ``this match on Command does not cover: Max(_), Columns``,
with help. My `max` used `var best: Int? = None` and `Some(n)`; the
answer's starts at `0`, which gives `0` for an empty table (I gave an
error). The answers here are `lume-skip`, though README says the answers
"are checked by the suite too". That is true for Part 1, not for most of
Part 3.

**Worked well:** the shape-first opening (20.1); the error messages that
say which line and which names exist; trying the tool out on built-in
data before reading files. This is the best lunch of my eight.

## Lunch 21: A log analyser

**Pace:** about 70 minutes (100 lines to type, plus building a million-line
log).

**Examples:** 21.5 printed exactly the eight lines shown. 21.6 built, made a
26 MiB `big.log`, and printed exactly the report shown. `time ./loglens
big.log`: 0.57 s ("about half a second", true). `lume run`: 1.43 s
("about 1.4 seconds", true).

**What was wrong or untaught:**

- `pad_right` (21.4) has never been taught. It works as you would guess.
- `r.slowest.map { |s| s.ms }`: **`map` on an optional** has never been
  taught. Neither has `sort` on a list of pairs, nor `sort_by { |p, avg| …
  }` taking a pair apart.
- 21.3: "keeping running totals — **nothing holds the whole log**". The
  whole log *is* held: `File.read(file)?` reads all 28 MB into one `Str`,
  and `analyse(text: Str)` walks it. What is not held is a list of every
  entry. For a lunch whose title promises "a large file", that is the
  first thing a careful reader asks about. A reader with a 10 GB log will
  try it. Say "nothing holds every *entry*", and say that the text is read
  whole. Or point to a line-by-line way of reading, if there is one.
- "the status (lunch 19)": the status codes were explained in lunch 19,
  but this lunch's reader may have skipped the preview chapters (README
  says they "may still change"). A short sentence would do.

**Exercises:** about 25 minutes. All doable, but 2 ("which minute had the
most?") needed `max_by` on map pairs and a `.map` on its optional, which
I guessed from 21.4. The answers use `Error(_) -> ()` (`()` as a value,
never shown) and, in 4, `status.some?`, which is never taught anywhere.

**Worked well:** the generator of `Entry or Error`, where a bad line is an
item, not the end. The three "what makes it fast" bullets. Numbers I
could reproduce.

## Lunch 22: A job runner

**Pace:** about 55 minutes.

**Examples:** 22.5 printed the five lines shown. 22.6's tool gave the
shown report, `failed: broken` on standard error, status 1, and usage with
status 2.

**What confused me:**

- **`line[cut + 2..line.len]`** (22.1). Lunch 3.6 says `s[a..b]` has "both
  included", so this asks for the character at position `line.len`, one
  past the end. I expected a crash. It works, because a slice past the
  end is cut short (I checked: `"ab: cd"[4..100]` is `cd`). Lunch 3 never
  says that. Either write `line[cut + 2...line.len]` (three dots, which
  stops before the end and needs no explaining), or say that a range past
  the end is cut short.
- **"`i` and `job` are named before the `spawn:`, so the task copies those
  values, not the changing `started`."** Lunch 16.5 says a task "gets its
  own copy of the names it uses, made when it starts", so `spawn:
  (started, run_job(jobs.at(started)))` would copy `started` at that
  moment too. I tried it, and it gives the same report. As a Python
  programmer I *know* the late-binding-closure trap, and this sentence
  made me think Lume had it, which 16.5 says it does not. Say "naming
  them first makes it plain which values the task takes", or drop the
  bullet.
- `r"""` raw strings are taught here, called "lunch 3's triple-quoted text
  with no escapes". That is a clear explanation, but lunch 3 never
  mentioned raw strings, so "lunch 3's" reads as if I missed something.
- `line.index_of(": ")` on **text**: lunch 6 taught `index_of` on lists only.
- `None -> ()`: `()` as a value has never been shown (lunch 7 showed it
  only as a block's result type).

**Exercises:** about 25 minutes.

- 1: done. Exit 127 with `sh: no-such-tool: command not found`, as the
  answer says.
- 2: my version matched the answer's shape. I also had to stop the outer
  `while` from spinning once `stop` was set, which the answer's `...`
  hides.
- 3: easy.
- 4 is misleading. "Compare the times of `nap`": the report's per-job
  times are the same with `-j 1` and `-j 4` (207 ms each), because each
  job still sleeps 0.2 s. Only the **whole run's** time changes (0.43 s
  against 0.21 s). The answer means the total ("take about 0.4 seconds
  between them"), but the question points the reader at the per-job
  number. Ask instead: "time the whole run with `time`".

I tried `unless o.ok?:` as a block, and the message (``unless cannot start
an expression … use `if not ...:` ``) got me straight out.

**Worked well:** the pool loop with `next_done`; failures as outcomes,
not errors; the exit-status discipline.

## Lunch 23: A health checker

**Pace:** about 60 minutes, though exercise 4 took 15 of them.

**Examples:** 23.5 printed the six lines shown, in about 1.4 s. 23.6
against example.com printed `up    example` / `DOWN  missing: status 404
(3 tries)` with status 1, as shown.

**What was wrong or confusing:**

- **`Time.sleep` with and without `await`.** Lunch 16.2 taught
  `await Time.sleep(...)`, and said an `async def` "every call to it is
  awaited". Here `Time.sleep(attempt * 20)` has no `await`. The only
  explanation is one sentence: "`Time.sleep(ms)` outside async code simply
  pauses the task it is in". I tried `Time.sleep(10)` with no `await` in
  an `async def main` and got ``error: `sleep` is an `async def`: its
  result arrives later``. So the same call is an `async def` in one place
  and a plain pause in another. This needs a short paragraph, ideally back
  in lunch 16, where `Time.sleep` is introduced.
- "pauses **the task** it is in": `pretend` in 23.5 is called from a task,
  but a reader who calls `run_check` from a plain `def main` is not in a
  task at all. "Pauses the thread it runs on" (as *Underneath* says) is
  clearer.
- "(A block type, from lunch 7, may be a field's type …)": good, needed.
  But nothing has shown a block **stored in a name** or written with typed
  parameters. Exercise 4 needs exactly that (below).

**Exercises:**

- 1–3: about 10 minutes, all doable. I guessed `2.pow(n)` and it worked;
  the answer confirms it, but `pow` has never been taught.
- **4 is not doable from the lunches.** "Count how many times `pretend` was
  called for each URL, with a `shared var` map". I tried:
  - storing a counting block in a name with a typed parameter, `counting =
    { |url: Str| ... }`. I got ``expected `|` after the block parameters,
    found `:` ``, which says what is wrong but not what to do.
  - a multi-line `{ }` block. Lunch 7 says a `{ }` block "holds one
    expression", and no lunch shows a `do` block passed as a struct field.

  The answer needs `calls: shared var {Str: Int}` **as a function
  parameter's type**, which lunch 16 never mentions: it shows `shared var`
  only as a local. The answer works (I ran it: the two down pages fetched
  twice, the good ones once), but a reader could not have found it.
  Either teach `shared var` parameters in 16.6, or give the hint in the
  exercise.

**Worked well:** the main idea, keeping the world behind a block in a
field so the logic can be tested with pretend pages, is explained well
and is worth the lunch. The pretend slow page that "fails however busy the
machine is" is a nice touch.

## Lunch 24: Talking to a language server

**Pace:** about 70 minutes. Typing about 90 lines of dense JSON-building
took 25 of them; the exercises took another 30.

**Setup:** git form of `json` worked as written.

**Examples:** `lume run` printed exactly the five lines shown, five runs
out of five.

**What was wrong:**

- **Exercise 3's answer gives an empty list.** "Ask for
  `textDocument/completion` after `puts double(4).` — the methods an `Int`
  has". I sent a text whose line 4 is `  puts double(4).` and asked at
  line 3, character 17 (just after the dot). The server answers
  `{"isIncomplete":false,"items":[]}` every time. The result is the same
  if the half-typed text follows a clean one ending in `.to_s`. Completion
  *does* work after a **name**: with `x = double(4)` checked cleanly and
  then `puts x.` sent, the first five labels are `["to_float", "to_int",
  "abs", "pad", "pad_right"]`. (docs/install.md says completion is offered
  "after `x.`", and the suite's `tools/lumecheck/lsp.lume`, which the
  lunch points to, only tries names.) So either the exercise should use a
  name, or the server should learn completion after a call. The answer is
  `lume-skip`, which is how this got through.
- **`import json.ToJson` is not needed.** 24.3 opens with three imports.
  With `import json.ToJson` deleted, `lume check` says `ok` and the
  program prints the same five lines. A reader will wonder what it is
  for. Drop it, or say why it is there. (Deleting `import json.Json` gives
  ``unknown type `Json` ``, whose help lists the built-in types and does
  not say "did you mean to import `json.Json`?")

**What was untaught or confusing:**

- 24.2 explains `json.obj`, `to_json`, `show`, `parse` and `path`, but the
  program also uses `Null`, `Arr([...])`, `.int`, `.str` and `.items`, none
  of them explained. I could guess them. A four-row table would help: "a
  value's `.str`, `.int`, `.items` give it as a `Str?`, `Int?`, `[Json]`;
  `Null` and `Arr([...])` make JSON null and an array".
- `x.path("id").map { |v| v.int == Some(id) }.or(false)` is a lot at once:
  `map` on an optional (never taught), compared with `Some(id)`. It is
  correct, but it is the hardest line in the tutorial.
- `{ |m| () }`: a block that does nothing, written with `()`. It has never
  been shown as a value.
- The exercises need LSP knowledge the lunch does not give: what params
  `definition`, `completion` and `rename` take. A link to the protocol's
  spec, or a line saying "they take the same `textDocument` and `position`
  as `hover`", would let a reader do them without the answers.

**Exercises:** 1, 2 and 4 worked as the answers say (`definition: line 1`;
``line 2: unknown name `nope` ``; ``textDocument/rename is not
supported``). 3 did not (above).

**Worked well:** 24.1's explanation of framing, and its link back to
lunch 18's `read(n)`; the honest note that `Content-Length` counts bytes
and `body.len` characters; the remark that hover answers come from the
last clean version, which explains why the hover is asked after the fix.
Ending on "you have now built one" lands.

---

## Across my lunches

- **Part 3 leans on things never taught.** The README promises Part 3 uses
  "what the first two parts taught". In 20–24 I met, untaught: list
  patterns, `map_error`, `error?`, `some?`, `map` on optionals and results,
  `+` on lists, `zip`, `pad_right`, `pow`, text `index_of`, `()` as a
  value, raw strings (taught in passing), `shared var` parameters, blocks
  that take a pair apart. Two of them come with false cross-references
  ("lunch 9", "lunch 10's table"). Each is easy to guess; together they
  make the reader feel they skipped a page. A short "new in this lunch"
  box at the top of each Part 3 lunch would fix most of it.
- **`await` rules are the main stumbling block** of 16–23: async
  generators and their questions need none, `async def` calls need one,
  `Time.sleep` needs one in async code and none outside. Each rule is
  stated once, in a different lunch. One table in lunch 17 would help.
- **Answers for 19–24 are `lume-skip`.** That is unavoidable for network
  and `lume` on `PATH`, but README says answers "are checked by the suite
  too", and the one answer that is wrong (24.3) is unchecked.
- The error messages were, nearly always, enough to get me out. The few
  that were not are listed above (`async async Iterator`, `import http`
  without a dependency, typed block parameters).

## The five things to fix, most important first

1. **Lunch 24, exercise 3 and its answer**: completion after `double(4).`
   returns no items. Use a name (`x = double(4)`, then `x.`), or make the
   server complete after a call. Check every `lume-skip` answer in 19–24
   by hand once more.
2. **Lunch 17, async generator types**: say that an async generator is
   declared `-> Iterator[T]`, and that `async Iterator[T]` is for non-`yield`
   functions and parameters. Writing `async def f -> async Iterator[T]`
   with `yield` gives ``to_list is an async def`` and ``cannot loop over
   an `async async Iterator[Str]` ``; refuse that with a clear message
   instead. Also add the sentence on why `orders().count` needs no
   `await` but `await count_all(...)` does.
3. **False statements**: 17's recap ("the first four stop at their
   answer"); 17.2's "at most one value ahead" (four dishes show two); 20's
   "list patterns (lunch 9)" and "`map_error` (lunch 10's table)"; 21.3's
   "nothing holds the whole log".
4. **Part 3's untaught features**: add a one-line "new here" note for each
   (list patterns, `map_error`, `error?`, `map` on `T?`, `+` on lists, map
   `to_list` pairs and two-name blocks, `pad_right`, `zip`, text
   `index_of`, `()`, JSON's `Null`/`Arr`/`.str`/`.int`/`.items`). Drop the
   unneeded `import json.ToJson` from 24.3.
5. **Exercises that cannot be done, or point the wrong way**: 23.4 needs
   `shared var` parameters (teach them in 16.6, or hint in the exercise);
   22.4 should compare the whole run's time, not `nap`'s; 17.4 needs
   values of different lengths. While there: 22.1's `line[cut +
   2..line.len]` should be `...`, or lunch 3 should say slices past the end
   are cut short; and 23's `Time.sleep` without `await` needs a paragraph
   (best in lunch 16).

# Reviewer B — lunches 9 to 16

Reader: a Python programmer, a little JavaScript, no Lume or Rust. Lume
0.5.0 from `compiler/target/release`, worked in `/tmp/lume-dryrun-b/`.
I skimmed lunches 1–8 first (no exercises) so I knew what had been taught.
I typed every example in lunches 9–16 and ran it, then poked at the edges a
reader would poke at, then did each *Try it* before opening `answers.md`.

One confession first: when I opened `answers.md` for lunch 15 I read on
past the end of the section and saw lunch 16's answers before doing lunch
16's exercises. My lunch 16 exercise notes are coloured by that. (It is
also a note in itself: see "answers.md" at the end of lunch 16.)

**Every example in lunches 9–16 printed exactly what the page says**,
`lume-bad` messages included (each `#!` line is a substring of the real
message). The `lume-skip` examples I could complete also matched. What
follows is everything else.

---

## Lunch 9: Enums and `match`

Pace: about 50 minutes with exercises. Comfortable.

**Worked well.** The order is right: plain variants, then `match`, then
the "does not cover" error, then variants with values. The sentence in 9.2
about adding a variant "months later" is the best motivation for
exhaustive matching I have read; exercise 4 makes you feel it. The real
message (`this match on Course does not cover: Dessert`, help: `add an arm
for Dessert, or a final _ -> arm`) is excellent.

**Confused me / tripped me:**

- **Named fields in a pattern are refused, and the page does not say so.**
  9.3 makes values with names, `Card(last4: "4242")`, and 9.1 says
  variants are "written like a struct's fields". So the first pattern I
  wrote by instinct was `Card(last4: d) -> ...`. Lume says
  `expected ) to close the variant pattern, found :` — which does not tell
  me named patterns are not a thing. 9.3 says "the values are taken in the
  order the variant lists them", which implies it, but one sentence —
  "a pattern names the values by position; `Card(last4: d)` is not
  allowed" — would save the trip. Better still, a help line on that error.
- **The fields of a variant cannot be read with a dot**, `p.last4`
  ("`Payment` has no field or method named `last4`"). Coming from lunch 8,
  where `soup.name` reads a field, I expected it to work at least after a
  `match`. Worth one sentence in 9.3: "the only way to get at a variant's
  values is a pattern".
- **Variants share one namespace.** I tried two enums that both have a
  `Main`; Lume says ``Main is a variant of more than one enum: Course,
  Meal`` and helps with `Course.Main`. Good message, but `Course.Main` is
  never shown in the lunch. Since a Python reader thinks of `Course.MAIN`
  first, mention that both spellings work and when you need the long one.
- 9.5 says a pattern can be "an exact value" and the lunch intro says
  "use `match` on numbers and text too", but every 9.5 example matches an
  `Int`. One text arm (`"tea" -> ...`) would show text works (it does).
- `->` now means two things: a function's result type (lunch 4) and a
  `match` arm. Not a problem in practice, but a half-sentence in 9.2 ("the
  same arrow as a function's result, read as 'gives'") would reassure.
- 9.5: "a `match` on one needs a last arm that catches the rest — `_`, or a
  name for the value". A name as a catch-all is first *used* in 9.6
  (`n -> n * 10`); I read 9.5 twice wondering what "a name for the value"
  looked like. Move the example up or point forward to 9.6.
- 9.4: `match self:` arms are written in a method, but a *statement*
  `match` (arms that `puts` instead of giving a value) first appears in
  lunch 10.3. It works; a reader may wonder whether `match` must give a
  value, since 9.2 stresses "`match` is a value, like an `if`".

**Error messages met.** `does not cover: _` on an `Int` match — the `_` in
"does not cover: _" read oddly, but the help line ("values of this type
have too many cases to list; add a final `_ ->` arm") rescued it.

**Exercises.** All four in about 15 minutes, from the lunch alone. Mine
matched the answers except trivially: in 2 I bound `Drink(name)` where the
answer binds `Drink(n)`. Both work. If `n` was chosen to avoid confusion
with the field `name`, saying so would teach something.

---

## Lunch 10: Absence and failure

Pace: about an hour. Full but fair; this is the most important lunch of
Part 1 and it earns its length.

**Worked well.** The table in 10.1 separating absence from failure is
clear and I will remember it. 10.8 "Choosing" is a great summary table.
The refusal in 10.5 (`save can fail, and nothing here looks at the result`)
with its help line listing `?`, `match` and `_ = ...` is exactly right.
Having `main -> () or Error` exit with status 1 is a nice, practical
detail.

**Confused me / tripped me:**

- **10.4 writes `.or(0 - 1)`, not `.or(-1)`.** I stopped and wondered
  whether Lume has no negative literals, or whether `-1` inside a call
  means something else. `.or(-1)` works fine (I tried it). As written it
  plants a false belief. Use `-1`, or say why not.
- **10.6, the `main -> () or Error` example is not a complete program.**
  It calls `seats`, which is defined in the block above but not in this
  one, and (in the rendered page) nothing marks it as different from the
  runnable examples. The README promises "Every program is complete and
  runs as shown." It also breaks the `#=>` convention with `# prints 6`
  and `# stops: ...`. Either add `seats` to the block or say "with `seats`
  from above".
- **`!` is named but never shown.** 10.8: "There is one more, `!`, which
  takes the value and stops the whole program if there is none." I did not
  know where the `!` goes until I guessed `"5".to_int!` (it works, with a
  warning). One line of example is enough.
- The 10.3 `lume-bad` message's help says "unwrap it first: `match` on
  `Some(x)`/`None`, `?`, or `.or(default)`". In that example `main` is a
  plain `def main`, where `?` is not allowed — following the help's second
  suggestion gives a new error. Fine once you know; slightly misleading
  for the reader the help is for.
- When I wrote `?` in a plain `main`, the error said ``main does not return
  `T or E` `` and the help said ``write def main(...) -> Type or Error``.
  `T or E` and `Type` are not the spellings the lunch teaches; for `main`
  the lunch says `-> () or Error`. Suggest the help says exactly that for
  `main`.
- 10.6 last paragraph: "`?` works on an optional too, in a function that
  gives an optional". True (I tried `xs.find {...}?` in a `-> Int?`
  function), but with no example it went past me on first reading.

**Exercises.** About 15 minutes. Exercise 1 I wrote with an explicit
`if prices.empty?: return None` before `prices.min`; the answer notes
`min` already gives `Int?` and writes it in one line. Good lesson, and I
liked learning it from the answer. 2–4 matched the answers.

---

## Lunch 11: Interfaces

Pace: about 45 minutes. A little short, which is welcome after lunch 10.

**Worked well.** "Nothing to declare" (11.2) is a clear, surprising, well
explained idea for a Python reader (it is duck typing, but checked). The
Napkin error and its help (`Priced needs: def price -> Int. Add it to
Napkin, or extend Napkin with Priced:`) is lovely. 11.6 "Interface or
enum?" is exactly the question I had.

**Confused me / tripped me:**

- **A field does not satisfy an interface's method, and the message hides
  why.** Lunch 8 taught that `soup.price` reads a field and `pie.cheap?`
  calls a method with the same dot, no parentheses. So I wrote a `Dish`
  with a *field* `price: Int` and expected it to be `Priced`. Lume says
  ``Dish ... has no `price` method`` — while `Dish` plainly has a `price`.
  The tutorial should say "an interface asks for methods; a field of the
  same name does not count — add `def price -> Int = ...`". The message
  could say "it has a field `price`, but `Priced` needs a method".
- 11.4 promises a warning and quotes its start — good, it is exactly what
  appeared. But the Try-it exercise 3 produces the same warning and
  `answers.md` does not mention it; a reader may think they did something
  wrong.
- 11.5: `extend Int with Priced:` — the word `with` is the only declaration
  of fit in the whole lunch, right after 11.2 said there is nothing to
  declare. One sentence: "`with Priced` here only says which methods you
  are adding; it is checked like any other fit" would close the loop.

**Exercises.** About 12 minutes, from the lunch alone; my structs differed
from the answer's (`English`/`French` vs `Waiter`/`Sign`), logic the same.

---

## Lunch 12: Generics, modules and tests

Pace: **too much for one hour** — 80–90 minutes for me. Three separate
ideas (generics, modules, tests) plus a packages section. Modules alone
needed three files typed in two folders. Consider splitting: generics in
one lunch; modules, tests and packages in another.

**Worked well.** 12.1's "Written for `[Str]` it would be the same line
again" is a perfect motivation. The `bigger` error (`T could be any type,
so < has nothing to compare`, help: `declare it [T: Ordered]`) teaches the
bound by itself. `lume test` output matched the page character for
character, including the failing case; showing `left:`/`right:` is great.

**Confused me / tripped me:**

- **`Hashable` appears only in the Recap.** "A bound — `[T: Ordered]`,
  `[T: Hashable]`, or any interface — says what `T` must be able to do."
  The lunch body never mentions `Hashable`. Either teach it (one line: what
  it allows — set items and map keys, I assume) or drop it from the recap.
- **"Only what is marked `pub` can be reached from another file" is not
  quite true.** In the café, `Order`'s methods `add`, `total` and
  `summary` are not marked `pub`, yet `main.lume` calls `o.add(...)` and
  `o.summary`. I stopped here for several minutes, expecting an error, then
  added `pub` to `add` to see what happened (it is accepted). The
  reference (`docs/reference/interfaces.md`, "`pub` and methods") says a
  method of a `pub struct` needs no `pub` of its own — the tutorial should
  say that too, in 12.4.
- **Paths "start from the folder that holds the file you run — inside
  `kitchen/` too".** I understood the rule, but it surprised me: in Python,
  a module's imports do not depend on which program imports it. And it has
  a cost the page does not mention: `lume check kitchen/orders.lume`
  fails with ``no module menu.Dish: neither kitchen/menu/Dish.lume nor
  kitchen/menu.lume exists`` and the help "module paths are relative to
  the directory of the file that holds `main`" — but `orders.lume` holds
  no `main`. A reader who checks the file they are editing (as lunch 1
  recommends) hits this at once. Say "check the program from its
  `main.lume`", or make the message say that.
- `main.lume` has both `import kitchen.orders` and
  `import kitchen.orders.Order`. It is explained afterwards (one is for
  `orders.KITCHEN`), but on first reading it looked like a mistake. A
  comment on each line would help.
- `pub KITCHEN = ...` — a `pub` constant is new syntax with no comment.
- 12.3 makes the queue with `var q: Queue[Str] = Queue(waiting: [])`. Good
  that it explains why. The message without the type is good too
  (`cannot tell what T is in this call to Queue`).
- 12.2 claims a bare `T` can be compared with `==`. True; I checked.

**Wrong / surprising (language, not page):** while doing exercise 1, my
first one-line attempt was `xs.skip(xs.len - 2)`. For a one-item list that
is `skip(-1)`, and **it silently gives `[]`** (and `take(-1)` gives the
whole list). Python's `xs[-2:]` would give `['a']`. No error, no warning.
The answer avoids it with an explicit `if`, but a reader who does not
think of it gets a wrong answer silently. Either a negative count should
be an error, or the exercise should warn about it.

**Exercises.** About 25 minutes. 1: see above. 2 and 4: same as the
answers; for 4 I wrote `empty: [Int] = []` as the answer does, then found
`count_of([], 5)` also works (the `x` fixes `T`), which is shorter. 3:
identical to the answer. The hint for 1 (`xs.skip(n)`) is the only place
`skip` is introduced; lunch 13.7 then uses `args.skip(1)` as if known.

---

## Lunch 13: Files and folders

Pace: about an hour. Good.

**Worked well.** Every example makes and removes its own folder under
`/tmp`, so I could run them twice without cleanup. `Dir.remove` "removes a
folder and everything in it, so be careful" — good warning. The small
`todo.lume` tool in 13.7 is a satisfying finish and behaved exactly as
shown. `Env.exit(2)` and `warn` worked as described.

**Wrong:**

- **13.4: "`Path.join` puts exactly one `/` between its two parts".**
  `Path.join("/home/ada/", "/recipes")` gives `/home/ada//recipes` — two.
  It handles a trailing slash on the first part, not a leading slash on the
  second. (Python's `os.path.join` would give `/recipes`, so a Python
  reader is doubly surprised.) Narrow the claim: "whether or not the first
  part ends with `/`".

**Confused me / tripped me:**

- 13.2: "Reading a file that does not exist is a failure, with a reason
  that names the file" — then the example prints `could not read it` and
  throws the reason away. I wanted to see the reason; print
  `e.message` instead.
- 13.1: `File.read(log)?.lines` — `?` in the middle of a chain is a new
  shape; lunch 10 only put `?` at the end. It works as you would hope; a
  half-sentence would say so.
- 13.7: `args.first == Some("add")` — comparing an optional with `==`
  against `Some(...)` is new. Natural, but untaught; lunch 10 said the ways
  in are `.or`, `match`, `?`.
- 13.6: the `check.lume` example shows no output, unlike its neighbours.
- 13.5: I tried `lume run greet.lume --verbose` and `-- -x`; both reach the
  program as arguments. Worth saying, since a reader will wonder whether
  `lume` eats flags.
- 13.2: `File` "also has `remove`, `size` ... and `modified`" — fine as a
  list, but with no example none of them stick.

**Exercises.** About 15 minutes; mine were nearly identical to the
answers. Exercise 2's hint ("`lines.last` gives a `Str?`") was exactly
enough.

---

## Lunch 14: Generators

Pace: about 55 minutes. One of the best lunches.

**Worked well.** 14.2's "cooking ... serving" interleaving makes laziness
visible in a way I have not seen done better, even in Python docs. The
pipeline in 14.5 with the "eighteen — and no more" count is satisfying.
The used-up error in 14.6 and its help line are clear.

**Confused me / tripped me:**

- **14.4 uses `left.min(per_pot)`** — `min` on an `Int`, taking an
  argument. Lunch 6 taught `min` only on a list, giving an `Int?`. I read
  the line twice expecting `Some(4)`. Mention it: "an `Int`'s `min(other)`
  gives the smaller of the two".
- **14.7 `menu.next` versus lunch 5's `next`.** The keyword `next` skips
  a loop iteration; now `next` is a method that takes a value. Worth a
  sentence: same word, different job.
- **When is `.to_list` needed?** 14.1 says a generator "has every method a
  list has" and then always writes `.to_list` before `puts`. But `puts
  courses()` and `puts courses().map(_.upcase)` print the list without it,
  and lunch 7.5 said Lume makes lists where wanted. I spent a few minutes
  working out whether `.to_list` mattered. Say when it does (keeping the
  values to walk twice, 14.6) and that `puts` does not need it.
- "Every method a list has" is not quite true: `courses()[0]` is refused
  ("Iterator[Str] values cannot be indexed"). Indexing is not a method, so
  this is pedantic, but "every method" invites trying.
- 14.6's error points at the *first* `to_list` ("used up here"), not at
  the second use that is the actual problem. Once read, it makes sense;
  the first time I looked at the wrong line.
- 14.8: "A generator method walks a copy of its value taken when it is
  called" — I tested this (push to the list after calling the method) and
  it is true. A tiny example would make it land.

**Exercises.** About 15 minutes, identical to the answers (I first wrote
countdown with `(1..from).reverse`, which also works).

---

## Lunch 15: Running other programs

Pace: about 45 minutes. A little light, which is fine.

**Worked well.** 15.3's distinction — a program that fails is an answer,
one that cannot start is an `Error` — is clear and the `grep` example
makes it stick. 15.4 on arguments not going through a shell is a lesson
many Python programmers learn the hard way with `shell=True`. The error
messages around tuples are excellent: `this tuple has 3 parts, but 2 are
named` (help: use `_`), and `(code, out, err) = ...` on a result without
`?` names the fix exactly.

**Confused me / tripped me:**

- **15.4: `"printf 'pie\\nsoup\\ntea\\n' | sort -r"`** — the doubled
  backslash is never explained. Lunch 3 taught `\n`, not `\\`. I needed it
  for exercise 3 and had to reason out that Lume turns `\\` into `\` so
  that `printf`, not Lume, sees `\n`. One sentence here.
- 15.6: `run_ok` builds its message as `"... failed with #{code}:
  #{err.trim}"`; when the program writes nothing to standard error the
  message ends in `": "` with a trailing space. Exercise 4 then produces
  exactly that, and the answer quietly adds `.trim` to the message. Better
  to fix `run_ok` in the lunch than to work around it in the answer.
- 15.5: the "folder that does not exist is an `Error`" claim is true; its
  message is good (`cannot run ls in /tmp/no-such-dir-b: no such folder`).
  Could be shown.

**Exercises.** About 15 minutes. Same as the answers, except exercise 3:
I left off the final `\n` in printf; output is the same.

---

## Lunch 16: Tasks

Pace: about 60–70 minutes. Dense: tasks, `async def`, lists of tasks,
blocking work, copies, `shared var` and `next_done` are seven ideas.

**Worked well.** The copy rule in 16.5 is explained well and the refusal
when a task changes an outside `var` (`served inside spawn: is a copy, so
changing it here would not be seen outside`, help: `shared var`) is
excellent. Results in start order, with `next_done` for finish order, is a
clean model. The `await t` twice error ("a task is the work itself, so
waiting for it uses it up") is very good.

**Confused me / tripped me:**

- **Why `spawn: await cook("pie", 9)` and not `spawn: cook("pie", 9)`?**
  16.3 writes the `await` inside the `spawn:` without comment. I tried
  without it; Lume refuses with ``cook is an async def: its result arrives
  later`` and the help **"write `await` in front of the call to wait for
  it, or `spawn:` to run it alongside other work"** — but it is already
  directly inside a `spawn:`. The help is wrong for that position, and the
  lunch never says that a `spawn:` body is ordinary code that must itself
  `await` an `async def`. This was my longest stall in the lunch
  (~10 minutes).
- **16.3: "gives their results in the order the tasks were started —
  whichever finishes first".** I read "whichever finishes first" as the
  opposite of what is meant. "— no matter which finishes first" says it.
- **`Time.sleep` in a plain `def main` just works** (it blocks; 1.5 s
  sleep took 1.5 s), but 16.2 presents it as something only an `async def`
  can wait for ("A function that waits for such things is an `async
  def`"). In an `async def main`, the same unawaited call is refused. A
  reader who meets the plain version first will be confused by the
  difference; the lunch should say what `Time.sleep` does outside async
  code.
- Relatedly, calling an `async def` unawaited from a *plain* `def main`
  gives the same help ("write `await` ... or `spawn:`"), and both
  suggestions fail there (`await only works inside an async def`,
  `spawn: needs the program to start with async def main`). The help
  should say "make `main` an `async def main`" in that case.
- **`spawn:` on one line only takes an expression.** I wrote
  `spawn: four += 1` (exercise 3, in a `map` block). Errors: on its own
  line, ``expected the end of the line, found += `` with help "wrap long
  expressions in parentheses" — doing that gives ``expected ) to close the
  parenthesis, found +=``. Inside `{ |w| spawn: four += 1 if ... }`:
  **"only a name or a field can be assigned to"** — but `four` *is* a
  name. Both messages misdirect. The lunch shows a one-line `spawn:` only
  with expressions and a block `spawn:` for statements, but never says
  that's the rule; one sentence ("a one-line `spawn:` takes a value; for a
  statement, indent it under `spawn:`") and a better message would fix it.
- **`Task[()]`** in 16.6 is the first time I saw `()` used as a type
  inside brackets. Lunch 7 used `()` as a block's result type, so it is
  derivable, but "a task that gives nothing is a `Task[()]`" would help.
  Without it I tried `ts = []` and got the help ``add the type: ts: [Int]
  = ...`` — `[Int]` is the wrong suggestion for a list of tasks.
- **`.to_list` after `map` of spawns**: every example writes
  `await xs.map { |n| spawn: ... }.to_list`; I left `.to_list` off and it
  works the same. Readers will wonder whether it starts the tasks, or
  matters. Say.
- 16.4: `Process.run("echo", [word]).or((1, "", ""))` — the double
  parentheses (a tuple as the argument) are right but read like a typo.
- The `shared var` warning appears even in a program with no tasks at all
  (a plain `shared var n = 0; n += 1`). Harmless; noted.

**Exercises.** I had (accidentally) seen the answers, so timing is not
honest; about 20 minutes, most of it on exercise 3 and the `spawn:` errors
above. My 2 left off `.to_list`; my 3 used a `do |w|` block with an
indented `spawn:` instead of the answer's `for` loop — shorter and needs
no `Task[()]` annotation. 1 and 4 matched.

**answers.md:** sections run straight on into each other; reading to the
end of one lunch's answers shows the next lunch's. A "spoiler space", or
one file per lunch, would prevent what happened to me.

---

## Things assumed but not taught by lunches 1–8 (or earlier in 9–16)

- `Int.min(other)` (14.4) — only list `.min` was taught.
- `\\` in a string (15.4) — only `\n` and `\t` were taught (3.7).
- `xs.skip(n)` — introduced only in a lunch 12 exercise hint, then used in
  13.7.
- Comparing an optional with `== Some(...)` (13.7).
- `?` in the middle of a chain (13.1).
- `Hashable` (12 recap).
- `()` as a type parameter, `Task[()]` (16.6).
- Statement-form `match` arms (`-> puts ...`), first in 10.3, after 9.2
  stressed `match` as a value.

---

## The five most important things to fix

1. **Lunch 16: explain `spawn: await f(...)`, and fix the help that
   misdirects.** Say that a `spawn:` body is ordinary code that must
   `await` an `async def` it calls. The "or `spawn:`" help appears even
   inside a `spawn:` and in a plain `main` where neither suggestion works;
   and one-line `spawn: x += 1` gives "only a name or a field can be
   assigned to" or "wrap ... in parentheses" (which does not work). Also say
   what `Time.sleep` does in a plain `def`, where it is accepted unawaited.
2. **Lunch 12: say that methods (and fields) of a `pub struct` need no
   `pub`**, since the café calls non-`pub` methods right after "Only what is
   marked `pub` can be reached from another file". And say how to check a
   module file that has no `main` (or fix the message that tells you paths
   are relative to "the file that holds `main`"). Consider splitting the
   lunch; it is the one that did not fit in an hour.
3. **Fix two false or untaught things on the page:** 13.4 "`Path.join`
   puts exactly one `/`" (false when the second part starts with `/`), and
   12's recap `Hashable` (never taught). Also 10.6's `main -> () or Error`
   block, which is not a complete program despite the README's promise.
4. **Lunches 9 and 11: say what a reader from lunch 8 will try.** A
   variant's values cannot be named (`Card(last4: d)`) or dotted
   (`p.last4`) in a pattern — by position only; and a *field* does not
   satisfy an interface's method (and the message "has no `price` method"
   should mention the field).
5. **Small unexplained choices that plant false beliefs**: `.or(0 - 1)`
   (10.4; `-1` works), `.to_list` written everywhere it is not needed (14,
   16), `left.min(per_pot)` (14.4), `\\n` (15.4), `Task[()]` (16.6).
   And, in the language: `skip`/`take` with a negative count silently give
   `[]` / the whole list — the lunch 12 exercise leads straight into it.

# Reviewer A: notes on lunches 1 to 8

I came to this as a Python programmer with a little JavaScript and no Rust.
I read `docs/tutorial/README.md` and `docs/install.md` first, then lunches 1
to 8 in order. I typed every example into `/tmp/lume-dryrun-a/`, ran it with
`compiler/target/release/lume` (0.5.0) on `PATH`, and did each *Try it*
before opening `answers.md`.

Overall the tutorial reads well. Each lunch was 35 to 50 minutes of typing,
running and exercises, so they fit in an hour with time to spare. Almost
every `#=>` line matched what I got. The compiler's messages were the best
part: nearly every mistake I made, on purpose or by accident, came with a
`help:` line that told me what to write. I found one wrong error message on
a page (2.6), one exercise whose natural answer crashes the compiler (6,
*Try it* 3), and a few places where a lunch relies on something not taught
until later.

---

## README.md and install.md

- **README.md** is clear. The table of lunches told me what I would be able
  to do after each one, which helped. "**Every example in these chapters is
  run by Lume's test suite**" made me trust the pages. That trust took a
  knock in lunch 2 (see 2.6 below).
- **install.md, top**: "`./install.sh`" assumes I already have the
  repository and am standing in it. Nothing says "clone this first" or
  gives a URL. A reader who found the tutorial on the web would be stuck
  here. The same goes for "`lume run examples/fib.lume`" in *Check it*,
  which only works from the repository root. Lunch 1 then says "Make a
  folder for these lunches", which is a different folder, so the reader has
  to work out where `examples/` is relative to them.
- **install.md, *The commands***: `lume --help` shows more flags than this
  table does (`fmt --check | --stdout`, `clean --cache`). That is minor, but
  the table looks like the full list.
- **install.md, *The commands***: "edit-and-run takes about 0.2 s". On this
  Mac a one-line edit to `hello.lume` took 0.33 to 0.37 s. The very first
  run took 2.5 s (lunch 1.1 says "a second or so"). Close enough, though
  saying "under half a second" would be more honest.
- **install.md** is long for a first read. *Editors* and the `--json`
  example are aimed at tool writers, not at someone about to type
  `hello.lume`. I skimmed past them. That is fine, but a line such as "you
  can stop reading here and start lunch 1" after *Check it* would help.
- All links in README.md and lunches 1 to 8 resolve, including the
  `reference/numbers.md#what-stops-the-program` anchor.

## Lunch 1: Hello, Lume (about 25 minutes)

- Every example printed what the page shows: 1.1, 1.2, 1.3 and the error
  messages in 1.4 and 1.5. The full error block in 1.5 matched character
  for character, column 20 included.
- **1.5, "The help line says what to write instead; lunch 2 shows it."**
  The page shows only the `#!` line, not the help, so I could not see what
  "the help line" was until I ran it. The help Lume printed (`to build a
  string, interpolate: "...#{12}..."`) is good. Showing it on the page, as
  1.5 does for `nmae`, would make the sentence make sense as I read it.
- **1.4**: Lume points its `^` at column 1 of the over-indented line, not at
  where the extra indentation ends. That is minor, but it looked odd next to
  1.5's careful explanation of the `^`.
- **1.6 / *Underneath***: `lume build hello.lume -o hello` worked (474 KiB
  binary). `lume emit` matched the page.
- **Exercises**: 5 minutes for all four. Mine matched the answers. For
  answer 4, "`lume fmt` puts one space between `puts` and its value, and
  indents each block by two spaces": it also fixed `def    main:`, `x   =
  1+2` → `x = 1 + 2`. The answer undersells what it does. Answer 3 covers
  only the missing `:`, not the first half of the exercise (a misspelled
  name), which is fine.
- **Worked well**: "`#=>` at the end of a line shows what that line prints.
  You do not type it — though you can". Knowing I could type it as a
  comment made typing the examples quicker.

## Lunch 2: Names and values (about 40 minutes)

- **Wrong, 2.6 *What a name can be***: the page says

  ```
  for = 3
  #! expected a name to loop with
  ```

  Lume 0.5.0 actually says `error: expected a loop variable, found `=``, at
  2:7, with **no help line**. So the page's message is wrong. Worse, the
  real message does not do what the sentence before it promises: "A few
  words belong to the language and cannot be names … Using one says so". It
  does not say so. Nothing tells me `for` is reserved. The other words fare
  no better: `match = 3` → "expected a value, found `=`", `var = 3` →
  "expected a variable name, found `=`", `puts = 3` → "expected a value,
  found `=`", `true = 3` → "only a name or a field can be assigned to".
  Coming from Python, where `for = 3` gives "invalid syntax", I would want
  "`for` is a keyword and cannot be a name". Either the message or the page
  needs fixing. The README's claim that the suite checks every page made me
  assume my typing was wrong, and I spent about 5 minutes re-checking a
  two-line file.
- **2.3 / Recap, "on the next lines of the same block"**: I read this as
  the transform having to come on the line right after. It does not: a
  `puts` between `t = "..."` and `t = t.trim` is accepted. The rule seems to
  be "the same block" and "the right-hand side starts with the name". A
  transform inside an `if` is refused, with a very good message ("a new `t`
  here would hide it and be lost when the block ends"). The page could say
  "later in the same block" and show that case.
- **2.3**: `t = t + 1` also counts as a transform. That surprised me, since
  the page only shows `order.trim`. "The right-hand side starts with the
  name itself" does cover it, but only if you read closely.
- **Error help with the wrong line**: for `t = 5`, `t = t + 1`, `t = 9`, the
  help says "declare it with `var t = ...` on line 3". Line 3 is the
  transform. The declaration is on line 2.
- **Exercises**: about 12 minutes. Exercises 1 to 3 matched the answers. For
  **exercise 4**, I wrote, naturally, `puts (price * 3).to_float + tip` and
  hit "a space between `puts` and `(` is ambiguous". That trap is not
  explained until **lunch 3.4**. The help message got me out in a minute,
  but the exercise walks the reader into a lunch-3 problem. The answer
  avoids it by naming `total` first. A hint in exercise 4 ("name the total
  first"), or moving 3.4 earlier, would fix it.
- Small: when I wrote `price * 3.to_float + tip`, Lume reported the `+`
  (Int and Float) and not the `*` that was also wrong. Fixing the `+` then
  showed the `*` error. One error per run is fine, but the first one shown
  was not the first one in the line.
- **Worked well**: the immutability message ("declare it with `var price =
  ...` on line 2 if it needs to change; `price = price.something` is
  allowed as a transform") taught the transform rule better than the prose
  did.

## Lunch 3: Numbers and text (about 40 minutes)

- Every `#=>` matched: arithmetic, `decimals`, `round`/`floor`/`ceil`,
  `to_int` giving `Ok(42)` and `Error("`lots` is not an integer")`, the text
  methods, slicing, `split`/`join`/`chars`, and triple-quoted text.
- **Missing for a Python reader, 3.1**: Lume's `Int` division and remainder
  are not Python's for negative numbers. `-7 / 2` is `-3` (Python gives
  `-4`), and `-7 % 2` is `-1` (Python gives `1`). The page says only "with
  any fraction dropped". One line, "rounds toward zero, as in Rust and C,
  not down as in Python", would prevent a real bug.
- **3.2**: `2.5.round` is `3.0`, which rounds half away from zero, while
  Python's `round(2.5)` is `2`. That is worth half a sentence.
- **3.1, runtime division by zero**: the page says the program "stops when
  it gets there, with `error: division by zero`". It does, but the output
  then says only "(set LUME_BACKTRACE=1 to see where in the generated
  Rust)". There is no Lume file or line. For a reader who is told "You never
  need to read this Rust" (lunch 1), being pointed at the generated Rust to
  find the failing line is a let-down. The same happens with `.at(5)` in
  lunch 6.
- **3.6, `s[a..b]` includes both ends**: the page is clear ("both
  included"). Still, every Python reader will type `word[0..4]` expecting
  `"sand"`. A short "unlike Python's `word[0:4]`" would earn its place.
  `1...4` (three dots) is the exclusive one, from 5.5, but slices never
  mention three dots.
- **3.5**: `capitalize` appears only inside the chain
  `order.trim.downcase.capitalize`, not in the list of methods above it, yet
  exercise 3 needs it.
- **Exercises**: about 10 minutes, and all matched. **Exercise 3** ("print
  `LUNCH BOX`, then `9`, then `Lunch box`") is a puzzle: it does not say the
  `9` is the length. I worked it out, but "its length, `9`" would be kinder.
  For exercise 2, I first wrote `"18".to_int` without `.or` and got an
  excellent message ("this value is an `Int or Error` — it may be an
  error…; unwrap it first: … `.or(default)`").
- **Worked well**: 3.4 explains *why* 3.1 named `first`, which closed a
  question I had from three sections earlier. The 0.1 + 0.2 section is
  honest and short.

## Lunch 4: Functions (about 30 minutes)

- Every example matched, and so did all five error messages, including
  the help on `price("soup", "two")`, which suggests `.to_int`.
- **4.6 vs 8.2, `()` on calls with no arguments**: a function with no
  parameters is defined without parentheses (`def special -> Str`) but must
  be *called* with them (`special()`). A method with no parameters is
  defined the same way and called *without* them (`pie.label`). The
  tutorial explains each rule where it comes up ("so that the call cannot
  be mistaken for a name"; "the dot already says it is a call"). Still, I
  had to stop and reconcile the two rules in lunch 8. A sentence in 8.2
  that refers back to 4.6 would help. Also, `def special() -> Str` is
  accepted too. The page could say so, since a Python reader will write it.
- **4.3**: the first `price` uses `if … : 4 else: 3` before lunch 5 teaches
  it. The page flags this ("[lunch 5] is about it"), which is enough.
- **4.4**: I tried mixing positional and named arguments. `f(b: 1, 2)` gives
  "a positional argument cannot follow a keyword argument", the same rule
  as Python's. The page never says whether mixing is allowed. One line
  would do.
- **Exercises**: about 8 minutes. My answers were identical to the given
  ones, character for character. Exercise 3 needs `if` and `<=`, which the
  4.7 example shows well enough to copy.
- **Worked well**: "Lume has no default values for parameters … write a
  second, shorter function that calls the first" heads off the first thing
  a Python programmer would try.

## Lunch 5: Making decisions (about 35 minutes)

- Every example matched, including `else if` → "`else if` is written `elif`
  in Lume" and the missing `else`. Typing `&&` gives "`and` joins
  conditions in Lume", and `unless x:` as a block gives "use `if not ...:`
  here". Both are very good.
- **5.5 ranges**: "`1..3` is 1, 2 and 3 — two dots include the last number
  — and `1...3` is 1 and 2". This is the reverse of what I expected (more
  dots, more numbers). The page states it clearly, but a mnemonic would
  help. *Underneath* says it is also the reverse of Rust, which will
  confuse readers who know Rust.
- **5.5**: `for i in 3..1:` prints nothing, silently. That is consistent
  with "A range never counts down", but a warning when both ends are
  literals would catch a common slip.
- **5.8**: `var rang: [Str] = []` and `rang.push(...)` come one lunch before
  lists are taught (6.2). They are guessable, but `var` with a type and
  `push` are both new here.
- **Exercises**: about 10 minutes, and all four matched. **Exercise 4**,
  "stopping at the first one that is a multiple of both", does not say
  whether 12 is printed. I printed it, and so does the answer, but "up to
  and including" would remove the doubt.
- **Worked well**: the bell program in 5.8 puts everything together, and
  the note on why `15` comes first is exactly the right lesson.

## Lunch 6: Lists, maps and sets (about 50 minutes, the longest)

- Every example matched: `Some`/`None` from indexing, `pop`, `sum`/`max`/
  `min`, map order, the tally, and set operations.
- **Wrong / compiler gap, exercise 3** ("Count the letters in `"banana"`
  with a map: `.chars` gives the letters"): I wrote the obvious thing,
  following 6.5's tally exactly:

  ```
  var letters: {Str: Int} = {}
  for c in "banana".chars:
    letters[c] = letters[c].or(0) + 1
  ```

  Lume type-checked it, then failed with "error: the generated Rust did not
  compile. This is a gap in the Lume compiler…" and a rustc error about
  `lume_as_str` on `char`. `puts m[c].or(0)` and `m[c] = 1` with a `{Str:
  Int}` map fail the same way. (`s: Str = c` is accepted and works, so a
  `Char` does turn into a `Str` in some places but not as a map key.) The
  answer uses `{Char: Int}`, but **`Char` has not been mentioned anywhere
  in lunches 1 to 6**. 3.6 shows `"tea".chars` printing `["t", "e", "a"]`,
  which looks exactly like a `[Str]`. So the exercise leads a reader straight
  into an internal compiler error that they cannot fix from the lunch. The
  fix is either in the compiler (reject `Char` as a `Str` key with a
  message, or convert it) or in the tutorial (introduce `Char` in 3.6 and
  say the key type in the exercise). Both would be best. It cost me about
  10 minutes, and a real reader would probably give up.
- **Exercise 4** ("Print, as a list, the dishes only the first one likes")
  needs `.to_list`, which is first mentioned in **lunch 7.5**. I guessed
  `.to_list` because Lume has `to_s`, `to_int` and `to_float`, and it
  worked. A reader who guesses `.list` gets "`{Str}` values have no method
  `list`" with **help: "did you mean `first`?"**, which points the wrong
  way. Either mention `to_list` in 6.6 or drop "as a list" from the
  exercise.
- **6.1 indexing**: `menu[-1]` gives `None` with no warning. Every Python
  reader will try `xs[-1]` for the last item, and silently getting nothing
  is a trap. `.last` exists (it appears in passing in 6.3's list). One line
  in 6.1, "`xs[-1]` is not the last item; use `xs.last`", would help.
- **6.1**: the error for `menu[0] + "b"` says "this value is a `Str?`". The
  `T?` notation is not taught until lunch 10. The error is otherwise good,
  and the page could say in passing "Lume writes this type `Str?`".
- **6.1 `.at`**: past the end, it prints "error: no item at 5 — the list has
  1" with no line number (see the note on lunch 3).
- **Exercises**: 1 and 2 matched (3 min). 3 was the crash above (10 min).
  4 matched the answer once I had guessed `to_list` (3 min).
- **Pace**: this lunch is heavy, with lists, maps, sets, `Option` and value
  semantics. It fit in an hour only because the examples are small.
- **Worked well**: 6.8, "A list is a value", is the opposite of Python and
  is shown with exactly the right example. The tally idiom
  `counts[o].or(0) + 1` is a nice payoff of the `.or` from lunch 3.

## Lunch 7: Blocks and chains (about 35 minutes)

- Every example matched: `map`/`filter`/`reject`, `_` shorthand, `any?`/
  `all?`/`count`/`find`/`sum`, `sort_by`/`max_by`, chains, `do`, `fold`,
  function names as blocks, and block parameters.
- **7.6, the `do` example**: `with_tax = p * 11 / 10` on `4, 2, 7` gives
  `4, 2, 7`, because of `Int` division, so `total` is `13`, the same as the
  plain sum. I read it twice, thinking I had mistyped, before I saw that the
  "tax" rounds away to nothing. The example teaches a block but quietly
  shows that the tax does nothing. Prices of `40, 20, 70` (total `143`), or
  a comment, would make the output look meant.
- **7.5**: "When you do want to, `.to_list` makes it explicit." This is where
  `to_list` first appears, but lunch 6's exercise 4 needed it (see above).
- **7.1**: blocks are Ruby syntax (`{ |p| p * 2 }`). The page explains them
  well enough that I did not miss Python's `lambda`. Typing `lambda x: x *
  2` out of habit gives "expected `,` or `)` in the argument list, found
  `x`", which does not point at blocks. A special-case message ("Lume
  writes this `{ |x| x * 2 }`"), like the ones for `&&` and `else if`,
  would match the rest of the compiler.
- **7.9**: `(Int) -> ()` is introduced well. `()` as "gives nothing" is
  new, and the page explains it in one clause, which was enough.
- **Exercises**: about 10 minutes, and all four matched the answers.
  Exercise 4's hint "(A block of two values is written `{ |a, b| ... }`)"
  was exactly what I needed. I wrote `0...(xs.len - 1)`, and later found that
  `0...xs.len - 1` also works. The page never says how `...` binds against
  `-`.
- **Worked well**: the *Underneath* box ("a chain of five steps costs no
  more than the loop you would have written by hand") answered the
  performance worry I had.

## Lunch 8: Structs (about 35 minutes)

- Every example matched, along with all three error messages. Structs in a
  set work, as 8.3 says.
- **Trap, 8.2 "a field is written by its name alone"**: the page never says
  what happens when a parameter has the same name as a field. I wrote the
  setter every Python programmer writes:

  ```
  def set_price(var self, price: Int):
    price = price
  ```

  It compiles with **no error and no warning** (`lume check` says `ok`), and
  it does nothing: the field stays `1`. `self.price = price` works. The
  rule seems to be that the parameter hides the field, and `price = price`
  is taken as a transform of the parameter. This deserves a sentence in
  8.2 or 8.5 ("a parameter with a field's name hides it; write `self.price`
  to mean the field"), and the compiler should warn about a
  self-assignment, or about a parameter hiding a field in a `var self`
  method.
- **8.2**: "One that takes nothing needs no `()`". See the lunch 4 note on
  the difference from plain functions.
- **8.5**: the *both halves* explanation, with `var self` in the
  definition and `var` at the call, is clear, and the messages for each
  half say exactly which half is missing.
- **8.6**: "Such a function has no value to work on, so it cannot read a
  field". Clear.
- **Exercises**: about 12 minutes, and all matched. Mine combined 1, 2 and 4
  into one `Guest`, as the answer does for 1 and 2. `done.push(dish)` inside
  a `var self` method worked first time, which was satisfying.
- **Worked well**: free `==`, printing and hashing for structs, and the
  `def self.` constructors. *Underneath* maps it all to Rust in four lines.

---

## The five most important things to fix

1. **Lunch 6, *Try it* 3, leads to an internal compiler error.** The
   natural answer, a `{Str: Int}` tally over `"banana".chars` written just
   like 6.5, type-checks and then fails in rustc ("the generated Rust did
   not compile … `lume_as_str` on `char`"). `Char` is never introduced in
   lunches 1 to 6, and `.chars` prints like a list of `Str`. Fix the
   compiler (convert the key, or reject it with a message), and introduce
   `Char` in 3.6 or name the key type in the exercise.
2. **Lunch 2.6 shows a message Lume does not give.** The page says `for =
   3` gives "expected a name to loop with". Lume 0.5.0 says "expected a loop
   variable, found `=`", with no help. No keyword used as a name gives a
   message that says the word is reserved, contrary to "Using one says so".
   Fix the message, or the page and the suite's check of it.
3. **A parameter with a field's name silently hides the field (lunch 8).**
   `def set_price(var self, price: Int): price = price` compiles cleanly and
   does nothing. Say so in 8.2/8.5, and make the compiler warn.
4. **Exercises that need what has not been taught yet**: lunch 2 exercise 4
   runs into the `puts (` trap from 3.4, and lunch 6 exercise 4 needs
   `.to_list`, first shown in 7.5. (Exercise 3 there also needs `Char`.)
   Add hints, or move the teaching earlier.
5. **Python habits that fail silently or surprisingly, not mentioned**:
   `xs[-1]` is `None` (6.1), `-7 / 2` is `-3` and `-7 % 2` is `-1` (3.1),
   `s[0..3]` includes position 3 (3.6), `2.5.round` is `3.0`. Also, runtime
   errors (division by zero, `.at` past the end) give no Lume line, only
   "set LUME_BACKTRACE=1 to see where in the generated Rust". A short "if
   you know Python" note in lunches 3 and 6 would prevent real bugs.

Also worth a look: install.md never says to get the repository before
`./install.sh`; the 7.6 tax example's tax rounds to nothing; and `.list`
gets the help "did you mean `first`?".

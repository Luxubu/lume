# Text, and the characters in it

## `len` counts characters, not bytes

> Reviewers guessed at this repeatedly, and guessed both ways.

**Every position and length in Lume is in characters.** `len`, `slice`,
`s[i]` and `s[a..b]` all count the same way, so a program never has to think
about UTF-8.

```lume
puts "héllo".len          #=> 5
puts "héllo".chars.len    #=> 5
puts "🙂ab".len           #=> 3
```

`"héllo"` is six bytes and five characters, and Lume says five. There is no
byte-length method.

## `Char`

`Str` and `Char` are different types. **There is no character literal** — `'a'`
is not valid Lume:

```lume-bad
c = 'a'
#! strings use double quotes
```

A `Char` comes from somewhere else instead. `s.chars` is a `[Char]`, and
`s[i]` is a `Char?` because the position may be past the end:

```lume
s = "héllo"
puts s[1]           #=> Some("é")
puts s[99]          #=> None
puts s[1].or("?")   #=> é
puts "abc".chars    #=> ["a", "b", "c"]
```

So `s[i]` is a `Char?` and `s[i].or("?")` is a `Char`. Once you have the
`Char`, it has its own methods:

```lume
c = "héllo"[1].or("?")
puts c.upcase     #=> É
puts c.code       #=> 233
puts c.alpha?     #=> true
puts c.digit?     #=> false
```

### Going between `Char` and `Str`

**A one-character string literal is a `Char` wherever a `Char` is wanted**, and
that is how you write one down:

```lume
c: Char = "a"
puts c          #=> a
puts c == "a"   #=> true
```

Going back is `.to_s`, but you rarely need it: a `Char` is accepted wherever a
`Str` is wanted, including in `+` and in a typed binding.

```lume
c: Char = "a"
puts c.to_s        #=> a
s: Str = c
puts s             #=> a
puts "x" + c       #=> xa
puts c + "x"       #=> ax
```

**`Str + Char` works, and so does `Char + Str`.** You do not need `.to_s`.

From a number, `Int.to_char` gives a `Char?`, because not every number is a
character:

```lume
puts 97.to_char            #=> Some("a")
puts 97.to_char.or("?")    #=> a
```

A `Char` compares, sorts, goes in a set and works as a map key:

```lume
var tally: {Char: Int} = {}
for c in "hello".chars:
  tally[c] = tally[c].or(0) + 1
puts tally["l"].or(0)    #=> 2
```

`for c in s` is not allowed — `for` takes a list, a set, a map or a range, so
write `for c in s.chars`:

```lume-bad
for c in "abc":
  puts c
#! cannot loop over a `Str`
```

## Case

`upcase` and `downcase` do the whole string. **`capitalize` raises the first
character and lowers everything after it** — it is not "first letter up, rest
untouched".

```lume
puts "hello world".capitalize    #=> Hello world
puts "HELLO WORLD".capitalize    #=> Hello world
puts "hELLO".capitalize          #=> Hello
puts "".capitalize               #=>
puts "héllo".upcase              #=> HÉLLO
```

## Predicates, and what they say about `""`

`digit?`, `alpha?` and `space?` ask whether **every** character is of that
kind. **On the empty string they are all `false`**, not `true`. The vacuous
reading would have made `if s.digit?` mean "yes, parse it" for a string with
nothing in it, so Lume answers `false` and leaves `empty?` to say the string
is empty.

```lume
puts "".digit?     #=> false
puts "".alpha?     #=> false
puts "".space?     #=> false
puts "".empty?     #=> true
puts "123".digit?  #=> true
puts "12a".digit?  #=> false
puts "ab c".alpha? #=> false
```

`empty?` is the one that is `true` for `""`. It is the predicate you want for
"is there anything here".

The `Char` predicates ask the same questions of one character, and a `Char` is
never absent, so there is no empty case:

```lume
c: Char = "7"
puts c.digit?    #=> true
puts c.alnum?    #=> true
puts c.upper?    #=> false
```

## Ordering

**`<` compares by code point, character by character.** There is no
case-insensitive comparison and no locale. Among the ASCII letters that puts
every capital before every lowercase one, because `A`–`Z` are 65–90 and
`a`–`z` are 97–122.

```lume
puts "abc" < "abd"     #=> true
puts "abc" < "abcd"    #=> true
puts "A" < "a"         #=> true
puts "Zebra" < "apple" #=> true
puts ["banana", "Apple", "cherry"].sort.join(",")   #=> Apple,banana,cherry
puts ["b", "A", "a", "B"].sort.join(",")            #=> A,B,a,b
```

`"Zebra" < "apple"` is `true` because `Z` is code point 90 and `a` is 97:

```lume
puts "Z"[0].or("?").code    #=> 90
puts "a"[0].or("?").code    #=> 97
```

Beyond ASCII it is still the code point and nothing cleverer — `É` is 201, so
it sorts after `a`:

```lume
puts "É"[0].or("?").code    #=> 201
puts "É" < "a"              #=> false
```

To sort the way a person reads, sort on the lowered form:

```lume
puts ["banana", "Apple", "cherry"].sort_by { |w| w.downcase }.join(",")
#=> Apple,banana,cherry
```

`Char` compares and sorts the same way:

```lume
puts "cab".chars.sort.join("")    #=> abc
```

## `slice(a, b)` is offset and *length*

> This was guessed wrong in every round that used it.

**`slice(offset, count)` takes `count` characters starting at `offset`.** The
second argument is a length, not an end position.

```lume
s = "hello world"
puts s.slice(0, 5)     #=> hello
puts s.slice(6, 5)     #=> world
puts "abcdef".slice(1, 3)   #=> bcd
```

`slice(6, 5)` is "five characters from position 6", not "positions 6 through
5". Both arguments are clamped, so nothing is ever out of range:

```lume
puts "[" + "abc".slice(1, 99) + "]"    #=> [bc]
puts "[" + "abc".slice(99, 2) + "]"    #=> []
puts "[" + "abc".slice(1, 0) + "]"     #=> []
```

## `s[a..b]` is start and end, and `..` includes the end

The range form is the other one: it takes a **start and an end position**.
`..` includes the end character and `...` stops before it.

```lume
s = "abcdef"
puts s[0..2]     #=> abc
puts s[0...2]    #=> ab
puts s[1..3]     #=> bcd
puts s[4..4]     #=> e
```

So `s.slice(1, 3)` and `s[1..3]` both give `bcd` here, for two different
reasons. When in doubt, use the range form — it reads the way it works.

Ranges are clamped to the string, so a slice can never fail:

```lume
s = "abcdef"
puts s[2..99]          #=> cdef
puts "[" + s[3..2] + "]"   #=> []
puts "héllo"[1..2]     #=> él
```

`s[a..b]` gives a `Str`, not an optional — clamping is what makes that safe.
`s[i]` with one index is the one that gives a `Char?`.

## `repeat`

```lume
puts "x".repeat(3)               #=> xxx
puts "ab".repeat(2)              #=> abab
puts "[" + "x".repeat(0) + "]"   #=> []
puts "-".repeat(10)              #=> ----------
```

**`repeat(0)` is the empty string**, which is what makes it safe to use for
indentation that may be zero deep.

## `split`, and getting the string back

There are two splits, and they are deliberately different.

**`split(sep)` keeps every piece**, including the empty ones at the ends and in
the middle. That is what makes it reversible: `s.split(sep).join(sep)` is `s`
again, always.

```lume
puts "a:b:c".split(":")    #=> ["a", "b", "c"]
puts "a:".split(":")       #=> ["a", ""]
puts ":a".split(":")       #=> ["", "a"]
puts "a::b".split(":")     #=> ["a", "", "b"]
puts "".split(":")         #=> [""]
puts "a:b:c".split(":").join(":")   #=> a:b:c
```

Note `"a:".split(":")` is **two** pieces, and `"".split(":")` is **one** — a
list holding the empty string, not an empty list. A `split(sep)` never gives
nothing back.

**Bare `split` splits on whitespace and drops the empties.** It is the one for
words, and it is the one that can give an empty list:

```lume
puts "a b c".split          #=> ["a", "b", "c"]
puts "  a  b  ".split       #=> ["a", "b"]
puts "".split               #=> []
puts "".split.len           #=> 0
puts "".split(":").len      #=> 1
```

The separator can be more than one character:

```lume
puts "a::b::c".split("::")   #=> ["a", "b", "c"]
```

`lines` splits on line breaks, and an empty string has no lines:

```lume
puts "a\nb\nc".lines   #=> ["a", "b", "c"]
puts "a".lines         #=> ["a"]
puts "".lines          #=> []
```

## `#` inside a string

**A `#` in a string literal is just a `#`.** Comments do not start inside
quotes, and `#` only begins an interpolation when the very next character is
`{`.

```lume
x = 5
puts "# not a comment"   #=> # not a comment
puts "a # b"             #=> a # b
puts "##{x}"             #=> #5
puts "#{x}#"             #=> 5#
```

`"##{x}"` lexes: the first `#` is a literal `#`, and the second starts the
`#{x}`.

## Escapes and raw strings

```lume
puts "a\nb"       #=> a
                  #=> b
puts "tab\there"  #=> tab	here
puts "\u{48}i"    #=> Hi
```

A raw string, written `r"..."`, has no escapes and no interpolation. It is for
templates, regexes and code:

```lume
puts r"#{not interpolated}"   #=> #{not interpolated}
puts r"a\nb"                  #=> a\nb
```

## Triple-quoted strings

Inside a `"""` block a `"` needs no backslash: `say "hi"` is written as
it is. `#{..}` still works; a raw `r"""` block has neither.

A `"""` block **drops the opening line break and the common indentation of the
lines inside**, so the text lines up with the code around it and still comes
out flush left.

```lume
def main:
  name = "Ada"
  text = """
    Dear #{name},
      indented more
    Goodbye.
    """
  puts text        #=> Dear Ada,
                   #=>   indented more
                   #=> Goodbye.
```

The common indentation is four spaces here, so it is removed from every line;
the `indented more` line had six, so it keeps the extra two. Escapes and
`#{}` work exactly as in a one-line string.

`r"""..."""` is the raw form of the same thing — indentation is still
stripped, but nothing is escaped or interpolated:

```lume
def main:
  t = r"""
    #{not interpolated}
    a\nb
    """
  puts t    #=> #{not interpolated}
            #=> a\nb
```

**There is no trailing newline.** The closing `"""` on its own line ends the
text after the last content line:

```lume
def main:
  text = """
    one
    two
    """
  puts text.lines.len            #=> 2
  puts text.ends_with?("\n")     #=> false
```

## Everything `Str` has

Ask for a method that does not exist and the compiler lists the real ones:

```lume-bad
puts "abc".skip(1)
#! `Str` has: len, empty?, to_int, to_float, upcase, downcase, trim, lines, split, chars, contains?, starts_with?, ends_with?, pad, pad_right, reverse, slice, replace, repeat, capitalize, index_of, digit?, alpha?, space?, trim_left, trim_right, to_s, to_str
```

`Char` has a shorter list:

```lume-bad
c: Char = "a"
puts c.len
#! `Char` has: digit?, alpha?, space?, alnum?, upper?, lower?, upcase, downcase, code, pad, pad_right, to_s, to_str
```

Note what is **not** on the `Str` list: no `skip`, no `take`, no `map`, no
`each`. The list methods live on `s.chars`. And `index_of` gives an `Int?`,
while `to_int` and `to_float` give a `T or Error`:

```lume
puts "banana".index_of("n")    #=> Some(2)
puts "banana".index_of("z")    #=> None
puts "42".to_int               #=> Ok(42)
puts "x".to_int                #=> Error("`x` is not an integer")
```

The rest behave as their names suggest:

```lume
puts "[" + "  hi  ".trim + "]"        #=> [hi]
puts "[" + "  hi  ".trim_left + "]"   #=> [hi  ]
puts "[" + "  hi  ".trim_right + "]"  #=> [  hi]
puts "banana".replace("a", "o")       #=> bonono
puts "banana".contains?("nan")        #=> true
puts "banana".starts_with?("ban")     #=> true
puts "banana".ends_with?("na")        #=> true
puts "abc".reverse                    #=> cba
```

`pad` and `pad_right` are on the [printing](printing.md) page.

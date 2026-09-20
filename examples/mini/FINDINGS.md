# What writing Mini exposed

The log kept while writing this program, before any compiler change. Every
entry is something the program wanted to say and could not; all seven are
fixed, and the code in this directory is the version written afterwards.
1. No top-level constants (`KEYWORDS = [...]` at file scope). Second time this has come up (m19 item 2). Workaround: a `def keywords -> [Str]`, which rebuilds the list on every call. FIX: top-level `NAME = value` constants, evaluated once.
2. A bare variant name in a *value* position is not resolved by the expected type: in a `def ... -> Expr or Error`, writing `Num(v)` is "a variant of more than one enum: Expr, lexer.Tok", though patterns already resolve by scrutinee type. Workaround: `Expr.Num(v)` everywhere. FIX: when the wanted type is known (return, argument, typed binding, list element), prefer that enum; ambiguity error only if it is still ambiguous.
3. BIG: a nested pattern through a recursive enum field is refused ("`right` holds a `Expr` inside a `Expr`; give it a name and `match` it in the arm"). Matching nested tree shapes is exactly what an AST is for: `Binary("+", _, Binary("*", _, _, _), _)`. Workaround: a cascade of `match` statements. FIX: compile arms with nested boxed patterns into an if-let chain so the pattern can be written whole.
4. `return if c: a else: b` does not parse (a one-line `if` after `return`), though `x = if c: a else: b` does. Workaround: `v = if c: a else: b` then `return v`. FIX: allow an inline `if` as the value of `return`.
5. Inside a method, a bare call resolves to a top-level function of the same name instead of the type's own method: `def run(var self, s: Stmt)` in a file that also has `pub def run(src: Str)` — `run(s)` called the free function ("`last` is a `Value`, but this is a `[Str]`"). Workaround: rename the method. FIX: a method's own name wins inside the type (Ruby's rule); the free function is still reachable as a module call.
6. `return match x: ...` with one arm giving a value and another an `Error` is rejected, though the same `match` as the function's last expression is fine (the implied Ok/Error only applies at the tail). Workaround: restructure so the match is the tail. FIX: `return <expr>` is a tail position too.

(Program done: 4 files, ~700 lines, 14 tests, all passing. fib(21) through the
interpreter in 34 ms.)
7. No `Variant(..)` pattern to ignore the fields of a variant: with `Binary(op, left, right, line)` you must write `Binary(_, _, _, _)`. FIX: `..` inside a variant pattern.

All seven fixed:

1. top-level constants, `pub` across modules
2. a bare variant name resolved by the type the position expects
3. nested patterns through a recursive enum's fields (a `matches!` guard plus
   a destructuring bind, so `match` semantics are unchanged)
4. `return if c: a else: b`
5. a method's own name wins over a free function inside the type
6. `return <expr>` is a tail position, so implied `Ok`/`Error` applies
7. `Variant(..)`

Found while fixing, not in the log: the exhaustiveness checker printed five
witnesses for a recursive enum, which is an endless family — it now shows the
three simplest and says there are more.

# What writing the site generator exposed

The log kept while writing the program, before any compiler change. The
language could compute; it could hardly talk to the world.
1. No raw string: a template (or any text holding `#{...}`) is interpolated, so every placeholder must be escaped `\#{`. Workaround here: use `{{KEY}}` placeholders, which is better template style anyway — but embedding Lume/shell/regex source in a string still needs it. FIX: `r"..."` and `r"""..."""` raw strings.
2. No `Path` namespace: `join`, `dir`, `base`, `ext`, `stem`. Every path is string surgery.
3. No `Dir`: `exists?`, `make`, `list`, `walk`. A program cannot find its own input files.
4. No way to write to stderr. Diagnostics go to stdout and pollute the output.
5. No way to set an exit code. A CLI that fails cannot say so to the shell.
6. No `File.append`, `File.remove`, `File.size`, `File.modified` (for skipping unchanged files).
7. No way to read standard input, so a filter (`cat x.md | site --stdin`) cannot be written. FIX: `Env.stdin`.

(Program done: 2 files, 336 lines, 6 tests. Builds the 3-page sample site,
skips unchanged pages, renders stdin to stdout, exits 2 on a missing directory.)

## Added for it
- `warn x` — `puts`'s twin, to stderr
- `Env.exit(code)`, `Env.stdin`
- `Dir.exists?/make/list/walk/remove`
- `Path.join/dir/base/ext/stem`
- `File.append/remove/size/modified`

All seven fixed. The shape of the fix, in Lume's style, is a few small
namespaces rather than a module system for I/O:

```ruby
warn "no such directory: #{dir}"     # puts, but to stderr
Env.exit(2)                          # an exit code for the shell
Env.stdin?                           # everything on standard input
Dir.exists?(p) / make / list / walk / remove
Path.join(a, b) / dir / base / ext / stem
File.append / remove / size / modified
r"raw #{not interpolated}"           # and r"""blocks"""
```

One thing the fixes took away and gave back: `Dir` and `Path` were briefly
reserved names, which broke a corpus program using `enum Dir` for compass
directions. A user's type now wins over a built-in namespace of the same
name — the namespace only applies when nothing else claims the name.

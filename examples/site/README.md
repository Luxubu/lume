# The site generator

`markdown.lume` renders a small Markdown dialect; `main.lume` walks a content
directory, writes an HTML tree, and keeps an index. 336 lines, 6 tests.

```
lume run examples/site/main.lume                 build content/ into out/
lume run examples/site/main.lume -- --force      rebuild everything
lume run examples/site/main.lume -- a b          another content/out pair
cat page.md | lume run examples/site/main.lume -- --stdin     one page to stdout
```

It rebuilds only pages whose source is newer than their output, lists every
page in `out/all.html`, writes diagnostics to stderr, and exits 2 when the
content directory is missing — the ordinary manners of a command-line tool.

This was milestone 21's dogfood program: Lume could compute but barely talk
to the world. Everything it needed was added for it, and `FINDINGS.md` is
the log kept while writing it.

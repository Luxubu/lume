#!/usr/bin/env python3
"""Which of the compiler's error messages does some test show?

Every message the compiler can give is a template in its source
(`LumeError::new(.., format!("`{}` is ...", ..))` or a plain string). A
message is covered when some text a test checks matches it: an
`examples/errors/*.expected`, a `#!` line of a `lume-bad` example in `docs/`,
or a `.expected` anywhere under `examples/` or `corpus/`.

  tests/message_coverage.py            the count, and every message not shown
  tests/message_coverage.py --check N  fails when fewer than N are shown
"""
import glob, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)

def templates():
    out = []
    for f in ["compiler/src/codegen.rs", "compiler/src/parser.rs", "compiler/src/lexer.rs",
              "compiler/src/loader.rs", "compiler/src/manifest.rs", "compiler/src/fetch.rs", "compiler/src/main.rs"]:
        src = open(f).read()
        # the message argument of LumeError::new / .err(...): a string literal,
        # or the first literal of a format!
        for m in re.finditer(r'(?:LumeError::new\([^,]+,[^,]+,\s*|self\.err\(\s*)(format!\(\s*)?"((?:[^"\\]|\\.)*)"', src):
            t = m.group(2)
            if len(t) < 12:
                continue
            line = src.count("\n", 0, m.start()) + 1
            out.append((f"{os.path.basename(f)}:{line}", t))
    return out

def to_regex(t):
    t = t.replace('\\"', '"').replace("\\\\", "\\")
    parts = re.split(r"\{[^{}]*\}", t.replace("{{", "\x00").replace("}}", "\x01"))
    parts = [re.escape(p).replace("\x00", "{").replace("\x01", "}") for p in parts]
    # the article before a filled-in name is chosen afterwards: "an `Int`"
    parts = [re.sub(r"(^|\\ )a\\ `", r"\1an?\\ `", p) for p in parts]
    return re.compile(".+?".join(parts))

def shown():
    texts = []
    files = glob.glob("examples/**/*.expected", recursive=True) + glob.glob("corpus/**/*.expected", recursive=True) + glob.glob("tests/*.expected")
    # the package error cases keep theirs in a file named just `expected`
    files += glob.glob("examples/**/expected", recursive=True)
    for f in files:
        texts.append(open(f, errors="replace").read())
    for f in glob.glob("docs/**/*.md", recursive=True):
        for l in open(f).read().splitlines():
            if l.startswith("#!") or l.startswith("error:") or l.startswith("warning:"):
                texts.append(l)
    return "\n".join(texts)

def main():
    ts = templates()
    text = shown()
    covered, missing = 0, []
    seen = set()
    for where, t in ts:
        if t in seen:
            continue
        seen.add(t)
        if to_regex(t).search(text):
            covered += 1
        else:
            missing.append((where, t))
    total = covered + len(missing)
    if "--check" in sys.argv:
        need = int(sys.argv[sys.argv.index("--check") + 1])
        print(f"{covered} of {total} messages shown by a test (at least {need} wanted)")
        sys.exit(0 if covered >= need else 1)
    print(f"{covered} of {total} messages shown by a test")
    for where, t in missing:
        print(f"  {where}: {t}")

main()

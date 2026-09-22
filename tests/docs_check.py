#!/usr/bin/env python3
"""Runs every example in docs/ and checks it against what the docs claim.

See tests/docs.sh for the conventions. This exists so that an answer in the
documentation cannot quietly stop being true.
"""
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LUME = os.path.join(ROOT, "compiler", "target", "release", "lume")
FENCE = re.compile(r"^```(lume|lume-bad|lume-skip)\s*$")

verbose = "-v" in sys.argv


def blocks(path):
    """Yield (kind, start_line, source) for every fenced Lume block."""
    with open(path) as f:
        lines = f.read().split("\n")
    i = 0
    while i < len(lines):
        m = FENCE.match(lines[i])
        if not m:
            i += 1
            continue
        kind, start = m.group(1), i + 1
        i += 1
        body = []
        while i < len(lines) and lines[i].strip() != "```":
            body.append(lines[i])
            i += 1
        i += 1
        yield kind, start, body


def program(body):
    """A block is a whole program, or statements to wrap in `def main`."""
    text = "\n".join(body)
    if re.search(r"^(async )?def main\b", text, re.M):
        return text + "\n"
    indented = "\n".join(("  " + ln) if ln.strip() else "" for ln in body)
    return "def main:\n" + indented + "\n"


def expected_out(body):
    """The `#=>` claims, in order."""
    out = []
    for ln in body:
        m = re.search(r"#=>\s?(.*)$", ln)
        if m:
            out.append(m.group(1).rstrip())
    return out


def run(kind, src, want, wants_err):
    """Returns None when the example behaves as documented, else a reason."""
    with tempfile.TemporaryDirectory() as d:
        f = os.path.join(d, "doc.lume")
        with open(f, "w") as fh:
            fh.write(src)
        cmd = [LUME, "check" if kind == "lume-bad" else "run", f]
        try:
            p = subprocess.run(cmd, capture_output=True, text=True, timeout=180, cwd=ROOT)
        except subprocess.TimeoutExpired:
            return "timed out"

        if kind == "lume-bad":
            if p.returncode == 0:
                return "expected this to be rejected, but it was accepted"
            said = p.stdout + p.stderr
            for phrase in wants_err:
                if phrase not in said:
                    return "error did not mention %r; it said:\n%s" % (phrase, said.strip()[:400])
            return None

        if p.returncode != 0:
            return "did not run:\n%s" % (p.stdout + p.stderr).strip()[:600]
        got = [ln for ln in p.stdout.split("\n")]
        while got and got[-1] == "":
            got.pop()
        if got != want:
            return "output does not match the docs\n  docs say: %r\n  it says:  %r" % (want, got)
        return None


def main():
    if not os.path.exists(LUME):
        print("build the compiler first: cd compiler && cargo build --release")
        return 1
    docs = []
    for dirpath, _, names in os.walk(os.path.join(ROOT, "docs")):
        for n in sorted(names):
            if n.endswith(".md"):
                docs.append(os.path.join(dirpath, n))
    docs.sort()

    ok = 0
    fails = []
    for path in docs:
        rel = os.path.relpath(path, ROOT)
        for kind, line, body in blocks(path):
            if kind == "lume-skip":
                continue
            wants_err = [m.group(1).strip() for m in
                         (re.search(r"#!\s?(.*)$", ln) for ln in body) if m]
            src = program([ln for ln in body if "#!" not in ln]) if kind == "lume-bad" else program(body)
            why = run(kind, src, expected_out(body), wants_err)
            where = "%s:%d" % (rel, line)
            if why is None:
                ok += 1
                if verbose:
                    print("ok   %s" % where)
            else:
                fails.append((where, why))
                print("FAIL %s\n    %s" % (where, why.replace("\n", "\n    ")))

    print("%d examples ok, %d failed" % (ok, len(fails)))
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())

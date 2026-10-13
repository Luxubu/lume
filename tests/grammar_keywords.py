#!/usr/bin/env python3
"""Every keyword the compiler knows is one the VS Code grammar colours: a
keyword added to the lexer and not to the grammar (as `yield` was, in
milestone 70) shows here."""
import json, re, sys
src = open("compiler/src/lexer.rs").read()
block = src[src.index("pub const KEYWORDS"):]
block = block[:block.index("];")]
keywords = re.findall(r'"([a-z_]+)"', block)
grammar = json.dumps(json.load(open("editors/vscode/syntaxes/lume.tmLanguage.json")))
missing = [k for k in keywords if not re.search(r"\b" + re.escape(k) + r"\b", grammar)]
print("%d keywords, %s" % (len(keywords), "all in the grammar" if not missing else "missing from the grammar: " + ", ".join(missing)))
sys.exit(1 if missing else 0)

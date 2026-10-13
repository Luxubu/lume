# Lume for VS Code

Highlighting for `.lume` files, and Lume's own errors and warnings in the
editor as you type: the extension starts `lume lsp`, which checks the
program a file belongs to — the package's `main.lume`, the program whose
imports reach the file, or the file on its own — using the text you have
not saved yet.

## Install

The `lume` command must be on your `PATH`, or its full path set in the
setting `lume.path`. Then, from this folder:

    npm install
    npx @vscode/vsce package
    code --install-extension lume-0.3.0.vsix

To try it without installing anything into your own VS Code, run it from
this folder in a window of its own:

    code --extensionDevelopmentPath="$PWD" path/to/your/program

`tests/lsp_test.py` drives `lume lsp` with the messages VS Code sends —
opening and changing files, hover, go to definition and completion — and
`tests/grammar_keywords.py` checks that the highlighting knows every
keyword; both run in `tests/run.sh`.

Hover over a name to see its type or signature. Go to definition
(F12, or Cmd/Ctrl-click) jumps to where it was written, across modules and
packages. Completion offers a value's fields and methods after `.`, a
module's items after its name and `.`, and the names in scope elsewhere.

## What it does not do yet

Rename and find-all-references. The server answers those
requests with "not supported", and editors carry on.

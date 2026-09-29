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
    code --install-extension lume-0.1.0.vsix

Hover over a name to see its type or signature. Go to definition
(F12, or Cmd/Ctrl-click) jumps to where it was written, across modules and
packages.

## What it does not do yet

Completion, rename and find-all-references. The server answers those
requests with "not supported", and editors carry on.

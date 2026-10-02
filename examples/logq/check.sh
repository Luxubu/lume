#!/usr/bin/env bash
# logq on the sample log: every command, standard input, and its errors.
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")" || exit 1
run() { echo "\$ logq $*"; "$LUME" run main.lume -- "$@" 2>&1; echo "(exit $?)"; }
run summary data/access.log
run top 3 data/access.log
run errors data/access.log
run slow 100 data/access.log
run since 2025-09-30T10:01:00Z data/access.log
echo "\$ grep /api data/access.log | logq top 2 -"
grep /api data/access.log | "$LUME" run main.lume -- top 2 - 2>&1; echo "(exit $?)"
run top many data/access.log
run summary missing.log
run since yesterday data/access.log
run
rm -rf .lume

#!/usr/bin/env bash
# csvq, run the way a user runs it: from inside the package, no file named.
# Prints each command and what it printed, for tests/run.sh to compare.
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")/csvq" || exit 1
q() { echo "\$ csvq $*"; "$LUME" run -- "$@" 2>&1; echo "exit: $?"; }
q data/people.csv
q data/people.csv where age ">=" 70 sort age desc show name,age
q data/people.csv where city = Boston count
q data/people.csv where name has Turing
q data/people.csv sort score limit 3 show name,score
q data/people.csv sort city show city,name
q data/people.csv where city has o where age "<" 50
q data/people.csv where nope = 1
q data/people.csv where age "~" 3
q data/people.csv limit x
q data/people.csv sort
q no-such.csv
echo "\$ cat data/people.csv | csvq - where city = Boston show name"
cat data/people.csv | "$LUME" run -- - where city = Boston show name 2>&1; echo "exit: $?"
echo "\$ csvq"
"$LUME" run 2>&1; echo "exit: $?"
for p in csv table csvq; do echo "\$ (cd $p && lume test)"; (cd ../$p && "$LUME" test 2>&1); echo "exit: $?"; done
rm -rf .lume ../csv/.lume ../table/.lume

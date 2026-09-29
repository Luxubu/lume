#!/usr/bin/env bash
# jobr, run the way a user runs it, from inside the package. Prints each
# command and what it printed, for tests/run.sh to compare. The jobs sleep,
# so one line checks the timing: the two 0.3 s fetches run side by side.
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")/jobr" || exit 1
tmp=$(mktemp -d)
q() { echo "\$ jobr $*"; "$LUME" run -- "$@" 2>&1 | grep -v "^warning\|^  -->\|^ *[0-9]* |\|^ *|\|^  help"; echo "exit: ${PIPESTATUS[0]}"; }
q jobs.txt --dry
start=$(python3 -c 'import time; print(time.time())')
q jobs.txt
python3 -c "import time,sys; print('the fetches ran side by side:', time.time() - $start < 0.55)"
printf '[a]\nrun = true\nafter = b\n[b]\nrun = true\nafter = a\n' > "$tmp/cycle.txt"
q "$tmp/cycle.txt" 2>&1 | sed "s#$tmp#<tmp>#g"
printf '[a]\nrun = true\nafter = nope\n' > "$tmp/unknown.txt"
q "$tmp/unknown.txt" 2>&1 | sed "s#$tmp#<tmp>#g"
printf '[a]\nretries = two\nrun = true\n' > "$tmp/bad.txt"
q "$tmp/bad.txt" 2>&1 | sed "s#$tmp#<tmp>#g"
q no-such-jobs.txt
echo "\$ (cd jobfile && lume test)"; (cd ../jobfile && "$LUME" test 2>&1); echo "exit: $?"
rm -rf out .lume ../jobfile/.lume "$tmp"

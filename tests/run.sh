#!/usr/bin/env bash
# Lume test suite.
#
#   tests/run.sh            run every example and error case, diff against .expected
#   tests/run.sh --update   regenerate the .expected files from current output
#
# It also runs tests/docs.sh, so a claim in the documentation that stops being
# true fails the suite like any other expectation.
#
# examples/*.lume         must compile and run; stdout is compared
# examples/lib/           a library (seq, table) and a program that uses it
# examples/match/         a concurrent record matcher: fan out, collect, report
# examples/errors/*.lume  must fail `lume check`; stderr is compared
# examples/tests/*.lume   `lume test` output and exit code are compared
# examples/fmt/*.lume     `lume fmt --stdout` is compared; every example
#                         must also format idempotently without changing
#                         the generated Rust
# corpus/**/*.lume        programs written by an independent reviewer; output
#                         and exit code are compared where a .expected exists
#                         (a file without one is a known gap: see tests/corpus.sh)
#
# The suite is the contract: a milestone is done when this passes.

set -u
cd "$(dirname "$0")/.."
LUME=compiler/target/release/lume
# every program is compiled afresh: a warning is shown when a program is
# compiled, not when an earlier build is reused, so a build left behind by
# an earlier run would change what a test sees
find examples corpus -name .lume -type d -prune -exec rm -rf {} +
UPDATE=0
[ "${1:-}" = "--update" ] && UPDATE=1

if [ ! -x "$LUME" ]; then
  echo "building compiler..."
  (cd compiler && cargo build --release 2>&1 | grep -E "^error" -A 8) && true
fi

pass=0; fail=0; failed=()

check() {  # name, expected-file, actual-text
  local name=$1 exp=$2 actual=$3
  if [ $UPDATE = 1 ]; then
    # a text file ends with a newline, so re-running --update changes nothing
    printf '%s\n' "$actual" > "$exp"
    echo "  updated $exp"
    return
  fi
  if [ ! -f "$exp" ]; then
    echo "FAIL $name: no expected file $exp (run with --update)"
    fail=$((fail+1)); failed+=("$name"); return
  fi
  if [ "$actual" == "$(cat "$exp")" ]; then
    pass=$((pass+1))
  else
    echo "FAIL $name"
    diff <(printf '%s\n' "$(cat "$exp")") <(printf '%s\n' "$actual") | head -20 | sed 's/^/    /'
    fail=$((fail+1)); failed+=("$name")
  fi
}

for f in examples/*.lume examples/port/*.lume examples/modules/*.lume examples/json/json.lume examples/mini/main.lume examples/lib/report.lume examples/travel/main.lume examples/shipping/main.lume examples/match/main.lume examples/packages/shelf/main.lume; do
  name=$(basename "$(dirname "$f")")/$(basename "$f" .lume); name=${name#examples/}
  [ "$(dirname "$f")" = "examples" ] && name=$(basename "$f" .lume)
  out=$("$LUME" run "$f" 2>&1); code=$?
  if [ $code -ne 0 ]; then
    echo "FAIL $name: exit $code"; printf '%s\n' "$out" | head -15 | sed 's/^/    /'
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  check "$name" "${f%.lume}.expected" "$out"
done

for f in examples/errors/*.lume examples/errors/travel/main.lume examples/errors/reexport/main.lume; do
  name="errors/$(basename "$f" .lume)"
  case "$f" in examples/errors/travel/*) name="errors/travel" ;; esac
  case "$f" in examples/errors/reexport/*) name="errors/reexport" ;; esac
  out=$("$LUME" check "$f" 2>&1); code=$?
  if [ $code -eq 0 ]; then
    echo "FAIL $name: expected a compile error, but it compiled"
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  if printf '%s' "$out" | grep -q "generated Rust did not compile"; then
    echo "FAIL $name: rustc error leaked instead of a Lume error"
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  exp="examples/errors/$(basename "$f" .lume).expected"
  case "$f" in examples/errors/travel/*) exp="examples/errors/travel/main.expected" ;; esac
  case "$f" in examples/errors/reexport/*) exp="examples/errors/reexport/main.expected" ;; esac
  check "$name" "$exp" "$out"
done

# packages: each refusal is one folder, the program that trips it in `app/`
for d in examples/errors/packages/*/; do
  d=${d%/}; name="errors/packages/$(basename "$d")"
  out=$("$LUME" check "$d/app/main.lume" 2>&1); code=$?
  if [ $code -eq 0 ]; then
    echo "FAIL $name: expected a compile error, but it compiled"
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  check "$name" "$d/expected" "$out"
done

# a library's tests, run from inside the package with no file named
# (these `cd`, so the compiler is named by its full path)
LUME_ABS="$(cd "$(dirname "$LUME")" && pwd)/$(basename "$LUME")"
out=$(cd examples/packages/tally && "$LUME_ABS" test 2>&1); code=$?
check "packages/tally tests" "examples/packages/tally/test.expected" "$out
exit: $code"

# packages from git and `lume.lock`, against local repositories only
out=$(tests/git_packages.sh "$LUME" 2>&1)
check "packages from git" "tests/git_packages.expected" "$out"

# every error message the compiler gives is shown by some test; the number
# shown may grow and never shrink (tests/message_coverage.min holds it)
out=$(python3 tests/message_coverage.py --check "$(cat tests/message_coverage.min)" 2>&1)
if [ $? -eq 0 ]; then pass=$((pass+1)); else echo "FAIL message coverage: $out"; python3 tests/message_coverage.py | tail -n +2 | head -10 | sed 's/^/    /'; fail=$((fail+1)); failed+=("message coverage"); fi

# overflow, as Rust has it: checked by run, wrapped by build, checked by build --checked
tmpo=$(mktemp -d)
out=$("$LUME" run tests/overflow.lume 2>&1; echo "exit: $?"
  "$LUME" build tests/overflow.lume -o "$tmpo/fast" >/dev/null 2>&1 && "$tmpo/fast" 2>&1; echo "exit: $?"
  "$LUME" build tests/overflow.lume -o "$tmpo/safe" --checked >/dev/null 2>&1 && "$tmpo/safe" 2>&1; echo "exit: $?")
rm -rf "$tmpo" tests/.lume
check "overflow by build mode" "tests/overflow.expected" "$out"

# editors: diagnostics as JSON, and the language server driven as an editor would
out=$("$LUME" check examples/errors/arity.lume --json 2>&1; echo "exit: $?"; "$LUME" check examples/fib.lume --json 2>&1; echo "exit: $?")
check "check --json" "examples/check_json.expected" "$out"
out=$(python3 tests/lsp_test.py "$LUME" 2>&1)
check "lume lsp" "tests/lsp_test.expected" "$out"

# the json package: its own tests, and a program that depends on it by path
LUME_ABS2="$(cd "$(dirname "$LUME")" && pwd)/$(basename "$LUME")"
out=$(cd packages/json && "$LUME_ABS2" test 2>&1; echo "exit: $?")
check "json package tests" "packages/json/test.expected" "$out"
out=$(cd examples/json_config && "$LUME_ABS2" run 2>&1; echo "exit: $?")
check "json_config" "examples/json_config/main.expected" "$out"

# the http package against a server on this machine (skipped where a
# sandbox allows no local connection)
out=$(tests/http_local.sh "$LUME" 2>&1)
if [ "$out" = "skipped: no local connection allowed here" ]; then
  echo "  http: skipped, no local connection allowed here"; pass=$((pass+1))
else
  check "http package, local server" "tests/http_local.expected" "$out"
fi
out=$(cd packages/http && "$LUME_ABS2" test 2>&1 | grep -v "^warning\|^ *|\|^  -->\|^  help"; echo "exit: ${PIPESTATUS[0]}")
check "http package tests" "packages/http/test.expected" "$out"

# the healthcheck dogfood program, against a server on this machine
out=$(examples/healthcheck/check.sh "$LUME" 2>&1)
if [ "$out" = "skipped: no local connection allowed here" ]; then
  echo "  healthcheck: skipped, no local connection allowed here"; pass=$((pass+1))
else
  check "healthcheck" "examples/healthcheck/check.expected" "$out"
fi

# the jobr dogfood program: dry run, a real run with its timing, broken job files
out=$(examples/jobr/check.sh "$LUME" 2>&1)
check "jobr" "examples/jobr/queries.expected" "$out"

# the csvq dogfood program: queries, a pipe, its errors, and three packages' tests
out=$(examples/csvq/queries.sh "$LUME" 2>&1)
check "csvq" "examples/csvq/queries.expected" "$out"

# `lume new`, then run and test what it made, with no file named
tmp=$(mktemp -d)
out=$(cd "$tmp" && "$LUME_ABS" new hello 2>&1 && cd hello && "$LUME_ABS" run 2>&1 && "$LUME_ABS" new ../greet --lib 2>&1 && cd ../greet && "$LUME_ABS" test 2>&1); code=$?
rm -rf "$tmp"
check "lume new" "examples/packages/new.expected" "$out
exit: $code"

for f in examples/tests/*.lume; do
  name="tests/$(basename "$f" .lume)"
  out=$("$LUME" test "$f" 2>&1); code=$?
  check "$name" "${f%.lume}.expected" "$out
exit: $code"
done

# programs that keep their tests beside the code they test
out=$("$LUME" test examples/json/json.lume 2>&1); code=$?
check "json tests" "examples/json/json.test.expected" "$out
exit: $code"
out=$("$LUME" test examples/mini/main.lume 2>&1); code=$?
check "mini tests" "examples/mini/main.test.expected" "$out
exit: $code"
out=$("$LUME" test examples/site/main.lume 2>&1); code=$?
check "site tests" "examples/site/main.test.expected" "$out
exit: $code"
out=$("$LUME" test examples/lib/report.lume 2>&1); code=$?
check "lib tests" "examples/lib/report.test.expected" "$out
exit: $code"

# the site generator writes files: build the sample site from scratch
rm -rf examples/site/out
out=$("$LUME" run examples/site/main.lume -- --force 2>&1); code=$?
check "site build" "examples/site/main.expected" "$out
exit: $code"
out=$("$LUME" run examples/site/main.lume 2>&1); code=$?
check "site rebuild" "examples/site/rebuild.expected" "$out
exit: $code"
out=$(cat examples/site/content/guide.md | "$LUME" run examples/site/main.lume -- --stdin 2>&1); code=$?
check "site stdin" "examples/site/stdin.expected" "$out
exit: $code"
out=$("$LUME" run examples/site/main.lume -- nope out 2>&1); code=$?
check "site missing dir" "examples/site/missing.expected" "$out
exit: $code"
rm -rf examples/site/out

for f in examples/fmt/*.lume; do
  name="fmt/$(basename "$f" .lume)"
  out=$("$LUME" fmt "$f" --stdout 2>&1); code=$?
  if [ $code -ne 0 ]; then
    echo "FAIL $name: exit $code"; printf '%s\n' "$out" | head -15 | sed 's/^/    /'
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  check "$name" "${f%.lume}.expected" "$out"
done

# The review corpus: real-user programs. A leaked rustc error is never accepted.
for f in corpus/*.lume corpus/edge/*.lume corpus/m17/*.lume corpus/m18/*.lume corpus/m18/mod1/main.lume corpus/m26/*.lume corpus/m28/*.lume corpus/m30/*/main.lume corpus/m36/*.lume corpus/m36/*/main.lume corpus/m42/*.lume corpus/m42/*/main.lume corpus/m45/*.lume corpus/m45/*/main.lume corpus/m49/*/app/main.lume corpus/m53/*.lume corpus/m53/*/main.lume corpus/m59/*.lume corpus/m59/*/main.lume corpus/m63/*.lume corpus/m63/*/main.lume corpus/m69/*.lume; do
  name="corpus/${f#corpus/}"; name=${name%.lume}
  case "$f" in corpus/m30/*|corpus/m36/*/main.lume|corpus/m42/*/main.lume|corpus/m45/*/main.lume) name="${f%/main.lume}" ;; esac
  case "$f" in corpus/m49/*) name="${f%/app/main.lume}" ;; esac
  case "$f" in corpus/m53/*/main.lume) name="${f%/main.lume}" ;; esac
  case "$f" in corpus/m59/*/main.lume) name="${f%/main.lume}" ;; esac
  case "$f" in corpus/m63/*/main.lume) name="${f%/main.lume}" ;; esac
  exp="${f%.lume}.expected"
  out=$("$LUME" run "$f" 2>&1); code=$?
  if printf '%s' "$out" | grep -q "generated Rust did not compile"; then
    if [ -f "$exp" ]; then echo "FAIL $name: rustc error leaked"; fail=$((fail+1)); failed+=("$name"); fi
    continue
  fi
  if [ $UPDATE = 1 ] || [ -f "$exp" ]; then
    check "$name" "$exp" "$out
exit: $code"
  fi
done
rm -rf corpus/.lume corpus/edge/.lume corpus/m17/.lume corpus/m18/.lume corpus/m18/mod1/.lume corpus/m26/.lume corpus/m26/mod/.lume corpus/m28/.lume
rm -rf corpus/m30/*/.lume corpus/m30/*/*/.lume corpus/m36/.lume corpus/m36/*/.lume corpus/m36/*/*/.lume corpus/m42/.lume corpus/m42/*/.lume corpus/m42/*/*/.lume corpus/m45/.lume corpus/m45/*/.lume corpus/m45/*/*/.lume corpus/m49/*/*/.lume corpus/m53/.lume corpus/m53/*/.lume corpus/m53/*/*/.lume corpus/m53/*/*/*/.lume corpus/m59/.lume corpus/m59/*/.lume corpus/m59/*/*/.lume corpus/m69/.lume

# The formatter must be idempotent and must not change what a program means.
tmp=$(mktemp -d)
for f in examples/*.lume examples/port/*.lume examples/modules/*.lume examples/modules/users/*.lume examples/tests/*.lume examples/json/*.lume examples/mini/*.lume examples/site/*.lume examples/lib/*.lume examples/travel/*.lume examples/shipping/*.lume; do
  name="fmt-roundtrip/${f#examples/}"
  mkdir -p "$tmp/$(dirname "$f")"
  cp -r examples/modules "$tmp/examples/" 2>/dev/null
  # a program that imports a neighbour needs it beside the copy too
  cp examples/json/json.lume "$tmp/examples/json/" 2>/dev/null
  mkdir -p "$tmp/examples/mini" && cp examples/mini/*.lume "$tmp/examples/mini/" 2>/dev/null
  mkdir -p "$tmp/examples/site" && cp examples/site/*.lume "$tmp/examples/site/" 2>/dev/null
  mkdir -p "$tmp/examples/lib" && cp examples/lib/*.lume "$tmp/examples/lib/" 2>/dev/null
  mkdir -p "$tmp/examples/travel" && cp examples/travel/*.lume "$tmp/examples/travel/" 2>/dev/null
  mkdir -p "$tmp/examples/shipping" && cp examples/shipping/*.lume "$tmp/examples/shipping/" 2>/dev/null
  if ! "$LUME" fmt "$f" --stdout > "$tmp/$f" 2>"$tmp/err"; then
    echo "FAIL $name: fmt failed"; head -5 "$tmp/err" | sed 's/^/    /'
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  "$LUME" fmt "$tmp/$f" --stdout > "$tmp/second" 2>/dev/null
  if ! cmp -s "$tmp/$f" "$tmp/second"; then
    echo "FAIL $name: formatting twice differs from formatting once"
    diff "$tmp/$f" "$tmp/second" | head -10 | sed 's/^/    /'
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  a=$("$LUME" emit "$f" 2>/dev/null | grep -v '^// Generated')
  b=$("$LUME" emit "$tmp/$f" 2>/dev/null | grep -v '^// Generated')
  if [ "$a" != "$b" ]; then
    echo "FAIL $name: formatted program compiles to different Rust"
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  pass=$((pass+1))
done
rm -rf "$tmp"

rm -rf examples/.lume examples/errors/.lume examples/port/.lume examples/modules/.lume examples/tests/.lume examples/json/.lume examples/mini/.lume examples/site/.lume examples/lib/.lume
rm -rf examples/packages/*/.lume examples/errors/packages/*/*/.lume
if [ $UPDATE = 1 ]; then echo "expected files updated"; exit 0; fi

# Every example in docs/ is run and checked against what the docs claim.
doc_out=$(tests/docs.sh 2>&1); doc_code=$?
echo "$doc_out" | tail -1
if [ $doc_code -ne 0 ]; then
  echo "$doc_out" | grep -A2 '^FAIL'
  fail=$((fail+1)); failed+=("docs")
fi

echo "$pass passed, $fail failed"
[ $fail -eq 0 ] || { printf '  %s\n' "${failed[@]}"; exit 1; }

#!/usr/bin/env bash
# Lume test suite.
#
#   tests/run.sh            run every example and error case, diff against .expected
#   tests/run.sh --update   regenerate the .expected files from current output
#
# examples/*.lume         must compile and run; stdout is compared
# examples/lib/           a library (seq, table) and a program that uses it
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

for f in examples/*.lume examples/port/*.lume examples/modules/*.lume examples/json/json.lume examples/mini/main.lume examples/lib/report.lume; do
  name=$(basename "$(dirname "$f")")/$(basename "$f" .lume); name=${name#examples/}
  [ "$(dirname "$f")" = "examples" ] && name=$(basename "$f" .lume)
  out=$("$LUME" run "$f" 2>&1); code=$?
  if [ $code -ne 0 ]; then
    echo "FAIL $name: exit $code"; printf '%s\n' "$out" | head -15 | sed 's/^/    /'
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  check "$name" "${f%.lume}.expected" "$out"
done

for f in examples/errors/*.lume; do
  name="errors/$(basename "$f" .lume)"
  out=$("$LUME" check "$f" 2>&1); code=$?
  if [ $code -eq 0 ]; then
    echo "FAIL $name: expected a compile error, but it compiled"
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  if printf '%s' "$out" | grep -q "generated Rust did not compile"; then
    echo "FAIL $name: rustc error leaked instead of a Lume error"
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  check "$name" "examples/errors/$(basename "$f" .lume).expected" "$out"
done

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
for f in corpus/*.lume corpus/edge/*.lume corpus/m17/*.lume corpus/m18/*.lume corpus/m18/mod1/main.lume; do
  name="corpus/${f#corpus/}"; name=${name%.lume}
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
rm -rf corpus/.lume corpus/edge/.lume corpus/m17/.lume corpus/m18/.lume corpus/m18/mod1/.lume

# The formatter must be idempotent and must not change what a program means.
tmp=$(mktemp -d)
for f in examples/*.lume examples/port/*.lume examples/modules/*.lume examples/modules/users/*.lume examples/tests/*.lume examples/json/*.lume examples/mini/*.lume examples/site/*.lume examples/lib/*.lume; do
  name="fmt-roundtrip/${f#examples/}"
  mkdir -p "$tmp/$(dirname "$f")"
  cp -r examples/modules "$tmp/examples/" 2>/dev/null
  # a program that imports a neighbour needs it beside the copy too
  cp examples/json/json.lume "$tmp/examples/json/" 2>/dev/null
  mkdir -p "$tmp/examples/mini" && cp examples/mini/*.lume "$tmp/examples/mini/" 2>/dev/null
  mkdir -p "$tmp/examples/site" && cp examples/site/*.lume "$tmp/examples/site/" 2>/dev/null
  mkdir -p "$tmp/examples/lib" && cp examples/lib/*.lume "$tmp/examples/lib/" 2>/dev/null
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
if [ $UPDATE = 1 ]; then echo "expected files updated"; exit 0; fi
echo "$pass passed, $fail failed"
[ $fail -eq 0 ] || { printf '  %s\n' "${failed[@]}"; exit 1; }

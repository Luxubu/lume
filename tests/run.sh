#!/usr/bin/env bash
# Lume test suite.
#
#   tests/run.sh            run every example and error case, diff against .expected
#   tests/run.sh --update   regenerate the .expected files from current output
#
# examples/*.lume         must compile and run; stdout is compared
# examples/errors/*.lume  must fail `lume check`; stderr is compared
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
    printf '%s' "$actual" > "$exp"
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

for f in examples/*.lume; do
  name=$(basename "$f" .lume)
  out=$("$LUME" run "$f" 2>&1); code=$?
  if [ $code -ne 0 ]; then
    echo "FAIL $name: exit $code"; printf '%s\n' "$out" | head -15 | sed 's/^/    /'
    fail=$((fail+1)); failed+=("$name"); continue
  fi
  check "$name" "examples/$name.expected" "$out"
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

rm -rf examples/.lume examples/errors/.lume
if [ $UPDATE = 1 ]; then echo "expected files updated"; exit 0; fi
echo "$pass passed, $fail failed"
[ $fail -eq 0 ] || { printf '  %s\n' "${failed[@]}"; exit 1; }

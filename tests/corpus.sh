#!/usr/bin/env bash
# Runs every program in corpus/ and classifies the outcome:
#   ok      - compiled and ran (exit 0)
#   lume    - rejected with a Lume error (acceptable: the message is ours)
#   leak    - "the generated Rust did not compile" (a compiler gap)
#   crash   - built, then panicked at run time
# The bar for milestones 15-17: zero leaks, and no wrong output.
cd "$(dirname "$0")/.."
LUME=compiler/target/release/lume
ok=0; lume=0; leak=0; crash=0
for f in corpus/*.lume corpus/edge/*.lume corpus/m17/*.lume corpus/m18/*.lume corpus/m18/mod1/main.lume; do
  out=$(timeout 120 "$LUME" run "$f" 2>&1); code=$?
  if printf '%s' "$out" | grep -q "generated Rust did not compile"; then
    leak=$((leak+1)); echo "leak   $f"
  elif [ $code -eq 0 ]; then
    ok=$((ok+1)); [ "${1:-}" = "-v" ] && echo "ok     $f"
  elif printf '%s' "$out" | grep -q "^error:"; then
    lume=$((lume+1)); [ "${1:-}" = "-v" ] && echo "lume   $f: $(printf '%s' "$out" | grep -m1 '^error:')"
  else
    crash=$((crash+1)); echo "crash  $f: $(printf '%s' "$out" | grep -m1 -i 'panicked\|error' )"
  fi
done
rm -rf corpus/.lume corpus/edge/.lume corpus/m17/.lume corpus/m18/.lume corpus/m18/mod1/.lume
echo "ok $ok, lume error $lume, rustc leak $leak, crash $crash  (of $((ok+lume+leak+crash)))"

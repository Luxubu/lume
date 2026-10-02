#!/usr/bin/env bash
# Lume's benchmarks, each against the Rust a person would have written.
#
#   tests/bench.sh            build both sides, run each five times, report the best
#   tests/bench.sh --update   write the current ratios into bench/BASELINE
#
# Every benchmark prints its own timing as its last line, so what is measured
# is the work, not the process start or the file read. Both sides are built at
# full optimisation: `lume build`, not `lume run`, which trades the program's
# opt-level for compile speed.
#
# The guard is the ratio against Rust, not the millisecond count, so the suite
# says the same thing on a slower machine. A benchmark fails when it is more
# than 25% slower than the ratio in bench/BASELINE -- a real regression, with
# room for the noise a shared machine adds.

set -u
cd "$(dirname "$0")/.."
LUME=compiler/target/release/lume
UPDATE=0
[ "${1:-}" = "--update" ] && UPDATE=1
RUNS=5
SLACK=125   # percent of the baseline ratio that still passes

# name:lume source:rust source          (rust side may be empty: no counterpart)
BENCHES="
b1_generic_blocks:bench/b1_generic_blocks.lume:bench/b1.rs
b2_interface_values:bench/b2_interface_values.lume:bench/b2.rs
b3_wordcount:bench/b3_wordcount.lume:bench/b3b.rs
b4_text:bench/b4_text.lume:bench/b4.rs
b5_printing:bench/b5_printing.lume:bench/b5.rs
b6_iterators:bench/b6_iterators.lume:bench/b6.rs
b7_generators:bench/b7_generators.lume:bench/b6.rs
b3c_split_only:bench/b3c_split_only.lume:
"

[ -f bench/words.txt ] || python3 bench/gen.py
mkdir -p bench/.build

# The last line of a run is "<n> ms"; take the smallest of RUNS.
best() {
  local cmd="$1" lo=999999 n i
  for i in $(seq $RUNS); do
    n=$($cmd 2>&1 >/dev/null | tail -1 | tr -dc '0-9')
    [ -n "$n" ] || n=$($cmd 2>/dev/null | tail -1 | tr -dc '0-9')
    [ -n "$n" ] || return 1
    [ "$n" -lt "$lo" ] && lo=$n
  done
  echo "$lo"
}

# "name ratio" lines for --update; a plain list, since macOS ships bash 3.2,
# which has no associative arrays
RATIOS=""
fail=0
printf '%-22s %8s %8s %7s %8s\n' benchmark lume rust ratio baseline
for row in $BENCHES; do
  name=${row%%:*}; rest=${row#*:}
  src=${rest%%:*}; rs=${rest#*:}

  "$LUME" build "$src" -o "bench/.build/$name" >/dev/null 2>&1 || { echo "$name: lume build failed"; fail=1; continue; }
  lms=$(best "bench/.build/$name") || { echo "$name: no timing line"; fail=1; continue; }

  if [ -n "$rs" ]; then
    rustc -O "$rs" -o "bench/.build/${name}_rust" 2>/dev/null || { echo "$name: rustc failed"; fail=1; continue; }
    rms=$(best "bench/.build/${name}_rust") || { echo "$name: rust gave no timing"; fail=1; continue; }
    [ "$rms" -lt 1 ] && rms=1
    ratio=$(( lms * 100 / rms ))
  else
    rms=0; ratio=0
  fi
  [ "$ratio" -gt 0 ] && RATIOS="${RATIOS}${name} ${ratio}
"

  base=$(grep "^$name " bench/BASELINE 2>/dev/null | awk '{print $2}')
  if [ -n "${base:-}" ] && [ "$ratio" -gt 0 ]; then
    limit=$(( base * SLACK / 100 ))
    mark=""
    [ "$ratio" -gt "$limit" ] && { mark="  SLOWER than $base"; fail=1; }
    printf '%-22s %6sms %6sms %6s%% %7s%%%s\n' "$name" "$lms" "$rms" "$ratio" "$base" "$mark"
  elif [ "$ratio" -gt 0 ]; then
    printf '%-22s %6sms %6sms %6s%% %8s\n' "$name" "$lms" "$rms" "$ratio" "-"
  else
    printf '%-22s %6sms %8s %7s %8s\n' "$name" "$lms" "-" "-" "-"
  fi
done

if [ $UPDATE -eq 1 ]; then
  {
    echo "# lume time as a percentage of the same program in Rust, best of $RUNS."
    echo "# tests/bench.sh fails a benchmark that is more than $(( SLACK - 100 ))% over its number here."
    printf '%s' "$RATIOS" | sort
  } > bench/BASELINE
  echo "wrote bench/BASELINE"
  exit 0
fi

[ $fail -eq 0 ] && echo "all benchmarks within budget" || echo "a benchmark regressed"
exit $fail

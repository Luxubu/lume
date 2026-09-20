#!/usr/bin/env bash
# Best of N runs. No pipes on the program's stdout (a closed pipe aborts it).
N=${N:-10}
best() {
  local cmd=("$@") bp=99999 bs=99999 out p s
  for i in $(seq $N); do
    out=$("${cmd[@]}" 2>/dev/null)
    nums=($(echo "$out" | grep -o '[0-9]* ms' | grep -o '[0-9]*'))
    p=${nums[0]}; s=${nums[1]}
    [ -n "$p" ] && [ "$p" -lt "$bp" ] && bp=$p
    [ -n "$s" ] && [ "$s" -lt "$bs" ] && bs=$s
  done
  printf "%5s  %5s\n" "$bp" "$bs"
}
echo "               parse  print   (ms, best of $N, 867 KB / 11774 objects)"
echo -n "  lume         "; best ./bench_lume big.json
echo -n "  rust         "; best ./rsjson/target/release/rsjson big.json
echo -n "  rust ([Str]) "; best ./rsjson2/target/release/rsjson2 big.json
echo -n "  python       "; N=3 best python3 pyjson.py big.json

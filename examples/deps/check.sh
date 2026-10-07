#!/usr/bin/env bash
# deps on two graphs: one with no cycles, one with three; then its errors.
LUME="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
cd "$(dirname "$0")" || exit 1
run() { echo "\$ deps $*"; "$LUME" run main.lume -- "$@" 2>&1; echo "(exit $?)"; }
run tree shop data/shop.deps
run order data/shop.deps
run why shop text data/shop.deps
run why text shop data/shop.deps
run users db data/shop.deps
run cycles data/shop.deps
run tree app data/loop.deps
run order data/loop.deps
run cycles data/loop.deps
run users core data/loop.deps
run tree nope data/shop.deps
run order data/bad.deps
run order data/missing.deps
run frob data/shop.deps
run
rm -rf .lume

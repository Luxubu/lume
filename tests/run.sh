#!/usr/bin/env bash
# Lume test suite.
#
#   tests/run.sh            run every check; each is compared with its .expected
#   tests/run.sh --update   regenerate the .expected files from current output
#   tests/run.sh -j 4       run four checks at a time (eight by default)
#   tests/run.sh -x         stop at the first failure
#   tests/run.sh -v         name every check that passes, as it passes
#
# The suite itself is written in Lume: tools/lumecheck/plan.lume lists every
# check, and tools/lumecheck/suite.lume runs them, on several tasks at once;
# each failure is told as soon as it is known.
# What it checks:
#
# examples/*.lume         must compile and run; output is compared
# examples/errors/*.lume  must fail `lume check`, with the expected message
#                         and never a rustc error
# examples/tests/*.lume   `lume test` output and exit code are compared
# examples/fmt/*.lume     `lume fmt --stdout` is compared; every example
#                         must also format idempotently without changing
#                         the generated Rust
# corpus/**/*.lume        programs written by independent reviewers; output
#                         and exit code are compared where a .expected exists
# packages, dogfood programs, the language server, overflow modes, `lume new`
# and the message coverage, and every example in docs/ (tests/docs.sh)
#
# Checks that measure time run alone, after the others.
#
# The suite is the contract: a milestone is done when this passes.
set -u
cd "$(dirname "$0")/.."
exec tests/lumecheck.sh suite "$@"

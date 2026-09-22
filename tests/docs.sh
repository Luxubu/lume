#!/usr/bin/env bash
# Every runnable example in docs/ is compiled, run, and checked against the
# output the documentation claims for it.
#
#   tests/docs.sh          check every example
#   tests/docs.sh -v       name each one as it passes
#
# The convention, in a ```lume fenced block:
#
#     puts [1, 2, 3].join("-")    #=> 1-2-3
#
# Each `#=>` is one line of expected output, in order, and together they must
# be the whole output — an example cannot claim less than it prints. A block
# holding `def main` is a whole program; anything else is wrapped in one.
#
#   ```lume-bad    the example must be REJECTED; `#!` lines are text the
#                  error message must contain.
#   ```lume-skip   a fragment, shown but not run.
#
# Documentation that is not run is documentation that drifts. Six review
# rounds found 210 things people could not look up; this is what keeps the
# answers true.

set -u
cd "$(dirname "$0")/.."
exec python3 tests/docs_check.py "$@"

#!/usr/bin/env bash
# tests-pass — oracle for the tdd-ro5 pipeline's tests-pass gates.
# Exit 0 = the relevant test suite passes, non-zero = fail.
# Stderr carries the failure reason.
set -uo pipefail

# The pipeline's gate prompts don't pass an artifact path; the oracle
# validates the workspace state. `just test` is the repo's full suite
# gate (lib + integration + python).
if ! just test >/tmp/wai-oracle-tests-pass.log 2>&1; then
    echo "tests-pass oracle FAILED — just test is red; tail of output:" >&2
    tail -20 /tmp/wai-oracle-tests-pass.log >&2
    exit 1
fi
echo "tests-pass: just test green"
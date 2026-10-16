#!/usr/bin/env bash
# pretender-pin.sh — enforce the pinned pretender version (specodelic-6bq2).
#
# Pretender's metric counting drifts between releases: 0.7.0 reds files
# (parse_typing_table, cmd_model_check, graph build, kernel atomic, the lint
# family fns, section_sync check) that CI's pinned 0.6.0 passes, because
# pretender.toml's ratchet-max values were recorded under 0.6.0's counting.
# The version is therefore a PIN, not a floor — the pinned value lives HERE
# (this script is the single source of truth; tests/ci_wiring.rs
# pretender_version_is_pinned_and_single_sourced pins ci.yml and justfile
# to it).
#
# Usage: pretender-pin.sh [repo-dir]   (default: current directory)
# Exit 0 = local pretender matches the pin. Exit 1 = mismatch/missing, with
# a remediation hint. Designed to run before every `pretender check`:
# justfile pretender-check, lefthook pre-commit (pretender-pin command),
# and ci.yml's smoke step.

set -u

PIN="0.6.0"

repo="${1:-.}"

got="$(pretender --version 2>/dev/null | awk '{print $NF}')"

if [ -z "$got" ]; then
  echo "pretender-pin: pretender not found on PATH" >&2
  echo "pretender-pin: remediation: cargo install --locked pretender@$PIN" >&2
  exit 1
fi

if [ "$got" != "$PIN" ]; then
  echo "pretender-pin: VIOLATION: local pretender $got != pinned $PIN" >&2
  echo "pretender-pin: the ratchet-max values in $repo/pretender.toml were" >&2
  echo "pretender-pin: recorded under $PIN's metric counting — $got's counting" >&2
  echo "pretender-pin: differs and reports false reds (specodelic-6bq2)." >&2
  echo "pretender-pin: remediation: cargo install --locked pretender@$PIN" >&2
  echo "pretender-pin: (to move the pin itself: bump PIN here, the ci.yml" >&2
  echo "pretender-pin: install line, and the ci_wiring test together —" >&2
  echo "pretender-pin: re-record any moved ratchet-max entries under the new" >&2
  echo "pretender-pin: version's counting first)" >&2
  exit 1
fi

exit 0

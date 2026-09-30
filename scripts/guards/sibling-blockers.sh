#!/usr/bin/env bash
# sibling-blockers.sh — hook-enforced guard for the AGENTS.md sibling-tool
# constraints (hard blockers): pretender / espectacular / vampiro.
#
# Checks (each failure prints a remediation hint and exits 1):
#   1. core.hooksPath is owned by beads (.beads/hooks) — pretender MUST NOT
#      claim it (AGENTS.md "pretender" section).
#   2. .git/hooks contains only .sample files — no unguarded writes by any
#     tool that does not own hooks.
#   3. No espectacular/vampiro wiring in .github/workflows — both are blocked
#      from CI (specodelic-4ae pending; no vampiro integration ticket).
#
# Usage: sibling-blockers.sh [repo-dir]   (default: current directory)
# Exit 0 = compliant. Designed to run in pre-commit/pre-push (lefthook.yml)
# and `just ci` so the constraints hold locally and in CI.

set -u

repo="${1:-.}"
fail=0

# Checks 1–2 are machine-local hygiene (core.hooksPath lives in .git/config,
# which is never committed): a fresh CI clone can neither satisfy nor violate
# them, and bd is not installed in CI. In CI (env CI) only check 3 — the
# committed-artifact constraint — applies. Locally all three run.
in_ci="${CI:-}"

fail=0

die() {
  echo "sibling-blockers: VIOLATION: $1" >&2
  echo "sibling-blockers: remediation: $2" >&2
  fail=1
}

# --- Check 1: beads owns core.hooksPath (local only — see in_ci note) -------
if [ -z "$in_ci" ]; then
hooks_path="$(git -C "$repo" config --get core.hooksPath || true)"
case "$hooks_path" in
  .beads/hooks|*/.beads/hooks)
    : ;;
  "")
    die "core.hooksPath is unset — beads does not own the git hooks" \
      "run 'bd hooks install' so core.hooksPath points at .beads/hooks" ;;
  *)
    die "core.hooksPath is '$hooks_path' — beads (.beads/hooks) must own it; a sibling tool claimed the hooks" \
      "re-claim with 'bd hooks install'; sibling tools must chain to .beads/hooks, never replace it (marker-guarded claim-or-chain)" ;;
esac
fi

# --- Check 2: .git/hooks purity (local only — see in_ci note) ----------------
git_hooks="$repo/.git/hooks"
if [ -z "$in_ci" ] && [ -d "$git_hooks" ]; then
  for f in "$git_hooks"/*; do
    [ -e "$f" ] || continue
    case "$(basename "$f")" in
      *.sample) ;;
      *) die "unguarded write in .git/hooks: $(basename "$f")" \
           "remove it or install via the marker-guarded chain (bd hooks install); no tool may write .git/hooks directly" ;;
    esac
  done
fi

# --- Check 3: no espectacular/vampiro wiring in CI ----------------------------
# Prose mentions in YAML comments are fine (see publish.yml); only actual
# wiring (non-comment lines invoking the tool) is a violation.
workflows="$repo/.github/workflows"
if [ -d "$workflows" ]; then
  matches="$(grep -rilE 'espectacular|vampiro' "$workflows" 2>/dev/null | while IFS= read -r f; do
    sed 's/#.*$//' "$f" | grep -qiE 'espectacular|vampiro' && echo "$f"
  done || true)"
  if [ -n "$matches" ]; then
    die "blocked sibling tool wired into CI: $(echo "$matches" | tr '\n' ' ')" \
      "espectacular is blocked on specodelic-4ae (adoption decision note); vampiro needs a filed integration ticket first — remove the wiring"
  fi
fi

if [ "$fail" -ne 0 ]; then
  echo "sibling-blockers: pre-commit/pre-push blocked (see AGENTS.md sibling-tool constraints)" >&2
  exit 1
fi
exit 0

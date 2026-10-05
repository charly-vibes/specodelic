#!/usr/bin/env bash
# gate-drill.sh — run a gate drill safely under concurrent sessions
# (specodelic-do8; remediation (a) of specodelic-dz4, incident e7bb19e).
#
# A drill is any seeded-violation run that temporarily poisons repo state
# (ah check seeded-violation exercises, guard-espectacular poison files).
# Isolation rules:
#   1. Inside a LINKED git worktree the drill runs bare — the worktree IS
#      the isolation, no lock needed, the main tree cannot see the poison.
#   2. In the main tree the drill must hold the advisory gate-drill lock
#      (scripts/guards/gate-drill-lock.sh) for its whole lifetime, so the
#      hook-side `check` refuses concurrent commits/pushes while it runs.
#   3. If the lock is held by another live session, the drill REFUSES with
#      exit 2 and a remediation hint (the do8 meter).
#
# Usage: gate-drill.sh <cmd...>      (run from anywhere inside the repo)
# Exit: cmd's status on success paths; 2 on refusal (with hint).
# Env: GATE_DRILL_TTL / GATE_DRILL_NOTE pass through to the lock script.
#
# House rules: chains via lefthook.yml — never touches core.hooksPath,
# .beads/hooks shims, or .git/hooks (AGENTS.md sibling constraints).

set -u

if [ $# -lt 1 ]; then
  echo "gate-drill.sh: usage: gate-drill.sh <cmd...>" >&2
  echo "gate-drill.sh: remediation: wrap the drill command, e.g. gate-drill.sh -- ah check --run-tests" >&2
  exit 2
fi

toplevel="$(git rev-parse --show-toplevel 2>/dev/null)" || {
  echo "gate-drill.sh: not inside a git repository" >&2
  echo "gate-drill.sh: remediation: gate drills need a repo checkout or a linked worktree" >&2
  exit 2
}
git_dir="$(git rev-parse --git-dir)"
common_dir="$(git rev-parse --git-common-dir)"
# Normalize for comparison (git may print .git relative to the cwd).
git_dir="$(cd "$git_dir" && pwd)"
common_dir="$(cd "$common_dir" && pwd)"

if [ "$git_dir" != "$common_dir" ]; then
  # Linked worktree: isolation already satisfied; run bare.
  exec "$@"
fi

# In-tree: lock for the drill's lifetime.
lock_script="$(cd "$(dirname "$0")" && pwd)/gate-drill-lock.sh"
lock="$toplevel/.beads/gate-drill.lock"

if ! "$lock_script" acquire "$toplevel" "$$"; then
  # acquire already printed the refusal + hint and exited 2.
  exit 2
fi

cleanup() { "$lock_script" release "$toplevel" "$$" >/dev/null 2>&1 || true; }
trap cleanup EXIT

"$@"
status=$?
exit $status
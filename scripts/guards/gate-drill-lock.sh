#!/usr/bin/env bash
# gate-drill-lock.sh — advisory concurrency lock for gate drills
# (specodelic-do8; remediation (a) of the specodelic-dz4 concurrent-session
# race, incident e7bb19e; design call in
# openspec/decisions/2026-10-05-precommit-sweep-policy.md).
#
# A "gate drill" is any seeded-violation run that temporarily poisons repo
# state (ah check seeded-violation exercises, guard-espectacular poison
# files). A drill holds this lock while live; the hook-side `check` refuses
# commit/push while a LIVE lock is held, so a concurrent session cannot
# sweep the drill's in-flight poison into a commit. Stale locks never wedge
# commits: dead pid, unparsable owner, or a cross-host lock older than
# GATE_DRILL_TTL is a warning only.
#
# Modes:
#   check   [repo]              hook-side: exit 1 if a live lock is held,
#                               exit 0 otherwise (warning on stale)
#   acquire [repo] [owner-pid]  drill-side: create the lock atomically
#                               (mkdir); exit 2 if a live lock is held
#   release [repo] [owner-pid]  drill-side: remove the lock iff the owner
#                               pid matches (never removes a live foreign
#                               lock)
#   owner   [repo]              print the owner file, if any
#
# Lock layout: <repo>/.beads/gate-drill.lock/owner = "host|pid|epoch|note"
# (mkdir is the atomic acquisition primitive; no flock dependency).
#
# House rules (AGENTS.md): this guard chains via lefthook.yml — it never
# touches core.hooksPath, .beads/hooks shims, or .git/hooks. Violations
# print a remediation hint and exit nonzero.

set -u

mode="${1:-}"
repo="${2:-.}"
lock="$repo/.beads/gate-drill.lock"
owner="$lock/owner"
# Cross-host locks carry no meaningful pid; they are live only within TTL.
ttl="${GATE_DRILL_TTL:-3600}"

warn() { echo "gate-drill-lock: WARNING: $1" >&2; }
die() { # die <message> [code=1]
  echo "gate-drill-lock: $1" >&2
  exit "${2:-1}"
}

epoch_now() { date +%s; }

# read_owner — sets HOST, PID, EPOCH, NOTE; fails on a missing/unparsable file.
read_owner() {
  [ -r "$owner" ] || return 1
  local line
  line="$(cat "$owner" 2>/dev/null)" || return 1
  IFS='|' read -r HOST PID EPOCH NOTE <<EOF
$line
EOF
  case "${PID:-}" in ''|*[!0-9]*) return 1 ;; esac
  case "${EPOCH:-}" in ''|*[!0-9]*) return 1 ;; esac
  return 0
}

# is_live — 0 if the lock is held by a live drill, 1 if stale.
is_live() {
  if ! read_owner; then
    warn "unparsable owner file at $owner — treating as stale"
    return 1
  fi
  local myhost
  myhost="$(uname -n)"
  if [ "$HOST" = "$myhost" ] && kill -0 "$PID" 2>/dev/null; then
    return 0
  fi
  if [ "$HOST" != "$myhost" ]; then
    local age=$(( $(epoch_now) - EPOCH ))
    [ "$age" -lt "$ttl" ] && return 0
  fi
  return 1
}

lock_hint() {
  echo "gate-drill-lock: a gate drill holds $lock (pid $PID on $HOST, epoch $EPOCH, note: ${NOTE:-n/a})"
  echo "gate-drill-lock: remediation: wait for the drill to finish, or coordinate with the holding session;"
  echo "gate-drill-lock: if that session crashed, remove the stale lock: rm -rf $lock"
}

acquire_hint() {
  echo "gate-drill-lock: remediation: run the drill inside a linked git worktree instead (git worktree add),"
  echo "gate-drill-lock: or wait for the lock holder to finish; if it crashed, remove: rm -rf $lock"
}

case "$mode" in
  check)
    [ -d "$lock" ] || exit 0
    if is_live; then
      echo "gate-drill-lock: COMMIT/PUSH REFUSED: a gate drill is in progress" >&2
      lock_hint >&2
      exit 1
    fi
    warn "stale drill lock at $lock (dead pid / beyond TTL ${ttl}s) — not blocking; clean up with: rm -rf $lock"
    exit 0
    ;;

  acquire)
    owner_pid="${3:-$$}"
    myhost="$(uname -n)"
    mkdir -p "$(dirname "$lock")" 2>/dev/null || true
    if mkdir "$lock" 2>/dev/null; then
      printf '%s|%s|%s|%s\n' "$myhost" "$owner_pid" "$(epoch_now)" "${GATE_DRILL_NOTE:-gate drill}" > "$owner"
      exit 0
    fi
    if is_live; then
      echo "gate-drill-lock: DRILL REFUSED: lock already held by a live session" >&2
      lock_hint >&2
      acquire_hint >&2
      exit 2
    fi
    warn "removing stale drill lock to acquire"
    rm -rf "$lock"
    if mkdir "$lock" 2>/dev/null; then
      printf '%s|%s|%s|%s\n' "$myhost" "$owner_pid" "$(epoch_now)" "${GATE_DRILL_NOTE:-gate drill}" > "$owner"
      exit 0
    fi
    echo "gate-drill-lock: DRILL REFUSED: could not acquire lock after stale cleanup" >&2
    acquire_hint >&2
    exit 2
    ;;

  release)
    owner_pid="${3:-}"
    if ! read_owner; then
      exit 0 # nothing to release
    fi
    if [ -n "$owner_pid" ] && [ "$PID" != "$owner_pid" ]; then
      warn "not releasing: lock is owned by pid $PID, not $owner_pid"
      exit 0
    fi
    rm -rf "$lock"
    exit 0
    ;;

  owner)
    if [ -r "$owner" ]; then cat "$owner"; fi
    exit 0
    ;;

  *)
    echo "usage: gate-drill-lock.sh {check|acquire|release|owner} [repo-dir] [owner-pid]" >&2
    echo "  env: GATE_DRILL_TTL (seconds; default 3600), GATE_DRILL_NOTE" >&2
    exit 1
    ;;
esac
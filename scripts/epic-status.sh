#!/usr/bin/env bash
# epic-status.sh — one-command snapshot of autonomous-orchestration state.
#
# Answers "where is the epic right now?" from the three sources that drift:
#   1. .wai/pipeline-runs/*.yml  — pipeline runs and their current step
#   2. bd                        — in-progress tickets
#   3. git                       — unpushed commits (orchestrator didn't push?)
# plus discoverable named subagent sessions for cost/usage auditing.
#
# Usage: scripts/epic-status.sh [repo-root]   (default: this repo)
# Exit 0 always — this is a status surface, not a gate.

set -u
root="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
runs_dir="$root/.wai/pipeline-runs"

echo "== Pipeline runs =="
if [ -d "$runs_dir" ] && ls "$runs_dir"/*.yml >/dev/null 2>&1; then
    # tdd-ro5 has 9 steps; per-pipeline step counts come from the TOMLs
    for f in "$runs_dir"/*.yml; do
        run_id=$(grep -m1 '^run_id:' "$f" | cut -d' ' -f2-)
        topic=$(grep -m1 '^topic:' "$f" | cut -d"'" -f2)
        step=$(grep -m1 '^current_step:' "$f" | grep -oE '[0-9]+$')
        pipe=$(grep -m1 '^pipeline:' "$f" | cut -d' ' -f2-)
        nsteps=$(grep -c '^\[\[steps\]\]' \
                 "$root/.wai/resources/pipelines/$pipe.toml" 2>/dev/null)
        [ -n "$nsteps" ] || nsteps='?'
        printf '  [%s/%s] %s\n    %s\n' "${step:-?}" "$nsteps" "$run_id" "$topic"
        if [ "${step:-0}" != "${nsteps:-x}" ]; then
            echo "    ⚠ run not at final step — hand-bumped or abandoned?"
        fi
    done
else
    echo "  (no runs)"
fi

echo "== Beads in-progress =="
bd list --status in_progress 2>/dev/null | sed 's/^/  /' | head -8 || echo "  (bd unavailable)"

echo "== Unpushed commits =="
if unpushed=$(git -C "$root" log --oneline '@{u}..HEAD' 2>/dev/null); then
    if [ -n "$unpushed" ]; then
        echo "$unpushed" | sed 's/^/  /'
        n=$(echo "$unpushed" | wc -l)
        [ "$n" -gt 10 ] && echo "  … and $((n - 10)) more"
    else
        echo "  (up to date with origin)"
    fi
else
    echo "  (no upstream configured)"
fi

echo "== Subagent sessions (usage audit) =="
# pi session-dir naming: cwd with leading '/' stripped, '/'→'-', wrapped '--…--'
# (verified against ~/.pi/agent/sessions contents, 2026-10-06)
key=$(printf '%s' "$root" | sed -e 's|^/||' -e 's|/$||' -e 's|/|-|g')
sess_dir="$HOME/.pi/agent/sessions/--${key}--"
if [ -d "$sess_dir" ]; then
    # Subagent runs = first user message starts with the brief header '# Task:'.
    # Attributed = the session carries a `-n "subagent:..."` name record.
    sub=0; unnamed=0
    for s in "$sess_dir"/*.jsonl; do
        first_task=$(grep -m1 -o '"role":"user"' "$s" 2>/dev/null)
        [ -n "$first_task" ] || continue
        head_txt=$(head -c 20000 "$s" | tr ',' '\n' | grep -A0 'You are a subagent' )
        if [ -z "$head_txt" ]; then
            # fall back: brief header on first message line
            grep -q '^# Task:' <(jq -r 'select(.type=="message") | select(.message.role=="user") | (.message.content | if type=="string" then . else ([.[] | select(.type=="text") | .text] | join("\\n")) end)' "$s" 2>/dev/null | head -1) || continue
        fi
        sub=$((sub + 1))
        if ! grep -q '"name":"subagent:' "$s"; then
            unnamed=$((unnamed + 1))
            echo "  ⚠ unattributed subagent run (no -n name; usage orphaned): $(basename "$s")"
        fi
    done
    echo "  $sub subagent runs, $unnamed unattributed / $(ls "$sess_dir"/*.jsonl 2>/dev/null | wc -l) sessions"
    if [ "$sub" -eq 0 ]; then echo "  ($sess_dir)"; fi
else
    echo "  (no session dir found: $sess_dir)"
fi

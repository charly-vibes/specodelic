#!/usr/bin/env bash
# validate-overlay.sh — applies the bundle's overlay to a clean copy of a
# specodelic checkout and runs the repo's own proposal gates that work offline,
# including an `openspec archive` dry run on a throwaway copy.
#
# Usage: SPK=/path/to/spk REPO=/path/to/checkout OPENSPEC=/path/to/openspec-0.19.0 \
#        OVERLAY=/path/to/repo-overlay [REBASE=/path/to/repo-overlay-main] \
#        bash validate-overlay.sh
#
#   OPENSPEC  must be @fission-ai/openspec@0.19.0, the version the repo's CI pins
#             (.github/workflows/ci.yml). Other versions parse dual-format deltas
#             differently (1.14.1 warns even on the repo's own add-conform).
#   REBASE    optional: use the main-branch variant of the `packs` delta and
#             apply its table patch after archiving (needed on upstream main).
set -u
: "${SPK:?}"; : "${REPO:?}"; : "${OPENSPEC:?}"; : "${OVERLAY:?}"
export OPENSPEC_TELEMETRY=0 DO_NOT_TRACK=1
W="$(mktemp -d)"
fresh() { rm -rf "$W/repo"; cp -r "$REPO" "$W/repo"; rm -rf "$W/repo/.git"; cp -r "$OVERLAY"/. "$W/repo/"
          if [ -n "${REBASE:-}" ]; then cp "$REBASE/openspec/changes/add-workspace-scope/specs/packs/spec.md" "$W/repo/openspec/changes/add-workspace-scope/specs/packs/spec.md"; fi; cd "$W/repo" || exit 2; }
echo "openspec: $("$OPENSPEC" --version)   spk: $("$SPK" --version)   rebase-variant: ${REBASE:+yes}${REBASE:-no}"
fresh
echo; echo "== V1 openspec validate --strict, each new change"
for c in add-svc-contracts-pack add-workspace-scope; do "$OPENSPEC" validate "$c" --strict --no-interactive 2>&1 | head -3; done
echo; echo "== V2 openspec validate --all --strict (the repo's openspec-validate recipe)"
"$OPENSPEC" validate --all --strict --no-interactive > "$W/all.txt" 2>&1; echo "exit=$?"; tail -1 "$W/all.txt"
echo; echo "== V3 spk lint openspec (the repo's lint-deltas recipe): baseline vs with overlay (both copies without .git, so pack discovery is the same)"
rm -rf "$W/base"; cp -r "$REPO" "$W/base"; rm -rf "$W/base/.git"
"$SPK" lint "$W/base/openspec" --human 2>&1 | head -1 | cut -c1-120
"$SPK" lint openspec --human 2>&1 | head -1 | cut -c1-120
echo; echo "== V4 section sync (the repo's sync-sections recipe)"
python3 scripts/check_section_sync.py openspec; echo "exit=$?"
echo; echo "== V5 summary completeness"
python3 scripts/check_summary_completeness.py . 2>&1 | tail -1
echo; echo "== V6 packs directory lints clean with the draft pack"
"$SPK" lint packs --human 2>&1 | head -1 | cut -c1-120
for c in add-svc-contracts-pack add-workspace-scope; do
  fresh; echo; echo "== V7 openspec archive $c (throwaway copy)"
  "$OPENSPEC" archive "$c" --yes > "$W/arch.txt" 2>&1; echo "exit=$?"
  grep -E "Applying|modified|added|failed|archived" "$W/arch.txt" | cut -c1-200
done
if [ -n "${REBASE:-}" ]; then
  echo; echo "== V8 apply packs-spec-tables.patch, then lint (end state on upstream main)"
  patch -p1 -s < "$REBASE/packs-spec-tables.patch"; echo "patch exit=$?"
  "$SPK" lint openspec --human 2>&1 | head -1 | cut -c1-120
fi
echo; echo "(not run: cargo build/test, just ci, pretender, guards — no Rust toolchain in this environment)"

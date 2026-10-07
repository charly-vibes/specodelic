# specodelic justfile — local/CI workflow for the specodelic CLI
#
# Run `just` for default (build + test), `just ci` for the full gate.
# The specs/ directory is the dogfood corpus: `just lint` runs specodelic on the
# format's own spec files.

set shell := ["bash", "-uc"]

# Default: build and test
default: build test

# === Build Commands ===

build:
    cargo build

build-release:
    cargo build --release

install:
    cargo install --path .

# === Test Commands ===

test:
    cargo test

test-name name:
    cargo test {{name}} -- --nocapture

# === Lint Commands ===

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

# === Spec Commands ===

# Lint the corpus with the freshly built specodelic (coverage gaps tracked in beads)
lint-specs:
    cargo run -q -- lint specs

# Derive the reference graph
graph-specs:
    cargo run -q -- graph specs

# Validate every active change and every capability spec (openspec, strict,
# non-interactive — CI safe)
openspec-validate:
    openspec validate --all --strict --no-interactive

# Lint the openspec tree with the freshly built spk: dual-format files
# (frontmatter + tables) are checked; plain prose files are exempt
# (no frontmatter — not a spec file)
lint-deltas:
    cargo run -q -- lint openspec

# Corpus-wide model-check gate (specodelic-12e): the model checker must be
# exercised on the corpus the tool ships with — its artifacts were once
# re-keyed by hand without a re-run, and just ci stayed green because no
# stage ran model-check at all. cargo run (not PATH spk) so the gate can
# never hit a stale installed binary. Fails the gate on any non-clean
# outcome (nonzero exit).
model-check-specs:
    cargo run -q -- model-check specs

# Epic status snapshot for autonomous orchestration (specodelic-k9v): pipeline
# run states, in-progress tickets, unpushed commits, subagent session audit
epic-status:
    bash scripts/epic-status.sh

# Section-sync check: in every dual-format file the ADDED Requirements
# and Requirements sections must carry identical requirement text
sync-sections:
    python3 scripts/check_section_sync.py openspec

# Lint fenced spec examples embedded in the corpus docs (USAGE.md's
# quick-start blocks are runnable format artifacts — they drifted once,
# specodelic-vpx; this gate keeps every frontmatter example lint-clean)
lint-doc-examples:
    python3 scripts/check_doc_examples.py specs

# Honest dogfood gate: corpus findings vs the shrink-only baseline
# (specs/.lint-baseline). Baseline is empty today — the corpus lints
# fully clean — so this is strict zero-findings until an entry is added,
# and an entry may only ever be REMOVED, never added.
lint-baseline:
    python3 scripts/check_lint_baseline.py specs

# SUMMARY completeness: every frontmatter-bearing specs/*.md and every
# openspec/specs/<cap>/spec.md must be linked from docs/src/SUMMARY.md
# (hand-written SUMMARY can silently miss new spec files, specodelic-b3p)
summary-completeness:
    python3 scripts/check_summary_completeness.py .

# Tests for the section-sync script itself (stdlib unittest): drift
# logic + the capability-format check (openspec/specs/<cap>/spec.md
# must be dual-format — frontmatter-less capability specs fail CI;
# spk lint cannot see them, they parse-skip)
sync-sections-test:
    python3 -m unittest discover -s scripts -p 'test_*.py'

# Archive a change bypassing the lossy spec regeneration, then copy each
# dual-format delta verbatim into openspec/specs/ (the specodelic layer
# survives into engineering truth)
archive-change id:
    #!/usr/bin/env bash
    set -euo pipefail
    # Delegates to the tool-level companion (specodelic-fzo / GH#7):
    # openspec archive --skip-specs + verbatim deploy of the archived
    # deltas, fail-closed on any delta lacking the dual-format layer.
    # cargo run (not PATH spk) so the recipe can never hit a stale
    # installed binary lacking the subcommand.
    cargo run -q -- archive-companion "{{id}}"

# Build the docs book locally, mirroring the docs.yml workflow steps
# (specodelic-2m7: the shared scripts/stamp_llms.py keeps both paths in
# lockstep — release page before the build, stamped llms.txt after)
docs-build:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf docs/src/specs docs/src/openspec
    cp -r specs docs/src/specs
    cp -r openspec/specs docs/src/openspec
    python3 scripts/stamp_llms.py page
    mdbook build
    python3 scripts/stamp_llms.py llms
    echo "docs built: book/index.html"

# === CI Pipeline ===

ci: fmt-check lint test build-release openspec-validate lint-deltas model-check-specs sync-sections sync-sections-test summary-completeness lint-doc-examples lint-baseline pretender-check guard-kernel-agreement guard-siblings guard-drill-lock guard-espectacular

# Structural-quality hard gate (pretender, gate mode — pretender.toml
# thresholds are a ratchet: entries only move DOWN; never touch .git/hooks,
# the gate runs here and via lefthook pre-commit instead)
pretender-check:
    pretender check

# Backend-agreement property (add-min-expr-kernel D2, specodelic-36n): the
# kernel fixture corpus's per-cell three-valued statuses are checked on
# every CI run. Rust-only until l8l — the py backend seat in the harness
# reports labeled Unsupported; the cross-backend assertion activates when
# l8l's emitter lands and is never commented out.
guard-kernel-agreement:
    cargo test --test kernel_agreement

# Sibling-tool constraint guard (AGENTS.md hard blockers) — also wired into
# pre-commit/pre-push via lefthook.yml and the .beads/hooks shim chain
guard-siblings:
    scripts/guards/sibling-blockers.sh .

# Gate-drill advisory lock guard (specodelic-do8, dz4 remediation-b): fails
# (exit 1) if a LIVE gate-drill lock is held — a drill's seeded-violation
# state must never be swept into a concurrent session's commit. Stale locks
# warn only. Wired into pre-commit AND pre-push via lefthook.yml; drills
# should be run through scripts/guards/gate-drill.sh (worktree or lock).
guard-drill-lock:
    scripts/guards/gate-drill-lock.sh check .

# Espectacular scenario-conformance gate (specodelic-c0o): every deployed
# openspec #### Scenario: block must resolve to a contract TOML in
# .espectacular/ whose bound test actually runs and passes. Espectacular is
# READ-ONLY over openspec/ — findings never drive spec edits. Pre-commit
# runs the structural-only half (ah check in lefthook.yml, EDGE-003: no
# test execution at commit time); this recipe adds the executed half.
guard-espectacular:
    ah check --run-tests

# Session start
prime:
    bd ready

# Publish to crates.io (run after `just ci` passes)
publish:
    cargo publish
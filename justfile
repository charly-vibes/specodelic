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

# Smart test run (default for agents / iterative red→green loops):
# testaruda computes the affected test set from the working diff via
# provenance analysis, runs only those tests, and ingests the results.
# Falls back to a plain `cargo test` if the store/config is missing or
# confidence is low (--safe). The full `cargo test` remains the CI gate
# (`just ci`); this recipe is for fast feedback, not the gate.
test-smart:
    testaruda select --safe --human

# Blast-radius preview: list the tests a change WOULD affect without
# running anything. Useful before a refactor; pass an explicit file
# list: just test-blast src/foo.rs,tests/foo.rs
test-blast *files:
    #!/usr/bin/env bash
    if [ -z "{{files}}" ]; then
        testaruda select --pre-edit --human
    else
        testaruda select --pre-edit --human --files "{{files}}"
    fi

# === Testaruda observability ===

# Usage counters: tests tracked, runs ingested, quarantined (flaky) tests
test-metrics:
    testaruda metrics --human

# Cross-tool health summary (also covers other .genesis/tools.toml tools)
test-status:
    testaruda status --human

# Calibration gate: does the predictive ranking actually recall failures?
# NOTE: reports NOT CALIBRATED while ingested runs contain zero failures
# (recall 0/0 degenerates to 0.0) — that is a no-signal state, not a
# regression. It becomes meaningful once a real failure has been ingested.
test-calibrate:
    testaruda calibrate --human

# Cross-validate selection semantics against the Soufflé Datalog oracle
test-oracle:
    testaruda oracle --human

# Why did (or didn't) a test get selected? Pass a test node_id or numeric
# ID — node_ids are store-relative, e.g. src::lint::anchors_resolve_but_name_no_row(Test)
# (not the full crate path cargo prints). List IDs: sqlite3 .testaruda/store.db \
#   "select id, node_id from test_items;"
test-why test_id:
    testaruda explain --human "{{test_id}}"

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

# Docs-accuracy gate (specodelic-lf4b.10): re-run the quickstart commands
# whose output is captured verbatim in docs/src/installation.md and
# docs/src/examples/worked-example.md, and diff the real output against
# the fenced blocks in the docs. Captured output rots as the tool evolves
# (the 7-vs-9 explain-topics drift is the existence proof); this fails on
# drift naming the doc and command. Scenario inputs live under
# docs/fixtures/doc-outputs (inputs only — expected outputs are parsed
# live from the docs, never duplicated). Never hand-edit an expected
# block to go green: re-capture from the real binary instead.
doc-examples:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -q
    python3 scripts/check_quickstart_outputs.py --spk target/debug/spk

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

# Explain-topic doc guard: README.md, docs/src/index.md and
# docs/src/installation.md must agree with the served `spk explain`
# TOPICS count parsed from src/guide.rs (README said six, index said
# eight, installation said seven — against a binary serving nine;
# specodelic-lf4b.1). The stdlib unittest for the guard needs no
# separate recipe: sync-sections-test below discovers all
# scripts/test_*.py, including test_check_explain_topic_docs.py.
check-explain-topic-docs:
    python3 scripts/check_explain_topic_docs.py .

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

# Derived graph views (add-graph-views D6, tasks 3.1–3.3): regenerate the
# Mermaid includes in docs/src/views/ from this repo's own corpus. The
# views are build-time artifacts — gitignored, never committed — so
# graph_is_derived_not_authored holds by construction (no hand-edit path,
# no staleness machinery). All envelopes come from the same freshly built
# binary (cargo run, never a PATH spk); the graph and schema envelopes
# feed scripts/graph_views.py; intermediate exports stay in target/
# (gitignored). Deliberately NOT in `just ci` (task 3.3): rendering is
# build-time only, and an artifact that is never stored cannot go stale.
docs-graphs:
    #!/usr/bin/env bash
    set -euo pipefail
    SCRATCH=target/docs-graphs
    OUT=docs/src/views
    rm -rf "$SCRATCH"
    mkdir -p "$SCRATCH" "$OUT"
    # Same binary, one build: the graph and schema envelopes (task 3.1)
    cargo run -q -- graph specs --format edges > "$SCRATCH/edges.tsv"
    cargo run -q -- graph specs --json > "$SCRATCH/graph.json"
    cargo run -q -- lint specs --json > "$SCRATCH/lint.json"
    cargo run -q -- guide --schema --json > "$SCRATCH/schema.json"
    # Derived views → docs/src/views/ (gitignored, D6)
    python3 scripts/graph_views.py schema "$SCRATCH/schema.json" \
        > "$OUT/schema.md"
    python3 scripts/graph_views.py states "$SCRATCH/edges.tsv" \
        --graph "$SCRATCH/graph.json" --lint "$SCRATCH/lint.json" \
        > "$OUT/states.md"
    python3 scripts/graph_views.py trace "$SCRATCH/edges.tsv" \
        --graph "$SCRATCH/graph.json" --lint "$SCRATCH/lint.json" \
        > "$OUT/trace.md"
    cargo run -q -- graph specs --view wiring --format mermaid \
        > "$OUT/wiring.md"
    # Clean-tree assertion (task 3.1): after a full build, git status
    # scoped to the output dir must stay empty — the views are gitignored
    # (D6), so any entry here is a leak into the tracked tree.
    if [ -n "$(git status --porcelain --untracked-files=all -- "$OUT")" ]; then
        echo "docs-graphs: generated views leaked into git status:" >&2
        git status --porcelain --untracked-files=all -- "$OUT" >&2
        exit 1
    fi
    echo "graph views regenerated: $OUT/{schema,states,trace,wiring}.md"

# Build the docs book locally, mirroring the docs.yml workflow steps
# (specodelic-2m7: the shared scripts/stamp_llms.py keeps both paths in
# lockstep — release page before the build, stamped llms.txt after).
# Depends on docs-graphs (add-graph-views task 3.3): the generated views
# must exist before mdbook resolves the {{#include}} directives.
# Fetch the vendored mermaid bundle if missing (see .gitignore for why it
# is not tracked). Version pinned to what mdbook-mermaid 0.17.1 bundles.
# Runs in docs-build AND docs.yml (CI) — keep both paths in lockstep.
[private]
docs-mermaid:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f vendor/mermaid.min.js ]; then
        mkdir -p vendor
        curl -fsSL -o vendor/mermaid.min.js \
            https://cdn.jsdelivr.net/npm/mermaid@11.6.0/dist/mermaid.min.js
        echo "fetched vendor/mermaid.min.js (mermaid 11.6.0)"
    fi

docs-build: docs-graphs docs-mermaid
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf docs/src/specs docs/src/openspec
    cp -r specs docs/src/specs
    cp -r openspec/specs docs/src/openspec
    python3 scripts/stamp_llms.py page
    mdbook build
    # Unresolved-include gate: mdbook only WARNS on a missing include
    # directive and emits the raw text — a green build can still ship a
    # page with unrendered views. Fail loudly instead (docs.yml runs the
    # same assertion after mdbook build — keep both in lockstep). The
    # {{'{{'}} escapes just's interpolation.
    if grep -rq "{{'{{'}}#include" book/; then
        echo "docs-build: unresolved include directive leaked into the built book:" >&2
        grep -rl "{{'{{'}}#include" book/ >&2
        exit 1
    fi
    python3 scripts/stamp_llms.py llms
    echo "docs built: book/index.html"

# === CI Pipeline ===

ci: fmt-check lint test build-release openspec-validate lint-deltas model-check-specs sync-sections sync-sections-test summary-completeness check-explain-topic-docs lint-doc-examples doc-examples lint-baseline pretender-check guard-kernel-agreement guard-siblings guard-drill-lock guard-espectacular

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
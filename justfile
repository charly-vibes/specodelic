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

# Section-sync check: in every dual-format file the ADDED Requirements
# and Requirements sections must carry identical requirement text
sync-sections:
    python3 scripts/check_section_sync.py openspec

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
    if openspec list 2>/dev/null | grep -q "{{id}}"; then
        openspec archive {{id}} --skip-specs --yes
    fi
    dir=$(find openspec/changes/archive -maxdepth 1 -type d -name "*-{{id}}" | sort | tail -1)
    for f in "$dir"/specs/*/spec.md; do
        cap=$(basename "$(dirname "$f")")
        mkdir -p "openspec/specs/$cap"
        cp "$f" "openspec/specs/$cap/spec.md"
        echo "archived: openspec/specs/$cap/spec.md (dual-format, verbatim)"
    done

# Build the docs book locally, mirroring the docs.yml workflow steps
docs-build:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -rf docs/src/specs docs/src/openspec
    cp -r specs docs/src/specs
    cp -r openspec/specs docs/src/openspec
    mdbook build docs
    cp llms.txt docs/book/llms.txt
    echo "docs built: docs/book/index.html"

# === CI Pipeline ===

ci: fmt-check lint test build-release openspec-validate lint-deltas sync-sections sync-sections-test guard-siblings

# Sibling-tool constraint guard (AGENTS.md hard blockers) — also wired into
# pre-commit/pre-push via lefthook.yml and the .beads/hooks shim chain
guard-siblings:
    scripts/guards/sibling-blockers.sh .

# Session start
prime:
    bd ready

# Publish to crates.io (run after `just ci` passes)
publish:
    cargo publish
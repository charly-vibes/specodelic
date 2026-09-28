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

# === CI Pipeline ===

ci: fmt-check lint test build-release

# Session start
prime:
    bd ready

# Publish to crates.io (run after `just ci` passes)
publish:
    cargo publish
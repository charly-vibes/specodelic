//! Drift guards for the CI/supply-chain hardening contract (specodelic-oet).
//!
//! CI workflows rot silently: someone edits ci.yml, drops a matrix leg or
//! the cargo-deny step, and no test notices until the next supply-chain
//! audit. These tests pin the hardening contract the same way
//! scripts/test_check_section_sync.py pins the dual-format sync contract —
//! by reading the repo artifacts directly.
//!
//! Contract (specodelic-oet):
//! 1. Cargo.toml declares an MSRV (`rust-version`) matching edition 2024's
//!    floor, and ci.yml runs a job pinned to that MSRV.
//! 2. ci.yml runs the test/clippy matrix over Linux AND macOS AND Windows.
//! 3. ci.yml runs cargo-deny; deny.toml enables advisories + licenses.
//! 4. ci.yml lints fenced spec examples extracted from the corpus docs
//!    (scripts/check_doc_examples.py), so USAGE-style quick-start blocks
//!    cannot silently drift out of the format.

use std::fs;
use std::path::Path;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// Edition 2024 requires Rust 1.85; the declared MSRV must not lag or
/// lead that floor, and CI must pin exactly it.
#[test]
fn msrv_declared_and_pinned_in_ci() {
    let toml = read("Cargo.toml");
    let msrv = toml
        .lines()
        .find_map(|l| l.trim().strip_prefix("rust-version = \""))
        .map(|s| s.trim_end_matches('"').to_string())
        .expect("Cargo.toml must declare rust-version (MSRV)");
    assert_eq!(
        msrv, "1.85",
        "edition 2024's floor is 1.85; if you bump the MSRV, bump this test and the CI pin together"
    );

    let ci = read(".github/workflows/ci.yml");
    assert!(
        ci.contains("1.85"),
        "ci.yml must run a job pinned to the declared MSRV ({msrv})"
    );
}

/// The crate is a CLI users install cross-platform: the test/clippy matrix
/// must cover all three desktop OSes, not just ubuntu.
#[test]
fn ci_matrix_covers_linux_macos_windows() {
    let ci = read(".github/workflows/ci.yml");
    for os in ["ubuntu-latest", "macos-latest", "windows-latest"] {
        assert!(ci.contains(os), "ci.yml matrix must include {os}");
    }
}

/// Supply-chain gate: cargo-deny must run in CI with advisories and
/// licenses checks enabled in deny.toml.
#[test]
fn cargo_deny_wired_with_advisories_and_licenses() {
    let ci = read(".github/workflows/ci.yml");
    assert!(
        ci.contains("cargo-deny"),
        "ci.yml must run cargo-deny (advisories + licenses)"
    );

    let deny = read("deny.toml");
    for section in ["[advisories]", "[licenses]"] {
        assert!(
            deny.contains(section),
            "deny.toml must enable the {section} check"
        );
    }
}

/// Doc examples rot: fenced ```markdown spec examples in specs/*.md are
/// runnable format artifacts; CI must lint them or USAGE-style blocks
/// drift (the specodelic-vpx class of bug). The pipeline lives in the
/// justfile (`just ci` is the local/CI parity point), so that is where
/// the gate must be wired.
#[test]
fn ci_lints_fenced_spec_examples() {
    let justfile = read("justfile");
    let ci_line = justfile
        .lines()
        .find(|l| l.starts_with("ci:"))
        .expect("justfile must define a ci: recipe");
    assert!(
        ci_line.contains("lint-doc-examples"),
        "justfile ci: must run the fenced-spec-example lint (lint-doc-examples)"
    );
}

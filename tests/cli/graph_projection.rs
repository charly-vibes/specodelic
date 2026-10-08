//! Projection-mode exit-code semantics (`specs/errors.md`
//! `exit_code_mapping`, specodelic-0zk F3+F8).
//!
//! Purpose: pin that `spk graph --format <projection>` obeys the corpus
//! 0/1/2 exit mapping in its text modes too, not just JSON.
//!
//! Responsibilities: a nonexistent/unreadable path exits 2 (invocation
//! error — a typo'd path must not read as an empty corpus), a real
//! path with zero spec files keeps the parsed-not-linted exit 0, and a
//! corpus with typing violations exits 1 while the violation annotation
//! rows still ride along (specs/graph.md design D3).
//!
//! Rationale: pipelines gate on exit codes; before specodelic-0zk the
//! projection modes reported success for both failure shapes, so a gate
//! could neither catch a typo'd path nor stop on a violations corpus.

use assert_cmd::Command;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

/// F3: a nonexistent path is NOT zero spec files — the projection
/// refuses with the invocation-error exit code, matching JSON mode
/// (`spk graph /nonexistent` exits 2).
#[test]
fn nonexistent_path_projection_exits_2() {
    let out = spk()
        .args(["graph", "/nonexistent/spk-0zk-path", "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "a nonexistent path is an invocation error, not a clean empty corpus"
    );
    assert!(
        !out.stdout.is_empty() || !out.stderr.is_empty(),
        "the refusal is labeled, never silent"
    );
}

/// F3 (dot and mermaid projections share the same exit semantics).
#[test]
fn nonexistent_path_dot_and_mermaid_also_exits_2() {
    for format in ["dot", "mermaid"] {
        let out = spk()
            .args(["graph", "/nonexistent/spk-0zk-path", "--format", format])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(2),
            "--format {format} must refuse a nonexistent path too"
        );
    }
}

/// The parsed-not-linted carve-out survives F3: a real path holding no
/// spec files is still a clean empty projection (the zero_file fixture's
/// contract, unchanged).
#[test]
fn existing_empty_path_still_exits_0() {
    let out = spk()
        .args([
            "graph",
            "tests/fixtures/graph/zero_file",
            "--format",
            "edges",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
}

/// F8: typing violations flip the projection exit to 1 (findings, per
/// `exit_code_mapping`) — matching JSON mode — while the violation
/// annotation rows still ride along (design D3: a view is never cleaner
/// than the artifact).
#[test]
fn violations_corpus_projection_exits_1_and_keeps_rows() {
    let out = spk()
        .args([
            "graph",
            "tests/fixtures/typing_violations",
            "--format",
            "edges",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "typing violations are findings: exit 1, not 0"
    );
    let rows: Vec<Vec<&str>> = std::str::from_utf8(&out.stdout)
        .unwrap()
        .lines()
        .map(|l| l.split('\t').collect())
        .collect();
    let annotations = rows
        .iter()
        .filter(|r| r[2].starts_with("violation:"))
        .count();
    assert_eq!(
        annotations, 7,
        "every violation annotation row rides along (JSON mode reports 7 for this fixture): {rows:?}"
    );
    assert!(
        rows.iter()
            .any(|r| r[0] == "c1.t1" && r[2] == "transitions.from"),
        "recorded transition edges survive next to the annotations: {rows:?}"
    );
}

/// F8 (wiring view): the wiring projection over the violations corpus
/// exits 1 too — the view selector must not launder the exit code.
#[test]
fn violations_corpus_wiring_view_exits_1() {
    let out = spk()
        .args([
            "graph",
            "tests/fixtures/typing_violations",
            "--format",
            "edges",
            "--view",
            "wiring",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
}

/// F8: the JSON envelope mode keeps its existing exit codes — 1 for the
/// violations corpus (anti-goal: don't change JSON-mode behavior).
#[test]
fn violations_corpus_json_mode_still_exits_1() {
    let out = spk()
        .args(["graph", "tests/fixtures/typing_violations", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
}

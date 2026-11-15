//! Projection-mode exit-code semantics (`openspec/specs/errors/spec.md`
//! `exit_code_mapping`, specodelic-0zk F3+F8).
//!
//! Purpose: pin that `spk graph --format <projection>` obeys the corpus
//! 0/1/2 exit mapping in its text modes too, not just JSON.
//!
//! Responsibilities: a nonexistent/unreadable path exits 2 (invocation
//! error — a typo'd path must not read as an empty corpus), a real
//! path with zero spec files keeps the parsed-not-linted exit 0, and a
//! corpus with typing violations exits 1 while the violation annotation
//! rows still ride along (openspec/specs/graph/spec.md design D3).
//!
//! Rationale: pipelines gate on exit codes; before specodelic-0zk the
//! projection modes reported success for both failure shapes, so a gate
//! could neither catch a typo'd path nor stop on a violations corpus.

use crate::common::UnwrapPanic;
use assert_cmd::Command;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// F3: a nonexistent path is NOT zero spec files — the projection
/// refuses with the invocation-error exit code, matching JSON mode
/// (`spk graph /nonexistent` exits 2).
#[test]
fn nonexistent_path_projection_exits_2() {
    let out = spk()
        .args(["graph", "/nonexistent/spk-0zk-path", "--format", "edges"])
        .output()
        .unwrap_p();
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
            .unwrap_p();
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
        .unwrap_p();
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
        .unwrap_p();
    assert_eq!(
        out.status.code(),
        Some(1),
        "typing violations are findings: exit 1, not 0"
    );
    let rows: Vec<Vec<&str>> = std::str::from_utf8(&out.stdout)
        .unwrap_p()
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
        .unwrap_p();
    assert_eq!(out.status.code(), Some(1));
}

/// F8: the JSON envelope mode keeps its existing exit codes — 1 for the
/// violations corpus (anti-goal: don't change JSON-mode behavior).
#[test]
fn violations_corpus_json_mode_still_exits_1() {
    let out = spk()
        .args(["graph", "tests/fixtures/typing_violations", "--json"])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(1));
}

/// specodelic-39w: `[[model.state]]` is a documented legal typed target
/// (`linter-referential_integrity`), so a corpus carrying one must graph —
/// every projection used to panic at `acset/instance.rs:126` with exit 101
/// and no envelope.
#[test]
fn model_anchor_link_graphs_without_panicking() {
    let dir = tempfile::tempdir().unwrap_p();
    let cap = dir.path().join("v-x");
    std::fs::create_dir_all(&cap).unwrap_p();
    std::fs::write(
        cap.join("spec.md"),
        "---\nid: v.x\nkind: intent\nstatement: \"WHEN the corpus is graphed THE v-x system SHALL resolve a typed model anchor.\"\n---\n\
         \n# v.x Specification\n\
         \n## Purpose\nModel anchor fixture.\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `x == x` | [[v.x]] |\n\
         | c2 | invariant | `y == y` | [[v.x]] ∧ [[model.state]] |\n\
         \n## Model\n\
         \n### States\n\
         \n- idle\n\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         | t1 | idle | idle | [[v.x.c1]] |\n\
         \n## Properties\n\
         \n| id | kind | predicate | derives_from |\n\
         |----|------|-----------|--------------|\n\
         | p1 | unit | `x == x` | [[v.x.c1]] |\n\
         | p2 | unit | `y == y` | [[v.x.c2]] |\n",
    )
    .unwrap_p();
    let path = dir.path().to_str().unwrap_p();
    for args in [
        vec!["graph", path, "--json"],
        vec!["graph", path, "--format", "edges"],
        vec!["graph", path, "--format", "dot"],
        vec!["graph", path, "--format", "mermaid"],
    ] {
        let out = spk().args(&args).output().unwrap_p();
        assert_eq!(
            out.status.code(),
            Some(0),
            "the model anchor must graph, not abort: {args:?} stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("panicked"),
            "no panic may reach stderr: {args:?}"
        );
    }
    let out = spk().args(["graph", path, "--json"]).output().unwrap_p();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap_p()).unwrap_p();
    assert!(
        json["data"]["edges"]
            .as_array()
            .unwrap_p()
            .iter()
            .any(|e| e["to"] == "v.x.model.state"),
        "the anchor edge is recorded: {json}"
    );
}

//! Graph-views fixture-corpus contracts — run the `specodelic` binary
//! over the checked-in corpora the Python transform's scope gate consumes
//! (add-graph-views 2.3 corpus legs, specodelic-gre.6).
//!
//! Purpose: pin what `spk graph` / `spk lint` emit over
//! `tests/fixtures/{graph/zero_file,single_intent,typing_violations,lint_dirty}`
//! — the artifact shapes `scripts/graph_views.py`'s `states` view renders
//! from and scope-gates on.
//!
//! Responsibilities: the four corpus legs (intentless projection empty
//! but envelope refusal; single-intent in-scope envelope; violation
//! annotation rows riding recorded transition edges; lint-dirty parses
//! but fails lint) plus the report's `intents` field the view layer
//! groups on.
//!
//! Rationale: scripts/test_graph_views.py pins what the transform renders
//! from constructed artifacts; this suite pins what the binary emits over
//! the same-shaped checked-in corpora — both sides fail loudly on drift,
//! and the fixture corpora stay files rather than inline strings
//! (parse_misc.rs sits at its file_lines cap).

use assert_cmd::Command;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap()
}

fn edges_rows(out: &[u8]) -> Vec<Vec<&str>> {
    std::str::from_utf8(out)
        .unwrap()
        .lines()
        .map(|l| l.split('\t').collect())
        .collect()
}

/// Tool half of `empty_corpus_valid`: the intentless fixture's raw
/// projection exits 0 with an empty TSV — while the JSON envelope the
/// scope gate consumes is a refusal (exit 2, ok:false).
#[test]
fn fixture_zero_file_corpus_projects_empty_but_envelope_refuses() {
    let out = spk()
        .args([
            "graph",
            "tests/fixtures/graph/zero_file",
            "--format",
            "edges",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "parsed-not-linted: exit 0");
    assert!(out.stdout.is_empty(), "an intentless corpus yields no rows");
    let json = spk()
        .args(["graph", "tests/fixtures/graph/zero_file", "--json"])
        .output()
        .unwrap();
    assert_eq!(json.status.code(), Some(2), "the scope-gate input refuses");
}

/// `single_intent_sane` (tool half): in scope (files == 1 in the JSON
/// envelope), zero edges — the transform renders the labeled
/// no_transitions view from this artifact pair.
#[test]
fn fixture_single_intent_corpus_is_in_scope_with_wellformed_rows() {
    let tsv = spk()
        .args(["graph", "tests/fixtures/single_intent", "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(tsv.status.code(), Some(0));
    for row in edges_rows(&tsv.stdout) {
        assert_eq!(row.len(), 6, "every row has exactly six columns: {row:?}");
    }
    let json = spk()
        .args(["graph", "tests/fixtures/single_intent", "--json"])
        .output()
        .unwrap();
    assert_eq!(json.status.code(), Some(0));
    let data: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(data["ok"], serde_json::json!(true));
    assert_eq!(
        data["data"]["files"],
        serde_json::json!(1),
        "the scope gate's intent leg reads files >= 1"
    );
    assert_eq!(
        data["data"]["intents"],
        serde_json::json!(["one"]),
        "the report carries the sorted intent ids the view layer groups on"
    );
}

/// The violation-bearing fixture projects both the recorded transition
/// edges and one annotation row per violation — including a
/// `violation:transitions.guard` row the state view must annotate (D3).
#[test]
fn fixture_typing_violations_corpus_projects_annotation_and_transition_rows() {
    let out = spk()
        .args([
            "graph",
            "tests/fixtures/typing_violations",
            "--format",
            "edges",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let rows = edges_rows(&out.stdout);
    for row in &rows {
        assert_eq!(row.len(), 6, "every row has exactly six columns: {row:?}");
    }
    assert!(
        rows.iter().any(|r| r[2] == "violation:transitions.guard"
            && r[3] == "c2.c2_effect"
            && !r[5].is_empty()),
        "the guard typing violation rides along with its reason: {rows:?}"
    );
    assert!(
        rows.iter()
            .any(|r| r[0] == "c1.t1" && r[2] == "transitions.from" && r[3] == "c1.alive"),
        "recorded transition edges survive next to the annotations: {rows:?}"
    );
}

/// The lint-dirty fixture's two halves: the graph projection stays
/// parsed-not-linted (exit 0, transitions.from/to rows), while `spk
/// lint` reports the invariant finding the transform's scope gate
/// refuses on (`out_of_scope_refused`, lint leg).
#[test]
fn fixture_lint_dirty_corpus_parses_but_fails_lint() {
    let out = spk()
        .args(["graph", "tests/fixtures/lint_dirty", "--format", "edges"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "graph extraction needs parsed, not linted"
    );
    let rows = edges_rows(&out.stdout);
    assert!(
        rows.iter()
            .any(|r| r[0] == "dirty.t" && r[2] == "transitions.to" && r[3] == "dirty.s2"),
        "the unguarded transition still projects its from/to edges: {rows:?}"
    );
    let lint = spk()
        .args(["lint", "tests/fixtures/lint_dirty", "--json"])
        .output()
        .unwrap();
    assert_ne!(lint.status.code(), Some(0), "invariant findings fail lint");
    let data: serde_json::Value = serde_json::from_slice(&lint.stdout).unwrap();
    let rules: Vec<&str> = data["data"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["rule_id"].as_str().unwrap())
        .collect();
    assert!(
        rules.contains(&"linter.guard_required"),
        "the scope gate names guard_required as the failed gate: {rules:?}"
    );
}

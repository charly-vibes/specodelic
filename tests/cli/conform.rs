// CLI wiring tests for `spk conform` (openspec/changes/add-conform
// phase 4, tasks 4.1/4.2): the report envelope over the standard
// conventions, the recorded `--closed-world` declaration, the
// open-world-never-forbidden property asserted at CLI level, the
// exit-code contract (0 = no forbidden/unsupported; 1 = any; 2 =
// invocation error / gate refusal with zero verdict records), and the
// read-only help contract.
use super::*;

/// A lint-clean, compilable single-file fixture (rust-fragment invariant
/// so the covering claim is executable — mirrors the phase-3 gate
/// fixture's shape with a distinct id).
const SPEC_TEXT: &str = r#"---
id: demo.cli_conform
kind: intent
statement: "THE system SHALL conform to the recorded oracle traces"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.cli_conform]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.cli_conform.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.cli_conform.locked]] | `g()` | `x` |
"#;

/// The same spec with edited structured content (a second transition) —
/// artifacts compiled before this edit are stale for the gate.
const SPEC_TEXT_EDITED: &str = r#"---
id: demo.cli_conform
kind: intent
statement: "THE system SHALL conform to the recorded oracle traces"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.cli_conform]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.cli_conform.locked]] |
| reopen | confirmed | held | [[demo.cli_conform.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.cli_conform.locked]] | `g()` | `x` |
"#;

const PERMITTED_LINE: &str =
    r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#;
/// `undo` is no declared transition — uncovered (underspecified
/// open-world, forbidden only under `--closed-world`).
const UNCOVERED_LINE: &str =
    r#"{"id": "undo-undeclared", "setup": {"state": "held"}, "trace": [{"action": "undo"}]}"#;
/// A declared transition exercised from a state it is not declared
/// from — positive contradiction, forbidden in ANY invocation mode.
const CONTRADICTION_LINE: &str =
    r#"{"id": "confirm-from-wrong-state", "setup": {"state": "confirmed"}, "trace": [{"action": "confirm"}]}"#;

/// Compile the fixture spec into a temp out-dir (the gate requires
/// current artifacts — conform never re-compiles).
fn compiled_out_dir(dir: &std::path::Path, spec: &std::path::Path) -> String {
    let out_dir = dir.join("out");
    spk()
        .args([
            "compile",
            spec.to_str().unwrap(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();
    out_dir.to_str().unwrap().to_string()
}

fn conform_json(spec: &std::path::Path, corpus: &std::path::Path, extra: &[&str]) -> (Option<i32>, serde_json::Value) {
    let dir = spec.parent().unwrap().to_path_buf();
    let out_dir = compiled_out_dir(dir.as_path(), spec);
    let out = spk()
        .args([
            "conform",
            spec.to_str().unwrap(),
            "--oracle",
            corpus.to_str().unwrap(),
            "--out-dir",
            &out_dir,
            "--json",
        ])
        .args(extra)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    (out.status.code(), json)
}

// ---- task 4.1: report envelope, recorded declaration, open-world ----

#[test]
fn conform_emits_report_envelope_open_world_never_forbidden() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n{UNCOVERED_LINE}\n")).unwrap();

    let (code, json) = conform_json(&spec, &corpus, &[]);
    // Coverage gaps never fail: permitted + underspecified → exit 0.
    assert_eq!(code, Some(0), "open-world gaps are non-failing: {json}");
    assert_eq!(json["ok"], serde_json::json!(true), "{json}");
    let data = &json["data"];
    // The conform-local report schema rides the envelope's data.
    assert_eq!(data["report_schema_version"], 1);
    assert_eq!(data["closed_world"], serde_json::json!(false));
    assert!(
        data["evidence_scope"].as_str().is_some_and(|s| !s.is_empty()),
        "evidence_scope present in the JSON view: {json}"
    );
    // Records sorted by scenario id; the uncovered trace is
    // underspecified — never forbidden without the declaration
    // (open_world_never_forbidden at CLI level).
    let records = data["records"].as_array().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["scenario_id"], "confirm-happy");
    assert_eq!(records[0]["verdict"], "permitted");
    assert_eq!(records[1]["scenario_id"], "undo-undeclared");
    assert_eq!(records[1]["verdict"], "underspecified");
    for record in records {
        assert_ne!(
            record["verdict"], "forbidden",
            "open-world runs never produce forbidden: {json}"
        );
        assert_eq!(record["closed_world"], serde_json::json!(false));
    }
    // Counts surfaced: the coverage gap is visible, not hidden.
    assert_eq!(data["verdict_counts"]["permitted"], 1);
    assert_eq!(data["verdict_counts"]["underspecified"], 1);
}

#[test]
fn conform_closed_world_recorded_and_uncovered_forbidden() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{UNCOVERED_LINE}\n")).unwrap();

    let (code, json) = conform_json(&spec, &corpus, &["--closed-world"]);
    // Uncovered + declared closed-world → forbidden → exit 1.
    assert_eq!(code, Some(1), "closed-world forbidden fails the run: {json}");
    assert_eq!(json["ok"], serde_json::json!(false));
    let data = &json["data"];
    // The declaration is recorded in the header AND every record (D2).
    assert_eq!(data["closed_world"], serde_json::json!(true));
    let records = data["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["verdict"], "forbidden");
    assert_eq!(records[0]["closed_world"], serde_json::json!(true));
    assert_eq!(data["verdict_counts"]["forbidden"], 1);
}

#[test]
fn conform_contradiction_forbidden_in_any_mode() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{CONTRADICTION_LINE}\n")).unwrap();

    // NO --closed-world: a positive contradiction is still forbidden.
    let (code, json) = conform_json(&spec, &corpus, &[]);
    assert_eq!(code, Some(1), "contradiction fails any mode: {json}");
    let records = json["data"]["records"].as_array().unwrap();
    assert_eq!(records[0]["verdict"], "forbidden");
    assert_eq!(records[0]["closed_world"], serde_json::json!(false));
    // The reason names the contradicted executable claim id.
    assert!(
        records[0]["reason"].as_str().unwrap().contains("locked"),
        "reason names the contradicted claim: {json}"
    );
}

// ---- task 4.2: exit-code contract — exit 2 paths, zero records ----

#[test]
fn conform_invocation_error_exits_2_without_records() {
    let dir = tempfile::tempdir().unwrap();
    // Missing spec path.
    let out = spk()
        .args([
            "conform",
            dir.path().join("nope.md").to_str().unwrap(),
            "--oracle",
            dir.path().join("scenarios.jsonl").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "missing spec is an invocation error");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(false));
    assert!(
        json["data"]["records"].is_null(),
        "zero verdict records on refusal: {json}"
    );
    // Missing oracle path too.
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let out_dir = compiled_out_dir(dir.path(), &spec);
    let out = spk()
        .args([
            "conform",
            spec.to_str().unwrap(),
            "--oracle",
            dir.path().join("absent.jsonl").to_str().unwrap(),
            "--out-dir",
            &out_dir,
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "missing oracle is an invocation error");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(false));
    assert!(json["data"]["records"].is_null(), "zero records: {json}");
}

#[test]
fn conform_gate_refusal_exits_2_without_records() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let _out_dir = compiled_out_dir(dir.path(), &spec);
    // Edit the structured content WITHOUT recompiling → stale artifacts.
    std::fs::write(&spec, SPEC_TEXT_EDITED).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{PERMITTED_LINE}\n")).unwrap();

    let out = spk()
        .args([
            "conform",
            spec.to_str().unwrap(),
            "--oracle",
            corpus.to_str().unwrap(),
            "--out-dir",
            dir.path().join("out").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "gate refusal stays exit 2");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["ok"], serde_json::json!(false), "{json}");
    assert!(
        json["data"]["records"].is_null(),
        "no verdict records from a gated-out invocation: {json}"
    );
    // The refusal names the failing stage's remediation (spk compile).
    let rendered = serde_json::to_string(&json).unwrap();
    assert!(
        rendered.contains("spk compile"),
        "refusal names the compile remediation: {json}"
    );
}

#[test]
fn conform_empty_corpus_yields_valid_empty_report_exit_0() {
    // Delta 'Empty corpus yields a valid empty report' at CLI level: a
    // zero-line corpus is a valid Ok report — zero records,
    // evidence_scope intact, exit 0 — never an error.
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("demo.cli_conform.md");
    std::fs::write(&spec, SPEC_TEXT).unwrap();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, "\n").unwrap();

    let (code, json) = conform_json(&spec, &corpus, &[]);
    assert_eq!(code, Some(0), "zero traces never fail: {json}");
    assert_eq!(json["ok"], serde_json::json!(true));
    let data = &json["data"];
    assert_eq!(data["report_schema_version"], 1);
    assert_eq!(data["records"], serde_json::json!([]));
    assert_eq!(data["verdict_counts"], serde_json::json!({}));
    assert!(
        data["evidence_scope"].as_str().is_some_and(|s| !s.is_empty()),
        "evidence_scope intact on the empty report: {json}"
    );
}

// ---- task 4.2: the help contract — READ-ONLY, stated up front ----

#[test]
fn conform_help_states_read_only_evaluation() {
    spk()
        .args(["conform", "--help"])
        .assert()
        .success()
        .stdout(contains("READ-ONLY"));
}
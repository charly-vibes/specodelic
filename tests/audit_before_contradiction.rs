//! `--before` diff tests for contradictions (specodelic-f1by): a
//! contradiction is live evidence, never a currency label. A row the
//! current run grades `contradicted` stays `contradicted` through the
//! diff — whether the prior report held the same contradiction or never
//! saw the row — so `exit_code` still fails the run. The currency
//! overlay keeps its meaning for every other row.

#[path = "common/mod.rs"]
mod common;

use common::UnwrapPanic;
use common::audit_fixtures::{CONFIRM_HAPPY, audit_json};

// ---------------------------------------------------------------------------
// Fixtures — the CLI spec shape from `audit_oracle_rows.rs`, with the
// corpus rows that produce a contradicted record
// ---------------------------------------------------------------------------

/// A lint-clean, compilable CLI fixture (rust-fragment invariant).
const CLI_SPEC_V1: &str = r#"---
id: demo.audit_before
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.audit_before]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.audit_before.locked]] |
| reopen | confirmed | held | [[demo.audit_before.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.audit_before.locked]] | `g()` | `x` |
"#;

/// `undo` is no declared transition — uncovered (underspecified open-world).
const UNCOVERED_LINE: &str =
    r#"{"id": "undo-undeclared", "setup": {"state": "held"}, "trace": [{"action": "undo"}]}"#;
/// A declared transition exercised from a state it is not declared
/// from — positive contradiction, forbidden in ANY invocation mode.
const CONTRADICTION_LINE: &str = r#"{"id": "confirm-from-wrong-state", "setup": {"state": "confirmed"}, "trace": [{"action": "confirm"}]}"#;

/// The corpus shared by both cases: one permitted, one contradicted,
/// one unchecked row.
fn write_fixtures() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap_p();
    let spec = dir.path().join("demo.audit_before.md");
    std::fs::write(&spec, CLI_SPEC_V1).unwrap_p();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(
        &corpus,
        format!("{CONFIRM_HAPPY}\n{CONTRADICTION_LINE}\n{UNCOVERED_LINE}\n"),
    )
    .unwrap_p();
    (dir, spec, corpus)
}

/// The envelope's class for one scenario id.
fn row_class<'a>(json: &'a serde_json::Value, id: &str) -> &'a str {
    json["data"]["records"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|r| r["scenario_id"] == id)
        .unwrap_or_else(|| panic!("record for {id} present: {json}"))["class"]
        .as_str()
        .unwrap_p()
}

// ---------------------------------------------------------------------------
// The two holes: sustained and unseen contradictions both stay live
// ---------------------------------------------------------------------------

/// A contradiction held from the prior report still fails the run: the
/// diff never overlays it into `current` (specodelic-f1by).
#[test]
fn before_sustained_contradiction_exits_1_and_stays_contradicted() {
    let (dir, spec, corpus) = write_fixtures();

    let (code, baseline) = audit_json(&spec, &corpus, &[]);
    assert_eq!(code, Some(1), "the contradicted baseline fails: {baseline}");
    assert_eq!(
        row_class(&baseline, "confirm-from-wrong-state"),
        "contradicted",
        "{baseline}"
    );
    let baseline_path = dir.path().join("baseline.json");
    std::fs::write(&baseline_path, baseline.to_string()).unwrap_p();

    let (code, json) = audit_json(
        &spec,
        &corpus,
        &["--before", baseline_path.to_str().unwrap_p()],
    );
    assert_eq!(
        code,
        Some(1),
        "a sustained contradiction still fails the run: {json}"
    );
    assert_eq!(
        row_class(&json, "confirm-from-wrong-state"),
        "contradicted",
        "the diff never relabels live evidence as current: {json}"
    );
    assert_eq!(json["data"]["class_counts"]["contradicted"], 1);
}

/// A contradiction the prior report never saw stays `contradicted`, not
/// `new`, and still fails the run (specodelic-f1by).
#[test]
fn before_unseen_contradiction_exits_1_and_stays_contradicted_not_new() {
    let (dir, spec, corpus) = write_fixtures();

    let (code, baseline) = audit_json(&spec, &corpus, &[]);
    assert_eq!(code, Some(1), "the contradicted baseline fails: {baseline}");
    // The prior report as a run that never saw the contradicted row:
    // same schema version, same scope digest, same expectation flag.
    let mut prior = baseline;
    let records = prior["data"]["records"].as_array_mut().unwrap_p();
    let before = records.len();
    records.retain(|r| r["scenario_id"] != "confirm-from-wrong-state");
    assert_eq!(
        records.len(),
        before - 1,
        "the prior report no longer carries the contradicted row"
    );
    let prior_path = dir.path().join("prior.json");
    std::fs::write(&prior_path, prior.to_string()).unwrap_p();

    let (code, json) = audit_json(
        &spec,
        &corpus,
        &["--before", prior_path.to_str().unwrap_p()],
    );
    assert_eq!(
        code,
        Some(1),
        "an unseen contradiction still fails the run: {json}"
    );
    assert_eq!(
        row_class(&json, "confirm-from-wrong-state"),
        "contradicted",
        "a missing prior row never downgrades a contradiction to new: {json}"
    );
    assert_eq!(json["data"]["class_counts"]["contradicted"], 1);
    assert!(
        json["data"]["class_counts"].get("new").is_none(),
        "the diff never labels a contradiction new: {json}"
    );
}

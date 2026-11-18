//! Scenario contracts for the archived `check` capability (specodelic-sqgv,
//! 2026-11-18): the verify-stage row mapping, the one-vocabulary rule, the
//! ladder rung derived from this run's verify verdict, and the no-oracle
//! scope statement — one test per `#### Scenario:` in
//! `openspec/specs/check/spec.md` that no existing test asserts end to end.
//! Row-level units live in `tests/check_rows.rs`; the CLI matrix in
//! `tests/check_matrix.rs`. These pins fill the gaps those leave: the exit
//! code of a refuted claim, the exit code and summary count of knowledge
//! absence, the rung BASIS strings, and both output views naming both
//! layers.
//!
//! Model-check evidence is produced by the real `spk model-check` run and
//! then relabelled the way a refuted or undecided run persists its report
//! (`outcome` plus one claim's `status`); every other field — the artifact
//! sha, the scope digest, the claim records — stays the real run's output,
//! so verify's model gate still accepts it as current evidence for this
//! revision. No checker is re-implemented.

#[path = "common/mod.rs"]
mod common;

use common::UnwrapPanic;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// A lint-clean spec with one executable invariant and one property whose
/// predicate is supplied (`**rust:** true` passes, `**rust:** false`
/// fails).
fn write_spec(path: &Path, id: &str, predicate: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL keep the ledger consistent\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | locked | invariant | `**rust:** state != \"open\"` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- held\n\
             - confirmed\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | confirm | held | confirmed | [[{id}.locked]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.locked]] | `g()` | `{predicate}` |\n"
        ),
    )
    .unwrap_p();
}

fn compile(spec: &Path, out_dir: &Path) {
    spk()
        .args([
            "compile",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap_p(),
        ])
        .assert()
        .success();
}

fn model_check(spec: &Path, out_dir: &Path) {
    spk()
        .args([
            "model-check",
            spec.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap_p(),
        ])
        .assert()
        .success();
}

/// One oracle corpus: a conforming trace and a contradicting one (the
/// declared target of `confirm` is `confirmed`, the trace observes `held`).
fn write_corpus(path: &Path) {
    std::fs::write(
        path,
        "{\"id\": \"confirm-happy\", \"setup\": {\"state\": \"held\"}, \"trace\": [{\"action\": \"confirm\", \"observations\": {\"state\": \"confirmed\"}}]}\n\
         {\"id\": \"wrong-obs\", \"setup\": {\"state\": \"held\"}, \"trace\": [{\"action\": \"confirm\", \"observations\": {\"state\": \"held\"}}]}\n",
    )
    .unwrap_p();
}

fn check_json(spec: &Path, out_dir: &Path, extra: &[&str]) -> (Option<i32>, serde_json::Value) {
    let out = spk()
        .args([
            "check",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out_dir.to_str().unwrap_p(),
            "--json",
        ])
        .args(extra)
        .output()
        .unwrap_p();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "check emits a JSON envelope ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    });
    (out.status.code(), json)
}

fn row<'a>(json: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    json["data"]["records"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|r| r["scenario_id"] == key)
        .unwrap_or_else(|| panic!("row {key} present: {json}"))
}

fn verified_fixture(td: &tempfile::TempDir) -> (PathBuf, PathBuf) {
    let spec = td.path().join("demo.contract.md");
    write_spec(&spec, "demo.contract", "**rust:** true");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    (spec, out)
}

/// The `<stem>.check.json` the real `spk model-check` run persisted for a
/// corpus-unique file (the flat layout `artifact_stem` keys on).
fn check_report_path(spec: &Path, out_dir: &Path) -> PathBuf {
    let stem = spec.file_stem().and_then(|s| s.to_str()).unwrap_p();
    out_dir.join(format!("{stem}.check.json"))
}

/// Relabel the real run report the way a refuted or undecided run would
/// have persisted it: the aggregate `outcome` plus the first required
/// claim's `status`. All freshness fields (artifact sha, scope digest,
/// claim ids/evaluators, expected/unchecked sets) stay the real run's, so
/// verify's model gate still accepts the report as current evidence.
/// Returns the claim id the gate will name as the blocking claim.
fn relabel_model_report(spec: &Path, out_dir: &Path, outcome: &str, status: &str) -> String {
    let path = check_report_path(spec, out_dir);
    let raw = std::fs::read_to_string(&path).unwrap_p();
    let mut json: serde_json::Value = serde_json::from_str(&raw).unwrap_p();
    json["outcome"] = serde_json::json!(outcome);
    let claims = json["claims"]
        .as_array_mut()
        .expect("the real run persists its qualified claim records");
    assert!(
        !claims.is_empty(),
        "the fixture's executable invariant is a required claim"
    );
    claims[0]["status"] = serde_json::json!(status);
    let id = claims[0]["id"].as_str().unwrap_p().to_string();
    std::fs::write(&path, serde_json::to_vec(&json).unwrap_p()).unwrap_p();
    id
}

/// The closed sets the capability spec declares (conform's `Verdict`
/// spellings and audit's `AuditClass` spellings). Pinned by literal so a
/// new enum value would fail this contract rather than widen it.
const VERDICTS: [&str; 6] = [
    "permitted",
    "forbidden",
    "underspecified",
    "unknown",
    "unsupported",
    "agree",
];
const CLASSES: [&str; 6] = [
    "current",
    "stale",
    "unchecked",
    "contradicted",
    "binds_nothing",
    "new",
];

/// Scenario "Every row carries both layers": every record of a check
/// report names one verdict from conform's closed set and one class from
/// audit's closed set, in the JSON view and in the human view (the same
/// report values rendered through audit's one rendering path) — including
/// the verify-stage rows riding an oracle report.
#[test]
fn contract_every_row_carries_both_layers_in_both_views() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("scenarios.jsonl");
    write_corpus(&corpus);
    let corpus_arg = corpus.to_str().unwrap_p();
    let (code, json) = check_json(&spec, &out, &["--oracle", corpus_arg]);
    assert_eq!(code, Some(1), "the corpus carries a contradiction: {json}");

    let records = json["data"]["records"].as_array().unwrap_p();
    assert!(!records.is_empty(), "a judgement was emitted: {json}");
    for record in records {
        let id = record["scenario_id"].as_str().unwrap_p();
        let class = record["class"]
            .as_str()
            .unwrap_or_else(|| panic!("row {id} names its audit class: {json}"));
        let verdict = record["conform_verdict"]
            .as_str()
            .unwrap_or_else(|| panic!("row {id} names its verdict: {json}"));
        assert!(
            CLASSES.contains(&class),
            "row {id} class {class} is declared"
        );
        assert!(
            VERDICTS.contains(&verdict),
            "row {id} verdict {verdict} is declared"
        );
        assert_ne!(class, "inconclusive", "reserved value never emitted");
        assert_ne!(verdict, "inconclusive", "reserved value never emitted");
    }

    // The human view renders both layers for every record, from the same
    // report values the JSON envelope carries.
    let human = spk()
        .args([
            "check",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out.to_str().unwrap_p(),
            "--oracle",
            corpus_arg,
            "--human",
        ])
        .output()
        .unwrap_p();
    let rendered = String::from_utf8_lossy(&human.stdout);
    for record in records {
        let id = record["scenario_id"].as_str().unwrap_p();
        let class = record["class"].as_str().unwrap_p();
        let verdict = record["conform_verdict"].as_str().unwrap_p();
        let expected = format!("{id}: {class} ({verdict})");
        assert!(
            rendered.contains(&expected),
            "the human view carries both layers ({expected}): {rendered}"
        );
    }
}

/// Scenario "A refuted claim is a contradiction": the blocking claim is
/// keyed `verify:model:<claim-id>` with verdict `forbidden` and class
/// `contradicted`, and the run exits 1.
#[test]
fn contract_refuted_claim_is_forbidden_contradicted_and_exits_one() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let claim = relabel_model_report(&spec, &out, "counterexample_found", "counterexample");
    let (code, json) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(1), "a refuted claim fails the run: {json}");
    let record = row(&json, &format!("verify:model:{claim}"));
    assert_eq!(record["conform_verdict"], serde_json::json!("forbidden"));
    assert_eq!(record["class"], serde_json::json!("contradicted"));
}

/// Scenario "Knowledge absence is not a contradiction": an undecided
/// claim's row is verdict `unknown`, class `unchecked`, counted in the
/// summary's advisory count, and the run exits 0.
#[test]
fn contract_knowledge_absence_is_unknown_unchecked_and_exits_zero() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let claim = relabel_model_report(&spec, &out, "exploration_only", "unknown");
    let (code, json) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(0), "knowledge absence never gates: {json}");
    let record = row(&json, &format!("verify:model:{claim}"));
    assert_eq!(record["conform_verdict"], serde_json::json!("unknown"));
    assert_eq!(record["class"], serde_json::json!("unchecked"));
    let counted = json["data"]["class_counts"]["unchecked"]
        .as_u64()
        .unwrap_or_else(|| panic!("the unchecked count is surfaced: {json}"));
    assert!(counted >= 1, "the advisory row is counted: {json}");
}

/// Scenario "A fully verified run reaches rung four": the reported rung is
/// 4 AND its basis states the verify pass (not a bare number).
#[test]
fn contract_fully_verified_run_states_the_verify_pass_basis() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let (code, json) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(0), "{json}");
    let rung = &json["data"]["ladder_rung"];
    assert_eq!(rung["rung"], serde_json::json!(4), "{json}");
    let basis = rung["basis"]
        .as_str()
        .unwrap_or_else(|| panic!("the rung states its basis: {json}"));
    assert!(
        basis.contains("matches-oracle") && basis.contains("verify pass"),
        "the basis names the verify pass that lifted the rung: {basis}"
    );
}

/// Scenario "A failed verify does not lift the rung": with the properties
/// gate failing, the rung is derived from the remaining current evidence
/// (the clean model-check report) and never reports 4, stating the basis
/// it did use.
#[test]
fn contract_failed_verify_rung_never_four_and_states_its_basis() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.contract.md");
    write_spec(&spec, "demo.contract", "**rust:** false");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    let (code, json) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(1), "a failed block is a contradiction: {json}");
    let rung = &json["data"]["ladder_rung"];
    assert_ne!(rung["rung"], serde_json::json!(4), "never rung 4: {json}");
    assert_eq!(
        rung["rung"],
        serde_json::json!(3),
        "the clean model-check report is the remaining current evidence: {json}"
    );
    let basis = rung["basis"]
        .as_str()
        .unwrap_or_else(|| panic!("the rung states its basis: {json}"));
    assert!(
        basis.contains("checked-within-bounds") && basis.contains("no_counterexample"),
        "the basis names the evidence it did use: {basis}"
    );
}

/// Scenario "Better-design still outranks a verify pass": a current
/// model-check report that refuted a claim reports rung 5 — even though
/// the compiled property block ran and passed, so a verify pass was
/// present — and `check` reports the ladder's derivation unmodified.
#[test]
fn contract_counterexample_outranks_a_verify_pass_at_rung_five() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    relabel_model_report(&spec, &out, "counterexample_found", "counterexample");
    let (_, json) = check_json(&spec, &out, &[]);
    let prop = row(&json, "verify:prop:p");
    assert_eq!(
        prop["conform_verdict"],
        serde_json::json!("permitted"),
        "the property block did run and pass: {json}"
    );
    let rung = &json["data"]["ladder_rung"];
    assert_eq!(rung["rung"], serde_json::json!(5), "{json}");
    let basis = rung["basis"]
        .as_str()
        .unwrap_or_else(|| panic!("the rung states its basis: {json}"));
    assert!(
        basis.contains("better-design") && basis.contains("counterexample_found"),
        "the rung is the ladder's own derivation, unmodified: {basis}"
    );
}

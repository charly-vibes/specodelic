//! Scenario-contract evidence for `check` (openspec/specs/check/spec.md):
//! Requirements "Single judge verb over existing evidence", "Recorded and
//! citation evidence mapping", "Report interoperability and determinism".
//!
//! Every test here pins one scenario's THEN clause at the level that can
//! actually observe it: the oracle rows compared against `conform`'s own
//! walk output, the unchecked verdict classes that must never gate, the
//! vacuity advisory riding check's own report, and the prior-report
//! currency overlay (stale / new) with its exit contract. Fixtures mirror
//! `tests/check_matrix.rs`; nothing here re-compiles the crate's sources.

#[path = "common/mod.rs"]
mod common;

use common::UnwrapPanic;
use std::path::Path;

use assert_cmd::Command;

/// One permitting trace: the declared transition `confirm` from `held`
/// observes its declared target.
const CONFIRM_HAPPY: &str = r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"state": "confirmed"}}]}"#;
/// One contradicting trace: the same declared transition observes its
/// source state instead of its declared target.
const WRONG_OBS: &str = r#"{"id": "wrong-obs", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"state": "held"}}]}"#;
/// A prose-guarded transition's trace: covered by the Model, but the
/// guard is prose — `unknown` by default, `agree` under structural-only.
const PROSE_COVERED: &str = r#"{"id": "prose-covered", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"state": "confirmed"}}]}"#;
/// An action no Model element declares — open-world `underspecified`.
const UNDECLARED_ACTION: &str =
    r#"{"id": "undeclared-action", "setup": {"state": "held"}, "trace": [{"action": "undo"}]}"#;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// A lint-clean spec with one executable invariant and one property whose
/// predicate is supplied; `action` names the only declared transition, so
/// renaming it across a revision uncovers a recorded trace.
fn write_spec(path: &Path, id: &str, predicate: &str, action: &str) {
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
             | {action} | held | confirmed | [[{id}.locked]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.locked]] | `g()` | `{predicate}` |\n"
        ),
    )
    .unwrap_p();
}

/// The prose-guard fixture from `tests/check_matrix.rs`: `unknown` in the
/// default mode, `agree` under `--structural-only`.
fn write_prose_spec(path: &Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "---\nid: {id}\nkind: intent\nstatement: \"THE system SHALL classify prose-guarded traces honestly\"\n---\n\
             \n## Constraints\n\
             \n| id | kind | expr | traces_to |\n\
             |----|------|------|-----------|\n\
             | gate | invariant | `a lock is held` | [[{id}]] |\n\
             \n## Model\n\
             \n### States\n\
             \n- held\n\
             - confirmed\n\
             \n### Transitions\n\
             \n| id | from | to | guard |\n\
             |----|------|----|-------|\n\
             | confirm | held | confirmed | [[{id}.gate]] |\n\
             \n## Properties\n\
             \n| id | kind | derives_from | generator | predicate |\n\
             |----|------|--------------|-----------|------------|\n\
             | p | unit | [[{id}.gate]] | `g()` | `**rust:** true` |\n"
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

fn check_json(
    spec: &Path,
    out_dir: &Path,
    extra: &[&str],
) -> (Option<i32>, serde_json::Value, Vec<u8>) {
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
    (out.status.code(), json, out.stdout)
}

/// `conform`'s own walk over the same compiled fixture and corpus — the
/// reference verdicts check's oracle rows must carry.
fn conform_json(spec: &Path, out_dir: &Path, corpus: &Path) -> serde_json::Value {
    let out = spk()
        .args([
            "conform",
            spec.to_str().unwrap_p(),
            "--oracle",
            corpus.to_str().unwrap_p(),
            "--out-dir",
            out_dir.to_str().unwrap_p(),
            "--json",
        ])
        .output()
        .unwrap_p();
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "conform emits a JSON envelope ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

fn row<'a>(json: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    json["data"]["records"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|r| r["scenario_id"] == key)
        .unwrap_or_else(|| panic!("row {key} present: {json}"))
}

fn verified_fixture(td: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let spec = td.path().join("demo.check.md");
    write_spec(&spec, "demo.check", "**rust:** true", "confirm");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    (spec, out)
}

/// The evidence rows are conform's walk, row for row: every trace id
/// conform classified appears in check's report carrying exactly the
/// verdict conform's own `--oracle` run produced, and check's report adds
/// no trace row conform never walked. (check/single_judge_verb)
#[test]
fn check_oracle_rows_match_the_conform_walk_verdicts() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{CONFIRM_HAPPY}\n{WRONG_OBS}\n")).unwrap_p();

    let reference = conform_json(&spec, &out, &corpus);
    let reference_rows = reference["data"]["records"]
        .as_array()
        .unwrap_or_else(|| panic!("conform's report carries records: {reference}"));
    assert_eq!(
        reference_rows.len(),
        2,
        "both traces were walked: {reference}"
    );

    let (code, json, _) = check_json(&spec, &out, &["--oracle", corpus.to_str().unwrap_p()]);
    assert_eq!(code, Some(1), "the corpus carries a contradiction: {json}");
    let records = json["data"]["records"].as_array().unwrap_p();
    for reference_row in reference_rows {
        let id = reference_row["scenario_id"].as_str().unwrap_p();
        let judged = records
            .iter()
            .find(|r| r["scenario_id"] == id)
            .unwrap_or_else(|| panic!("check emits the oracle row for {id}: {json}"));
        assert_eq!(
            judged["conform_verdict"], reference_row["verdict"],
            "the row for {id} carries the verdict conform's walk produced"
        );
    }
    // The converse: no trace-looking row invents an id conform never saw.
    for record in records {
        let id = record["scenario_id"].as_str().unwrap_p();
        if id.starts_with("verify:") || id.starts_with("bind:") {
            continue;
        }
        assert!(
            reference_rows
                .iter()
                .any(|r| r["scenario_id"] == record["scenario_id"]),
            "row {id} is a trace conform's walk produced: {json}"
        );
    }
}

/// A permitted trace keeps its corpus id and verdict with class
/// `current`, and the run exits 0 when nothing contradicts.
/// (check/verdict_mapping)
#[test]
fn permitted_only_oracle_run_exits_zero_with_current_rows() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("happy.jsonl");
    std::fs::write(&corpus, format!("{CONFIRM_HAPPY}\n")).unwrap_p();
    let (code, json, _) = check_json(&spec, &out, &["--oracle", corpus.to_str().unwrap_p()]);
    assert_eq!(code, Some(0), "a permitted trace never fails: {json}");
    let happy = row(&json, "confirm-happy");
    assert_eq!(happy["conform_verdict"], "permitted");
    assert_eq!(happy["class"], "current");
}

/// `underspecified`, `unknown`, and structural-only `agree` oracle rows
/// keep their verdict, carry class `unchecked`, and leave the exit at 0 —
/// knowledge absence is never a gate. (check/verdict_mapping)
#[test]
fn unchecked_oracle_verdicts_keep_their_verdict_without_gating() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.prose.md");
    write_prose_spec(&spec, "demo.prose");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    let corpus = td.path().join("trace.jsonl");
    std::fs::write(&corpus, format!("{PROSE_COVERED}\n{UNDECLARED_ACTION}\n")).unwrap_p();

    let (code, json, _) = check_json(&spec, &out, &["--oracle", corpus.to_str().unwrap_p()]);
    assert_eq!(
        code,
        Some(0),
        "unchecked verdicts never gate the run: {json}"
    );
    let unknown = row(&json, "prose-covered");
    assert_eq!(unknown["conform_verdict"], "unknown");
    assert_eq!(unknown["class"], "unchecked");
    let underspecified = row(&json, "undeclared-action");
    assert_eq!(underspecified["conform_verdict"], "underspecified");
    assert_eq!(underspecified["class"], "unchecked");

    // The `agree` value exists only under structural-only, and is
    // unchecked there too.
    let (code_struct, struct_json, _) = check_json(
        &spec,
        &out,
        &["--oracle", corpus.to_str().unwrap_p(), "--structural-only"],
    );
    assert_eq!(code_struct, Some(0), "{struct_json}");
    let agree = row(&struct_json, "prose-covered");
    assert_eq!(agree["conform_verdict"], "agree");
    assert_eq!(agree["class"], "unchecked");
}

/// An evidence unit citing no declared model element is a `binds_nothing`
/// row on check's warnings channel, counted in the summary, and the exit
/// code is unchanged. (check/verdict_mapping)
#[test]
fn vacuous_check_rows_ride_the_warnings_channel_without_gating() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("happy.jsonl");
    std::fs::write(&corpus, format!("{CONFIRM_HAPPY}\n")).unwrap_p();
    let (code, json, _) = check_json(&spec, &out, &["--oracle", corpus.to_str().unwrap_p()]);
    assert_eq!(code, Some(0), "vacuity never gates the run: {json}");

    let bind = row(&json, "bind:locked");
    assert_eq!(bind["class"], "binds_nothing");
    assert_eq!(bind["conform_verdict"], "agree");
    let records = json["data"]["records"].as_array().unwrap_p();
    let binds = records
        .iter()
        .filter(|r| r["class"] == "binds_nothing")
        .count();
    assert!(binds >= 1, "the vacuous fragment is visible: {json}");
    assert_eq!(
        json["data"]["class_counts"]["binds_nothing"],
        serde_json::json!(binds),
        "the advisory rows are countable in the summary: {json}"
    );
    let warnings = json["warnings"]
        .as_array()
        .unwrap_or_else(|| panic!("the envelope has a warnings channel: {json}"));
    assert!(
        warnings.iter().any(|w| {
            w["message"]
                .as_str()
                .is_some_and(|s| s.contains("binds_nothing"))
        }),
        "the warnings channel names the vacuity advisory: {json}"
    );
}

/// A recorded trace whose row worsened since the prior report is `stale`
/// and keeps THIS run's verdict; stale fails (exit 1) only when the prior
/// was recorded with a declared expectation, and is informational (exit
/// 0) otherwise. (check/verdict_mapping, check/exit_contract)
#[test]
fn prior_overlay_labels_stale_without_changing_the_verdict() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.check.md");
    write_spec(&spec, "demo.check", "**rust:** true", "confirm");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    let corpus = td.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{CONFIRM_HAPPY}\n")).unwrap_p();
    let corpus_arg = corpus.to_str().unwrap_p();

    // Two baselines over the same corpus: one declaring expectations.
    let (code_declared, _, declared_bytes) = check_json(
        &spec,
        &out,
        &["--oracle", corpus_arg, "--declare-expectation"],
    );
    assert_eq!(code_declared, Some(0), "the baseline run itself passes");
    let declared_path = td.path().join("declared.json");
    std::fs::write(&declared_path, &declared_bytes).unwrap_p();
    let (code_plain, _, plain_bytes) = check_json(&spec, &out, &["--oracle", corpus_arg]);
    assert_eq!(code_plain, Some(0), "the plain baseline passes");
    let plain_path = td.path().join("plain.json");
    std::fs::write(&plain_path, &plain_bytes).unwrap_p();

    // The revision renames the declared transition: the recorded trace
    // now names no declared action, so its class worsens current →
    // unchecked (⇒ stale) against the unchanged corpus.
    write_spec(&spec, "demo.check", "**rust:** true", "advance");
    compile(&spec, &out);
    model_check(&spec, &out);

    let (code, json, _) = check_json(
        &spec,
        &out,
        &[
            "--oracle",
            corpus_arg,
            "--before",
            declared_path.to_str().unwrap_p(),
        ],
    );
    assert_eq!(
        code,
        Some(1),
        "stale under a declared baseline fails the run: {json}"
    );
    let stale = row(&json, "confirm-happy");
    assert_eq!(stale["class"], "stale");
    assert_eq!(
        stale["conform_verdict"], "underspecified",
        "the row keeps this run's verdict, never the prior report's: {json}"
    );

    let (code_plain, plain_json, _) = check_json(
        &spec,
        &out,
        &[
            "--oracle",
            corpus_arg,
            "--before",
            plain_path.to_str().unwrap_p(),
        ],
    );
    assert_eq!(
        code_plain,
        Some(0),
        "the same stale row without a declared expectation never fails: {plain_json}"
    );
    assert_eq!(row(&plain_json, "confirm-happy")["class"], "stale");
}

/// A row absent from the prior report is `new`, keeps this run's verdict,
/// and never affects the exit. (check/verdict_mapping)
#[test]
fn prior_overlay_labels_new_without_changing_the_verdict() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let (_, _, baseline_bytes) = check_json(&spec, &out, &[]);
    let mut prior: serde_json::Value = serde_json::from_slice(&baseline_bytes).unwrap_p();
    if let Some(records) = prior["data"]["records"].as_array_mut() {
        records.retain(|r| r["scenario_id"] != "verify:prop:p");
    }
    let prior_path = td.path().join("prior.json");
    std::fs::write(&prior_path, serde_json::to_vec(&prior).unwrap_p()).unwrap_p();

    let (code, json, _) = check_json(&spec, &out, &["--before", prior_path.to_str().unwrap_p()]);
    assert_eq!(code, Some(0), "a new row never affects the exit: {json}");
    let fresh = row(&json, "verify:prop:p");
    assert_eq!(fresh["class"], "new");
    assert_eq!(
        fresh["conform_verdict"], "permitted",
        "the row keeps this run's judgement: {json}"
    );
    assert_eq!(
        row(&json, "verify:model")["class"],
        "current",
        "held rows stay current under the overlay: {json}"
    );
}

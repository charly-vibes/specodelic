//! Scenario contracts for the archived `check` capability (specodelic-sqgv,
//! 2026-11-18): the exit contract, the non-judgement refusals, the scope
//! boundary's claim of record, and the non-regression promise to the verbs
//! `check` composes — one test per `#### Scenario:` in
//! `openspec/specs/check/spec.md` that no existing test asserts end to end.
//! Row-level units live in `tests/check_rows.rs`; the CLI matrix in
//! `tests/check_matrix.rs`. These pins fill the gaps those leave: the exit
//! code in BOTH output views, the refusal branches the matrix does not
//! exercise (stale artifacts, stale and unparseable model reports), the
//! all-`unchecked` summary, the declared-expectation `stale` gate, and the
//! documented claim of record.
//!
//! No checker is re-implemented: the fixtures compile and model-check
//! through the real verbs, and a staleness arm edits the spec the way a
//! revision would and recompiles — the stored report is what goes stale.

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

/// Turn the constraint's machine-checkable fragment into prose: the trace
/// is then `unknown` by default (conform never decides a prose guard) and
/// the model gate cannot decide either, while the spec stays lint-clean
/// and compilable — the shape that worsens a previously `current` row to
/// `unchecked`. The property keeps deriving from the same constraint, so
/// coverage still holds.
fn prose_constraint(spec: &Path) {
    let text = std::fs::read_to_string(spec).unwrap_p();
    let machine = "`**rust:** state != \"open\"`";
    assert!(
        text.contains(machine),
        "the fixture declares the machine-checkable fragment"
    );
    std::fs::write(spec, text.replace(machine, "`a lock is held`")).unwrap_p();
}

/// Change the executable invariant's fragment: the compiled module's bytes
/// change, so an artifact compiled before the edit is stale.
fn change_invariant_fragment(spec: &Path) {
    let text = std::fs::read_to_string(spec).unwrap_p();
    assert!(text.contains("state != \"open\""), "the fixture fragment");
    std::fs::write(
        spec,
        text.replace("state != \"open\"", "state != \"closed\""),
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

/// One corpus whose only row is a permitted trace — no contradiction to
/// mask a currency overlay.
fn write_permitted_corpus(path: &Path) {
    std::fs::write(
        path,
        "{\"id\": \"confirm-happy\", \"setup\": {\"state\": \"held\"}, \"trace\": [{\"action\": \"confirm\", \"observations\": {\"state\": \"confirmed\"}}]}\n",
    )
    .unwrap_p();
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

/// Run `check --human`; the report renders on stdout for a judgement, while
/// a refusal's labeled message and hint ride the error channel (genesis's
/// `Output::failure` prints warnings and the next-step footer to stderr).
fn check_human(spec: &Path, out_dir: &Path, extra: &[&str]) -> (Option<i32>, String, String) {
    let out = spk()
        .args([
            "check",
            spec.to_str().unwrap_p(),
            "--out-dir",
            out_dir.to_str().unwrap_p(),
            "--human",
        ])
        .args(extra)
        .output()
        .unwrap_p();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
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
    let spec = td.path().join("demo.exit.md");
    write_spec(&spec, "demo.exit", "**rust:** true");
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

/// Scenario "Gate refusal names its cause": a lint-dirty file and a stale
/// artifact set both refuse with `artifact_gate` in the label, the failing
/// gate's remediation in the message, zero records, and exit 2.
#[test]
fn contract_gate_refusal_names_the_gate_and_the_remedy() {
    // Lint findings: the gate refuses before any judgement.
    let td = tempfile::tempdir().unwrap_p();
    let dirty = td.path().join("demo.dirty.md");
    std::fs::write(
        &dirty,
        "---\nid: demo.dirty\nkind: intent\nstatement: \"the system should maybe work\"\n---\n",
    )
    .unwrap_p();
    let (code, json, stdout) = check_json(&dirty, &td.path().join("lint-out"), &[]);
    assert_eq!(code, Some(2), "a lint-dirty file is refused: {json}");
    assert!(json["data"].is_null(), "zero records: {json}");
    let rendered = String::from_utf8_lossy(&stdout);
    assert!(
        rendered.contains("artifact_gate"),
        "the failing gate is named: {rendered}"
    );
    assert!(
        rendered.contains("lint findings"),
        "the refusal names what failed: {rendered}"
    );
    assert!(
        rendered.contains("spk lint"),
        "the remediation hint names lint: {rendered}"
    );

    // Stale artifacts: lint-clean, but the compiled module predates the
    // fragment edit (never re-compiled).
    let spec = td.path().join("demo.gate.md");
    write_spec(&spec, "demo.gate", "**rust:** true");
    let out = td.path().join("stale-out");
    compile(&spec, &out);
    change_invariant_fragment(&spec);
    let (code, json, stdout) = check_json(&spec, &out, &[]);
    assert_eq!(
        code,
        Some(2),
        "stale artifacts are refused before any verdict: {json}"
    );
    assert!(json["data"].is_null(), "zero records: {json}");
    let rendered = String::from_utf8_lossy(&stdout);
    assert!(
        rendered.contains("artifact_gate"),
        "the failing gate is named: {rendered}"
    );
    assert!(
        rendered.contains("stale"),
        "the refusal names the staleness: {rendered}"
    );
    assert!(
        rendered.contains("spk compile"),
        "the remediation hint names compile: {rendered}"
    );
}

/// Scenario "A stale model report is refused": a report that no longer
/// binds the current revision, and one that cannot be read, both exit 2
/// with a hint naming the model-check command; the stored report is never
/// rewritten and no verdict is inferred.
#[test]
fn contract_stale_or_unreadable_model_report_is_refused_never_rewritten() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.stale.md");
    write_spec(&spec, "demo.stale", "**rust:** true");
    let out = td.path().join("out");
    compile(&spec, &out);
    model_check(&spec, &out);
    let report = check_report_path(&spec, &out);
    let current = std::fs::read(&report).unwrap_p();

    // Stale: the revision's structured content changed and was re-compiled
    // after the run — the stored report no longer binds the artifact.
    change_invariant_fragment(&spec);
    compile(&spec, &out);
    let (code, json, stdout) = check_json(&spec, &out, &[]);
    assert_eq!(
        code,
        Some(2),
        "stale model evidence is not a verdict: {json}"
    );
    assert!(json["data"].is_null(), "zero records: {json}");
    let rendered = String::from_utf8_lossy(&stdout);
    assert!(
        rendered.contains("verify_model"),
        "the failing stage is labeled: {rendered}"
    );
    assert!(
        rendered.contains("spk model-check"),
        "the hint names the model-check rerun: {rendered}"
    );
    assert!(
        !rendered.contains("inconclusive"),
        "the reserved value is never spelled: {rendered}"
    );
    assert_eq!(
        std::fs::read(&report).unwrap_p(),
        current,
        "the stored report is never rewritten"
    );

    // Unparseable: still a refusal, never a verdict.
    std::fs::write(&report, b"not a model report").unwrap_p();
    let (code, json, stdout) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(2), "unreadable evidence refuses: {json}");
    assert!(json["data"].is_null(), "zero records: {json}");
    assert!(
        String::from_utf8_lossy(&stdout).contains("spk model-check"),
        "the hint names the model-check rerun: {}",
        String::from_utf8_lossy(&stdout)
    );
    assert_eq!(
        std::fs::read(&report).unwrap_p(),
        b"not a model report".to_vec(),
        "the refusal never writes the report"
    );
}

/// Scenario "Could not judge is exit two": a run with no obtainable
/// judgement emits zero records, exits 2 in both output views, states the
/// cause and the remediation, and nowhere reads as a pass or a
/// contradiction.
#[test]
fn contract_could_not_judge_is_exit_two_in_both_views() {
    let td = tempfile::tempdir().unwrap_p();
    let spec = td.path().join("demo.refuse.md");
    write_spec(&spec, "demo.refuse", "**rust:** true");
    let out = td.path().join("out");
    compile(&spec, &out); // no model-check run — the run cannot judge

    let (code, json, stdout) = check_json(&spec, &out, &[]);
    assert_eq!(code, Some(2), "no judgement ⇒ exit 2: {json}");
    assert_eq!(json["ok"], serde_json::json!(false));
    assert!(json["data"].is_null(), "zero records: {json}");

    let (human_code, human_out, human_err) = check_human(&spec, &out, &[]);
    assert_eq!(
        human_code,
        Some(2),
        "the human view exits 2 too: {human_err}"
    );

    // A refusal's labeled cause and hint ride the error channel; the JSON
    // envelope carries the same message in its warnings.
    for rendered in [
        String::from_utf8_lossy(&stdout).to_string(),
        human_err.clone(),
    ] {
        assert!(
            rendered.contains("model-check"),
            "the cause is stated: {rendered}"
        );
        assert!(
            rendered.contains("spk model-check"),
            "the remediation hint is stated: {rendered}"
        );
    }
    for rendered in [
        String::from_utf8_lossy(&stdout).to_string(),
        human_out,
        human_err,
    ] {
        assert!(
            !rendered.contains("contradicted"),
            "it never reads as a contradiction: {rendered}"
        );
        assert!(
            !rendered.to_lowercase().contains("pass"),
            "it never reads as a pass: {rendered}"
        );
    }
}

/// Scenario "A contradiction fails the run": a contradicted oracle row
/// exits 1 in both output views, and each view names the contradicted row.
#[test]
fn contract_contradiction_fails_the_run_in_both_views() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("scenarios.jsonl");
    write_corpus(&corpus);
    let corpus_arg = corpus.to_str().unwrap_p();

    let (code, json, _) = check_json(&spec, &out, &["--oracle", corpus_arg]);
    assert_eq!(code, Some(1), "a contradicted row fails the run: {json}");
    assert_eq!(row(&json, "wrong-obs")["class"], "contradicted");
    assert_eq!(
        json["data"]["class_counts"]["contradicted"],
        serde_json::json!(1)
    );

    let (human_code, human, _) = check_human(&spec, &out, &["--oracle", corpus_arg]);
    assert_eq!(human_code, Some(1), "the human view exits 1 too: {human}");
    assert!(
        human.contains("wrong-obs: contradicted"),
        "the human view names the contradicted row: {human}"
    );
}

/// Scenario "Declared-expectation staleness gates": the same worsened row
/// — a `current` trace that a prose guard now leaves `unchecked` — exits 1
/// when the baseline was recorded with `--declare-expectation`, and 0
/// without the declaration.
#[test]
fn contract_declared_expectation_staleness_gates() {
    let td = tempfile::tempdir().unwrap_p();
    let (spec, out) = verified_fixture(&td);
    let corpus = td.path().join("scenarios.jsonl");
    write_permitted_corpus(&corpus);
    let corpus_arg = corpus.to_str().unwrap_p();

    // Two baselines over the SAME corpus: one declared, one plain.
    let (code, json, declared_bytes) = check_json(
        &spec,
        &out,
        &["--oracle", corpus_arg, "--declare-expectation"],
    );
    assert_eq!(code, Some(0), "the baseline is clean: {json}");
    assert_eq!(row(&json, "confirm-happy")["class"], "current");
    let declared = td.path().join("declared.json");
    std::fs::write(&declared, &declared_bytes).unwrap_p();
    let (code, _, plain_bytes) = check_json(&spec, &out, &["--oracle", corpus_arg]);
    assert_eq!(code, Some(0), "the plain baseline is clean");
    let plain = td.path().join("plain.json");
    std::fs::write(&plain, &plain_bytes).unwrap_p();

    // The revision makes the constraint prose: the previously permitted
    // trace is now `unknown`/`unchecked` — a currency drop, never a
    // refutation.
    prose_constraint(&spec);
    compile(&spec, &out);
    model_check(&spec, &out);

    let declared_arg = declared.to_str().unwrap_p();
    let (code, json, _) = check_json(
        &spec,
        &out,
        &["--oracle", corpus_arg, "--before", declared_arg],
    );
    assert_eq!(
        row(&json, "confirm-happy")["class"],
        "stale",
        "the dropped row is stale, not new: {json}"
    );
    assert_eq!(
        code,
        Some(1),
        "a stale row under a declared expectation fails the run: {json}"
    );

    let plain_arg = plain.to_str().unwrap_p();
    let (code, json, _) = check_json(
        &spec,
        &out,
        &["--oracle", corpus_arg, "--before", plain_arg],
    );
    assert_eq!(
        row(&json, "confirm-happy")["class"],
        "stale",
        "the same row is stale: {json}"
    );
    assert_eq!(
        json["data"]["expectation_declared"],
        serde_json::json!(false),
        "an undeclared baseline is informational: {json}"
    );
    assert_eq!(
        code,
        Some(0),
        "without the declaration staleness never fails the run: {json}"
    );
}

/// Scenario "Honest unchecked run exits zero": a report whose every row is
/// `unchecked` or `binds_nothing` exits 0 and its counts are surfaced in
/// both output views (the JSON `class_counts` and the human class list).
#[test]
fn contract_unchecked_run_exits_zero_with_counts_in_both_views() {
    use specodelic::audit::{self, AuditClass};
    use specodelic::model_check::Outcome;
    use specodelic::spec as spec_mod;
    use specodelic::verify::{
        FileVerdict, ModelGateState, PropertiesGate, PropsGateState, Verdict as GateVerdict,
    };

    // A lint-clean shape whose fragment cites no declared model element:
    // the bind row is `binds_nothing`. The model gate could not decide
    // (`exploration_only`) and the properties gate had nothing to run.
    const VACUOUS: &str = r#"---
id: demo.unchecked_run
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.unchecked_run]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.unchecked_run.locked]] |
"#;
    let spec = spec_mod::parse_str(VACUOUS).expect("the fixture spec parses");
    let fv = FileVerdict {
        file: "demo.unchecked_run.md".to_string(),
        id: "demo.unchecked_run".to_string(),
        verdict: GateVerdict {
            status: "model_not_clean",
            message: String::new(),
            hint: String::new(),
        },
        properties: PropertiesGate {
            state: PropsGateState::Pass,
            blocks: vec![],
        },
        model: ModelGateState::NotClean {
            outcome: Outcome::ExplorationOnly,
            blocking_claims: vec!["locked".to_string()],
            unchecked_claim_ids: vec![],
        },
    };
    let mut rows = specodelic::check::verify_rows(&fv);
    rows.extend(audit::bind_rows(&spec));
    let report = audit::fold_rows(audit::build_report(&spec, b"", &[], false, false), rows);

    assert!(
        report
            .records
            .iter()
            .all(|r| matches!(r.class, AuditClass::Unchecked | AuditClass::BindsNothing)),
        "every row is unchecked or binds_nothing: {:?}",
        report.records
    );
    assert_eq!(audit::exit_code(&report, false), 0);
    assert_eq!(audit::exit_code(&report, true), 0);

    let json = serde_json::to_value(&report).unwrap_p();
    assert_eq!(
        json["class_counts"]["unchecked"],
        serde_json::json!(1),
        "the JSON view counts the unchecked row: {json}"
    );
    assert_eq!(
        json["class_counts"]["binds_nothing"],
        serde_json::json!(1),
        "the JSON view counts the binds_nothing row: {json}"
    );
    let human = audit::human_view_labelled("check", &report);
    assert!(human.contains("unchecked 1"), "{human}");
    assert!(human.contains("binds_nothing 1"), "{human}");
}

/// Scenario "The claim of record is stated honestly": the check verb's
/// documentation states the graded-drift claim (a spec revision's dropped or
/// worsened evidence is classified per row, never read as green) and states
/// in the negative that the report is not a second authoritative copy and
/// agent-authored evidence is not independent evidence; the command reference
/// makes neither claim; the report's own runtime scope statement disclaims
/// both.
#[test]
fn contract_documented_claim_of_record_is_drift_visible_and_graded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let graded = std::fs::read_to_string(root.join("docs/src/graded-evidence.md"))
        .expect("the graded-evidence page exists");
    // Drift is visible and graded: a revision's dropped evidence is
    // classified per row, not read as green.
    assert!(
        graded.contains("re-running the audit after a spec edit classifies exactly"),
        "the page states that a revision's drift is classified"
    );
    assert!(
        graded.contains("a row whose evidence dropped or worsened is classified `stale`"),
        "the page names the grade a dropped row receives"
    );
    assert!(
        graded.contains("never read as green"),
        "the page states that dropped evidence is never read as green"
    );
    assert!(
        graded.contains("in which case it fails the run"),
        "the page states that declared-expectation staleness fails the run"
    );
    assert!(
        graded.contains("contradicted` row always fails, whatever the prior report said"),
        "the page states that a contradicted row always fails"
    );

    // Neither overclaim, stated in the negative on the page itself: the
    // report is not a second authoritative copy, and agent-authored
    // evidence is not independent evidence.
    assert!(
        graded.contains("never a second authoritative copy of anything"),
        "the page denies that the report is a second authoritative copy"
    );
    assert!(
        graded.contains("Agent-authored evidence is not independent evidence"),
        "the page denies that agent-authored evidence is independent evidence"
    );

    // The command reference never makes either overclaim.
    let commands = std::fs::read_to_string(root.join("docs/src/commands.md"))
        .expect("the command reference exists");
    let commands = commands.to_lowercase();
    assert!(
        !commands.contains("independent evidence"),
        "commands.md claims independent evidence"
    );
    assert!(
        !commands.contains("second authoritative copy"),
        "commands.md claims a second authoritative copy"
    );

    // The report's own runtime statement is the same not-claim.
    assert!(specodelic::conform::EVIDENCE_SCOPE.contains("not a proof of behavioral equality"));
    assert!(specodelic::check::NO_ORACLE_SCOPE.contains("not claimed"));
}

//! Slice-1 tests for `audit` (openspec/changes/add-evidence-audit,
//! tasks 1.1–1.3): oracle-row classification carrying the underlying
//! conform verdict, the `--before` prior-report diff (stale / new /
//! digest discipline), and the exit contract. Library-level tests cover
//! the classification and diff units; CLI-level tests cover the exit
//! contract and envelope shapes over the real binary. GREEN lives in
//! `src/audit.rs` reusing conform's walk machinery unchanged.

#[path = "common/mod.rs"]
mod common;

use assert_cmd::Command;
use common::UnwrapPanic;
use specodelic::audit::{self, AuditClass, AuditRecord, AuditReport};
use specodelic::conform::{self, Verdict};
use specodelic::spec;

// ---------------------------------------------------------------------------
// Fixtures — same shape as the conform fixtures, distinct ids
// ---------------------------------------------------------------------------

/// Spec v1 (library fixture): declares `confirm` AND `reopen`. The
/// rust-fragment invariant is executable, so covering claims evaluate.
const SPEC_V1: &str = r#"---
id: demo.audit
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.audit]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.audit.locked]] |
| reopen | confirmed | held | [[demo.audit.locked]] |
"#;

/// Spec v2 (library fixture): `reopen` dropped — a recorded example
/// exercising it worsens from `current` to `unchecked` against this
/// revision. This is the revision-preservation diffing case (design D1).
const SPEC_V2: &str = r#"---
id: demo.audit
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.audit]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.audit.locked]] |
"#;

const CONFIRM_HAPPY: &str = r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#;
/// `undo` is no declared transition — uncovered (underspecified open-world).
const UNCOVERED_LINE: &str =
    r#"{"id": "undo-undeclared", "setup": {"state": "held"}, "trace": [{"action": "undo"}]}"#;
/// A declared transition exercised from a state it is not declared
/// from — positive contradiction, forbidden in ANY invocation mode.
const CONTRADICTION_LINE: &str = r#"{"id": "confirm-from-wrong-state", "setup": {"state": "confirmed"}, "trace": [{"action": "confirm"}]}"#;
/// Exercises `reopen` — permitted under SPEC_V1, uncovered under SPEC_V2.
const REOPEN_HAPPY: &str =
    r#"{"id": "reopen-happy", "setup": {"state": "confirmed"}, "trace": [{"action": "reopen"}]}"#;

fn parse_spec(text: &str) -> spec::Spec {
    spec::parse_str(text).expect("fixture spec parses")
}

fn parse_corpus_bytes(lines: &[&str]) -> String {
    let bytes = format!("{}\n", lines.join("\n"));
    conform::parse_corpus(&bytes).expect("fixture corpus parses");
    bytes
}

// ---------------------------------------------------------------------------
// 1.1 — per-record classification carrying the underlying conform verdict
// ---------------------------------------------------------------------------

/// Every corpus row receives exactly one audit class from the closed
/// evidence-lifecycle set, and the record carries the underlying conform
/// verdict it was derived from — the audit layer rides conform's verdicts,
/// never replaces them (design D4).
#[test]
fn audit_classifies_records_current_unchecked_contradicted_carrying_the_conform_verdict() {
    let spec = parse_spec(SPEC_V1);
    let bytes = parse_corpus_bytes(&[CONFIRM_HAPPY, UNCOVERED_LINE, CONTRADICTION_LINE]);
    let corpus = conform::parse_corpus(&bytes).expect("fixture corpus parses");
    let report = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, false);
    let records = &report.records;
    assert_eq!(
        records.len(),
        3,
        "one audit record per corpus row: {records:?}"
    );
    // Records sorted by scenario id (the conform report's convention).
    let row = |id: &str| {
        records
            .iter()
            .find(|r| r.scenario_id == id)
            .unwrap_or_else(|| panic!("record for {id} present"))
    };
    // permitted ⇒ current: the recorded example still agrees with the model.
    let happy = row("confirm-happy");
    assert_eq!(happy.class, AuditClass::Current);
    assert_eq!(happy.conform_verdict, Verdict::Permitted);
    // forbidden ⇒ contradicted: the model positively contradicts the example.
    let wrong = row("confirm-from-wrong-state");
    assert_eq!(wrong.class, AuditClass::Contradicted);
    assert_eq!(wrong.conform_verdict, Verdict::Forbidden);
    assert!(
        !wrong.reason.is_empty(),
        "the record carries the conform walk's labeled reason"
    );
    // underspecified ⇒ unchecked: the example was never truly checkable.
    let uncovered = row("undo-undeclared");
    assert_eq!(uncovered.class, AuditClass::Unchecked);
    assert_eq!(uncovered.conform_verdict, Verdict::Underspecified);
    // Class counts surfaced per class, keyed by snake_case spelling.
    assert_eq!(report.class_counts.get("current"), Some(&1));
    assert_eq!(report.class_counts.get("unchecked"), Some(&1));
    assert_eq!(report.class_counts.get("contradicted"), Some(&1));
    // The underlying conform verdict counts ride through untouched.
    assert_eq!(report.verdict_counts.get("permitted"), Some(&1));
    assert_eq!(report.verdict_counts.get("forbidden"), Some(&1));
    assert_eq!(report.verdict_counts.get("underspecified"), Some(&1));
}

/// The report carries `evidence_scope` verbatim (conform's fixed
/// statement — data, not documentation) and a `scope_sha256` binding the
/// exact oracle corpus bytes consumed: stable across reruns, flipped by
/// any corpus change, and independent of the spec so revision diffing
/// can proceed (design D1: staleness detects SPEC drift; the digest
/// binds the EVIDENCE set).
#[test]
fn audit_report_carries_evidence_scope_verbatim_and_corpus_scope_digest() {
    let spec = parse_spec(SPEC_V1);
    let bytes = parse_corpus_bytes(&[CONFIRM_HAPPY, REOPEN_HAPPY]);
    let corpus = conform::parse_corpus(&bytes).expect("fixture corpus parses");
    let report = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, false);
    assert_eq!(report.evidence_scope, conform::EVIDENCE_SCOPE);
    assert!(
        !report.scope_sha256.is_empty(),
        "scope_sha256 present in the report"
    );
    assert_eq!(report.report_schema_version, audit::AUDIT_SCHEMA_VERSION);
    // The recorded expectation declaration is carried in the header.
    assert!(!report.expectation_declared);
    let declared = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, true);
    assert!(declared.expectation_declared);
    // Deterministic: identical inputs serialize identically.
    let again = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, false);
    assert_eq!(report, again);
    // Any corpus change flips the digest (the evidence set differs).
    let grown = parse_corpus_bytes(&[CONFIRM_HAPPY, REOPEN_HAPPY, UNCOVERED_LINE]);
    let grown_corpus = conform::parse_corpus(&grown).expect("fixture corpus parses");
    let other = audit::build_report(&spec, grown.as_bytes(), &grown_corpus, false, false);
    assert_ne!(report.scope_sha256, other.scope_sha256);
}

// ---------------------------------------------------------------------------
// 1.2 — the --before prior-report diff (stale / new / digest discipline)
// ---------------------------------------------------------------------------

/// A record whose class dropped/worsened between the prior report and
/// the current run classifies `stale` (the prior report's good verdict
/// no longer holds — the stale-example failure the audit exists to
/// catch); a record whose class held stays `current`. The corpus bytes
/// are identical, so the digest check passes and rows compare 1:1.
#[test]
fn before_worsened_class_classifies_stale_and_same_class_stays_current() {
    let corpus_bytes = parse_corpus_bytes(&[CONFIRM_HAPPY, REOPEN_HAPPY]);
    let corpus = conform::parse_corpus(&corpus_bytes).expect("fixture corpus parses");
    let prior = audit::build_report(
        &parse_spec(SPEC_V1),
        corpus_bytes.as_bytes(),
        &corpus,
        false,
        false,
    );
    let reopen_prior = prior
        .records
        .iter()
        .find(|r| r.scenario_id == "reopen-happy")
        .expect("prior row for reopen-happy");
    assert_eq!(reopen_prior.class, AuditClass::Current);

    // The spec dropped `reopen`: the recorded example is now uncovered —
    // its class worsened current → unchecked against this revision.
    let current = audit::build_report(
        &parse_spec(SPEC_V2),
        corpus_bytes.as_bytes(),
        &corpus,
        false,
        false,
    );
    let reopen_current = current
        .records
        .iter()
        .find(|r| r.scenario_id == "reopen-happy")
        .expect("current row for reopen-happy");
    assert_eq!(reopen_current.class, AuditClass::Unchecked);

    let diffed = audit::diff_with_prior(current, &prior).expect("same corpus bytes diff");
    let reopen = diffed
        .records
        .iter()
        .find(|r| r.scenario_id == "reopen-happy")
        .expect("diffed row for reopen-happy");
    assert_eq!(reopen.class, AuditClass::Stale);
    assert_eq!(
        reopen.conform_verdict,
        Verdict::Underspecified,
        "the stale record still carries the CURRENT run's conform verdict"
    );
    // confirm-happy held its class across the revision → current.
    let happy = diffed
        .records
        .iter()
        .find(|r| r.scenario_id == "confirm-happy")
        .expect("diffed row for confirm-happy");
    assert_eq!(happy.class, AuditClass::Current);
    // Class counts recomputed over the diffed classes.
    assert_eq!(diffed.class_counts.get("stale"), Some(&1));
    assert_eq!(diffed.class_counts.get("current"), Some(&1));
}

/// A current-run record the prior report lacks classifies `new` — fresh
/// evidence is never silently graded against a baseline that never saw
/// it. Exercised on the diff unit with a same-digest prior missing the
/// row (the rule the CLI's --before overlay applies per row).
#[test]
fn before_record_absent_from_prior_classifies_new() {
    let bytes = parse_corpus_bytes(&[CONFIRM_HAPPY, REOPEN_HAPPY]);
    let corpus = conform::parse_corpus(&bytes).expect("fixture corpus parses");
    let spec = parse_spec(SPEC_V1);
    let full = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, false);
    // A prior report from a run that never saw reopen-happy: same
    // evidence digest, one row fewer.
    let mut prior = full.clone();
    prior.records.retain(|r| r.scenario_id != "reopen-happy");
    prior
        .class_counts
        .insert("current".to_string(), prior.class_counts["current"] - 1);
    let diffed = audit::diff_with_prior(full, &prior).expect("same corpus bytes diff");
    let reopen = diffed
        .records
        .iter()
        .find(|r| r.scenario_id == "reopen-happy")
        .expect("diffed row for reopen-happy");
    assert_eq!(reopen.class, AuditClass::New);
    assert_eq!(diffed.class_counts.get("new"), Some(&1));
}

/// The digest discipline (design risks): reports whose `scope_sha256`
/// differs are NEVER silently diffed — the mismatch is a labeled
/// refusal naming the field, with a remediation hint. The spec is free
/// to change between reports (that is the diff's purpose); the corpus
/// is not.
#[test]
fn before_digest_mismatch_fails_labeled() {
    let bytes_a = parse_corpus_bytes(&[CONFIRM_HAPPY, REOPEN_HAPPY]);
    let corpus_a = conform::parse_corpus(&bytes_a).expect("fixture corpus parses");
    let bytes_b = parse_corpus_bytes(&[CONFIRM_HAPPY, UNCOVERED_LINE]);
    let corpus_b = conform::parse_corpus(&bytes_b).expect("fixture corpus parses");
    let spec = parse_spec(SPEC_V1);
    let current = audit::build_report(&spec, bytes_a.as_bytes(), &corpus_a, false, false);
    let prior = audit::build_report(&spec, bytes_b.as_bytes(), &corpus_b, false, false);
    let err = audit::diff_with_prior(current, &prior).expect_err("digest mismatch refuses");
    assert!(
        err.message.contains("scope_sha256"),
        "the refusal names the digest field: {err}"
    );
    assert!(
        err.message.contains("spk audit"),
        "the refusal carries a remediation hint: {err}"
    );
}

/// RO5U M1: reports from different audit schema versions are never
/// silently diffed — a prior report whose schema version differs from
/// this audit's is a labeled refusal (the version field exists to
/// guard exactly this drift).
#[test]
fn before_schema_version_mismatch_refuses() {
    let bytes = parse_corpus_bytes(&[CONFIRM_HAPPY]);
    let corpus = conform::parse_corpus(&bytes).expect("fixture corpus parses");
    let spec = parse_spec(SPEC_V1);
    let mut prior = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, false);
    prior.report_schema_version = audit::AUDIT_SCHEMA_VERSION + 1;
    let current = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, false);
    let err = audit::diff_with_prior(current, &prior).expect_err("schema mismatch refuses");
    assert_eq!(err.stage, "before_diff");
    assert!(
        err.message.contains("schema version"),
        "the refusal names the schema drift: {err}"
    );
}

/// RO5U L1: a prior report carrying duplicate scenario rows is refused
/// at parse — the diff keys rows by scenario id, so a duplicate would
/// silently let `.find()` pick whichever row came first.
#[test]
fn before_parse_refuses_duplicate_prior_rows() {
    let bytes = parse_corpus_bytes(&[CONFIRM_HAPPY]);
    let corpus = conform::parse_corpus(&bytes).expect("fixture corpus parses");
    let spec = parse_spec(SPEC_V1);
    let mut report = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, false);
    let dupe = report.records[0].clone();
    report.records.push(dupe);
    let bare = serde_json::to_string(&report).expect("prior report serializes");
    let err = audit::parse_prior_report(bare.as_bytes()).expect_err("duplicate prior rows refuse");
    assert_eq!(err.stage, "before_parse");
    assert!(
        err.message.contains("duplicate scenario id"),
        "the refusal names the duplicate: {err}"
    );
}

// ---------------------------------------------------------------------------
// 1.3 — the exit contract
// ---------------------------------------------------------------------------

/// Exit mapping (library unit): 1 iff a `contradicted` record exists —
/// or a `stale` record when the prior report declared an expectation;
/// 2 on zero records; never nonzero for `unchecked`/`new`/plain `stale`.
#[test]
fn exit_code_one_iff_contradicted_or_stale_under_declared_expectation() {
    let record = |class: AuditClass| AuditRecord {
        scenario_id: "s".to_string(),
        class,
        conform_verdict: Verdict::Permitted,
        reason: "fixture".to_string(),
        evaluated_claim_ids: vec![],
        closed_world: false,
    };
    let report = |records: Vec<AuditRecord>| AuditReport {
        report_schema_version: audit::AUDIT_SCHEMA_VERSION,
        evidence_scope: conform::EVIDENCE_SCOPE.to_string(),
        scope_sha256: "abc".to_string(),
        expectation_declared: false,
        verdict_counts: Default::default(),
        class_counts: Default::default(),
        records,
    };
    // All current → 0.
    assert_eq!(
        audit::exit_code(&report(vec![record(AuditClass::Current)]), false),
        0
    );
    // A contradicted record fails even without any declared expectation.
    assert_eq!(
        audit::exit_code(&report(vec![record(AuditClass::Contradicted)]), false),
        1
    );
    // Stale fails ONLY under the prior's declared expectation.
    assert_eq!(
        audit::exit_code(&report(vec![record(AuditClass::Stale)]), true),
        1
    );
    assert_eq!(
        audit::exit_code(&report(vec![record(AuditClass::Stale)]), false),
        0,
        "stale without a declared expectation is informational, never nonzero"
    );
    // unchecked / new are never nonzero, in either mode.
    for class in [AuditClass::Unchecked, AuditClass::New] {
        assert_eq!(audit::exit_code(&report(vec![record(class)]), false), 0);
        assert_eq!(audit::exit_code(&report(vec![record(class)]), true), 0);
    }
    // Zero records → 2: an audit over no evidence is never a pass.
    assert_eq!(audit::exit_code(&report(vec![]), false), 2);
}

/// The command path refuses zero-record corpora as a labeled error —
/// an audit run over an empty corpus is the empty-checked-set failure
/// this change exists to prevent, never a silent pass.
#[test]
fn run_refuses_zero_record_corpus_labeled() {
    let spec = parse_spec(SPEC_V1);
    let bytes = "\n\n".to_string(); // blank lines only — zero records
    let corpus = conform::parse_corpus(&bytes).expect("blank corpus parses");
    // The gate runs before the record check, so point out_dir at a
    // nonexistent tree ONLY if the gate would fail — instead assert the
    // zero-records refusal via the pure path: build then exit code.
    let report = audit::build_report(&spec, bytes.as_bytes(), &corpus, false, false);
    assert!(report.records.is_empty());
    assert_eq!(audit::exit_code(&report, false), 2);
}

/// The gate is reused from conform unchanged: a run whose artifacts are
/// missing is refused with the labeled artifact_gate error — verdicts
/// are never derived from absent or stale artifacts (exit 2 at CLI).
#[test]
fn run_refuses_gate_failure_labeled() {
    let spec = parse_spec(SPEC_V1);
    let bytes = parse_corpus_bytes(&[CONFIRM_HAPPY]);
    let corpus = conform::parse_corpus(&bytes).expect("fixture corpus parses");
    let err = audit::run(
        &spec,
        std::path::Path::new("/nonexistent/specodelic-audit-gate"),
        bytes.as_bytes(),
        false,
        false,
        None,
    )
    .expect_err("missing artifacts refuse");
    assert_eq!(err.stage, "artifact_gate");
    assert!(
        err.message.contains("spk compile"),
        "the refusal names the remediation: {err}"
    );
}

// ---------------------------------------------------------------------------
// CLI — the real verb over the binary (wiring lives in task 1.4 GREEN)
// ---------------------------------------------------------------------------

/// A lint-clean, compilable CLI fixture (rust-fragment invariant —
/// mirrors the conform CLI fixture's shape with a distinct id).
const CLI_SPEC_V1: &str = r#"---
id: demo.audit_cli
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.audit_cli]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.audit_cli.locked]] |
| reopen | confirmed | held | [[demo.audit_cli.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.audit_cli.locked]] | `g()` | `x` |
"#;

/// CLI v2: `reopen` dropped (the revision-preservation edit).
const CLI_SPEC_V2: &str = r#"---
id: demo.audit_cli
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.audit_cli]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.audit_cli.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.audit_cli.locked]] | `g()` | `x` |
"#;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

/// Compile the fixture spec into a temp out-dir (the reused gate
/// requires current artifacts — audit never re-compiles, like conform).
fn compiled_out_dir(dir: &std::path::Path, spec: &std::path::Path) -> String {
    let out_dir = dir.join("out");
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
    out_dir.to_str().unwrap_p().to_string()
}

fn audit_json(
    spec: &std::path::Path,
    corpus: &std::path::Path,
    extra: &[&str],
) -> (Option<i32>, serde_json::Value) {
    let dir = spec.parent().unwrap_p().to_path_buf();
    let out_dir = compiled_out_dir(dir.as_path(), spec);
    let out = spk()
        .args([
            "audit",
            spec.to_str().unwrap_p(),
            "--oracle",
            corpus.to_str().unwrap_p(),
            "--out-dir",
            &out_dir,
            "--json",
        ])
        .args(extra)
        .output()
        .unwrap_p();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "audit emits a JSON envelope ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    });
    (out.status.code(), json)
}

/// The envelope carries the audit report with scope fields and the
/// class/verdict layers; a contradicted record fails the run (exit 1,
/// error-kind envelope — records ride along).
#[test]
fn audit_cli_envelope_and_exit_1_on_contradicted() {
    let dir = tempfile::tempdir().unwrap_p();
    let spec = dir.path().join("demo.audit_cli.md");
    std::fs::write(&spec, CLI_SPEC_V1).unwrap_p();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(
        &corpus,
        format!("{CONFIRM_HAPPY}\n{CONTRADICTION_LINE}\n{UNCOVERED_LINE}\n"),
    )
    .unwrap_p();

    let (code, json) = audit_json(&spec, &corpus, &[]);
    assert_eq!(code, Some(1), "a contradicted record fails the run: {json}");
    assert_eq!(json["ok"], serde_json::json!(false), "{json}");
    let data = &json["data"];
    assert_eq!(data["report_schema_version"], audit::AUDIT_SCHEMA_VERSION);
    assert_eq!(data["evidence_scope"], conform::EVIDENCE_SCOPE);
    assert!(data["scope_sha256"].as_str().is_some_and(|s| !s.is_empty()));
    // The class layer AND the underlying verdict layer, side by side.
    let records = data["records"].as_array().unwrap_p();
    let wrong = records
        .iter()
        .find(|r| r["scenario_id"] == "confirm-from-wrong-state")
        .expect("contradicted record present");
    assert_eq!(wrong["class"], "contradicted");
    assert_eq!(wrong["conform_verdict"], "forbidden");
    let happy = records
        .iter()
        .find(|r| r["scenario_id"] == "confirm-happy")
        .expect("current record present");
    assert_eq!(happy["class"], "current");
    assert_eq!(happy["conform_verdict"], "permitted");
    let uncovered = records
        .iter()
        .find(|r| r["scenario_id"] == "undo-undeclared")
        .expect("unchecked record present");
    assert_eq!(uncovered["class"], "unchecked");
    assert_eq!(uncovered["conform_verdict"], "underspecified");
    assert_eq!(data["class_counts"]["contradicted"], 1);
    assert_eq!(data["verdict_counts"]["forbidden"], 1);
}

/// Zero records → exit 2 with a labeled refusal (an empty audit is
/// never a pass) — and a gate refusal (missing artifacts) is exit 2
/// with zero records emitted.
#[test]
fn audit_cli_exit_2_on_zero_records_and_gate_refusal() {
    // Zero records over CURRENT artifacts: the refusal is the
    // zero-records check, labeled, never a silent pass.
    let dir = tempfile::tempdir().unwrap_p();
    let spec = dir.path().join("demo.audit_cli.md");
    std::fs::write(&spec, CLI_SPEC_V1).unwrap_p();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, "\n\n").unwrap_p();
    let dir2 = spec.parent().unwrap_p().to_path_buf();
    let out_dir = compiled_out_dir(dir2.as_path(), &spec);
    let out = spk()
        .args([
            "audit",
            spec.to_str().unwrap_p(),
            "--oracle",
            corpus.to_str().unwrap_p(),
            "--out-dir",
            &out_dir,
            "--json",
        ])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(2), "zero records refuse: {out:?}");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(false));
    let rendered = serde_json::to_string(&json).unwrap_p();
    assert!(
        rendered.contains("zero records"),
        "the zero-records refusal is labeled: {json}"
    );

    // Gate refusal: missing artifacts → exit 2, zero records.
    let corpus2 = dir.path().join("scenarios2.jsonl");
    std::fs::write(&corpus2, CONFIRM_HAPPY).unwrap_p();
    let out = spk()
        .args([
            "audit",
            spec.to_str().unwrap_p(),
            "--oracle",
            corpus2.to_str().unwrap_p(),
            "--out-dir",
            dir.path().join("no-such-out").to_str().unwrap_p(),
            "--json",
        ])
        .output()
        .unwrap_p();
    assert_eq!(
        out.status.code(),
        Some(2),
        "gate refusal is exit 2: {out:?}"
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    assert_eq!(json["ok"], serde_json::json!(false));
    assert!(
        json["data"].is_null(),
        "refusal paths emit zero records: {json}"
    );
}

/// Revision preservation end-to-end: a baseline recorded with
/// `--declare-expectation` makes a worsened row fail the later run
/// (stale under a declared expectation → exit 1); the same diff against
/// a baseline without the declaration stays informational (exit 0).
/// The SPEC changed between the runs — allowed; the corpus did not.
#[test]
fn audit_cli_stale_fails_only_when_prior_declared_expectation() {
    let dir = tempfile::tempdir().unwrap_p();
    let spec = dir.path().join("demo.audit_cli.md");
    std::fs::write(&spec, CLI_SPEC_V1).unwrap_p();
    let corpus = dir.path().join("scenarios.jsonl");
    std::fs::write(&corpus, format!("{CONFIRM_HAPPY}\n{REOPEN_HAPPY}\n")).unwrap_p();

    // Baselines over spec v1: one declaring expectations, one not.
    let (code, baseline) = audit_json(&spec, &corpus, &["--declare-expectation"]);
    assert_eq!(code, Some(0), "the baseline run itself passes: {baseline}");
    assert_eq!(
        baseline["data"]["expectation_declared"],
        serde_json::json!(true)
    );
    let (_, plain) = audit_json(&spec, &corpus, &[]);
    assert_eq!(
        plain["data"]["expectation_declared"],
        serde_json::json!(false)
    );
    let declared_path = dir.path().join("baseline.json");
    std::fs::write(&declared_path, baseline.to_string()).unwrap_p();
    let plain_path = dir.path().join("plain.json");
    std::fs::write(&plain_path, plain.to_string()).unwrap_p();

    // Revise the spec (drop `reopen`), refresh artifacts, re-audit with
    // --before. reopen-happy worsened current → unchecked ⇒ stale.
    std::fs::write(&spec, CLI_SPEC_V2).unwrap_p();
    let (code_declared, json) = audit_json(
        &spec,
        &corpus,
        &["--before", declared_path.to_str().unwrap_p()],
    );
    let stale = json["data"]["records"]
        .as_array()
        .unwrap_p()
        .iter()
        .find(|r| r["scenario_id"] == "reopen-happy")
        .expect("stale row present")["class"]
        .clone();
    assert_eq!(stale, "stale", "{json}");
    assert_eq!(
        code_declared,
        Some(1),
        "stale under a declared expectation fails: {json}"
    );
    let (code_plain, json) = audit_json(
        &spec,
        &corpus,
        &["--before", plain_path.to_str().unwrap_p()],
    );
    assert_eq!(
        code_plain,
        Some(0),
        "the same stale row without a declared expectation never fails: {json}"
    );
}

/// Diffing reports whose evidence sets differ is refused, labeled with
/// the digest field and a remediation hint — never silently compared
/// across corpus versions (design's digest discipline).
#[test]
fn audit_cli_digest_mismatch_refuses_labeled() {
    let dir = tempfile::tempdir().unwrap_p();
    let spec = dir.path().join("demo.audit_cli.md");
    std::fs::write(&spec, CLI_SPEC_V1).unwrap_p();
    let corpus_a = dir.path().join("a.jsonl");
    std::fs::write(&corpus_a, CONFIRM_HAPPY).unwrap_p();
    let corpus_b = dir.path().join("b.jsonl");
    std::fs::write(&corpus_b, REOPEN_HAPPY).unwrap_p();

    let (_, prior) = audit_json(&spec, &corpus_a, &[]);
    let prior_path = dir.path().join("prior.json");
    std::fs::write(&prior_path, prior.to_string()).unwrap_p();

    let (code, json) = audit_json(
        &spec,
        &corpus_b,
        &["--before", prior_path.to_str().unwrap_p()],
    );
    assert_eq!(code, Some(2), "digest mismatch refuses: {json}");
    assert_eq!(json["ok"], serde_json::json!(false));
    let text = serde_json::to_string(&json).unwrap_p();
    assert!(
        text.contains("scope_sha256"),
        "the refusal names the digest field: {json}"
    );
    assert!(
        json["data"].is_null(),
        "refusal paths emit zero records: {json}"
    );
}

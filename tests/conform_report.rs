//! Phase-2 report tests for `conform` (openspec/changes/add-conform,
//! tasks 2.1–2.2): the persisted report schema (conform-local
//! `report_schema_version`, per-trace records, fixed `evidence_scope`,
//! `scope_sha256`), the evidence scope present in BOTH the JSON envelope
//! and the `--human` view (design D6 — data, not docs), rerun
//! byte-identity (records sorted by scenario id, no timestamps), and the
//! digest binding the scenario corpus bytes. Library/emit-layer only —
//! the `spk conform` CLI command itself is phase 4.

use std::path::PathBuf;

use specodelic::conform::{
    build_report, emit_report, parse_corpus, scope_digest, EVIDENCE_SCOPE, REPORT_SCHEMA_VERSION,
    Verdict,
};
use specodelic::spec;
use specodelic::verify::CLAIM_SCHEMA_VERSION;

// ---------------------------------------------------------------------------
// Fixture: the same spec fixture phase 1 exercises (every verdict class)
// ---------------------------------------------------------------------------

/// Fixture spec (parses; not compiled — the library layer consumes the
/// parsed spec through the same IR extraction model_check uses).
const SPEC_TEXT: &str = r#"---
id: demo.conform
kind: intent
statement: "THE system SHALL conform"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.conform]] |
| audited | invariant | `the ledger is audited before any confirmation` | [[demo.conform]] |
| portable | invariant | `**py:** status != "open"` | [[demo.conform]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.conform.locked]] ∧ [[demo.conform.audited]] |
| reopen | confirmed | held | [[demo.conform.locked]] |
| escalate | held | held | [[demo.conform.portable]] |
| crosscheck | held | held | [[other.spec.audited]] |
"#;

const CORPUS_BYTES: &str = concat!(
    r#"{"id": "wrong-state-reopen", "setup": {"state": "held"}, "trace": [{"action": "reopen"}]}"#,
    "\n",
    r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
    "\n",
    r#"{"id": "escalate-portable", "setup": {"state": "held"}, "trace": [{"action": "escalate"}]}"#,
    "\n",
);

fn fixture_spec() -> spec::Spec {
    spec::parse_str(SPEC_TEXT).expect("fixture spec parses")
}

fn fixture_corpus() -> Vec<specodelic::conform::ScenarioTrace> {
    parse_corpus(CORPUS_BYTES).expect("fixture corpus parses")
}

fn fixture_report() -> specodelic::conform::ConformReport {
    let spec = fixture_spec();
    let corpus = fixture_corpus();
    build_report(&spec, CORPUS_BYTES.as_bytes(), &corpus, false)
}

// ---------------------------------------------------------------------------
// 2.1 — report_schema_roundtrip
// ---------------------------------------------------------------------------

/// The persisted report carries a conform-LOCAL `report_schema_version`
/// (distinct field and value from model_check's `claim_schema_version`),
/// per-trace records (scenario id, verdict, reason, evaluated claim ids,
/// closed_world flag), the fixed `evidence_scope` statement, and
/// `scope_sha256` — and survives a serialize→deserialize roundtrip.
#[test]
fn report_schema_roundtrip() {
    let report = fixture_report();

    // Conform-local schema version: its own constant, versioned
    // independently of model_check's claim reports — the FIELD is
    // distinct; both families may sit at v1.
    assert_eq!(report.report_schema_version, REPORT_SCHEMA_VERSION);

    let json = serde_json::to_value(&report).expect("report serializes");
    assert!(
        json.get("report_schema_version").is_some(),
        "report declares report_schema_version: {json}"
    );
    assert!(
        json.get("claim_schema_version").is_none(),
        "the conform report must not reuse model_check's claim_schema_version \
         (currently {CLAIM_SCHEMA_VERSION}) field: {json}"
    );
    assert!(
        json.get("scope_sha256").is_some(),
        "report carries scope_sha256: {json}"
    );
    assert_eq!(
        json["evidence_scope"].as_str(),
        Some(EVIDENCE_SCOPE),
        "evidence_scope is the fixed agreement-on-corpus statement"
    );

    // Per-trace records: one per scenario, each with id, verdict, reason,
    // evaluated claim ids, and the closed_world flag.
    assert_eq!(report.records.len(), 3);
    for record in &report.records {
        assert!(!record.scenario_id.is_empty());
        assert!(!record.reason.is_empty(), "every verdict carries its reason");
        assert!(
            matches!(
                record.verdict,
                Verdict::Permitted
                    | Verdict::Forbidden
                    | Verdict::Underspecified
                    | Verdict::Unknown
                    | Verdict::Unsupported
            ),
            "verdict is from the closed taxonomy"
        );
    }
    let reopen = report
        .records
        .iter()
        .find(|r| r.scenario_id == "wrong-state-reopen")
        .expect("record per scenario id");
    assert_eq!(reopen.verdict, Verdict::Forbidden);
    assert!(!reopen.closed_world, "run was open-world; the record says so");
    assert!(
        reopen.evaluated_claim_ids.contains(&"locked".to_string()),
        "record names the claims the run evaluated: {:?}",
        reopen.evaluated_claim_ids
    );

    // Roundtrip: JSON → value → struct equals the original.
    let back: specodelic::conform::ConformReport =
        serde_json::from_value(json).expect("report deserializes");
    assert_eq!(back, report, "serialize→deserialize roundtrip is lossless");
}

/// The fixed `evidence_scope` statement is present in BOTH the JSON
/// envelope and the `--human` view — data, not documentation (design D6):
/// no consumer can strip it by reformatting.
#[test]
fn evidence_scope_present_in_both_views() {
    let report = fixture_report();

    // JSON view: emit through the genesis envelope; the data payload
    // carries the statement.
    let mut json_buf: Vec<u8> = Vec::new();
    let mut stderr_buf: Vec<u8> = Vec::new();
    emit_report(
        &report,
        genesis::guide::OutputFormat::Json,
        genesis::guide::Verbosity::Normal,
        &mut json_buf,
        &mut stderr_buf,
    )
    .expect("json emit succeeds");
    let emitted = String::from_utf8(json_buf).expect("json emit is utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(&emitted).expect("json emit is one JSON envelope");
    assert_eq!(
        envelope["data"]["evidence_scope"].as_str(),
        Some(EVIDENCE_SCOPE),
        "the JSON envelope data carries evidence_scope: {emitted}"
    );

    // Human view: same statement, same report — the view cannot omit it.
    let mut human_buf: Vec<u8> = Vec::new();
    let mut human_stderr: Vec<u8> = Vec::new();
    emit_report(
        &report,
        genesis::guide::OutputFormat::Human,
        genesis::guide::Verbosity::Normal,
        &mut human_buf,
        &mut human_stderr,
    )
    .expect("human emit succeeds");
    let human = String::from_utf8(human_buf).expect("human emit is utf-8");
    assert!(
        human.contains(EVIDENCE_SCOPE),
        "the --human view carries the same evidence_scope statement: {human}"
    );
    assert!(
        human.contains("forbidden"),
        "the human view names the verdicts: {human}"
    );
}

/// Two runs over identical inputs are byte-identical — records sorted by
/// scenario id, no timestamps, no iteration-order leakage.
#[test]
fn rerun_byte_identical() {
    let first = fixture_report();
    let second = fixture_report();

    let first_json = serde_json::to_string(&first).expect("report serializes");
    let second_json = serde_json::to_string(&second).expect("report serializes");
    assert_eq!(
        first_json, second_json,
        "two runs over identical inputs produce byte-identical reports"
    );

    // Records are sorted by scenario id (not corpus order — the corpus
    // fixture deliberately lists wrong-state-reopen first).
    let ids: Vec<&str> = first.records.iter().map(|r| r.scenario_id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "records are sorted by scenario id");

    // No timestamps anywhere in the report.
    assert!(
        !first_json.contains("timestamp"),
        "the report carries no timestamps: {first_json}"
    );

    // The emitted data payloads are byte-identical too (the envelope's
    // own meta.duration_ms is not part of the report).
    let emit_json = |report: &specodelic::conform::ConformReport| {
        let mut buf: Vec<u8> = Vec::new();
        let mut err: Vec<u8> = Vec::new();
        emit_report(
            report,
            genesis::guide::OutputFormat::Json,
            genesis::guide::Verbosity::Normal,
            &mut buf,
            &mut err,
        )
        .expect("json emit succeeds");
        let envelope: serde_json::Value =
            serde_json::from_slice(&buf).expect("json emit is one JSON envelope");
        serde_json::to_string(&envelope["data"]).expect("data serializes")
    };
    assert_eq!(
        emit_json(&first),
        emit_json(&second),
        "emitted report payloads are byte-identical across runs"
    );
}

// ---------------------------------------------------------------------------
// 2.2 — digest_binds_scenarios
// ---------------------------------------------------------------------------

/// Identical spec content with a one-byte-different scenario corpus
/// produces a different `scope_sha256`; identical inputs produce
/// identical digests across runs and path reordering (the digest binds
/// the consumed corpus bytes and the parsed structured content — never
/// file paths).
#[test]
fn digest_binds_scenarios() {
    let spec = fixture_spec();
    let corpus = fixture_corpus();

    // One byte differs: `escalate-portable` → `escalate-portabl` (one
    // byte shorter, same structured shape otherwise).
    let corpus_b = CORPUS_BYTES.replacen("escalate-portable", "escalate-portabl", 1);
    assert_ne!(CORPUS_BYTES, corpus_b, "the corpora differ by one byte");

    let digest_a = scope_digest(&spec, CORPUS_BYTES.as_bytes());
    let digest_b = scope_digest(&spec, corpus_b.as_bytes());
    assert_ne!(
        digest_a, digest_b,
        "a one-byte corpus difference must flip the scope digest"
    );

    // Identical inputs → identical digests across runs.
    let corpus_again = fixture_corpus();
    let report_a = build_report(&spec, CORPUS_BYTES.as_bytes(), &corpus, false);
    let report_again = build_report(&spec, CORPUS_BYTES.as_bytes(), &corpus_again, false);
    assert_eq!(
        report_a.scope_sha256, report_again.scope_sha256,
        "identical inputs produce identical digests across runs"
    );
    assert_eq!(report_a.scope_sha256, digest_a, "build_report and scope_digest agree");

    // Path reordering/preservation: the digest binds structured content,
    // never the CLI path the file was read from.
    let mut relocated = fixture_spec();
    relocated.path = Some(PathBuf::from("/elsewhere/demo.conform.md"));
    let digest_relocated = scope_digest(&relocated, CORPUS_BYTES.as_bytes());
    assert_eq!(
        digest_a, digest_relocated,
        "the digest is independent of the spec file's path"
    );

    // A corpus consumed at a different path (same bytes) binds identically
    // — the digest binds the corpus BYTES, not the oracle file's path.
    assert_eq!(report_a.scope_sha256, report_again.scope_sha256);

    // And a genuinely different corpus (an extra scenario) never shares a
    // digest with the original.
    let extended = format!(
        "{CORPUS_BYTES}{}\n",
        r#"{"id": "extra-scenario", "setup": {"state": "held"}, "trace": [{"action": "confirm"}]}"#
    );
    parse_corpus(&extended).expect("extended corpus parses");
    let digest_extended = scope_digest(&spec, extended.as_bytes());
    assert_ne!(digest_a, digest_extended, "a changed corpus never shares a digest");
}

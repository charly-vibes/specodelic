//! Phase-3 input-gate tests for `conform` (openspec/changes/add-conform,
//! tasks 3.1–3.4): the command-path gate (design D5 — parse + lint clean +
//! current compiled artifacts, reusing orchestrate's stage discipline),
//! the scenario-corpus validation refusals (design D3 — malformed lines,
//! missing/duplicate ids, unknown fields are refused with remediation
//! hints, never silently ignored), and the zero-line corpus (a valid
//! report with zero records, `evidence_scope` intact — never an error).
//! Library layer only — the `spk conform` CLI command itself is phase 4.

use std::path::PathBuf;

use specodelic::conform::{
    gate, parse_corpus, run, ConformError, EVIDENCE_SCOPE, REPORT_SCHEMA_VERSION,
};
use specodelic::spec;
use specodelic::{compile, orchestrate};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A lint-clean, compilable fixture spec (subset of the phase-1 fixture:
/// rust fragment claim only — py/ts fragments are emitter-gated at
/// compile validation and citations to foreign specs stay out so the
/// single-file gate can pass).
const SPEC_TEXT: &str = r#"---
id: demo.gate
kind: intent
statement: "THE system SHALL conform"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.gate]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.gate.locked]] |
"#;

/// The same spec with edited structured content (a second transition) —
/// compiled artifacts produced before this edit are stale.
const SPEC_TEXT_EDITED: &str = r#"---
id: demo.gate
kind: intent
statement: "THE system SHALL conform"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.gate]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.gate.locked]] |
| reopen | confirmed | held | [[demo.gate.locked]] |
"#;

/// A spec that parses but fails the lint gate (statement carries no
/// EARS `SHALL` form — the ears_syntax checker refuses it).
const LINT_DIRTY_TEXT: &str = r#"---
id: demo.dirty
kind: intent
statement: "the system conforms to the ledger"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| locked | invariant | `**rust:** status != "open"` | [[demo.dirty]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.dirty.locked]] |
"#;

/// Write spec text to a temp file, parse it back (path set — the artifact
/// stem derives from it), and compile its artifacts into the temp
/// out-dir. Returns (parsed spec with path, out-dir path).
fn compiled_fixture(dir: &std::path::Path, text: &str) -> (spec::Spec, PathBuf) {
    let file = dir.join("demo.gate.md");
    std::fs::write(&file, text).expect("fixture spec writes");
    let mut spec = spec::parse_str(text).expect("fixture spec parses");
    spec.path = Some(file.clone());
    let out_dir = dir.join("out");
    let compiled = compile::compile_spec(&spec).expect("fixture spec compiles");
    compile::write_artifacts(&spec, &compiled, out_dir.to_str().expect("utf8 out-dir"))
        .expect("fixture artifacts write");
    (spec, out_dir)
}

const CORPUS: &str = concat!(
    r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}"#,
    "\n",
);

// ---------------------------------------------------------------------------
// 3.1/3.2 — artifact gate (D5)
// ---------------------------------------------------------------------------

/// Stale artifacts refused: editing the spec's structured content after
/// compile makes the on-disk `.tla` non-current — the gate refuses with a
/// remediation hint naming `spk compile`, and the run emits ZERO verdict
/// records (an Err, never a report).
#[test]
fn stale_artifacts_refused() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let (original, out_dir) = compiled_fixture(dir.path(), SPEC_TEXT);

    // Sanity: with current artifacts the gate passes.
    gate(&original, &out_dir).expect("gate passes over current artifacts");

    // Edit the structured content WITHOUT recompiling.
    let file = original.path.as_ref().expect("fixture path").clone();
    std::fs::write(&file, SPEC_TEXT_EDITED).expect("spec edit writes");
    let edited = spec::parse_str(SPEC_TEXT_EDITED).expect("edited spec parses");
    let edited = spec::Spec {
        path: Some(file),
        ..edited
    };

    let err: ConformError = run(&edited, &out_dir, CORPUS.as_bytes(), false)
        .expect_err("stale artifacts must be refused");
    assert!(
        err.message.contains("spk compile"),
        "refusal names the compile remediation: {err}"
    );
    assert!(
        err.message.contains("stale") || err.message.contains("current"),
        "refusal states the staleness fact: {err}"
    );
}

/// Missing artifacts refused too: conform never compiles (read-only over
/// artifacts) — an invocation without compiled artifacts is a refusal
/// naming `spk compile`, never a silent run.
#[test]
fn missing_artifacts_refused() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let (spec, _out_dir) = compiled_fixture(dir.path(), SPEC_TEXT);
    let empty_out = dir.path().join("nowhere");

    let err = run(&spec, &empty_out, CORPUS.as_bytes(), false)
        .expect_err("missing artifacts must be refused");
    assert!(
        err.message.contains("spk compile"),
        "refusal names the compile remediation: {err}"
    );
}

/// Lint-dirty file refused: a spec with lint findings never reaches
/// classification — the gate refuses with a remediation hint naming
/// `spk lint`, and the run emits ZERO verdict records.
#[test]
fn lint_dirty_refused() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let file = dir.path().join("demo.dirty.md");
    std::fs::write(&file, LINT_DIRTY_TEXT).expect("fixture spec writes");
    let mut spec = spec::parse_str(LINT_DIRTY_TEXT).expect("dirty fixture parses");
    spec.path = Some(file.clone());
    let out_dir = dir.path().join("out");

    // The lint stage itself flags the fixture (orchestrate's own gate
    // discipline — the gate reuses the same stage, not a re-derivation).
    let stage = orchestrate::run_lint_stage(std::slice::from_ref(&spec), &[]);
    assert_eq!(stage.status, "failed", "fixture is lint-dirty by the shared stage");

    let err = run(&spec, &out_dir, CORPUS.as_bytes(), false)
        .expect_err("lint-dirty file must be refused");
    assert!(
        err.message.contains("spk lint"),
        "refusal names the lint remediation: {err}"
    );
}

/// The gate is part of the run path: a lint-clean, current run succeeds
/// and the report carries the schema contract (the happy-path control
/// for the refusal tests above).
#[test]
fn gated_run_produces_the_report() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let (spec, out_dir) = compiled_fixture(dir.path(), SPEC_TEXT);

    let report = run(&spec, &out_dir, CORPUS.as_bytes(), false).expect("gated run succeeds");
    assert_eq!(report.report_schema_version, REPORT_SCHEMA_VERSION);
    assert_eq!(report.evidence_scope, EVIDENCE_SCOPE);
    assert_eq!(report.records.len(), 1);
}

// ---------------------------------------------------------------------------
// 3.3 — scenario corpus validation (D3)
// ---------------------------------------------------------------------------

/// A malformed JSONL line is refused with a remediation hint — never
/// silently ignored.
#[test]
fn malformed_line_refused() {
    let err = parse_corpus("{not json").expect_err("malformed line refused");
    assert!(
        err.message.contains("one JSON object per line"),
        "refusal carries the remediation hint: {err}"
    );
}

/// A record with no `id` field is refused with a remediation hint.
#[test]
fn missing_id_refused() {
    let err = parse_corpus(r#"{"setup": {"state": "held"}, "trace": [{"action": "confirm"}]}"#)
        .expect_err("missing id refused");
    assert!(
        err.message.contains("missing field `id`"),
        "refusal names the missing id: {err}"
    );
    assert!(
        err.message.contains("one JSON object per line") || err.message.contains("remediation"),
        "refusal carries a remediation hint: {err}"
    );
}

/// Two records sharing one id are refused — duplicate ids are never
/// silently accepted (the report is keyed by scenario id).
#[test]
fn duplicate_id_refused() {
    let corpus = concat!(
        r#"{"id": "same", "trace": [{"action": "confirm"}]}"#,
        "\n",
        r#"{"id": "same", "trace": [{"action": "confirm"}]}"#,
        "\n",
    );
    let err = parse_corpus(corpus).expect_err("duplicate id refused");
    assert!(
        err.message.contains("same"),
        "refusal names the duplicated id: {err}"
    );
    assert!(
        err.message.to_lowercase().contains("unique") || err.message.contains("duplicate"),
        "refusal states the uniqueness rule: {err}"
    );
}

/// An unknown field is refused with a remediation hint — never silently
/// ignored (design D3; envelope discipline).
#[test]
fn unknown_field_refused() {
    let err = parse_corpus(
        r#"{"id": "ok", "trace": [{"action": "confirm"}], "expected": "confirmed"}"#,
    )
    .expect_err("unknown field refused");
    assert!(
        err.message.contains("one JSON object per line") || err.message.contains("unknown"),
        "refusal carries the remediation hint: {err}"
    );
}

// ---------------------------------------------------------------------------
// 3.4 — zero-line corpus
// ---------------------------------------------------------------------------

/// A zero-line corpus is a valid run: the report carries zero records,
/// `evidence_scope` intact, and the run is Ok — never an error.
#[test]
fn empty_corpus_yields_valid_report() {
    let dir = tempfile::tempdir_in(std::env::temp_dir()).expect("tempdir");
    let (spec, out_dir) = compiled_fixture(dir.path(), SPEC_TEXT);

    let report = run(&spec, &out_dir, b"", false).expect("empty corpus is a valid run");
    assert_eq!(report.records.len(), 0, "zero records");
    assert_eq!(report.evidence_scope, EVIDENCE_SCOPE);
    assert_eq!(report.report_schema_version, REPORT_SCHEMA_VERSION);
    // The digest still binds the (empty) corpus bytes.
    assert!(!report.scope_sha256.is_empty());
    assert!(report.verdict_counts.is_empty(), "no verdict counts: {:?}",
        report.verdict_counts);
}

/// parse_corpus over a zero-line corpus is Ok(vec![]) — the emptiness is
/// the caller's report, never a parse error.
#[test]
fn empty_corpus_parses_to_zero_scenarios() {
    let corpus = parse_corpus("").expect("empty corpus parses");
    assert!(corpus.is_empty());
}

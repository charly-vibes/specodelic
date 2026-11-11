//! Slice-4 CLI-level pin for `audit` (openspec/changes/add-evidence-audit,
//! tasks 4.1 red): the per-file ladder-rung aggregate rides the real
//! binary end-to-end — the delta's "Evidence rung aggregate" scenarios
//! over the actual gate + summary + emit path. The gate's lint stage
//! establishes "lint-clean" for real; a current `<stem>.check.json`
//! stands in for the model-check run a prior `spk model-check` wrote
//! (same artifact_sha256 staleness key verify's model gate checks).
//! The derivation pins live in `src/audit.rs` (`mod rung_tests`);
//! this file pins what only the CLI can: the rung surfacing in BOTH
//! output views of a real run.
//!
//! Rationale: lint/orchestrate stay byte-identical (hard scope guard) —
//! this file edits nothing but its own fixtures; audit is read-only
//! over the artifacts it consumes.

#[path = "common/mod.rs"]
mod common;

use assert_cmd::Command;
use common::UnwrapPanic;
use specodelic::audit::{
    self, AuditClass, AuditRecord, AuditReport, EvidenceMix, PackCheck, RungAggregate, fold_rows,
    human_view, with_derived_rung,
};
use specodelic::model_check;
use specodelic::spec;
use std::collections::BTreeMap;

/// Lint-clean modelful fixture (the gate requires it): the fragment
/// `status != "open"` cites no declared element, so the run also
/// carries a binds_nothing row — the honesty intersection pinned
/// end-to-end: the advisory row exists, counts, and must NOT lift or
/// cap the rung.
const SPEC: &str = r#"---
id: demo.rung_cli
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|------------|
| locked | invariant | `**rust:** status != "open"` | [[demo.rung_cli]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.rung_cli.locked]] |
"#;

/// The Properties row the coverage checker demands for lint-clean.
const PROPS: &str = r#"

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[demo.rung_cli.locked]] | `g()` | `x` |
"#;

/// One permitted oracle row (the fragment holds against the final
/// observation).
const CORPUS: &str = r#"{"id": "confirm-happy", "setup": {"state": "held"}, "trace": [{"action": "confirm", "observations": {"status": "confirmed"}}]}
"#;

fn spk() -> Command {
    Command::cargo_bin("specodelic").unwrap_p()
}

// ---------------------------------------------------------------------------
// Derivation pins (tasks 4.1–4.2, moved from the in-module
// `rung_tests`: the pretender file_lines ratchet on src/audit.rs
// keeps the module lean, and these pins ride the same pub surface the
// layer's p_rung `arbitrary_evidence_mix()` generator will reach).
// The aggregate is DERIVED at report time from evidence actually
// present and never exceeds it (design D3 "derived, not stored"; the
// delta's "Evidence rung aggregate" scenarios; the layer row
// `rung_derived`). Ladder semantics:
// docs/src/verification-boundaries.md — 1 lint-clean, 2 well-modeled,
// 3 checked-within-bounds, 4 matches-oracle, 5 better-design.
// ---------------------------------------------------------------------------

/// A shell report carrying only the pins' load-bearing fields.
fn report() -> AuditReport {
    AuditReport {
        report_schema_version: audit::AUDIT_SCHEMA_VERSION,
        evidence_scope: String::new(),
        scope_sha256: "digest".to_string(),
        expectation_declared: false,
        verdict_counts: BTreeMap::new(),
        class_counts: BTreeMap::new(),
        ladder_rung: RungAggregate::default(),
        records: vec![],
    }
}

fn record(id: &str, class: AuditClass) -> AuditRecord {
    AuditRecord {
        scenario_id: id.to_string(),
        class,
        conform_verdict: specodelic::conform::Verdict::Permitted,
        reason: "fixture".to_string(),
        evaluated_claim_ids: Vec::new(),
        closed_world: false,
    }
}

/// Parse a fixture spec (path-less — fine for the pure-core and
/// tmpdir pins; the CLI pins below parse from the file).
fn parse(text: &str) -> spec::Spec {
    spec::parse_str(text).expect("fixture spec parses")
}

/// Modelful fixture: nonempty States + Transitions (the rung-2
/// evidence, lint's `model_present` predicate), and a constraint
/// fragment that cites the declared state `held` — no bind-row
/// noise.
const MODELFUL: &str = r#"---
id: demo.rung_modelful
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|------------|
| locked | invariant | `**rust:** state == "held"` | [[demo.rung_modelful]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.rung_modelful.locked]] |
"#;

/// Modelful fixture whose fragment cites nothing declared (the
/// bowling-rev1 shape — `status != "open"`): bind rows EXIST and
/// the honesty pins need them.
const MODELFUL_VACUOUS: &str = r#"---
id: demo.rung_vacuous
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|------------|
| locked | invariant | `**rust:** status != "open"` | [[demo.rung_vacuous]] |

## Model

### States

- held
- confirmed

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| confirm | held | confirmed | [[demo.rung_vacuous.locked]] |
"#;

/// Model-free fixture: constraints only. Never lint-clean at the
/// CLI (lint's `model_present` checker refuses it, so the gate
/// never emits a report for it) — the floor shape is the pure
/// core's and the prior-parse serde default's, and the pins say
/// so.
const MODEL_FREE: &str = r#"---
id: demo.rung_free
kind: intent
statement: "THE system SHALL keep the ledger consistent"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|------------|
| locked | invariant | `**rust:** status != "open"` | [[demo.rung_free]] |
"#;

fn mix(
    model_present: bool,
    model_check: Option<model_check::Outcome>,
    verify_pass: bool,
) -> EvidenceMix {
    EvidenceMix {
        model_present,
        model_check,
        verify_pass,
    }
}

/// The delta scenario "Rung matches evidence", verbatim: a file
/// lint-clean but only model-checked, with no verify pass, is
/// rung 3 — and never 4 or 5.
#[test]
fn model_checked_only_reports_rung_3_never_4_or_5() {
    let rung = audit::rung_from_evidence(&mix(
        true,
        Some(model_check::Outcome::NoCounterexample),
        false,
    ));
    assert_eq!(rung.rung, 3, "model-checked-only is rung 3");
    assert_ne!(rung.rung, 4, "never 4 without verify evidence");
    assert_ne!(rung.rung, 5, "never 5 without counterexample evidence");
    assert!(
        rung.basis.contains("no_counterexample"),
        "the basis names the evidence that lifted it: {}",
        rung.basis
    );
}

/// The delta scenario "Rung never claims without evidence",
/// verbatim: no audit, conform, or verify evidence at all is rung
/// 1 (lint-clean at most), stated as such.
#[test]
fn no_evidence_reports_rung_1_stated_as_such() {
    let rung = audit::rung_from_evidence(&mix(false, None, false));
    assert_eq!(rung.rung, 1);
    assert!(
        rung.basis.contains("at most"),
        "the floor is stated as lint-clean AT MOST, never a claim: {}",
        rung.basis
    );
}

/// The full mix table (p_rung's `arbitrary_evidence_mix()` shape):
/// rung == highest evidence-backed rung over arbitrary mixes.
/// `exploration_only` and `timed_out` are rung-3's own honest
/// assurances ("found no violation within the bound — or honestly
/// reported it could not finish"), never 4/5;
/// `counterexample_found` is rung-5 evidence (the design lesson);
/// a verify pass alone is evidence-backed rung 4.
#[test]
fn rung_is_the_highest_evidence_backed_rung_across_mixes() {
    assert_eq!(audit::rung_from_evidence(&mix(true, None, false)).rung, 2);
    assert_eq!(
        audit::rung_from_evidence(&mix(
            true,
            Some(model_check::Outcome::ExplorationOnly),
            false
        ))
        .rung,
        3,
        "exploration_only is checked-within-bounds, never clean-or-higher"
    );
    assert_eq!(
        audit::rung_from_evidence(&mix(true, Some(model_check::Outcome::TimedOut), false)).rung,
        3,
        "timed_out honestly reported non-finish is rung 3"
    );
    assert_eq!(
        audit::rung_from_evidence(&mix(
            true,
            Some(model_check::Outcome::NoCounterexample),
            true
        ))
        .rung,
        4,
        "a verify pass lifts to matches-oracle"
    );
    assert_eq!(
        audit::rung_from_evidence(&mix(
            true,
            Some(model_check::Outcome::CounterexampleFound),
            false
        ))
        .rung,
        5,
        "a counterexample is better-design evidence"
    );
    assert_eq!(
        audit::rung_from_evidence(&mix(
            true,
            Some(model_check::Outcome::CounterexampleFound),
            true
        ))
        .rung,
        5,
        "highest-evidence semantics: the counterexample still tops the mix"
    );
    assert_eq!(
        audit::rung_from_evidence(&mix(false, None, true)).rung,
        4,
        "verify-only is evidence-backed rung 4 (real inputs always chain the lower rungs)"
    );
}

/// The honesty intersection (layer `honesty_channels` ×
/// `rung_derived`): unchecked and binds_nothing rows are
/// countable, NEVER verified — they must not lift the rung. A
/// report full of vacuous advisory rows derives exactly what the
/// spec+artifact evidence alone supports — modelful ⇒ 2 (model
/// presence only, never 3+), model-free ⇒ the floor 1 — even
/// though the rows EXIST and count in the summary.
#[test]
fn vacuous_rows_never_lift_the_rung() {
    let dir = tempfile::tempdir().unwrap_p();
    let vacuous = parse(MODELFUL_VACUOUS);
    let free = parse(MODEL_FREE);
    let rows = [
        audit::bind_rows(&vacuous),
        audit::bind_rows(&free),
        audit::pack_check_rows(&[PackCheck {
            pack_id: "packs.a".to_string(),
            subject_id: "demo.rung_vacuous".to_string(),
            declared_rules: vec!["packs.a.t".to_string()],
            violations: vec![],
        }]),
    ]
    .concat();
    assert!(!rows.is_empty(), "the vacuous rows exist and count");
    let lifted = with_derived_rung(fold_rows(report(), rows.clone()), &vacuous, dir.path());
    assert_eq!(
        lifted.ladder_rung.rung, 2,
        "model presence alone lifts; the vacuous rows add nothing"
    );
    assert!(
        lifted.ladder_rung.rung < 3,
        "never checked-within-bounds without check evidence"
    );
    let floored = with_derived_rung(fold_rows(report(), rows), &free, dir.path());
    assert_eq!(
        floored.ladder_rung.rung, 1,
        "no model, no current check, no verify: the floor, whatever the row count says"
    );
}

/// Staleness (design D3: CURRENT evidence): a
/// `<stem>.check.json` whose `artifact_sha256` matches the
/// on-disk `<stem>.tla` lifts to 3; a stale or missing one does
/// not lift. (Path-less parse here: the pins write the `.tla`
/// under the SAME stem artifact_stem derives, so reader and pin
/// agree on the names.)
#[test]
fn stale_check_report_does_not_lift_current_one_does() {
    let dir = tempfile::tempdir().unwrap_p();
    let sp = parse(MODELFUL);
    let stem = specodelic::compile::artifact_stem(&sp);
    let tla = dir.path().join(format!("{stem}.tla"));
    std::fs::write(&tla, b"module demo").unwrap_p();
    let check = dir.path().join(format!("{stem}.check.json"));
    let stored = |sha: String| model_check::RunReport {
        backend: model_check::Backend {
            engine: "stateright".to_string(),
            version: "0".to_string(),
        },
        bound: model_check::Bound::default(),
        outcome: model_check::Outcome::NoCounterexample,
        invariants_checked: vec!["locked".to_string()],
        invariant_statuses: vec![],
        violated_invariant_id: None,
        trace: None,
        artifact_sha256: sha,
        states_explored: 3,
    };
    std::fs::write(
        &check,
        serde_json::to_vec(&stored("a-stale-sha".to_string())).unwrap_p(),
    )
    .unwrap_p();
    let stale = with_derived_rung(report(), &sp, dir.path());
    assert_eq!(
        stale.ladder_rung.rung, 2,
        "stale check evidence never lifts — the model run is not current"
    );
    let current_sha = model_check::artifact_sha256(b"module demo");
    std::fs::write(&check, serde_json::to_vec(&stored(current_sha)).unwrap_p()).unwrap_p();
    let current = with_derived_rung(report(), &sp, dir.path());
    assert_eq!(current.ladder_rung.rung, 3, "current check evidence lifts");
    assert!(
        current.ladder_rung.basis.contains("no_counterexample"),
        "the basis names the outcome: {}",
        current.ladder_rung.basis
    );
}

/// The rung is derived, never stored (design D3): a prior
/// report's rung never drives the current one. An OLD prior
/// (written before this field existed) parses via the serde
/// default and diffs without touching the current report's
/// derived rung; a new-shape prior round-trips the field
/// verbatim.
#[test]
fn prior_rung_never_drives_the_current_rung() {
    // An old-shape prior: serialize a report, strip the field —
    // the exact bytes a pre-slice-4 `spk audit --json` wrote.
    let mut old = report();
    old.records = vec![record("confirm-happy", AuditClass::Current)];
    let mut value = serde_json::to_value(&old).unwrap_p();
    value
        .as_object_mut()
        .expect("report serializes to an object")
        .remove("ladder_rung");
    let prior = audit::parse_prior_report(&serde_json::to_vec(&value).unwrap_p())
        .expect("an old-shape prior parses");
    assert_eq!(
        prior.ladder_rung,
        RungAggregate::default(),
        "old priors parse at the serde-default floor"
    );
    // The current report carries its OWN derived rung; diffing
    // must not let the prior's floor drive it.
    let mut current = report();
    current.records = old.records.clone();
    current.ladder_rung = RungAggregate {
        rung: 3,
        basis: "checked-within-bounds".to_string(),
    };
    let diffed = audit::diff_with_prior(current, &prior).expect("same digest diffs");
    assert_eq!(
        diffed.ladder_rung.rung, 3,
        "the prior's rung never drives the current derivation"
    );
    // A new-shape prior round-trips the field verbatim.
    let bytes = serde_json::to_vec(&diffed).unwrap_p();
    let parsed = audit::parse_prior_report(&bytes).expect("new-shape prior parses");
    assert_eq!(parsed.ladder_rung, diffed.ladder_rung);
}

/// Both views from the same report values (the single rendering
/// path discipline): the rung surfaces in the human view exactly
/// as the envelope carries it, so the views cannot drift.
#[test]
fn human_view_surfaces_the_rung() {
    let mut rpt = report();
    rpt.ladder_rung = RungAggregate {
        rung: 3,
        basis: "checked-within-bounds: current model-check run reported no_counterexample"
            .to_string(),
    };
    let human = human_view(&rpt);
    assert!(
        human.contains("ladder_rung: 3 — checked-within-bounds"),
        "the human view carries the rung and its basis: {human}"
    );
}

// ---------------------------------------------------------------------------
// CLI pins — the delta scenarios through the real binary
// ---------------------------------------------------------------------------

fn write_spec(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("demo.rung_cli.md");
    std::fs::write(&path, format!("{SPEC}{PROPS}")).unwrap_p();
    path
}

fn write_corpus(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("corpus.jsonl");
    std::fs::write(&path, CORPUS).unwrap_p();
    path
}

fn compile(dir: &std::path::Path, spec_path: &std::path::Path) -> String {
    let out_dir = dir.join("out");
    spk()
        .args([
            "compile",
            spec_path.to_str().unwrap_p(),
            "--json",
            "--out-dir",
            out_dir.to_str().unwrap_p(),
        ])
        .assert()
        .success();
    out_dir.to_str().unwrap_p().to_string()
}

/// Stand in for a prior `spk model-check` run: write a
/// `<stem>.check.json` whose `artifact_sha256` matches the on-disk
/// `<stem>.tla` exactly (the current-evidence key).
fn write_current_check(out_dir: &str, spec_path: &std::path::Path) {
    // Parse FROM THE FILE: artifact_stem keys on the file stem
    // (`demo.rung_cli`), which is the name compile wrote the `.tla`
    // under — a path-less parse would fall back to the dash-form id
    // and miss the artifact.
    let sp = specodelic::parse::parse_file(spec_path).expect("fixture spec parses");
    let stem = specodelic::compile::artifact_stem(&sp);
    let tla = std::path::Path::new(out_dir).join(format!("{stem}.tla"));
    let sha = model_check::artifact_sha256(&std::fs::read(&tla).unwrap_p());
    let report = model_check::RunReport {
        backend: model_check::Backend {
            engine: "stateright".to_string(),
            version: "0".to_string(),
        },
        bound: model_check::Bound::default(),
        outcome: model_check::Outcome::NoCounterexample,
        invariants_checked: vec!["locked".to_string()],
        invariant_statuses: vec![],
        violated_invariant_id: None,
        trace: None,
        artifact_sha256: sha,
        states_explored: 3,
    };
    let check = std::path::Path::new(out_dir).join(format!("{stem}.check.json"));
    std::fs::write(&check, serde_json::to_vec(&report).unwrap_p()).unwrap_p();
}

/// Delta scenario "Rung matches evidence", through the real binary:
/// lint-clean (the gate ran), model-checked (a current check report),
/// no verify pass ⇒ the envelope's summary reports rung 3 — never 4
/// or 5 — and the binds_nothing advisory row present in the same
/// report neither lifts nor caps it. The human view carries the same
/// rung from the same report values.
#[test]
fn audit_cli_model_checked_only_reports_rung_3_in_both_views() {
    let dir = tempfile::tempdir().unwrap_p();
    let spec_path = write_spec(dir.path());
    let corpus = write_corpus(dir.path());
    let out_dir = compile(dir.path(), &spec_path);
    write_current_check(&out_dir, &spec_path);
    let out = spk()
        .args([
            "audit",
            spec_path.to_str().unwrap_p(),
            "--oracle",
            corpus.to_str().unwrap_p(),
            "--out-dir",
            &out_dir,
            "--json",
        ])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(0), "no contradicted rows");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    let rung = &json["data"]["ladder_rung"];
    assert_eq!(rung["rung"], 3, "model-checked-only is rung 3: {json}");
    assert_ne!(rung["rung"], 4);
    assert_ne!(rung["rung"], 5);
    assert!(
        rung["basis"]
            .as_str()
            .is_some_and(|b| b.contains("no_counterexample")),
        "the basis names the lifting evidence: {rung}"
    );
    // The honesty intersection, end-to-end: the vacuous fragment's
    // binds_nothing row counts — and the rung is still exactly 3.
    assert_eq!(
        json["data"]["class_counts"]["binds_nothing"], 1,
        "the advisory row exists and counts: {json}"
    );
    // The human view: same report values, same rung.
    let human = spk()
        .args([
            "audit",
            spec_path.to_str().unwrap_p(),
            "--oracle",
            corpus.to_str().unwrap_p(),
            "--out-dir",
            &out_dir,
            "--human",
        ])
        .output()
        .unwrap_p();
    let stdout = String::from_utf8_lossy(&human.stdout);
    assert!(
        stdout.contains("ladder_rung: 3"),
        "the human view carries the rung: {stdout}"
    );
}

/// The floor above the gate: lint-clean and audited (current oracle
/// evidence) but NO model-check run ⇒ rung 2 — the model section is
/// present, and nothing claims checked-within-bounds. (Rung 1 is the
/// no-evidence-at-all floor, unreachable through the CLI because the
/// gate's lint-clean implies model presence — pinned at the pure core
/// in `src/audit.rs`.)
#[test]
fn audit_cli_without_check_evidence_reports_rung_2_never_higher() {
    let dir = tempfile::tempdir().unwrap_p();
    let spec_path = write_spec(dir.path());
    let corpus = write_corpus(dir.path());
    let out_dir = compile(dir.path(), &spec_path);
    let out = spk()
        .args([
            "audit",
            spec_path.to_str().unwrap_p(),
            "--oracle",
            corpus.to_str().unwrap_p(),
            "--out-dir",
            &out_dir,
            "--json",
        ])
        .output()
        .unwrap_p();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_p();
    assert_eq!(
        json["data"]["ladder_rung"]["rung"], 2,
        "lint+oracle evidence alone is well-modeled at most: {json}"
    );
    assert_ne!(json["data"]["ladder_rung"]["rung"], 3);
    assert_ne!(json["data"]["ladder_rung"]["rung"], 4);
    assert_ne!(json["data"]["ladder_rung"]["rung"], 5);
}

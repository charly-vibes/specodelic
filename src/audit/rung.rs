//! audit::rung — the ladder-rung aggregate submodule (add-evidence-audit,
//! slice 4, tasks 4.1–4.3).
//!
//! Purpose: the per-file verification-ladder rung summary — derived at
//! report time from evidence actually present, never stored state
//! (design D3).
//! Responsibilities: the [`RungAggregate`] type (rung 1–5 plus a stated
//! basis; the serde-default floor keeps reports written before this
//! field existed parseable as `--before` priors); the pure derivation
//! core [`rung_from_evidence`] over [`EvidenceMix`] (arbitrary evidence
//! mixes checkable without a filesystem — the p_rung
//! `arbitrary_evidence_mix()` seam); the on-disk current-evidence
//! collector `current_check_evidence` (reusing model_check's
//! `artifact_sha256` staleness key — no new staleness machinery); and
//! [`with_derived_rung`], which attaches the derivation to the merged
//! report before any `--before` diff (the diff never re-derives and
//! never reads a prior report's rung).
//! Rationale: relocated verbatim from src/audit.rs (specodelic-6hou,
//! tasks.md 4.3) — the pretender file_lines ratchet had that module at
//! 2294/2300, and a cohesive-unit module split is the sanctioned shape
//! (the acset module shows the pattern). Re-exported from `audit`
//! (`pub use rung::…`) so every `audit::<name>` path — the
//! tests/audit_rung.rs pins, the CLI handler, lib consumers — is
//! unchanged; zero behavior change, proven by byte-identical
//! `spk audit` envelopes over frozen fixtures.

use serde::{Deserialize, Serialize};

use super::AuditReport;
use crate::model_check;
use crate::spec::Spec;
// ---------------------------------------------------------------------------
// The ladder-rung aggregate (slice 4 — tasks 4.1–4.2, supersedes 4m3y):
// the per-file verification-ladder rung, DERIVED at report time from
// evidence actually present — never stored state, never staleness
// machinery, never exceeding the evidence (design D3 "derived, not
// stored"). Ladder semantics: docs/src/verification-boundaries.md —
// 1 lint-clean, 2 well-modeled, 3 checked-within-bounds, 4
// matches-oracle, 5 better-design. Honesty intersection (layer
// `honesty_channels` × `rung_derived`): unchecked/binds_nothing rows
// are countable, NEVER verified — they never lift a rung.
// ---------------------------------------------------------------------------

/// The summary's ladder-rung aggregate: a rung number (1–5) plus the
/// evidence basis that lifted it, stated (the floor reads "lint-clean
/// at most", never a claim). Serialized inside the audit report; the
/// [`Default`] is the no-evidence floor — the serde default that keeps
/// reports written before this field parseable as `--before` priors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RungAggregate {
    /// The ladder rung, 1–5 (docs/src/verification-boundaries.md).
    pub rung: u8,
    /// What the evidence says — the basis the derivation lifted the
    /// rung on, stated so the number is never an undifferentiated
    /// PASS.
    pub basis: String,
}

impl Default for RungAggregate {
    /// The floor: rung 1, stated as "at most" — the value old reports
    /// parse with and the no-evidence derivation lands on.
    fn default() -> Self {
        RungAggregate {
            rung: 1,
            basis: "no current evidence beyond the floor — lint-clean at most, stated as such"
                .to_string(),
        }
    }
}

/// The snake_case outcome name (the serde spelling records use) — a
/// private local map because [`model_check::Outcome`] has no Display
/// and the basis must not carry Debug formatting.
fn outcome_name(outcome: model_check::Outcome) -> &'static str {
    match outcome {
        model_check::Outcome::NoCounterexample => "no_counterexample",
        model_check::Outcome::CounterexampleFound => "counterexample_found",
        model_check::Outcome::TimedOut => "timed_out",
        model_check::Outcome::ExplorationOnly => "exploration_only",
    }
}

/// The evidence inputs the rung derivation reads. IO collects them
/// ([`with_derived_rung`]); the pure core
/// ([`rung_from_evidence`]) decides — so arbitrary evidence mixes are
/// checkable without a filesystem (p_rung's `arbitrary_evidence_mix()`
/// shape). `pub` because the pins live in `tests/audit_rung.rs` (the
/// pretender file_lines ratchet keeps this module lean) and the layer
/// property generator will reach the same seam.
pub struct EvidenceMix {
    /// The Model section is present with both sections nonempty — the
    /// same predicate lint's `model_present` checker gates on, read
    /// from the spec the walk already has.
    pub model_present: bool,
    /// The CURRENT model-check run's outcome (a
    /// `<stem>.check.json` whose `artifact_sha256` matches the
    /// current `<stem>.tla`); stale, foreign, or missing ⇒ `None` —
    /// a run report that is not current evidence never lifts.
    pub model_check: Option<model_check::Outcome>,
    /// A verify pass (the matches-oracle rung's evidence).
    ///
    /// V1 HONESTY BOUNDARY: verify persists no report — its verdict is
    /// re-evaluated live every run ("verified is not cached"), and the
    /// properties blocks' pass results exist nowhere on disk for the
    /// audit to read. No audit input can carry verify evidence today,
    /// so the IO collector NEVER sets this — rung 4 is unreachable
    /// through `spk audit` in v1, and the aggregate never claims it
    /// from evidence it cannot see. The pure core still accepts the
    /// input (a future persisted verify report is one collector
    /// branch, not a redesign), and the delta's scenario stays honest:
    /// a rung without its backing evidence is impossible.
    pub verify_pass: bool,
}

/// The pure derivation core (tasks 4.2, design D3): the rung is the
/// HIGHEST rung the mix carries evidence for, floor 1 — never
/// exceeding the evidence, never reading anything else. Unchecked and
/// binds_nothing rows do not appear here at all: advisory rows are
/// countable, never verified, so they cannot lift a rung by
/// construction, not by per-site discipline.
pub fn rung_from_evidence(mix: &EvidenceMix) -> RungAggregate {
    // Rung 5 — better-design: a counterexample is the design lesson
    // (the ladder's highest-value rung, above a clean verify).
    if mix.model_check == Some(model_check::Outcome::CounterexampleFound) {
        return RungAggregate {
            rung: 5,
            basis: "better-design: the current model-check run reported counterexample_found — the refuted claim names the design lesson".to_string(),
        };
    }
    // Rung 4 — matches-oracle.
    if mix.verify_pass {
        return RungAggregate {
            rung: 4,
            basis: "matches-oracle: a verify pass is present — the compiled property blocks executed and passed on the current model".to_string(),
        };
    }
    // Rung 3 — checked-within-bounds. All three non-counterexample
    // outcomes are rung-3's own honest assurances: "found no violation
    // within the stated bound — or honestly reported it could not
    // finish". `exploration_only` is a completed exploration with zero
    // predicates executed — bounded exploration, never clean, never
    // 4/5; `timed_out` is the honest non-finish.
    if let Some(outcome) = mix.model_check {
        return RungAggregate {
            rung: 3,
            basis: format!(
                "checked-within-bounds: current model-check run reported {}",
                outcome_name(outcome)
            ),
        };
    }
    // Rung 2 — well-modeled (lint's model_present predicate).
    if mix.model_present {
        return RungAggregate {
            rung: 2,
            basis: "well-modeled: the Model section is present (states + transitions declared); no current model-check run".to_string(),
        };
    }
    // Rung 1 — the floor, stated as such (never a claim).
    RungAggregate::default()
}

/// The on-disk current-evidence reader: the model-check run report a
/// prior `spk model-check` persisted beside the compiled module
/// (`<stem>.check.json` + `<stem>.tla` in the audit's out-dir — the
/// same artifacts conform's gate and verify's model gate consume).
/// Returns the outcome ONLY when the stored run is current evidence:
/// its `artifact_sha256` must equal the sha of the on-disk `.tla`
/// (the same staleness key `model_check::is_stale` and verify's model
/// gate apply) — stale, foreign, unreadable, or missing reports are
/// `None` (never evidence, never an error: absence of a check run is
/// the honest floor, not a failure — the audit grades evidence, it
/// does not demand it).
fn current_check_evidence(spec: &Spec, out_dir: &std::path::Path) -> Option<model_check::Outcome> {
    let stem = crate::compile::artifact_stem(spec);
    let tla = std::fs::read(out_dir.join(format!("{stem}.tla"))).ok()?;
    let check_json = std::fs::read_to_string(out_dir.join(format!("{stem}.check.json"))).ok()?;
    let report: model_check::RunReport = serde_json::from_str(&check_json).ok()?;
    if model_check::is_stale(&report, &model_check::artifact_sha256(&tla)) {
        return None;
    }
    Some(report.outcome)
}

/// Derive and attach the rung aggregate at report time (tasks 4.2):
/// reads the evidence the run already has — the spec (model presence)
/// and the persisted artifacts (the current model-check run) — and
/// overwrites the report's summary field. Called on the merged report
/// (all row kinds folded) BEFORE any `--before` diff, so the summary
/// always carries THIS run's derivation; the diff never re-derives
/// and never reads the prior report's rung — a prior rung never
/// drives the current one (the field is derived, not stored; its
/// serde default exists only so old priors parse).
pub fn with_derived_rung(
    mut report: AuditReport,
    spec: &Spec,
    out_dir: &std::path::Path,
) -> AuditReport {
    let mix = EvidenceMix {
        model_present: !spec.states.is_empty() && !spec.transitions.is_empty(),
        model_check: current_check_evidence(spec, out_dir),
        // The v1 honesty boundary (see [`EvidenceMix::verify_pass`]):
        // verify persists no report, so no audit input carries verify
        // evidence — rung 4 stays unclaimable rather than guessed.
        verify_pass: false,
    };
    report.ladder_rung = rung_from_evidence(&mix);
    report
}

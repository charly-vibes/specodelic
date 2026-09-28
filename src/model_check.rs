//! The model_check step — run a model-check backend against a compiled
//! model (specs/model_check.md).
//!
//! Purpose: implement `specs/model_check.md`'s run state machine —
//! `not_run → running → clean | counterexample_found | timed_out` — over
//! the compiled `ModelIR`, with a native default backend (stateright,
//! embedded) and a run report that `verify` can consume with artifact
//! provenance (`rerun_on_model_change`). Responsibilities: interpret the
//! compiled automaton (program counter, transitions as actions — the same
//! semantics the committed `.tla` emission commits to), explore
//! exhaustively within a stated bound, and attribute the report to the
//! backend engine + version (`backend_identified`). Rationale: guards and
//! predicates in the corpus language are prose (no executable predicate
//! language exists — openspec change `add-model-check`, Decision 3
//! Option A), so the native backend honestly reports
//! `invariants_checked: []` and never fabricates a counterexample.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use serde::Serialize;
use sha2::{Digest, Sha256};
use stateright::{Checker, Model, Property};

use crate::compile::ModelIr;

/// The native default backend's engine name (`backend_identified`).
pub const BACKEND_ENGINE: &str = "stateright";

/// The native backend's engine version — must match the `stateright`
/// entry in Cargo.toml. A const (not read at runtime) so the report is
/// reproducible from source alone; the test below pins the pair.
pub const BACKEND_VERSION: &str = "0.31.0";

/// The stated bound (`exhaustive_within_bound`): a run may only report
/// `no_counterexample` after exploring all reachable states up to this
/// bound. The bound is part of the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Bound {
    pub max_depth: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_states: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

impl Default for Bound {
    fn default() -> Self {
        Bound {
            max_depth: 100,
            max_states: None,
            timeout_secs: None,
        }
    }
}

/// The run outcome — `model_check.md`'s terminal states. `timed_out` is
/// a real outcome, never collapsed into clean or counterexample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    NoCounterexample,
    CounterexampleFound,
    TimedOut,
}

/// Backend attribution (`backend_identified`): engine + version, so two
/// backends' reports on the same compiled model are comparable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Backend {
    pub engine: String,
    pub version: String,
}

/// One completed run — the payload persisted as `<stem>.check.json` and
/// consumed by `verify` (`no_counterexample_feeds_verify`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunReport {
    pub backend: Backend,
    pub bound: Bound,
    pub outcome: Outcome,
    /// The Constraints-table ids of the invariants actually checked. The
    /// native v0 backend executes none (prose predicates — Decision 3,
    /// Option A), so this is empty rather than implying a semantic check.
    pub invariants_checked: Vec<String>,
    /// Set iff `outcome == counterexample_found` — names exactly one
    /// violated invariant by its Constraints-table id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violated_invariant_id: Option<String>,
    /// The full state trace leading to the violation, when found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<String>>,
    /// SHA-256 of the compiled `.tla` module the run consumed — the
    /// staleness key for `rerun_on_model_change`.
    pub artifact_sha256: String,
    /// Number of unique states the backend actually explored.
    pub states_explored: u64,
}

/// A labeled failure — never a silent partial result.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelCheckError {
    pub stage: String,
    pub message: String,
}

fn err(stage: &str, message: impl Into<String>) -> ModelCheckError {
    ModelCheckError {
        stage: stage.to_string(),
        message: message.into(),
    }
}

/// SHA-256 of an artifact, hex-encoded — the staleness key format.
pub fn artifact_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// `rerun_on_model_change`: a stored clean report does not satisfy
/// `no_counterexample` once the compiled artifact changed since that run.
pub fn is_stale(report: &RunReport, current_artifact_sha256: &str) -> bool {
    report.artifact_sha256 != current_artifact_sha256
}

// ---------------------------------------------------------------------------
// Native backend: the compiled automaton as a stateright program-counter
// model
// ---------------------------------------------------------------------------

/// The ModelIR as a stateright model: state = the automaton's state
/// (as an index into `ModelIr.states`), actions = transitions whose
/// `from` matches — always enabled, exactly the semantics the committed
/// `.tla` emission commits to (guards carried as annotations).
struct PcModel {
    init: usize,
    transitions: Vec<(usize, usize)>,
}

impl Model for PcModel {
    type State = usize;
    type Action = usize;

    fn init_states(&self) -> Vec<Self::State> {
        vec![self.init]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        for (i, (from, _)) in self.transitions.iter().enumerate() {
            if from == state {
                actions.push(i);
            }
        }
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let (from, to) = self.transitions.get(action)?;
        (from == last_state).then_some(*to)
    }

    fn properties(&self) -> Vec<Property<Self>> {
        // Decision 3 Option A: prose invariants are uninterpreted — the
        // native backend runs no semantic properties and reports that
        // honestly via `invariants_checked: []`.
        //
        // The placeholder below is an engine requirement, not a claim:
        // stateright's workers only expand states while some property is
        // still "awaiting discoveries" (src/checker/bfs.rs
        // `is_awaiting_discoveries`), so with zero properties the space
        // would never be explored. A never-satisfied `sometimes` property
        // forces exhaustive exploration and can never yield a discovery —
        // no fabricated counterexample, `checker_invoked` still honest.
        vec![Property::sometimes(
            "specodelic.exhaustive_exploration",
            |_, _| false,
        )]
    }
}

/// Run the native backend against the compiled model. `tla_artifact` is
/// the compiled `.tla` module bytes (hashed for provenance — the run
/// never re-compiles).
pub fn run(ir: &ModelIr, tla_artifact: &[u8], bound: &Bound) -> Result<RunReport, ModelCheckError> {
    let artifact_sha256 = artifact_sha256(tla_artifact);
    let backend = Backend {
        engine: BACKEND_ENGINE.into(),
        version: BACKEND_VERSION.into(),
    };
    // Native v0 checks no semantic invariants (Decision 3 Option A) —
    // reported honestly as empty. Keep the binding self-documenting.
    let invariants_checked = Vec::<String>::new();
    let _ = &invariants_checked;
    let states = &ir.states;
    if states.is_empty() {
        return Err(err(
            "empty_model",
            "the compiled model has no states — model_present requires a Model section with states and transitions",
        ));
    }
    // Program-counter indices; unknown from/to refs are a labeled failure
    // (lint's every_transition_valid should have caught these upstream).
    let mut state_index: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, s) in states.iter().enumerate() {
        state_index.insert(s.id.as_str(), i);
    }
    let mut transitions = Vec::with_capacity(ir.transitions.len());
    for t in &ir.transitions {
        let from = *state_index.get(t.from.as_str()).ok_or_else(|| {
            err(
                "invalid_model",
                format!(
                    "transition `{}` has unknown from-state `{}` — not in the Model section's states",
                    t.id, t.from
                ),
            )
        })?;
        let to = *state_index.get(t.to.as_str()).ok_or_else(|| {
            err(
                "invalid_model",
                format!(
                    "transition `{}` has unknown to-state `{}` — not in the Model section's states",
                    t.id, t.to
                ),
            )
        })?;
        transitions.push((from, to));
    }
    // Initial state: first listed in the spec (the `.tla` Init convention).
    let model = PcModel {
        init: 0,
        transitions,
    };

    // The bound maps 1:1 onto the checker's knobs. Targets are
    // NonZeroUsize upstream, so 0 would be silently ignored — clamp to 1.
    let max_depth = bound.max_depth.max(1) as usize;
    let mut builder = model.checker().target_max_depth(max_depth);
    if let Some(n) = bound.max_states {
        builder = builder.target_state_count(n.max(1) as usize);
    }
    let started = Instant::now();
    if let Some(secs) = bound.timeout_secs {
        builder = builder.timeout(Duration::from_secs(secs));
    }
    let checker = builder.spawn_bfs().join();
    let elapsed = started.elapsed();
    let states_explored = checker.unique_state_count() as u64;
    let depth_reached = checker.max_depth();

    // exhaustive_within_bound: a run may report clean only when the whole
    // reachable space was explored within the stated bound. If a cap was
    // actually reached, exhaustiveness cannot be proven — report the
    // budget-exhausted outcome (`timed_out`), conservatively. This can
    // mislabel a space whose true diameter equals the cap, which is the
    // honest direction: never claim clean without proof.
    let depth_capped = depth_reached >= max_depth;
    let states_capped = bound
        .max_states
        .map(|n| states_explored >= n.max(1))
        .unwrap_or(false);
    let time_capped = bound
        .timeout_secs
        .map(|s| elapsed >= Duration::from_secs(s))
        .unwrap_or(false);
    let outcome = if depth_capped || states_capped || time_capped {
        Outcome::TimedOut
    } else {
        Outcome::NoCounterexample
    };

    Ok(RunReport {
        backend,
        bound: bound.clone(),
        outcome,
        invariants_checked: vec![],
        violated_invariant_id: None,
        trace: None,
        artifact_sha256,
        states_explored,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixture: a three-state automaton a → b → c (first listed state is
    /// the initial one, matching the `.tla` Init convention).
    fn chain_ir() -> ModelIr {
        serde_json::from_str(
            r#"{
                "states": [{"id": "a"}, {"id": "b"}, {"id": "c"}],
                "transitions": [
                    {"id": "t1", "from": "a", "to": "b", "guard": null},
                    {"id": "t2", "from": "b", "to": "c", "guard": null}
                ],
                "emits": {}
            }"#,
        )
        .unwrap()
    }

    fn artifact() -> Vec<u8> {
        b"fake tla module".to_vec()
    }

    #[test]
    fn bound_defaults_to_max_depth_100() {
        let b = Bound::default();
        assert_eq!(b.max_depth, 100);
        assert_eq!(b.max_states, None);
        assert_eq!(b.timeout_secs, None);
    }

    #[test]
    fn report_serializes_outcome_snake_case_and_omits_absent_violation() {
        let report = RunReport {
            backend: Backend {
                engine: BACKEND_ENGINE.into(),
                version: BACKEND_VERSION.into(),
            },
            bound: Bound::default(),
            outcome: Outcome::NoCounterexample,
            invariants_checked: vec![],
            violated_invariant_id: None,
            trace: None,
            artifact_sha256: "abc".into(),
            states_explored: 3,
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["outcome"], "no_counterexample");
        assert_eq!(json["backend"]["engine"], "stateright");
        assert_eq!(json["artifact_sha256"], "abc");
        assert_eq!(json["states_explored"], 3);
        assert!(json.get("violated_invariant_id").is_none());
        assert!(json.get("trace").is_none());
        // The bound is part of the report (exhaustive_within_bound).
        assert_eq!(json["bound"]["max_depth"], 100);
    }

    #[test]
    fn clean_run_names_backend_and_restates_bound() {
        let report = run(&chain_ir(), &artifact(), &Bound::default()).unwrap();
        assert_eq!(report.outcome, Outcome::NoCounterexample);
        assert_eq!(report.backend.engine, "stateright");
        assert_eq!(report.backend.version, BACKEND_VERSION);
        assert_eq!(report.bound, Bound::default());
        assert_eq!(report.artifact_sha256, artifact_sha256(&artifact()));
    }

    #[test]
    fn exploration_is_exhaustive_within_the_bound() {
        // a → b → c: the full reachable set is explored.
        let report = run(&chain_ir(), &artifact(), &Bound::default()).unwrap();
        assert_eq!(report.states_explored, 3);
        assert_eq!(report.outcome, Outcome::NoCounterexample);
    }

    #[test]
    fn exploration_starts_at_the_first_listed_state() {
        // Only states reachable from `a` are explored — `c` is outside.
        let mut ir = chain_ir();
        ir.transitions.truncate(1); // keep only a → b
        let report = run(&ir, &artifact(), &Bound::default()).unwrap();
        assert_eq!(report.states_explored, 2);
    }

    #[test]
    fn prose_invariants_are_reported_unchecked_not_fabricated() {
        // Decision 3 Option A: no executable predicate language exists,
        // so the native backend checks nothing semantic — and never
        // invents a counterexample.
        let report = run(&chain_ir(), &artifact(), &Bound::default()).unwrap();
        assert!(report.invariants_checked.is_empty());
        assert_eq!(report.violated_invariant_id, None);
        assert_eq!(report.trace, None);
    }

    #[test]
    fn depth_budget_exhaustion_yields_timed_out() {
        // The chain's diameter (2) exceeds the stated depth bound, so
        // exhaustiveness cannot be proven within it.
        let bound = Bound {
            max_depth: 1,
            ..Default::default()
        };
        let report = run(&chain_ir(), &artifact(), &bound).unwrap();
        assert_eq!(report.outcome, Outcome::TimedOut);
    }

    #[test]
    fn state_budget_exhaustion_yields_timed_out() {
        let bound = Bound {
            max_states: Some(1),
            ..Default::default()
        };
        let report = run(&chain_ir(), &artifact(), &bound).unwrap();
        assert_eq!(report.outcome, Outcome::TimedOut);
    }

    #[test]
    fn time_budget_exhaustion_yields_timed_out() {
        // A zero-second budget is exhausted before the run can complete.
        let bound = Bound {
            timeout_secs: Some(0),
            ..Default::default()
        };
        let report = run(&chain_ir(), &artifact(), &bound).unwrap();
        assert_eq!(report.outcome, Outcome::TimedOut);
    }

    #[test]
    fn state_budget_wide_enough_for_the_space_yields_clean() {
        let bound = Bound {
            max_states: Some(100),
            ..Default::default()
        };
        let report = run(&chain_ir(), &artifact(), &bound).unwrap();
        assert_eq!(report.outcome, Outcome::NoCounterexample);
    }

    #[test]
    fn transition_referencing_an_unknown_state_is_a_labeled_error() {
        let mut ir = chain_ir();
        ir.transitions[0].from = "nope".into();
        let e = run(&ir, &artifact(), &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "invalid_model");
    }

    #[test]
    fn empty_model_is_a_labeled_error() {
        let ir = ModelIr::default();
        let e = run(&ir, &artifact(), &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "empty_model");
    }

    #[test]
    fn stale_report_detected_by_artifact_hash_change() {
        let report = run(&chain_ir(), &artifact(), &Bound::default()).unwrap();
        assert!(!is_stale(&report, &artifact_sha256(&artifact())));
        assert!(is_stale(&report, &artifact_sha256(b"edited module")));
    }

    #[test]
    fn sha256_is_hex_and_deterministic() {
        let a = artifact_sha256(b"fake tla module");
        let b = artifact_sha256(b"fake tla module");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn backend_version_matches_the_cargo_dep() {
        // backend_identified honesty: the const must track Cargo.toml.
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains(&format!("stateright = \"{}\"", BACKEND_VERSION)));
    }
}

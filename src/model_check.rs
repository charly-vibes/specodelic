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
//! `invariants_checked: []` and never fabricates a counterexample — and
//! a completed exploration reports `exploration_only`, never
//! `no_counterexample` (specodelic-len): exhaustiveness is not a clean
//! verdict.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
/// `exploration_only` (specodelic-len) is the native backend's completed
/// run: the space WAS explored exhaustively within the bound, but zero
/// invariant predicates were executed (prose exprs — Decision 3,
/// Option A), so it is explicitly NOT `no_counterexample` — a consumer
/// can never read it as counterexample-free verification. Only a backend
/// that actually executes invariant predicates may report
/// `no_counterexample`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    NoCounterexample,
    CounterexampleFound,
    TimedOut,
    ExplorationOnly,
}

/// Backend attribution (`backend_identified`): engine + version, so two
/// backends' reports on the same compiled model are comparable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backend {
    pub engine: String,
    pub version: String,
}

/// One completed run — the payload persisted as `<stem>.check.json` and
/// consumed by `verify` (`no_counterexample_feeds_verify`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Clone for PcModel {
    fn clone(&self) -> Self {
        PcModel {
            init: self.init,
            transitions: self.transitions.clone(),
        }
    }
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
/// never re-compiles). The native backend interprets the automaton and
/// explores it exhaustively within the bound but executes no invariant
/// predicates (prose — Decision 3, Option A), so a completed run reports
/// `exploration_only`, never `no_counterexample`.
pub fn run(ir: &ModelIr, tla_artifact: &[u8], bound: &Bound) -> Result<RunReport, ModelCheckError> {
    let artifact_sha256 = artifact_sha256(tla_artifact);
    let backend = Backend {
        engine: BACKEND_ENGINE.into(),
        version: BACKEND_VERSION.into(),
    };
    let states = &ir.states;
    if states.is_empty() {
        return Err(err(
            "empty_model",
            "the compiled model has no states — model_present requires a Model section with states and transitions",
        ));
    }
    // Program-counter indices; unknown from/to refs are a labeled failure
    // (lint's every_transition_valid should have caught these upstream).
    // IR integrity is validated before IR↔artifact agreement — a
    // corrupted IR is invalid_model, not stale_artifact.
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
    // checker_invoked's consistency half: the run must be against the
    // *current compiled model* — the IR extracted from the live spec and
    // the on-disk `.tla` must describe the same automaton, or the run
    // would check one model while hashing another (the stale-claim trap,
    // inverted).
    assert_artifact_consistent(ir, tla_artifact)?;
    // Initial state: first listed in the spec (the `.tla` Init convention).
    let model = PcModel {
        init: 0,
        transitions,
    };

    // The bound maps 1:1 onto the checker's knobs. Targets are
    // NonZeroUsize upstream, so 0 would be silently ignored — clamp to 1.
    let max_depth = bound.max_depth.max(1) as usize;
    let (checker, elapsed) = explore(model.clone(), bound, max_depth);
    let mut states_explored = checker.unique_state_count() as u64;

    // exhaustive_within_bound governs whether the space was fully
    // explored — but exhaustiveness alone is NOT clean (specodelic-len):
    // the native backend executes zero invariant predicates, so a
    // completed exploration is reported as `exploration_only`, never as
    // `no_counterexample`. `depth_reached == cap` is ambiguous — the space
    // may end exactly at the cap — so when the depth cap was the ONLY
    // constraint, one confirmation re-run at cap+1 resolves it: completing
    // below cap+1 proves the space ends within the bound (`exploration_only`);
    // hitting cap+1 too means genuinely truncated (`timed_out`). With a
    // simultaneous state/time cap the confirmation could itself be
    // truncated by the other budget, so those stay conservative. A cap
    // that was actually reached without confirmation means exhaustiveness
    // cannot be proven — `timed_out`. This can mislabel a space whose true
    // size equals a state/time cap, which is the honest direction: never
    // claim more than the run proved.
    let depth_was_hit = checker.max_depth() >= max_depth;
    let mut confirmed_exhaustive = false;
    if depth_was_hit && bound.max_states.is_none() && bound.timeout_secs.is_none() {
        let deeper_bound = Bound {
            max_depth: max_depth as u32 + 1,
            ..bound.clone()
        };
        let (deeper, _) = explore(model.clone(), &deeper_bound, max_depth + 1);
        if deeper.max_depth() < max_depth + 1 {
            confirmed_exhaustive = true;
            // The completed exploration saw every reachable state.
            states_explored = deeper.unique_state_count() as u64;
        }
    }
    let states_capped = bound
        .max_states
        .map(|n| states_explored >= n.max(1))
        .unwrap_or(false);
    let time_capped = bound
        .timeout_secs
        .map(|s| elapsed >= Duration::from_secs(s))
        .unwrap_or(false);
    let outcome = if confirmed_exhaustive {
        // The cap+1 re-run proved the space ends within the stated bound.
        Outcome::ExplorationOnly
    } else if depth_was_hit || states_capped || time_capped {
        Outcome::TimedOut
    } else {
        // Completed exploration below every cap, zero predicates
        // executed — never a clean verdict.
        Outcome::ExplorationOnly
    };

    Ok(RunReport {
        backend,
        bound: bound.clone(),
        outcome,
        invariants_checked: vec![],
        // Native backend truth: no predicate executed, so this report is
        // never a clean verdict — `no_counterexample` is reserved for
        // backends that actually execute invariant predicates.
        violated_invariant_id: None,
        trace: None,
        artifact_sha256,
        states_explored,
    })
}

/// Spawn the BFS checker under the given effective depth cap and join it.
/// Returns the joined checker and the wall time the exploration took.
fn explore(model: PcModel, bound: &Bound, max_depth: usize) -> (impl Checker<PcModel>, Duration) {
    let mut builder = model.checker().target_max_depth(max_depth);
    if let Some(n) = bound.max_states {
        builder = builder.target_state_count(n.max(1) as usize);
    }
    let started = Instant::now();
    if let Some(secs) = bound.timeout_secs {
        builder = builder.timeout(Duration::from_secs(secs));
    }
    let checker = builder.spawn_bfs().join();
    (checker, started.elapsed())
}

/// `checker_invoked`'s consistency half: the on-disk `.tla` module and the
/// IR extracted from the live spec must describe the same automaton.
/// Parses the machine-generated format `add-tla-emitter` commits (the
/// `StateValues` set line and the per-disjunct `\* <id>: <from> -> <to>`
/// comments) and compares both against the IR; any mismatch is a stale
/// artifact — a labeled error, never a run against a mixed pair.
fn assert_artifact_consistent(ir: &ModelIr, tla_artifact: &[u8]) -> Result<(), ModelCheckError> {
    let stale = |what: &str| {
        err(
            "stale_artifact",
            format!(
                "{what} — the compiled module does not match the spec's current Model section (run: specodelic compile <files>)"
            ),
        )
    };
    let text = std::str::from_utf8(tla_artifact).map_err(|_| {
        err(
            "artifact_unreadable",
            "the compiled .tla module is not valid UTF-8",
        )
    })?;
    let state_values = text
        .lines()
        .find(|l| l.trim_start().starts_with("StateValues =="))
        .ok_or_else(|| {
            err(
                "artifact_unreadable",
                "the compiled .tla module has no StateValues line — not a compile-emitted module",
            )
        })?;
    let artifact_states: std::collections::BTreeSet<&str> = state_values
        .split("==")
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split(',')
        .map(|s| s.trim().trim_matches('"'))
        .filter(|s| !s.is_empty())
        .collect();
    let ir_states: std::collections::BTreeSet<&str> =
        ir.states.iter().map(|s| s.id.as_str()).collect();
    if artifact_states != ir_states {
        return Err(stale(
            "the module's StateValues differ from the spec's states",
        ));
    }
    let mut artifact_transitions: Vec<&str> = vec![];
    let mut artifact_edges: Vec<(&str, &str)> = vec![];
    for line in text.lines() {
        let comment = line.trim_start().strip_prefix("\\*").map(str::trim_start);
        if let Some(c) = comment
            && c.contains(" -> ")
            && c.contains("(guard:")
            && let Some(id) = c.split(':').next()
        {
            artifact_transitions.push(id.trim());
        }
        // specodelic-8nt: the code disjuncts, not just their comments —
        // an edited/deleted edge with its comment left behind must not
        // pass. Every `\/` line is a disjunct; the stutter disjunct is
        // not a transition, anything else must parse or the module is
        // not a compile-emitted one (fail closed on hand edits).
        let trimmed = line.trim_start();
        if trimmed.starts_with("\\/") {
            if trimmed.starts_with("\\/ UNCHANGED") {
                continue;
            }
            let (from, to) = parse_vpc_disjunct(trimmed).ok_or_else(|| {
                err(
                    "artifact_unreadable",
                    "a Next disjunct is not of the emitted `\\/ vpc = \"…\" /\\ vpc' = \"…\"` shape — not a compile-emitted module",
                )
            })?;
            artifact_edges.push((from, to));
        }
    }
    let mut ir_transitions: Vec<&str> = ir.transitions.iter().map(|t| t.id.as_str()).collect();
    artifact_transitions.sort_unstable();
    ir_transitions.sort_unstable();
    if artifact_transitions != ir_transitions {
        return Err(stale(
            "the module's Next disjuncts differ from the spec's transitions",
        ));
    }
    // specodelic-8nt: the edges the module actually steps through — the
    // id set alone reads comments, which the disjunct code can outlive.
    let mut artifact_edges_sorted: Vec<(&str, &str)> = artifact_edges.clone();
    artifact_edges_sorted.sort_unstable();
    let mut ir_edges: Vec<(&str, &str)> = ir
        .transitions
        .iter()
        .map(|t| (t.from.as_str(), t.to.as_str()))
        .collect();
    ir_edges.sort_unstable();
    if artifact_edges_sorted != ir_edges {
        return Err(stale(
            "the module's Next disjunct edges differ from the spec's transitions",
        ));
    }
    Ok(())
}

/// A Next disjunct's program-counter edge: the two quoted values of the
/// emitted `\\/ vpc = "…" /\\ vpc' = "…"` shape. State ids are emitter-
/// generated (letters/digits), so a plain quote scan suffices.
fn parse_vpc_disjunct(line: &str) -> Option<(&str, &str)> {
    let mut quoted = line.match_indices('"').map(|(i, _)| i);
    let (q1, q2, q3, q4) = (quoted.next()?, quoted.next()?, quoted.next()?, quoted.next()?);
    if quoted.next().is_some() {
        return None;
    }
    let from = line.get(q1 + 1..q2)?;
    let to = line.get(q3 + 1..q4)?;
    Some((from, to))
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
        // A compile-emitted-format module for chain_ir() — the consistency
        // check parses this shape (StateValues + disjunct comments).
        artifact_for(&["a", "b", "c"], &[("t1", "a", "b"), ("t2", "b", "c")])
    }

    /// A minimal `.tla` module in exactly the shape `add-tla-emitter`
    /// commits: one StateValues line, one comment+disjunct per transition.
    fn artifact_for(states: &[&str], transitions: &[(&str, &str, &str)]) -> Vec<u8> {
        let mut out = String::new();
        out.push_str(&format!(
            "StateValues == {{{}}}\n\nNext ==\n",
            states
                .iter()
                .map(|s| format!("\"{s}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        for (id, from, to) in transitions {
            out.push_str(&format!(
                "  \\* {id}: {from} -> {to} (guard: some-guard)\n  \\/ vpc = \"{from}\" /\\ vpc' = \"{to}\"\n"
            ));
        }
        out.into_bytes()
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
    fn exploration_run_names_backend_and_restates_bound() {
        let report = run(&chain_ir(), &artifact(), &Bound::default()).unwrap();
        assert_eq!(report.outcome, Outcome::ExplorationOnly);
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
        assert_eq!(report.outcome, Outcome::ExplorationOnly);
    }

    #[test]
    fn exploration_starts_at_the_first_listed_state() {
        // Only states reachable from `a` are explored — `c` is outside.
        let mut ir = chain_ir();
        ir.transitions.truncate(1); // keep only a → b
        let artifact = artifact_for(&["a", "b", "c"], &[("t1", "a", "b")]);
        let report = run(&ir, &artifact, &Bound::default()).unwrap();
        assert_eq!(report.states_explored, 2);
    }

    #[test]
    fn exhaustive_run_with_zero_invariants_is_never_reported_clean() {
        // specodelic-len: `no_counterexample` over an empty invariant set
        // implies verification that did not happen. The native backend
        // executes no predicate (Decision 3, Option A), so a completed
        // exploration — however exhaustive — must carry an explicitly
        // exploratory outcome a consumer cannot read as
        // counterexample-free verification.
        let report = run(&chain_ir(), &artifact(), &Bound::default()).unwrap();
        assert!(report.invariants_checked.is_empty());
        assert_eq!(report.outcome, Outcome::ExplorationOnly);
    }

    #[test]
    fn exploration_only_outcome_serializes_snake_case() {
        let report = RunReport {
            backend: Backend {
                engine: BACKEND_ENGINE.into(),
                version: BACKEND_VERSION.into(),
            },
            bound: Bound::default(),
            outcome: Outcome::ExplorationOnly,
            invariants_checked: vec![],
            violated_invariant_id: None,
            trace: None,
            artifact_sha256: "abc".into(),
            states_explored: 3,
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["outcome"], "exploration_only");
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
    fn state_budget_wide_enough_for_the_space_yields_exploration_only() {
        let bound = Bound {
            max_states: Some(100),
            ..Default::default()
        };
        let report = run(&chain_ir(), &artifact(), &bound).unwrap();
        assert_eq!(report.outcome, Outcome::ExplorationOnly);
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
    fn edited_model_without_recompile_is_a_stale_artifact_error() {
        // CORR-001: the run interprets the live spec's IR, so an on-disk
        // module compiled from an older Model section must be rejected —
        // never a run against a mixed (IR, artifact) pair.
        let drifted = artifact_for(
            &["a", "b", "c", "d"], // spec gained a state the module lacks
            &[("t1", "a", "b"), ("t2", "b", "c")],
        );
        let e = run(&chain_ir(), &drifted, &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "stale_artifact");
        assert!(e.message.contains("specodelic compile"));
    }

    /// A drifted module in the emitted shape: comment lines and code
    /// disjuncts are supplied separately so tests can decouple them
    /// (comment left behind with its code line deleted, etc.).
    fn drifted_artifact(
        comments: &[(&str, &str, &str)],
        code: &[(&str, &str)],
        output: &[(&str, &str)],
    ) -> Vec<u8> {
        let mut out = String::from("StateValues == {\"a\", \"b\", \"c\"}\n\nNext ==\n");
        for (id, from, to) in comments {
            out.push_str(&format!(
                "  \\* {id}: {from} -> {to} (guard: some-guard)\n"
            ));
        }
        for (from, to) in code {
            out.push_str(&format!(
                "  \\/ vpc = \"{from}\" /\\ vpc' = \"{to}\"\n"
            ));
        }
        out.push_str("  \\/ UNCHANGED vpc\n\n");
        if output.is_empty() {
            out.push_str("Output == << >>\n");
        } else {
            out.push_str("Output ==\n");
            let entries: Vec<String> = output
                .iter()
                .map(|(s, v)| format!("\"{s}\" :> \"{v}\""))
                .collect();
            out.push_str(&entries.join(" @@\n"));
            out.push('\n');
        }
        out.into_bytes()
    }

    #[test]
    fn edited_code_edge_without_recompile_is_rejected() {
        // specodelic-8nt: the disjunct ids still match, but t2's code edge
        // (b -> c) was hand-edited to b -> a — states and id sets alone
        // pass silently; the edge comparison must catch it.
        let drifted = drifted_artifact(
            &[("t1", "a", "b"), ("t2", "b", "c")],
            &[("a", "b"), ("b", "a")],
            &[],
        );
        let e = run(&chain_ir(), &drifted, &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "stale_artifact");
    }

    #[test]
    fn deleted_disjunct_code_with_comment_left_behind_is_rejected() {
        // t2's code line deleted, its comment left behind: the id set
        // matches, the edge multiset does not — never a silent run.
        let drifted = drifted_artifact(&[("t1", "a", "b"), ("t2", "b", "c")], &[("a", "b")], &[]);
        let e = run(&chain_ir(), &drifted, &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "stale_artifact");
    }

    #[test]
    #[test]
    fn edited_transitions_without_recompile_are_rejected_too() {
        let drifted = artifact_for(&["a", "b", "c"], &[("t1", "a", "b")]); // t2 missing
        let e = run(&chain_ir(), &drifted, &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "stale_artifact");
    }

    #[test]
    fn non_emitted_module_is_rejected_not_silently_run() {
        let e = run(&chain_ir(), b"not a module", &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "artifact_unreadable");
    }

    #[test]
    fn depth_cap_equal_to_the_diameter_still_confirms_exploration() {
        // CORR-002: cap == diameter is ambiguous ("stopped at cap" vs "the
        // space ends at cap"); the confirmation re-run at cap+1 proves the
        // space is exhausted, so the outcome is exploration_only (never
        // timed_out) — but still NOT no_counterexample (specodelic-len).
        let bound = Bound {
            max_depth: 3,
            ..Default::default()
        };
        let report = run(&chain_ir(), &artifact(), &bound).unwrap();
        assert_eq!(report.outcome, Outcome::ExplorationOnly);
        assert_eq!(report.states_explored, 3);
    }

    #[test]
    fn genuinely_truncated_depth_budget_still_yields_timed_out() {
        // cap 1: the confirmation re-run at 2 also hits its cap — truncated.
        let bound = Bound {
            max_depth: 1,
            ..Default::default()
        };
        let report = run(&chain_ir(), &artifact(), &bound).unwrap();
        assert_eq!(report.outcome, Outcome::TimedOut);
    }

    #[test]
    fn depth_cap_confirmation_never_runs_alongside_other_caps() {
        // A state cap alongside the depth cap stays conservative: the
        // confirmation could itself be truncated by the other budget.
        let bound = Bound {
            max_depth: 3,
            max_states: Some(1),
            ..Default::default()
        };
        let report = run(&chain_ir(), &artifact(), &bound).unwrap();
        assert_eq!(report.outcome, Outcome::TimedOut);
    }

    #[test]
    fn cyclic_automaton_terminates_and_explores_each_state_once() {
        // EDGE-001: a self-loop must not hang the exploration.
        let ir: ModelIr = serde_json::from_str(
            r#"{
                "states": [{"id": "a"}, {"id": "b"}],
                "transitions": [
                    {"id": "loop", "from": "a", "to": "a", "guard": null},
                    {"id": "t", "from": "a", "to": "b", "guard": null}
                ],
                "emits": {}
            }"#,
        )
        .unwrap();
        let artifact = artifact_for(&["a", "b"], &[("loop", "a", "a"), ("t", "a", "b")]);
        let report = run(&ir, &artifact, &Bound::default()).unwrap();
        assert_eq!(report.outcome, Outcome::ExplorationOnly);
        assert_eq!(report.states_explored, 2);
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

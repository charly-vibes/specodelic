//! The model_check step — run a model-check backend against a compiled
//! model (specs/model_check.md).
//!
//! Purpose: implement `specs/model_check.md`'s run state machine —
//! `not_run → running → clean | counterexample_found | timed_out` — over
//! the compiled `ModelIR`, with the two specced backends and a run report
//! that `verify` can consume with artifact provenance
//! (`rerun_on_model_change`). Responsibilities: interpret the compiled
//! automaton (program counter, transitions as actions — the same
//! semantics the committed `.tla` emission commits to), explore
//! exhaustively within a stated bound, and attribute the report to the
//! backend engine + version (`backend_identified`) — the embedded
//! stateright default and the opt-in TLC JVM reference engine (`run_tlc`,
//! `--backend tlc`; a missing binary/jar is a `missing_checker` error,
//! never a verdict). Rationale: guards and
//! predicates in the corpus language are prose unless a cell opts in
//! with the `**rust:**` marker (executable predicate fragments —
//! specodelic.md Revision 15, specodelic-rjb; before that Revision no
//! executable predicate language existed — `add-model-check`, Decision 3
//! Option A). Fragment-less input honestly reports
//! `invariants_checked: []` and never fabricates a counterexample — and
//! a completed exploration reports `exploration_only`, never
//! `no_counterexample` (specodelic-len): exhaustiveness is not a clean
//! verdict. Fragment-bearing IR goes through the scratch-crate run
//! (`run_executable`) where `no_counterexample` becomes producible.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stateright::{Checker, Model, Property};

use crate::compile::ModelIr;

// Guard citation algebra (slice 1, specodelic-txo): the grammar and the
// three-valued evaluator live beside the spec language (compile), the
// evaluation callers live here — re-exported so the citation API is
// reachable from the model_check surface.
pub use crate::compile::{
    CitationExpr, ThreeValued, evaluate_citation, parse_citation_expr,
};

/// One invariant's evaluated status in a run report — the id is the
/// Constraints-table id, the status is the three-valued citation/exec
/// outcome (`unknown` persisted honestly, never coerced).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantStatus {
    pub id: String,
    pub status: ThreeValued,
}

/// The native default backend's engine name (`backend_identified`).
pub const BACKEND_ENGINE: &str = "stateright";

/// The opt-in TLC backend's engine name (`backend_identified`) — the JVM
/// reference engine run as a subprocess over the compiled `.tla` module
/// (specs/model_check.md, Backends).
pub const TLC_ENGINE: &str = "tlc";

/// The resolved TLC invocation: the JVM binary and the `tla2tools.jar`
/// classpath entry. A missing binary or jar is a `missing_checker` error —
/// never a verdict (specodelic-ug3 MUST).
#[derive(Clone)]
pub struct TlcPaths {
    pub java: std::path::PathBuf,
    pub jar: std::path::PathBuf,
}

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
    /// Per-invariant evaluated status (slice 1, specodelic-txo):
    /// executable invariants get their exec outcome; guard citations
    /// evaluate under the citation algebra. Absent from old reports and
    /// empty when nothing was evaluated — `unknown` persists verbatim,
    /// never coerced.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariant_statuses: Vec<InvariantStatus>,
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
        // Guard citations evaluate under the citation algebra; with no
        // executed outcome in the native backend every cited id is
        // undischargable → unknown, never pass (slice 1, specodelic-txo).
        invariant_statuses: citation_statuses(ir, &BTreeMap::new()),
        // Native backend truth: no predicate executed, so this report is
        // never a clean verdict — `no_counterexample` is reserved for
        // backends that actually execute invariant predicates.
        violated_invariant_id: None,
        trace: None,
        artifact_sha256,
        states_explored,
    })
}

// ---------------------------------------------------------------------------
// TLC backend: the JVM reference engine as a subprocess (opt-in)
// ---------------------------------------------------------------------------

/// The classification of one TLC run's output. The parse is pinned to the
/// tla2tools output shapes external differential runs agreed with (25/25,
/// vv8's harness carries the in-repo conformance evidence).
pub(crate) struct TlcOutput {
    /// `Model checking completed` — the full reachable space was explored.
    pub completed: bool,
    /// `The behavior up to this point is error-free` — a behavior was cut
    /// at the `-depth` bound before the space was exhausted.
    pub depth_cutoff: bool,
    /// An invariant TLC executed was violated — by its module-local name.
    pub violated_invariant: Option<String>,
    /// The run's `N states generated` stat.
    pub states_generated: Option<u64>,
}

/// Classify TLC's combined stdout+stderr. Unknown shapes classify as
/// neither completed nor cut — the caller fails closed on them.
pub(crate) fn parse_tlc_output(text: &str) -> TlcOutput {
    let mut violated_invariant = None;
    for line in text.lines() {
        // "Error: Invariant <name> is violated."
        if let Some(rest) = line.trim().strip_prefix("Error: Invariant ")
            && let Some(name) = rest.strip_suffix(" is violated.")
        {
            violated_invariant = Some(name.trim().to_string());
        }
    }
    let states_generated = text.lines().find_map(|l| {
        let idx = l.find(" states generated")?;
        let start = l[..idx]
            .rsplit(|c: char| !c.is_ascii_digit())
            .next()
            .filter(|d| !d.is_empty())?;
        start.parse().ok()
    });
    TlcOutput {
        completed: text.contains("Model checking completed"),
        depth_cutoff: text.contains("The behavior up to this point is error-free"),
        violated_invariant,
        states_generated,
    }
}

/// Parse the version probe's output (`tlc2.TLC -version`, e.g.
/// `TLC2 version 1.20.0 of 12 May 2024`) — `backend_identified`'s version.
pub(crate) fn parse_tlc_version(text: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let rest = l.trim().strip_prefix("TLC2 version ")?;
        Some(rest.split_whitespace().next()?.to_string())
    })
}

/// Run the TLC backend against the compiled module. `tla_path` is the
/// on-disk module TLC reads (copied into a scratch dir — TLC drops
/// `states/` artifacts beside its input); `tla_artifact` is the same
/// module's bytes, hashed for provenance exactly like the native backend.///
/// The contract mirrors `run` (same RunReport, same IR/artifact
/// validation) so the two backends' reports are attributable and
/// comparable (`backend_identified`). MUST: a missing JVM binary or jar
/// is a `missing_checker` error, never a verdict. Zero corpus invariants
/// are executable in the module (prose predicates — Decision 3, Option
/// A), so a completed run reports `exploration_only`, never
/// `no_counterexample` — the same honesty the native backend reports.
/// The counterexample leg waits on mp1's predicate-fragment decision; a
/// violated engine invariant (TypeOK — not a Constraints-table id) is a
/// labeled error, never `counterexample_found`.
pub fn run_tlc(
    ir: &ModelIr,
    tla_path: &std::path::Path,
    tla_artifact: &[u8],
    bound: &Bound,
    paths: &TlcPaths,
) -> Result<RunReport, ModelCheckError> {
    // Same integrity half as the native run: the IR must be well-formed
    // and agree with the on-disk module (checker_invoked).
    if ir.states.is_empty() {
        return Err(err(
            "empty_model",
            "the compiled model has no states — model_present requires a Model section with states and transitions",
        ));
    }
    assert_artifact_consistent(ir, tla_artifact)?;
    if bound.max_states.is_some() {
        return Err(err(
            "unsupported_bound",
            "--max-states has no TLC equivalent (its stated bound is -depth); drop it or use the stateright backend",
        ));
    }
    if !paths.jar.is_file() {
        return Err(err(
            "missing_checker",
            format!(
                "no tla2tools.jar at {} — the TLC backend needs the tla2tools distribution (download from https://github.com/tlaplus/tlaplus/releases and pass it via --tlc-jar)",
                paths.jar.display()
            ),
        ));
    }
    // Version probe (`backend_identified`): doubles as the binary/jar
    // smoke test — a JVM that cannot run tlc2.TLC cannot run the module.
    let mut probe_cmd = std::process::Command::new(&paths.java);
    probe_cmd
        .arg("-cp")
        .arg(&paths.jar)
        .arg("tlc2.TLC")
        .arg("-version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let probe = spawn_with_etxtbsy_retry(&mut probe_cmd)
        .map_err(|e| missing_checker(&paths.java, &e))?
        .wait_with_output()
        .map_err(|e| missing_checker(&paths.java, &e))?;
    let version = if probe.status.success() {
        String::from_utf8_lossy(&probe.stdout)
    } else {
        String::from_utf8_lossy(&probe.stderr)
    };
    let version = parse_tlc_version(&version).ok_or_else(|| {
        err(
            "missing_checker",
            format!(
                "{} -cp {} tlc2.TLC -version did not report a version — is this a tla2tools.jar?",
                paths.java.display(),
                paths.jar.display()
            ),
        )
    })?;

    // Scratch dir: TLC writes `states/` beside its input — never beside
    // the committed artifact. The module keeps its (emitter-sanitized)
    // name: TLA+ module names must be identifiers.
    let stem = tla_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "module".to_string());
    let scratch = scratch_dir(&stem)?;
    let module_in_scratch = scratch.join(format!("{stem}.tla"));
    std::fs::write(&module_in_scratch, tla_artifact).map_err(|e| {
        // Don't leave a half-created scratch dir behind on the error path.
        let _ = std::fs::remove_dir_all(&scratch);
        err("scratch_dir", e.to_string())
    })?;

    let mut command = std::process::Command::new(&paths.java);
    command
        .arg("-cp")
        .arg(&paths.jar)
        .arg("tlc2.TLC")
        .arg("-depth")
        .arg(bound.max_depth.max(1).to_string())
        .arg(&module_in_scratch)
        .current_dir(&scratch)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child =
        spawn_with_etxtbsy_retry(&mut command).map_err(|e| missing_checker(&paths.java, &e))?;
    let started = Instant::now();
    let budget = bound.timeout_secs.map(Duration::from_secs);
    // The backend enforces the wall-clock bound itself — a JVM ignoring
    // its budget is killed and the run reports timed_out (the bound was
    // not reached before the budget expired, model_check.md's
    // finish_timeout guard).
    let mut killed = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if !killed && budget.is_some_and(|t| started.elapsed() >= t) {
                    killed = child.kill().is_ok();
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(err("tlc_error", e.to_string())),
        }
    }
    let out = child
        .wait_with_output()
        .map_err(|e| err("tlc_error", e.to_string()))?;
    let mut output_bytes = out.stdout;
    output_bytes.extend_from_slice(&out.stderr);
    let output_text = String::from_utf8_lossy(&output_bytes);
    let parsed = parse_tlc_output(&output_text);
    // Best-effort scratch cleanup — every return path below is final.
    let _ = std::fs::remove_dir_all(&scratch);
    if let Some(name) = parsed.violated_invariant {
        return Err(err(
            "tlc_invariant_violated",
            format!(
                "TLC violated engine invariant `{name}` — not a Constraints-table id, so this is never a counterexample verdict; the emitted module's TypeOK should be unviolatable (an emitter bug), and corpus predicates are prose (Decision 3, Option A)"
            ),
        ));
    }
    let outcome = if killed || parsed.depth_cutoff {
        // Killed on the wall-clock budget, or a depth-cut behavior: the
        // bound was not exhausted — timed_out, never a verdict either way.
        Outcome::TimedOut
    } else if parsed.completed {
        Outcome::ExplorationOnly
    } else {
        // Zero exit but no known verdict shape — fail closed.
        return Err(err(
            "tlc_error",
            format!(
                "TLC exited without a recognizable verdict — output tail: {}",
                tail(&output_text, 400)
            ),
        ));
    };
    Ok(RunReport {
        backend: Backend {
            engine: TLC_ENGINE.into(),
            version,
        },
        bound: bound.clone(),
        outcome,
        invariants_checked: vec![],
        invariant_statuses: citation_statuses(ir, &BTreeMap::new()),
        violated_invariant_id: None,
        trace: None,
        artifact_sha256: artifact_sha256(tla_artifact),
        states_explored: parsed.states_generated.unwrap_or(0),
    })
}

// ---------------------------------------------------------------------------
// Executable invariants: the scratch-crate run (Revision 15, specodelic-rjb)
// ---------------------------------------------------------------------------

/// The executable mode's engine attribution (`backend_identified`): the
/// scratch-crate BFS run is NOT stateright — it is this tool's own
/// dependency-free breadth-first search over the same program-counter
/// model, and the report must not imply otherwise.
pub const EXEC_ENGINE: &str = "native-bfs";
pub const EXEC_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The scratch crate's manifest: deliberately dependency-free — std-only
/// BFS, no registry access, builds offline and instantly. Fragments are
/// verbatim user Rust; hygiene (compile.md's `fragment_hygiene`) was
/// validated labeled at compile time, and the Rust compiler is not a
/// sandbox — fragments run with the invoking user's privileges, exactly
/// like every other compiled artifact.
pub const SCRATCH_RUN_MANIFEST: &str = "[package]\n\nname = \"specodelic-model-check-scratch\"\n\nversion = \"0.0.0\"\n\nedition = \"2021\"\n";

/// The scratch binary's output: the raw facts of one BFS run. The parent
/// (spk) owns the outcome policy — the mapping to `Outcome` stays here,
/// unit-tested, not in generated code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunFacts {
    pub violated_invariant_id: Option<String>,
    #[serde(default)]
    pub trace: Option<Vec<String>>,
    pub states_explored: u64,
    pub deadline_hit: bool,
    pub states_capped: bool,
    #[serde(default)]
    pub depth_capped: bool,
}

/// Transition from/to cells as state indices — labeled `invalid_model` on
/// an unknown reference (lint's every_transition_valid should have caught
/// these upstream; IR integrity before execution, same as `run`).
fn resolve_edges(ir: &ModelIr) -> Result<Vec<(usize, usize)>, ModelCheckError> {
    let mut index = BTreeMap::new();
    for (i, s) in ir.states.iter().enumerate() {
        index.insert(s.id.as_str(), i);
    }
    ir.transitions
        .iter()
        .map(|t| {
            let from = *index.get(t.from.as_str()).ok_or_else(|| {
                err(
                    "invalid_model",
                    format!(
                        "transition `{}` has unknown from-state `{}` — not in the Model section's states",
                        t.id, t.from
                    ),
                )
            })?;
            let to = *index.get(t.to.as_str()).ok_or_else(|| {
                err(
                    "invalid_model",
                    format!(
                        "transition `{}` has unknown to-state `{}` — not in the Model section's states",
                        t.id, t.to
                    ),
                )
            })?;
            Ok((from, to))
        })
        .collect()
}

/// Generate the scratch crate's `src/main.rs`: a std-only BFS over the
/// same program-counter model the embedded interpreter walks, with one
/// `inv_<i>` fn per executable invariant (fragment verbatim) evaluated at
/// every visited state through `catch_unwind` (`invariant_totality`: a
/// panicking fragment is a violation, never a pass). BFS order keeps
/// `counterexample_is_minimal` holding by construction — the first
/// violated state discovered carries the shortest path to it.
pub fn scratch_run_source(ir: &ModelIr, edges: &[(usize, usize)], bound: &Bound) -> String {
    let names: Vec<String> = ir.states.iter().map(|s| s.id.clone()).collect();
    let ids: Vec<String> = ir.invariants.iter().map(|i| i.id.clone()).collect();
    let max_states = bound.max_states.map(|n| n as usize).unwrap_or(usize::MAX);
    let timeout_nanos: u128 = bound
        .timeout_secs
        .map(|s| s as u128 * 1_000_000_000)
        .unwrap_or(u128::MAX);

    let mut out = String::new();
    out.push_str(
        "// Generated by `spk model-check` — the executable-invariant run\n\
         // (specodelic.md Revision 15). Source of truth: the spec file;\n\
         // edit there, not here.\n\n\
         use std::collections::VecDeque;\n\
         use std::time::Instant;\n\n",
    );
    out.push_str(&format!("const MAX_STATES: usize = {max_states};\n"));
    out.push_str(&format!("const MAX_DEPTH: u32 = {};\n", bound.max_depth));
    out.push_str(&format!("const TIMEOUT_NANOS: u128 = {timeout_nanos};\n"));
    out.push_str("const INIT: usize = 0usize;\n\n");
    out.push_str(&format!(
        "fn state_names() -> Vec<&'static str> {{\n    vec![{}]\n}}\n\n",
        names
            .iter()
            .map(|n| format!("{n:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str(&format!(
        "fn edges() -> Vec<(usize, usize)> {{\n    vec![{}]\n}}\n\n",
        edges
            .iter()
            .map(|(f, t)| format!("({f}, {t})"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    for (i, inv) in ir.invariants.iter().enumerate() {
        out.push_str(&format!(
            "// INVARIANT {} — fragment verbatim\nfn inv_{i}(state: &str) -> bool {{\n    {}\n}}\n\n",
            inv.id, inv.fragment
        ));
    }
    out.push_str(
        r#"
fn eval_inv(i: usize, state: &str) -> bool {
    // invariant_totality: a panicking fragment is a violation at that
    // state — never a pass, never silently skipped.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match i {
"#,
    );
    for i in 0..ids.len() {
        out.push_str(&format!("        {i} => inv_{i}(state),\n"));
    }
    out.push_str(
        r#"
        _ => true,
    }))
    .unwrap_or(false)
}

fn json_escape(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
"#,
    );
    out.push_str(&format!(
        "fn main() {{\n\
             let names = state_names();\n\
             let edges = edges();\n\
             let inv_ids: Vec<&str> = vec![{}];\n\
             let t0 = Instant::now();\n\
             let n = names.len();\n\
             let mut visited = vec![false; n];\n\
             let mut parent = vec![usize::MAX; n];\n\
             let mut depth = vec![0u32; n];\n\
             let mut queue: VecDeque<usize> = VecDeque::new();\n\
             visited[INIT] = true;\n\
             queue.push_back(INIT);\n\
             let mut states_explored: usize = 0;\n\
             let mut violated: Option<(usize, usize)> = None;\n\
             let mut deadline_hit = false;\n\
             let mut states_capped = false;\n\
             let mut depth_capped = false;\n\
             'search: while let Some(s) = queue.pop_front() {{\n\
                 if TIMEOUT_NANOS != u128::MAX\n\
                     && (Instant::now() - t0).as_nanos() >= TIMEOUT_NANOS\n\
                 {{\n\
                     deadline_hit = true;\n\
                     break 'search;\n\
                 }}\n\
                 states_explored += 1;\n\
                 if states_explored > MAX_STATES {{\n\
                     states_capped = true;\n\
                     break 'search;\n\
                 }}\n\
                 for i in 0..inv_ids.len() {{\n\
                     if !eval_inv(i, &names[s]) {{\n\
                         violated = Some((s, i));\n\
                         break 'search;\n\
                     }}\n\
                 }}\n\
                 if depth[s] >= MAX_DEPTH {{\n\
                     // exhaustive_within_bound: the stated depth bound\n\
                     // refused to explore this state's successors — the\n\
                     // space is not proven exhausted (conservative).\n\
                     if edges.iter().any(|(from, _)| *from == s) {{\n\
                         depth_capped = true;\n\
                         break 'search;\n\
                     }}\n\
                     continue 'search;\n\
                 }}\n\
                 for (from, to) in edges.iter() {{\n\
                     if *from == s && !visited[*to] {{\n\
                         visited[*to] = true;\n\
                         parent[*to] = s;\n\
                         depth[*to] = depth[s] + 1;\n\
                         queue.push_back(*to);\n\
                     }}\n\
                 }}\n\
             }}\n",
        ids.iter()
            .map(|i| format!("{i:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str(r#"
    let trace = violated.map(|(s, _)| {
        let mut path = Vec::new();
        let mut cur = s;
        loop {
            path.push(names[cur].to_string());
            if cur == INIT {
                break;
            }
            cur = parent[cur];
        }
        path.reverse();
        path
    });
    let violated_json = match &violated {
        Some((_, i)) => json_escape(inv_ids[*i]),
        None => "null".to_string(),
    };
    let trace_json = match &trace {
        Some(t) => format!(
            "[{}]",
            t.iter().map(|s| json_escape(s)).collect::<Vec<_>>().join(",")
        ),
        None => "null".to_string(),
    };
    println!(
        "{{\"violated_invariant_id\":{},\"trace\":{},\"states_explored\":{},\"deadline_hit\":{},\"states_capped\":{}}}",
        violated_json,
        trace_json,
        states_explored,
        deadline_hit,
        states_capped
    );
}
"#);
    out
}

/// The outcome policy for one scratch run's facts (unit-tested here, not
/// in generated code): a violation is `counterexample_found` (the BFS
/// trace is minimal by construction); a budget breach is `timed_out`; a
/// completed exploration with ≥ 1 executed invariant and no violation is
/// `no_counterexample` — the leg the gate could never reach before
/// Revision 15. The run executes every invariant at every visited state,
/// so `invariants_checked` is exactly the fragment set.
pub fn report_from_facts(
    facts: &RunFacts,
    artifact_sha256: String,
    invariant_ids: &[String],
    backend: &Backend,
    bound: &Bound,
) -> RunReport {
    let outcome = if facts.violated_invariant_id.is_some() {
        Outcome::CounterexampleFound
    } else if facts.deadline_hit || facts.states_capped || facts.depth_capped {
        Outcome::TimedOut
    } else {
        Outcome::NoCounterexample
    };
    // Per-invariant exec status (slice 1, specodelic-txo): the violated
    // id is a counterexample; the other executed ids are verified only
    // when the exploration completed within the bound — a truncated run
    // proves nothing about them (honest unknown).
    let invariant_statuses = invariant_ids
        .iter()
        .map(|id| InvariantStatus {
            id: id.clone(),
            status: if facts.violated_invariant_id.as_deref() == Some(id.as_str()) {
                ThreeValued::Counterexample
            } else if outcome == Outcome::NoCounterexample {
                ThreeValued::Verified
            } else {
                ThreeValued::Unknown
            },
        })
        .collect();
    RunReport {
        backend: backend.clone(),
        bound: bound.clone(),
        outcome,
        invariants_checked: invariant_ids.to_vec(),
        invariant_statuses,
        violated_invariant_id: facts.violated_invariant_id.clone(),
        trace: facts.trace.clone(),
        artifact_sha256,
        states_explored: facts.states_explored,
    }
}

/// Run the native backend's executable mode: one invariant fragment or
/// more is present in the IR, so the report may honestly say
/// `no_counterexample` (or name a real minimal counterexample). The run
/// is a dependency-free scratch crate (fragments are verbatim Rust —
/// `executable_invariants_execute`), compiled and executed by cargo with
/// a wall-clock backstop; a missing toolchain or a fragment that does not
/// compile is a labeled error, never a verdict.
pub fn run_executable(
    ir: &ModelIr,
    tla_artifact: &[u8],
    bound: &Bound,
) -> Result<RunReport, ModelCheckError> {
    let artifact_sha = artifact_sha256(tla_artifact);
    assert_artifact_consistent(ir, tla_artifact)?;
    if ir.states.is_empty() {
        return Err(err(
            "empty_model",
            "the compiled model has no states — model_present requires a Model section with states and transitions",
        ));
    }
    if ir.invariants.is_empty() {
        // Defense-in-depth: the caller routes executable IR here only.
        return run(ir, tla_artifact, bound);
    }
    let edges = resolve_edges(ir)?;

    // Per-invocation scratch crate under the shared scratch base (the
    // CARGO_TARGET_DIR there is shared with verify's runner, so cargo
    // locks coordinate concurrent invocations).
    let unique = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let base = crate::verify::scratch_base()
        .join("model-check")
        .join(&unique);
    std::fs::create_dir_all(base.join("src")).map_err(|e| {
        err(
            "scratch_dir",
            format!("could not create {}: {e}", base.display()),
        )
    })?;
    // Unique package (hence binary) name per invocation: `cargo run`
    // releases the target-dir build lock before executing, so a parallel
    // scratch build of the SAME binary name could overwrite it mid-run
    // — two fragment files model-checked concurrently must not race.
    let manifest = SCRATCH_RUN_MANIFEST.replace(
        "specodelic-model-check-scratch",
        &format!("spk-mc-{unique}"),
    );
    std::fs::write(base.join("Cargo.toml"), manifest)
        .map_err(|e| err("scratch_dir", e.to_string()))?;
    std::fs::write(
        base.join("src").join("main.rs"),
        scratch_run_source(ir, &edges, bound),
    )
    .map_err(|e| err("scratch_dir", e.to_string()))?;
    let result = run_scratch(&base, bound);
    let _ = std::fs::remove_dir_all(&base); // best effort
    let stdout = result?;

    // The scratch binary prints exactly one JSON line (the facts).
    let line = stdout
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .ok_or_else(|| {
            err(
                "scratch_output",
                "the executable-invariant run printed no facts line — not a verdict",
            )
        })?;
    let facts: RunFacts = serde_json::from_str(line.trim()).map_err(|e| {
        err(
            "scratch_output",
            format!("the run's facts line does not parse: {e} — not a verdict"),
        )
    })?;
    let backend = Backend {
        engine: EXEC_ENGINE.into(),
        version: EXEC_VERSION.into(),
    };
    let ids: Vec<String> = ir.invariants.iter().map(|i| i.id.clone()).collect();
    let mut report = report_from_facts(
        &facts,
        artifact_sha,
        &ids,
        &backend,
        bound,
    );
    // Guard citations evaluate against the executable run's outcomes —
    // a cited executable invariant discharges from its exec status; a
    // cited prose property or unknown id stays unknown (slice 1).
    let outcomes: BTreeMap<String, ThreeValued> = report
        .invariant_statuses
        .iter()
        .map(|s| (s.id.clone(), s.status))
        .collect();
    report.invariant_statuses.extend(citation_statuses(ir, &outcomes));
    Ok(report)
}

/// Per-invariant status for the IR's guard citations, evaluated under
/// the citation algebra against the given outcomes map — a cited id
/// with no known outcome is undischargable → unknown, never pass.
fn citation_statuses(
    ir: &ModelIr,
    outcomes: &BTreeMap<String, ThreeValued>,
) -> Vec<InvariantStatus> {
    ir.guard_citations
        .iter()
        .filter_map(|(id, cell)| {
            parse_citation_expr(cell).map(|expr| InvariantStatus {
                id: id.clone(),
                status: evaluate_citation(&expr, outcomes),
            })
        })
        .collect()
}

/// Spawn `cargo run` on the scratch crate and wait under a wall-clock
/// backstop (`bound.timeout_secs` + grace — the generated BFS bounds
/// itself first; this backstop covers a pathological fragment). Returns
/// the child's stdout on clean exit.
fn run_scratch(base: &std::path::Path, bound: &Bound) -> Result<String, ModelCheckError> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = std::process::Command::new(&cargo);
    command
        .args(["run", "--quiet", "--manifest-path"])
        .arg(base.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            crate::verify::scratch_base().join("target"),
        )
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = spawn_with_etxtbsy_retry(&mut command)
        .map_err(|e| err("missing_runner", format!("the executable-invariant run compiles and runs a scratch crate — the Rust toolchain must be available ({e})")))?;
    let backstop = bound
        .timeout_secs
        .map(|s| Duration::from_secs(s.saturating_add(10)));
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = String::new();
                if let Some(mut out) = child.stdout.take() {
                    use std::io::Read;
                    let _ = out.read_to_string(&mut stdout);
                }
                if !status.success() {
                    let mut stderr = String::new();
                    if let Some(mut err_out) = child.stderr.take() {
                        use std::io::Read;
                        let _ = err_out.read_to_string(&mut stderr);
                    }
                    return Err(err(
                        "fragment_compile",
                        format!(
                            "the executable-invariant scratch run failed ({status}) — fragments are verbatim Rust: fix the **rust:** expression (first errors: {})",
                            tail(&stderr, 400)
                        ),
                    ));
                }
                return Ok(stdout);
            }
            Ok(None) => {
                if let Some(limit) = backstop
                    && started.elapsed() >= limit
                {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(err(
                        "scratch_timeout",
                        "the executable-invariant run exceeded its wall-clock backstop and was killed — a pathological fragment is an unbounded run; raise --timeout-secs",
                    ));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(err("scratch_spawn", e.to_string())),
        }
    }
}

/// A fresh scratch directory for one TLC run (pid + a process-local
/// counter — unique across sequential runs in one invocation). Removed
/// best-effort when the run finishes; a leftover from a killed process
/// is harmless (the next run of the same stem creates it anew).
fn scratch_dir(stem: &str) -> Result<std::path::PathBuf, ModelCheckError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("spk-tlc-{}-{n}-{stem}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| err("scratch_dir", e.to_string()))?;
    Ok(dir)
}

/// The missing-checker labeled error with its remediation hint — the
/// specodelic-ug3 MUST: a missing checker binary is an ERROR, never a
/// no_counterexample result.
fn missing_checker(java: &std::path::Path, e: &std::io::Error) -> ModelCheckError {
    err(
        "missing_checker",
        format!(
            "cannot run the TLC backend ({}): {e} — the JVM binary must be on PATH (or SPK_TLC_JAVA) and --tlc-jar must point at a tla2tools.jar",
            java.display()
        ),
    )
}

/// Spawn a command, retrying the `ETXTBSY` race (os error 26): spawning a
/// just-written script shim can hit `Text file busy` under parallel test
/// load — a fresh write's close and the exec raced on some filesystems.
/// A handful of short retries removes the flake without masking genuinely
/// missing binaries (any other error, or exhaustion, surfaces immediately).
fn spawn_with_etxtbsy_retry(
    command: &mut std::process::Command,
) -> std::io::Result<std::process::Child> {
    const ETXTBSY: i32 = 26;
    const MAX_ATTEMPTS: u32 = 10;
    for attempt in 0..=MAX_ATTEMPTS {
        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(e) if e.raw_os_error() == Some(ETXTBSY) && attempt < MAX_ATTEMPTS => {
                std::thread::sleep(Duration::from_millis(20 * (attempt as u64 + 1)));
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!("retry loop returns on success or error")
}

fn tail(text: &str, max: usize) -> &str {
    let trimmed = text.trim_end();
    if trimmed.len() <= max {
        trimmed
    } else {
        &trimmed[trimmed.len() - max..]
    }
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
/// Extracts the artifact's structure (backend-neutral, reusable by the
/// TLC backend) and compares it against the IR; any mismatch is a stale
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
    let shape = parse_artifact_shape(tla_artifact)?;
    let ir_states: std::collections::BTreeSet<&str> =
        ir.states.iter().map(|s| s.id.as_str()).collect();
    let ir_states_owned: std::collections::BTreeSet<String> =
        ir_states.iter().copied().map(String::from).collect();
    if shape.states != ir_states_owned {
        return Err(stale(
            "the module's StateValues differ from the spec's states",
        ));
    }
    let ir_transition_ids: std::collections::BTreeSet<String> =
        ir.transitions.iter().map(|t| t.id.clone()).collect();
    if shape.transition_ids != ir_transition_ids {
        return Err(stale(
            "the module's Next disjuncts differ from the spec's transitions",
        ));
    }
    // specodelic-8nt: the edges the module actually steps through — the
    // id set alone reads comments, which the disjunct code can outlive.
    let mut artifact_edges = shape.edges.clone();
    artifact_edges.sort_unstable();
    let mut ir_edges: Vec<(String, String)> = ir
        .transitions
        .iter()
        .map(|t| (t.from.clone(), t.to.clone()))
        .collect();
    ir_edges.sort_unstable();
    if artifact_edges != ir_edges {
        return Err(stale(
            "the module's Next disjunct edges differ from the spec's transitions",
        ));
    }
    // specodelic-8nt (Output half): the Output function must match the
    // IR's emitted values verbatim — the emitter writes the effect-
    // Constraint's expr (or raw target), and the IR carries the same
    // computation (ModelIr::emits_values), so a hand-edited or stale
    // Output section cannot pass silently.
    if shape.output != ir.emits_values {
        return Err(stale(
            "the module's Output function differs from the spec's emits",
        ));
    }
    // Revision 15 (specodelic-rjb): the executable-invariant manifest —
    // id + verbatim **rust:** fragment per invariant — must match the
    // IR's invariants. A fragment edited after the run changes these
    // bytes, so rerun_on_model_change fails the stale report closed; a
    // manifest absent while the IR carries fragments is equally stale.
    let mut artifact_invariants = shape.invariants.clone();
    artifact_invariants.sort();
    let mut ir_invariants: Vec<(String, String)> = ir
        .invariants
        .iter()
        .map(|i| (i.id.clone(), i.fragment.clone()))
        .collect();
    ir_invariants.sort();
    if artifact_invariants != ir_invariants {
        return Err(stale(
            "the module's invariant manifest differs from the spec's executable invariants",
        ));
    }
    Ok(())
}

/// The automaton structure a compile-emitted `.tla` module carries:
/// the StateValues range, the per-disjunct transition ids (from the
/// `\* <id>: <from> -> <to>` comments), the code edges the Next action
/// actually steps through, and the Output function.
pub(crate) struct ArtifactShape {
    pub states: std::collections::BTreeSet<String>,
    pub transition_ids: std::collections::BTreeSet<String>,
    /// One `(from, to)` per code disjunct — unsorted, may repeat for
    /// parallel transitions.
    pub edges: Vec<(String, String)>,
    pub output: std::collections::BTreeMap<String, String>,
    /// The executable-invariant manifest comment: `(id, fragment)` per
    /// `\* INVARIANT <id> **rust:** <fragment>` line — empty for modules
    /// compiled before Revision 15 or with no fragment invariants.
    pub invariants: Vec<(String, String)>,
}

/// Parse the machine-generated format `add-tla-emitter` commits: the
/// `StateValues` set line, one `\* <id>: <from> -> <to> (guard: …)`
/// comment plus `\/ vpc = "…" /\ vpc' = "…"` code line per transition,
/// and the `Output` function (`"state" :> "value"` entries joined with
/// `@@`). Parsing is backend-neutral — the TLC backend (specodelic-ug3)
/// reuses this same extraction to validate its own artifacts. Anything
/// outside the emitted shape is a labeled error, never a guess.
pub(crate) fn parse_artifact_shape(tla_artifact: &[u8]) -> Result<ArtifactShape, ModelCheckError> {
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
    let states: std::collections::BTreeSet<String> = state_values
        .split("==")
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut transition_ids: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut edges: Vec<(String, String)> = vec![];
    let mut output: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut invariants: Vec<(String, String)> = vec![];
    let mut in_output = false;
    for line in text.lines() {
        let comment = line.trim_start().strip_prefix("\\*").map(str::trim_start);
        if let Some(c) = comment
            && let Some(rest) = c.strip_prefix("INVARIANT ")
            && let Some((id, fragment)) = rest.split_once(" **rust:** ")
        {
            invariants.push((id.trim().to_string(), fragment.trim().to_string()));
        }
        if let Some(c) = comment
            && c.contains(" -> ")
            && c.contains("(guard:")
            && let Some(id) = c.split(':').next()
        {
            transition_ids.insert(id.trim().to_string());
        }
        // specodelic-8nt: the code disjuncts, not just their comments —
        // an edited/deleted edge with its comment left behind must not
        // pass. Every `\/` line is a disjunct; the stutter disjunct is
        // not a transition, anything else must parse or the module is
        // not a compile-emitted one (fail closed on hand edits).
        let trimmed = line.trim_start();
        if trimmed.starts_with("Output ==") {
            in_output = true;
            continue;
        }
        if in_output {
            if trimmed.starts_with("====") {
                in_output = false;
            } else if !trimmed.starts_with("<< >>") {
                // One entry per emitting state, `"state" :> "value"`,
                // joined with `@@` — parse each fragment; a malformed
                // entry means the module was hand-edited (fail closed).
                for fragment in trimmed.split("@@") {
                    let f = fragment.trim();
                    if f.is_empty() {
                        continue;
                    }
                    let (state, value) = parse_output_entry(f).ok_or_else(|| {
                        err(
                            "artifact_unreadable",
                            "an Output entry is not of the emitted `\"state\" :> \"value\"` shape — not a compile-emitted module",
                        )
                    })?;
                    output.insert(state.to_string(), value.to_string());
                }
            }
            continue;
        }
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
            edges.push((from.to_string(), to.to_string()));
        }
    }
    Ok(ArtifactShape {
        states,
        transition_ids,
        edges,
        output,
        invariants,
    })
}

/// A Next disjunct's program-counter edge: the two quoted values of the
/// emitted `\\/ vpc = "…" /\\ vpc' = "…"` shape. State ids are emitter-
/// generated (letters/digits), so a plain quote scan suffices.
fn parse_vpc_disjunct(line: &str) -> Option<(&str, &str)> {
    let mut quoted = line.match_indices('"').map(|(i, _)| i);
    let (q1, q2, q3, q4) = (
        quoted.next()?,
        quoted.next()?,
        quoted.next()?,
        quoted.next()?,
    );
    if quoted.next().is_some() {
        return None;
    }
    let from = line.get(q1 + 1..q2)?;
    let to = line.get(q3 + 1..q4)?;
    Some((from, to))
}

/// An Output entry's state and value: the two quoted halves of the
/// emitted `"state" :> "value"` shape, split at the first ` :> `.
fn parse_output_entry(entry: &str) -> Option<(&str, &str)> {
    let (left, right) = entry.split_once(" :> ")?;
    let state = left.trim().strip_prefix('"')?.strip_suffix('"')?;
    let value = right.trim().strip_prefix('"')?.strip_suffix('"')?;
    Some((state, value))
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
        artifact_for(&["a", "b", "c"], &[("t1", "a", "b"), ("t2", "b", "c")], &[])
    }

    /// A minimal `.tla` module in exactly the shape `add-tla-emitter`
    /// commits: one StateValues line, one comment+disjunct per transition,
    /// and an Output function (empty when no output entries are given —
    /// the emitter always writes the section).
    fn artifact_for(
        states: &[&str],
        transitions: &[(&str, &str, &str)],
        output: &[(&str, &str)],
    ) -> Vec<u8> {
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

    // --- specodelic-rjb: executable invariants (Revision 15) ---

    use crate::compile::InvariantIr;

    fn ir_with_invariants(invariants: &[(&str, &str)]) -> ModelIr {
        let base = chain_ir();
        ModelIr {
            invariants: invariants
                .iter()
                .map(|(id, f)| InvariantIr {
                    id: (*id).to_string(),
                    fragment: (*f).to_string(),
                })
                .collect(),
            ..base
        }
    }

    fn artifact_with_manifest(
        states: &[&str],
        transitions: &[(&str, &str, &str)],
        invariants: &[(&str, &str)],
    ) -> Vec<u8> {
        let mut text = String::from_utf8(artifact_for(states, transitions, &[])).unwrap();
        if !invariants.is_empty() {
            let manifest = invariants
                .iter()
                .map(|(id, f)| format!("\\* INVARIANT {id} **rust:** {f}\n"))
                .collect::<Vec<_>>()
                .join("");
            text = text.replace(
                "StateValues ==",
                &format!(
                    "\\* Executable invariant fragments — id + verbatim **rust:** fragment;\n\\* carried in the module so the artifact hash covers fragment edits.\n{manifest}\nStateValues =="
                ),
            );
        }
        text.into_bytes()
    }

    #[test]
    fn artifact_manifest_parses_into_shape() {
        let tla = artifact_with_manifest(
            &["a", "b", "c"],
            &[("t1", "a", "b"), ("t2", "b", "c")],
            &[("c_hold", "state != \"blackhole\"")],
        );
        let shape = parse_artifact_shape(&tla).expect("parses");
        assert_eq!(
            shape.invariants,
            vec![("c_hold".to_string(), "state != \"blackhole\"".to_string())],
            "the manifest comment carries id + fragment verbatim"
        );
        // Manifest absence means zero invariants (the corpus modules keep
        // their bytes — the manifest is only emitted when one exists).
        let plain = artifact_with_manifest(&["a"], &[("t1", "a", "a")], &[]);
        assert!(parse_artifact_shape(&plain).unwrap().invariants.is_empty());
    }

    #[test]
    fn consistency_checks_the_manifest_against_the_ir() {
        let ir = ir_with_invariants(&[("c_hold", "state != \"blackhole\"")]);
        let tla = artifact_with_manifest(
            &["a", "b", "c"],
            &[("t1", "a", "b"), ("t2", "b", "c")],
            &[("c_hold", "state != \"blackhole\"")],
        );
        assert!(assert_artifact_consistent(&ir, &tla).is_ok());
        // A fragment edit changes the manifest — the run fails closed as
        // stale (rerun_on_model_change, Revision 15 rewording).
        let drifted = artifact_with_manifest(
            &["a", "b", "c"],
            &[("t1", "a", "b"), ("t2", "b", "c")],
            &[("c_hold", "state != \"doom\"")],
        );
        assert!(assert_artifact_consistent(&ir, &drifted).is_err());
        // Manifest absent while the IR carries fragments — also stale.
        let bare = artifact_for(&["a", "b", "c"], &[("t1", "a", "b"), ("t2", "b", "c")], &[]);
        assert!(assert_artifact_consistent(&ir, &bare).is_err());
    }

    #[test]
    fn scratch_source_compiles_fragments_verbatim() {
        let ir = ir_with_invariants(&[
            ("c_hold", "state != \"blackhole\""),
            ("c_other", "state != \"doom\""),
        ]);
        let edges = resolve_edges(&ir).unwrap();
        let src = scratch_run_source(&ir, &edges, &Bound::default());
        assert!(
            src.contains("fn inv_0(state: &str) -> bool {\n    state != \"blackhole\"\n}"),
            "fragment verbatim in the generated invariant fn: {src}"
        );
        assert!(src.contains("fn inv_1(state: &str) -> bool {\n    state != \"doom\"\n}"));
        assert!(
            src.contains("catch_unwind"),
            "panicking fragment → violated"
        );
        assert!(src.contains("\"a\", \"b\", \"c\""), "state names");
        assert!(src.contains("(0, 1)"), "edges from the IR");
        assert!(src.contains("usize::MAX"), "unbounded state cap");
        assert!(
            !SCRATCH_RUN_MANIFEST.contains("[dependencies]"),
            "dependency-free scratch crate — builds offline, instantly"
        );
    }

    #[test]
    fn facts_parse_from_the_scratch_line() {
        let facts: RunFacts = serde_json::from_str(
            r#"{"violated_invariant_id":"c_hold","trace":["a","b","c"],"states_explored":3,"deadline_hit":false,"states_capped":false}"#,
        )
        .expect("the scratch line parses");
        assert_eq!(facts.violated_invariant_id.as_deref(), Some("c_hold"));
        assert_eq!(
            facts.trace,
            Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn facts_map_to_outcomes() {
        let ids = ["c_hold".to_string()];
        let backend = Backend {
            engine: EXEC_ENGINE.into(),
            version: EXEC_VERSION.into(),
        };
        let clean = RunFacts {
            violated_invariant_id: None,
            trace: None,
            states_explored: 3,
            deadline_hit: false,
            states_capped: false,
            depth_capped: false,
        };
        let report = report_from_facts(
            &clean,
            "sha256".to_string(),
            &ids,
            &backend,
            &Bound::default(),
        );
        assert_eq!(report.outcome, Outcome::NoCounterexample);
        assert_eq!(report.invariants_checked, ids);
        assert_eq!(report.violated_invariant_id, None);

        let violated = RunFacts {
            violated_invariant_id: Some("c_hold".into()),
            trace: Some(vec!["a".into(), "b".into(), "c".into()]),
            states_explored: 3,
            deadline_hit: false,
            states_capped: false,
            depth_capped: false,
        };
        let report = report_from_facts(
            &violated,
            "sha256".to_string(),
            &ids,
            &backend,
            &Bound::default(),
        );
        assert_eq!(report.outcome, Outcome::CounterexampleFound);
        assert_eq!(report.violated_invariant_id.as_deref(), Some("c_hold"));
        assert_eq!(
            report.trace,
            Some(
                ["a", "b", "c"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            )
        );

        let capped = RunFacts {
            violated_invariant_id: None,
            trace: None,
            states_explored: 1000,
            deadline_hit: true,
            states_capped: false,
            depth_capped: false,
        };
        let report = report_from_facts(
            &capped,
            "sha256".to_string(),
            &ids,
            &backend,
            &Bound::default(),
        );
        assert_eq!(report.outcome, Outcome::TimedOut);
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
            invariant_statuses: vec![],
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
        let artifact = artifact_for(&["a", "b", "c"], &[("t1", "a", "b")], &[]);
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
            invariant_statuses: vec![],
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
        // Decision 3 Option A, unchanged for fragment-less IR: prose
        // predicates are uninterpreted — the in-process native backend
        // checks nothing semantic and never invents a counterexample.
        // Fragment-bearing IR takes run_executable instead.
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
            &[],
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
            out.push_str(&format!("  \\* {id}: {from} -> {to} (guard: some-guard)\n"));
        }
        for (from, to) in code {
            out.push_str(&format!("  \\/ vpc = \"{from}\" /\\ vpc' = \"{to}\"\n"));
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

    /// An emitting IR: state `a` emits effect-constraint `C`, whose
    /// emitted Output value is the constraint's expr (as model_to_tla
    /// writes it).
    fn emitting_ir() -> ModelIr {
        serde_json::from_str(
            r#"{
                "states": [{"id": "a"}, {"id": "b"}, {"id": "c"}],
                "transitions": [
                    {"id": "t1", "from": "a", "to": "b", "guard": null},
                    {"id": "t2", "from": "b", "to": "c", "guard": null}
                ],
                "emits": {"a": "C"},
                "emits_values": {"a": "`C's expr`"}
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn output_missing_when_the_spec_emits_is_rejected() {
        // specodelic-8nt (Output half): the module has no Output entries
        // while the spec's model emits from `a` — the Output function
        // must be compared against the IR, never skipped.
        let artifact = artifact_for(&["a", "b", "c"], &[("t1", "a", "b"), ("t2", "b", "c")], &[]);
        let e = run(&emitting_ir(), &artifact, &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "stale_artifact");
    }

    #[test]
    fn output_value_drift_without_recompile_is_rejected() {
        // The entry exists but its value was hand-edited — the domain
        // alone is insufficient; the emitted value must match the IR.
        let artifact = artifact_for(
            &["a", "b", "c"],
            &[("t1", "a", "b"), ("t2", "b", "c")],
            &[("a", "a hand-edited value")],
        );
        let e = run(&emitting_ir(), &artifact, &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "stale_artifact");
    }

    #[test]
    fn artifact_output_entry_without_spec_emits_is_rejected() {
        // The spec emits nothing; the module claims an Output entry.
        let artifact = artifact_for(
            &["a", "b", "c"],
            &[("t1", "a", "b"), ("t2", "b", "c")],
            &[("a", "anything")],
        );
        let e = run(&chain_ir(), &artifact, &Bound::default()).unwrap_err();
        assert_eq!(e.stage, "stale_artifact");
    }

    #[test]
    fn output_matching_the_ir_passes() {
        // The positive leg: a module whose Output function matches the
        // IR verbatim runs clean (well — exploratory, specodelic-len).
        let artifact = artifact_for(
            &["a", "b", "c"],
            &[("t1", "a", "b"), ("t2", "b", "c")],
            &[("a", "`C's expr`")],
        );
        let report = run(&emitting_ir(), &artifact, &Bound::default()).unwrap();
        assert_eq!(report.outcome, Outcome::ExplorationOnly);
    }

    #[test]
    fn edited_transitions_without_recompile_are_rejected_too() {
        let drifted = artifact_for(&["a", "b", "c"], &[("t1", "a", "b")], &[]); // t2 missing
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
        let artifact = artifact_for(&["a", "b"], &[("loop", "a", "a"), ("t", "a", "b")], &[]);
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

    // ---- specodelic-ug3: the opt-in TLC backend (JVM reference engine) ----

    /// The version probe's output shape (`java -cp tla2tools.jar tlc2.TLC
    /// -version`): a single line naming the engine and its version.
    #[test]
    fn tlc_version_line_parses() {
        assert_eq!(
            parse_tlc_version("TLC2 version 1.20.0 of 12 May 2024\n"),
            Some("1.20.0".to_string())
        );
        assert_eq!(parse_tlc_version("Error: could not open jar"), None);
    }

    #[test]
    fn tlc_completed_run_parses_as_completed_with_states() {
        let out = parse_tlc_output(
            "Model checking completed. No error has been found.\n\
             43 states generated, 42 distinct states found, 0 states left on queue.\n",
        );
        assert!(out.completed);
        assert!(!out.depth_cutoff);
        assert_eq!(out.states_generated, Some(43));
        assert_eq!(out.violated_invariant, None);
    }

    #[test]
    fn tlc_depth_cutoff_parses_as_not_completed() {
        // TLC's depth-bound stop message: behaviors cut at -depth are
        // reported as "error-free so far" — the run did NOT complete.
        let out = parse_tlc_output(
            "The behavior up to this point is error-free.\n\
             5 states generated, 5 distinct states found.\n",
        );
        assert!(!out.completed);
        assert!(out.depth_cutoff);
        assert_eq!(out.states_generated, Some(5));
    }

    #[test]
    fn tlc_invariant_violation_is_detected() {
        let out = parse_tlc_output(
            "Error: Invariant TypeOK is violated.\n\
             3 states generated.\n",
        );
        assert_eq!(out.violated_invariant, Some("TypeOK".to_string()));
        assert!(!out.completed);
    }

    #[test]
    fn tlc_run_with_missing_binary_is_a_labeled_error_never_a_verdict() {
        // MUST (specodelic-ug3): a missing checker binary is an ERROR —
        // it can never degrade into a no_counterexample result.
        let dir = tempfile::tempdir().unwrap();
        let module = dir.path().join("chain.tla");
        std::fs::write(&module, artifact()).unwrap();
        let paths = TlcPaths {
            java: dir.path().join("no-such-java"),
            jar: dir.path().join("tla2tools.jar"),
        };
        let e = run_tlc(&chain_ir(), &module, &artifact(), &Bound::default(), &paths).unwrap_err();
        assert_eq!(e.stage, "missing_checker");
    }

    #[test]
    fn tlc_run_with_missing_jar_is_a_labeled_error_never_a_verdict() {
        let dir = tempfile::tempdir().unwrap();
        let module = dir.path().join("chain.tla");
        std::fs::write(&module, artifact()).unwrap();
        let paths = TlcPaths {
            java: std::path::PathBuf::from("sh"),
            jar: dir.path().join("no-such.jar"),
        };
        let e = run_tlc(&chain_ir(), &module, &artifact(), &Bound::default(), &paths).unwrap_err();
        assert_eq!(e.stage, "missing_checker");
    }

    /// A stand-in JVM: prints the canned version line for `-version`, and
    /// the given run output for any other invocation — the seam the TLC
    /// backend drives, so the classification logic is testable without a
    /// real JVM (real-TLC agreement is external evidence, vv8).
    #[cfg(unix)]
    fn fake_java(dir: &tempfile::TempDir, run_output: &str, exit_code: i32) -> TlcPaths {
        let shim = dir.path().join("fake-java.sh");
        let script = format!(
            "#!/bin/sh\nfor arg in \"$@\"; do case \"$arg\" in -version) echo 'TLC2 version 1.20.0 of 12 May 2024'; exit 0;; esac; done\ncat <<'TLCOUT'\n{run_output}\nTLCOUT\nexit {exit_code}\n"
        );
        std::fs::write(&shim, script).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(dir.path().join("tla2tools.jar"), b"placeholder jar").unwrap();
        TlcPaths {
            java: shim,
            jar: dir.path().join("tla2tools.jar"),
        }
    }

    #[cfg(unix)]
    fn module_file(dir: &tempfile::TempDir) -> std::path::PathBuf {
        let module = dir.path().join("chain.tla");
        std::fs::write(&module, artifact()).unwrap();
        module
    }

    #[cfg(unix)]
    #[test]
    fn tlc_completed_run_reports_backend_tlc_and_exploration_only() {
        // Zero corpus invariants are executable in the module (prose —
        // Decision 3, Option A), so even the reference engine's completed
        // run is exploration_only, never no_counterexample — the two
        // backends' reports are comparable (backend_identified).
        let dir = tempfile::tempdir().unwrap();
        let module = module_file(&dir);
        let paths = fake_java(
            &dir,
            "Model checking completed. No error has been found.\n43 states generated, 42 distinct states found, 0 states left on queue.",
            0,
        );
        let report = run_tlc(&chain_ir(), &module, &artifact(), &Bound::default(), &paths).unwrap();
        assert_eq!(report.backend.engine, "tlc");
        assert_eq!(report.backend.version, "1.20.0");
        assert_eq!(report.outcome, Outcome::ExplorationOnly);
        assert_eq!(report.states_explored, 43);
        assert!(report.invariants_checked.is_empty());
        assert_eq!(report.artifact_sha256, artifact_sha256(&artifact()));
        assert_eq!(report.bound, Bound::default());
    }

    #[cfg(unix)]
    #[test]
    fn tlc_depth_cutoff_reports_timed_out() {
        let dir = tempfile::tempdir().unwrap();
        let module = module_file(&dir);
        let paths = fake_java(
            &dir,
            "The behavior up to this point is error-free.\n5 states generated, 5 distinct states found.",
            0,
        );
        let report = run_tlc(&chain_ir(), &module, &artifact(), &Bound::default(), &paths).unwrap();
        assert_eq!(report.outcome, Outcome::TimedOut);
        assert_eq!(report.backend.engine, "tlc");
    }

    #[cfg(unix)]
    #[test]
    fn tlc_engine_invariant_violation_is_a_labeled_error_not_a_counterexample() {
        // The module's only invariant is TypeOK — engine-generated, not a
        // Constraints-table id. The contract requires a counterexample to
        // name a corpus invariant (counterexample_names_violated_invariant,
        // deferred on mp1's predicate-fragment decision), so an engine-
        // invariant violation is a labeled error, never counterexample_found.
        let dir = tempfile::tempdir().unwrap();
        let module = module_file(&dir);
        let paths = fake_java(
            &dir,
            "Error: Invariant TypeOK is violated.\n3 states generated.",
            1,
        );
        let e = run_tlc(&chain_ir(), &module, &artifact(), &Bound::default(), &paths).unwrap_err();
        assert_eq!(e.stage, "tlc_invariant_violated");
    }

    #[cfg(unix)]
    #[test]
    fn tlc_nonzero_exit_without_completion_is_a_labeled_error() {
        let dir = tempfile::tempdir().unwrap();
        let module = module_file(&dir);
        let paths = fake_java(&dir, "Exception in thread main ...", 1);
        let e = run_tlc(&chain_ir(), &module, &artifact(), &Bound::default(), &paths).unwrap_err();
        assert_eq!(e.stage, "tlc_error");
    }

    #[cfg(unix)]
    #[test]
    fn tlc_exit_zero_without_a_known_verdict_fails_closed() {
        // Unrecognized output on a zero exit is never silently mapped to a
        // verdict — the run reports a labeled error instead.
        let dir = tempfile::tempdir().unwrap();
        let module = module_file(&dir);
        let paths = fake_java(&dir, "something unexpected", 0);
        let e = run_tlc(&chain_ir(), &module, &artifact(), &Bound::default(), &paths).unwrap_err();
        assert_eq!(e.stage, "tlc_error");
    }

    #[cfg(unix)]
    #[test]
    fn tlc_wall_clock_budget_exhaustion_kills_the_run_as_timed_out() {
        // A shim that ignores its budget: the backend must enforce the
        // wall-clock bound itself (kill the JVM), reporting timed_out.
        let dir = tempfile::tempdir().unwrap();
        let module = module_file(&dir);
        let shim = dir.path().join("slow-java.sh");
        std::fs::write(
            &shim,
            "#!/bin/sh\nfor arg in \"$@\"; do case \"$arg\" in -version) echo 'TLC2 version 1.20.0 of 12 May 2024'; exit 0;; esac; done\nexec sleep 30\n",
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(dir.path().join("tla2tools.jar"), b"placeholder").unwrap();
        let paths = TlcPaths {
            java: shim,
            jar: dir.path().join("tla2tools.jar"),
        };
        let bound = Bound {
            timeout_secs: Some(0),
            ..Default::default()
        };
        let report = run_tlc(&chain_ir(), &module, &artifact(), &bound, &paths).unwrap();
        assert_eq!(report.outcome, Outcome::TimedOut);
    }

    #[cfg(unix)]
    #[test]
    fn tlc_max_states_budget_is_unsupported_and_labeled() {
        // TLC has no state-count cap (its bound is -depth); an unsupported
        // budget is a labeled error, never silently ignored.
        let dir = tempfile::tempdir().unwrap();
        let module = module_file(&dir);
        let paths = fake_java(
            &dir,
            "Model checking completed. No error has been found.\n1 states generated.",
            0,
        );
        let bound = Bound {
            max_states: Some(10),
            ..Default::default()
        };
        let e = run_tlc(&chain_ir(), &module, &artifact(), &bound, &paths).unwrap_err();
        assert_eq!(e.stage, "unsupported_bound");
    }

    // --- slice 1 (specodelic-txo): guard citation semantics ---
    // The citation-algebra and report-status tests live in
    // tests/citation_algebra.rs (integration scope — the file_lines
    // ratchet pins this module; pretender.toml is shrink-only).
}

//! conform — external-oracle trace conformance verdict engine.
//!
//! Purpose: check a spec against reality instead of against itself —
//! classify traces recorded from an external oracle against what the
//! spec declares and can execute (openspec/changes/add-conform).
//! Responsibilities: the closed five-valued verdict taxonomy (design D1),
//! claim classification reusing model_check's machinery (D4), mechanical
//! trace classification against the compiled Model's transition relation
//! (D3 — string identity after trimming), and the closed-world evidence
//! gate (D2). Read-only: it evaluates, it never generates.
//! Rationale: phase 1 is the pure engine — no CLI wiring, no report
//! envelope, no input gate (later phases).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::compile::{self, ModelIr};
use crate::spec::Spec;

// ---------------------------------------------------------------------------
// The closed verdict taxonomy (design D1 — five distinct values, never
// collapsed to pass/fail)
// ---------------------------------------------------------------------------

/// One oracle trace's verdict. The set is closed: `permitted`,
/// `forbidden`, `underspecified`, `unknown`, `unsupported` — distinct
/// values with distinct remediations, never coerced into a binary
/// pass/fail (report §13: never collapse the ladder into one PASS label).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The trace is explainable by the declared Model and executable
    /// claims over it.
    Permitted,
    /// Positive contradiction with a declared executable claim (any
    /// invocation mode), or absence of coverage under an explicitly
    /// declared closed-world mode (design D2).
    Forbidden,
    /// No declared Model element or claim covers the trace — author a
    /// claim or declare the trace's vocabulary.
    Underspecified,
    /// A claim covers the trace but its content is not interpretable by
    /// this run (prose-only expr) — make the claim interpretable.
    Unknown,
    /// The covering claim's evaluator kind exists in the format but is
    /// not executable in this run (e.g. `**py:**` — no emitter yet).
    Unsupported,
}

impl Verdict {
    /// The verdict's snake_case spelling (report records, exit-code
    /// mapping, and reason text all use this one form).
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Permitted => "permitted",
            Verdict::Forbidden => "forbidden",
            Verdict::Underspecified => "underspecified",
            Verdict::Unknown => "unknown",
            Verdict::Unsupported => "unsupported",
        }
    }
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Claim classification (design D4 — reuse model_check's machinery, no new
// status vocabulary: the required/unchecked split of
// `required_claims_classified`, extended only by the existing
// no_emitter_labeled_failure evaluator kinds)
// ---------------------------------------------------------------------------

/// The evaluator kind a claim opts into — the format's own closed set
/// (rust/py/ts fragments, kernel, citation); `prose` is the default that
/// opts into nothing (`required_claims_classified`: prose never implies
/// evaluation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evaluator {
    /// `**rust:**` fragment — executable in this run.
    RustFragment,
    /// `**kernel:**` expression — executable in this run.
    Kernel,
    /// Citation expression (`[[a]] ∧ [[b]]`) — executable via the
    /// citation algebra.
    Citation,
    /// Prose — not interpretable by this run; never implies a judgment.
    Prose,
    /// `**py:**` fragment — grammar-valid, no emitter in this run
    /// (`no_emitter_labeled_failure`).
    Py,
    /// `**ts:**` fragment — grammar-valid, no emitter in this run.
    Ts,
}

impl Evaluator {
    /// The kind's label as written in the format (`rust`, `kernel`,
    /// `citation`, `prose`, `py`, `ts`).
    pub fn label(self) -> &'static str {
        match self {
            Evaluator::RustFragment => "rust",
            Evaluator::Kernel => "kernel",
            Evaluator::Citation => "citation",
            Evaluator::Prose => "prose",
            Evaluator::Py => "py",
            Evaluator::Ts => "ts",
        }
    }

    /// True when the kind is executable in this run — the required set of
    /// `required_claims_classified`.
    pub fn is_executable(self) -> bool {
        matches!(
            self,
            Evaluator::RustFragment | Evaluator::Kernel | Evaluator::Citation
        )
    }
}

/// One invariant-kind Constraint row and the evaluator kind its expr
/// cell opts into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedClaim {
    pub id: String,
    pub evaluator: Evaluator,
}

impl ClassifiedClaim {
    /// The kind's label (`unsupported_kind_named` names the kind).
    pub fn evaluator_label(&self) -> &'static str {
        self.evaluator.label()
    }
}

/// The `required_claims_classified` partition for one file's invariant
/// claims: `required` (executable in this run), `unchecked` (prose-only —
/// never implied verified or judged), `unsupported` (kind exists in the
/// format but has no emitter in this run).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClaimClassification {
    pub required: Vec<ClassifiedClaim>,
    pub unchecked: Vec<ClassifiedClaim>,
    pub unsupported: Vec<ClassifiedClaim>,
}

impl ClaimClassification {
    /// The claim with this id, if the file declares it as an invariant.
    pub fn claim(&self, id: &str) -> Option<&ClassifiedClaim> {
        self.required
            .iter()
            .chain(self.unchecked.iter())
            .chain(self.unsupported.iter())
            .find(|c| c.id == id)
    }
}

/// Partition the file's invariant-kind Constraints into
/// required/unchecked/unsupported (design D4 — the same classification
/// model_check's `required_claims_classified` fixes: executable =
/// rust fragment, kernel, or citation; prose-only = unchecked; py/ts =
/// grammar-valid but emitter-less in this run). Advisory/effect/
/// extension-point rows are never required, so they never appear.
pub fn classify_claims(spec: &Spec) -> ClaimClassification {
    let ir = extract_ir(spec);
    let mut out = ClaimClassification::default();
    for row in &spec.constraints {
        if row.cells.get("kind").map(String::as_str) != Some("invariant") {
            continue;
        }
        let expr = row.cells.get("expr").map(String::as_str).unwrap_or("");
        let evaluator = match compile::fragment_of_strict(expr) {
            // A marker-shaped cell — its language decides executability.
            Ok(Some(fragment)) => match fragment.lang {
                compile::FragmentLang::Rust => Evaluator::RustFragment,
                compile::FragmentLang::Py => Evaluator::Py,
                compile::FragmentLang::Ts => Evaluator::Ts,
            },
            // No marker: kernel opt-in, else citation expr, else prose.
            _ => {
                if ir.guard_kernel.contains_key(row.id.as_str()) {
                    Evaluator::Kernel
                } else if compile::parse_citation_expr(expr).is_some() {
                    Evaluator::Citation
                } else {
                    Evaluator::Prose
                }
            }
        };
        let claim = ClassifiedClaim {
            id: row.id.clone(),
            evaluator,
        };
        match evaluator {
            e if e.is_executable() => out.required.push(claim),
            Evaluator::Prose => out.unchecked.push(claim),
            _ => out.unsupported.push(claim),
        }
    }
    out
}

/// Extract the compiled Model IR once per classification pass (the same
/// extraction model_check consumes — design D4). Memoized behind a cell
/// is unnecessary: the IR extraction is cheap and pure.
fn extract_ir(spec: &Spec) -> ModelIr {
    compile::extract_model_ir(spec)
}

// ---------------------------------------------------------------------------
// Scenario corpus (design D3 — one JSON object per line, mechanical name
// identity after trimming)
// ---------------------------------------------------------------------------

/// One recorded step of an oracle trace: the action taken (a transition
/// id/name) and optional observations (state/effect references). The
/// field is named `observations` — never `observes` — to avoid colliding
/// with the format's typed reference field of the same name (design D3).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TraceStep {
    pub action: String,
    /// Observed state/effect references: the string values of the JSON
    /// object (keys are free-variable names, opaque to conform).
    #[serde(default)]
    pub observations: BTreeMap<String, serde_json::Value>,
}

/// One scenario: a unique id, the setup (opaque except for the initial
/// state it names), and the ordered trace.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioTrace {
    pub id: String,
    #[serde(default)]
    pub setup: BTreeMap<String, serde_json::Value>,
    pub trace: Vec<TraceStep>,
}

/// Rule 1's verdict depends on the recorded declaration (design D2):
/// absence of coverage is `underspecified` in the default open-world mode
/// (`uncovered_trace_never_forbidden`) and `forbidden` only under the
/// explicitly declared closed-world mode (`closed_world_forbidden_recorded`).
/// The detail names the uncovered element; the declaration is named in
/// the forbidden reason.
fn uncovered_record(
    scenario_id: &str,
    detail: String,
    evaluated: Vec<String>,
    closed_world: bool,
) -> VerdictRecord {
    let (verdict, rule) = if closed_world {
        (Verdict::Forbidden, "closed_world_forbidden_recorded")
    } else {
        (Verdict::Underspecified, "uncovered_trace_never_forbidden")
    };
    let mut reason = format!("{rule}: {detail}");
    if closed_world {
        reason.push_str(" — forbidden under the declared closed-world mode (--closed-world)");
    }
    VerdictRecord {
        scenario_id: scenario_id.to_string(),
        verdict,
        reason,
        evaluated_claim_ids: evaluated,
        closed_world,
    }
}

/// A labeled failure — never a silent partial result. Carries a
/// remediation hint per the error contract (design D3: unknown fields
/// are refused with a hint, never silently ignored).
#[derive(Debug, Clone, PartialEq)]
pub struct ConformError {
    pub stage: String,
    pub message: String,
}

impl std::fmt::Display for ConformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.stage, self.message)
    }
}

impl std::error::Error for ConformError {}

/// Parse a JSONL scenario corpus: one object per line, each
/// `{id, setup?, trace: [{action, observations?}]}`. Unknown fields and
/// malformed lines are refused with a remediation hint, never silently
/// ignored (design D3). Full corpus validation (duplicate ids, missing
/// id) lands with the phase-3 input gate.
pub fn parse_corpus(jsonl: &str) -> Result<Vec<ScenarioTrace>, ConformError> {
    let mut out = Vec::new();
    for (n, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let scenario: ScenarioTrace = serde_json::from_str(line).map_err(|e| ConformError {
            stage: "corpus_parse".into(),
            message: format!(
                "line {}: {e} — fix the scenario record (one JSON object per line: {{id, setup?, trace}}; unknown fields are refused, never ignored)",
                n + 1
            ),
        })?;
        out.push(scenario);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Trace classification — the verdict engine
// ---------------------------------------------------------------------------

/// One trace's verdict record: the scenario it classifies, the verdict,
/// the labeled reason (naming the rule and the claims/elements that
/// produced it), the ids of the claims the run evaluated, and whether
/// the invocation declared closed-world mode (recorded in EVERY record,
/// never inferred — design D2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerdictRecord {
    pub scenario_id: String,
    pub verdict: Verdict,
    pub reason: String,
    pub evaluated_claim_ids: Vec<String>,
    pub closed_world: bool,
}

/// Classify every corpus trace against the spec's compiled Model
/// transition relation and its invariant claims. Every trace receives
/// exactly one verdict from the closed taxonomy (totality); classification
/// is mechanical — string identity after trimming, never semantic (D3).
///
/// The rules, in evaluation order per step:
///
/// 1. **Uncovered** — the trace references a name (initial state from
///    `setup`, action, or observation value) that no declared Model
///    element declares, or `setup` names no declared state: no Model
///    element or claim covers the trace. Open-world → `underspecified`
///    (`uncovered_trace_never_forbidden`); with the closed-world
///    declaration → `forbidden` (`closed_world_forbidden_recorded`).
///    Undeclared names alone never produce `forbidden` without the
///    declaration (design D2/D3).
/// 2. **Positive contradiction** — a declared transition exercised from
///    a state it is not declared from, or an observation naming a state
///    other than the transition's declared target: the trace asserts a
///    step/outcome the compiled Model forbids — the same fact
///    model_check reports as `counterexample_found`. `forbidden` in ANY
///    invocation mode (`contradiction_forbidden_any_mode`), naming the
///    executable claim(s) covering the violated transition.
/// 3. **Unsupported** — a covering claim's evaluator kind has no emitter
///    in this run (`**py:**`/`**ts:**`): `unsupported`, naming the kind.
/// 4. **Unknown** — a covering claim is prose-only: `unknown`, naming
///    the claim (prose never implies a permitted/forbidden judgment).
/// 5. **Permitted** — the walk explains the trace through declared
///    transitions and any executable covering claims are consistent
///    (their guards enabled each step).
pub fn classify_traces(
    spec: &Spec,
    corpus: &[ScenarioTrace],
    closed_world: bool,
) -> Vec<VerdictRecord> {
    let ir = extract_ir(spec);
    let claims = classify_claims(spec);
    corpus
        .iter()
        .map(|scenario| classify_trace(spec, &ir, &claims, scenario, closed_world))
        .collect()
}

/// A declared transition, trimmed (D3 — mechanical name identity).
struct DeclaredTransition {
    id: String,
    from: String,
    to: String,
}

/// The classified verdict for one scenario, by the rules documented on
/// [`classify_traces`].
fn classify_trace(
    spec: &Spec,
    ir: &ModelIr,
    claims: &ClaimClassification,
    scenario: &ScenarioTrace,
    closed_world: bool,
) -> VerdictRecord {
    // Declared vocabulary, trimmed — string identity is the only
    // normalization (design D3, mechanical never semantic).
    let transitions: BTreeMap<&str, DeclaredTransition> = ir
        .transitions
        .iter()
        .map(|t| {
            (
                t.id.trim(),
                DeclaredTransition {
                    id: t.id.clone(),
                    from: t.from.trim().to_string(),
                    to: t.to.trim().to_string(),
                },
            )
        })
        .collect();
    let states: BTreeSet<&str> = ir.states.iter().map(|s| s.id.trim()).collect();
    // Effect ids: the constraints the Model's `emits` references.
    let effects: BTreeSet<&str> = ir.emits.values().map(|v| v.trim()).collect();

    let record = |verdict: Verdict, reason: String, evaluated: Vec<String>| VerdictRecord {
        scenario_id: scenario.id.clone(),
        verdict,
        reason,
        evaluated_claim_ids: evaluated,
        closed_world,
    };

    // Rule 1 (anchor): setup names the initial state; an absent or
    // undeclared state leaves no Model element anchoring the trace.
    let initial = scenario
        .setup
        .get("state")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .map(str::to_string);
    let Some(initial) = initial else {
        return uncovered_record(
            &scenario.id,
            format!(
                "scenario `{}` declares no initial state in setup — no Model element anchors the trace",
                scenario.id
            ),
            Vec::new(),
            closed_world,
        );
    };
    if !states.contains(initial.as_str()) {
        return uncovered_record(
            &scenario.id,
            format!(
                "setup state `{initial}` is not a declared Model state — no Model element anchors the trace"
            ),
            Vec::new(),
            closed_world,
        );
    }

    let mut current = initial;
    let mut covering: Vec<ClassifiedClaim> = Vec::new();
    let mut exercised: Vec<&str> = Vec::new();

    for (i, step) in scenario.trace.iter().enumerate() {
        let action = step.action.trim();
        // Rule 1: the action must name a declared transition.
        let Some(t) = transitions.get(action) else {
            return uncovered_record(
                &scenario.id,
                format!(
                    "step {} action `{action}` is not a declared Model transition — no declared Model element or claim covers the trace",
                    i + 1
                ),
                covering_ids(&covering),
                closed_world,
            );
        };
        // Rule 2: positive contradiction — the step is not enabled from
        // the current state; the compiled Model forbids this move.
        if t.from != current {
            let guards = covering_claims_of(spec, claims, &t.id);
            let reason = contradiction_reason(&t.id, &current, &guards);
            let evaluated = covering_ids(&guards);
            return record(Verdict::Forbidden, reason, evaluated);
        }
        // Rule 1/2 on observations: an observation value must name a
        // declared state or effect (else uncovered); a state observation
        // must equal the transition's declared target (else the trace
        // asserts an outcome the Model forbids).
        for value in observed_names(step) {
            let value = value.trim();
            if states.contains(value) {
                if value != t.to.as_str() {
                    let guards = covering_claims_of(spec, claims, &t.id);
                    let reason = format!(
                        "contradiction_forbidden_any_mode: step {} observed state `{value}` but transition `{}` declares target `{}` — the compiled Model forbids this outcome{}",
                        i + 1,
                        t.id,
                        t.to,
                        claims_suffix(&guards)
                    );
                    return record(Verdict::Forbidden, reason, covering_ids(&guards));
                }
            } else if !effects.contains(value) {
                return uncovered_record(
                    &scenario.id,
                    format!(
                        "step {} observation `{value}` names no declared Model state or effect — no declared Model element or claim covers the trace",
                        i + 1
                    ),
                    covering_ids(&covering),
                    closed_world,
                );
            }
        }
        // The step is explained; accumulate its covering claims.
        let guards = covering_claims_of(spec, claims, &t.id);
        for claim in guards {
            if !covering.iter().any(|c| c.id == claim.id) {
                covering.push(claim);
            }
        }
        exercised.push(&t.id);
        current = t.to.clone();
    }

    let _ = exercised;
    // Rule 3: an unsupported covering claim names its kind — the kind
    // exists in the format but is not executable in this run.
    if let Some(claim) = covering
        .iter()
        .find(|c| !c.evaluator.is_executable() && c.evaluator != Evaluator::Prose)
    {
        let reason = format!(
            "unsupported_evaluator_kind: claim `{}` covers this trace but its evaluator kind `{}` has no emitter in this run — the claim exists in the format, is not executable here",
            claim.id,
            claim.evaluator.label()
        );
        return record(Verdict::Unsupported, reason, covering_ids(&covering));
    }
    // Rule 4: a prose-only covering claim is not interpretable — unknown,
    // naming the claim; prose never implies a judgment.
    if let Some(claim) = covering.iter().find(|c| c.evaluator == Evaluator::Prose) {
        let reason = format!(
            "prose_claim_not_interpretable: claim `{}` covers this trace but its expr is prose — not interpretable by this run, never implied permitted or forbidden",
            claim.id
        );
        return record(Verdict::Unknown, reason, covering_ids(&covering));
    }
    // Rule 5: the walk explains the trace; executable covering claims'
    // guards enabled each step — permitted.
    let reason = match exercised.as_slice() {
        [] => format!(
            "permitted: scenario `{}` declares an empty trace from state `{current}` — explainable by the declared Model",
            scenario.id
        ),
        steps => format!(
            "permitted: trace through {} explainable by the declared Model's transition relation{}",
            steps
                .iter()
                .map(|s| format!("`{s}`"))
                .collect::<Vec<_>>()
                .join(", "),
            if covering.is_empty() {
                String::new()
            } else {
                format!(
                    " — executable claims evaluated: {}",
                    covering_ids(&covering).join(", ")
                )
            }
        ),
    };
    record(Verdict::Permitted, reason, covering_ids(&covering))
}

/// The string values of a step's observations object — the observed
/// state/effect references (keys are free-variable names, opaque).
fn observed_names(step: &TraceStep) -> Vec<String> {
    let mut out = Vec::new();
    for value in step.observations.values() {
        match value {
            serde_json::Value::String(s) => out.push(s.clone()),
            serde_json::Value::Array(items) => {
                out.extend(items.iter().filter_map(|i| i.as_str().map(str::to_string)));
            }
            _ => {}
        }
    }
    out
}

/// The invariant claims covering a transition: the local constraint ids
/// its guard cell cites (`[[...]]` targets, resolved to local row ids).
/// Mechanical extraction — a guard's cited claim ids are exactly the
/// links the format declares between the Model and the Constraints.
fn covering_claims_of(
    spec: &Spec,
    claims: &ClaimClassification,
    transition_id: &str,
) -> Vec<ClassifiedClaim> {
    let Some(t) = spec
        .transitions
        .iter()
        .find(|t| t.id.trim() == transition_id)
    else {
        return Vec::new();
    };
    let Some(guard) = t.guard.as_deref() else {
        return Vec::new();
    };
    cited_claim_ids(guard)
        .into_iter()
        .filter_map(|id| claims.claim(&id).cloned())
        .collect()
}

/// Extract `[[...]]` citation targets from a guard cell and resolve them
/// to local invariant claim ids: `demo.conform.locked` → `locked` when
/// the file prefix matches the file's own intent id. Mechanical string
/// surgery, never semantic interpretation of the guard's meaning.
fn cited_claim_ids(guard: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = guard;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else { break };
        let target = after[..end].trim();
        // `file.row` → local row id when the prefix is the file's own id;
        // a bare `row` target is already local. Non-invariant or foreign
        // targets resolve to nothing (claims.claim misses).
        let local = target
            .rsplit_once('.')
            .map(|(_, row)| row.to_string())
            .unwrap_or_else(|| target.to_string());
        out.push(local);
        rest = &after[end + 2..];
    }
    out
}

/// The `contradiction_forbidden_any_mode` reason: names the violated
/// transition, the state it was exercised from, and the executable
/// claim(s) covering it (the claim id the record must name).
fn contradiction_reason(transition: &str, from_state: &str, guards: &[ClassifiedClaim]) -> String {
    format!(
        "contradiction_forbidden_any_mode: transition `{transition}` is declared from another state — exercised from `{from_state}`, the compiled Model's transition relation forbids this step{}",
        claims_suffix(guards)
    )
}

/// The executable-claim suffix a contradiction reason carries: names the
/// contradicted executable claim id(s) covering the violated transition.
fn claims_suffix(guards: &[ClassifiedClaim]) -> String {
    let executable: Vec<&ClassifiedClaim> = guards
        .iter()
        .filter(|c| c.evaluator.is_executable())
        .collect();
    if executable.is_empty() {
        String::new()
    } else {
        format!(
            " — contradicts executable claim(s) {}",
            executable
                .iter()
                .map(|c| format!("`{}`", c.id))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Evaluated claim ids, deduplicated in first-seen order.
fn covering_ids(covering: &[ClassifiedClaim]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    covering
        .iter()
        .filter(|c| seen.insert(c.id.clone()))
        .map(|c| c.id.clone())
        .collect()
}

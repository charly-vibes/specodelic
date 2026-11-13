//! Id-literal resolution in the kernel evaluator (specodelic-8mta, E2
//! finding F1).
//!
//! Purpose: pin the honesty rule that a kernel id literal is
//! RESOLVED against the evaluation corpus — it must name a declared
//! node (Intent ids are file ids; row ids are file-qualified). An
//! unresolvable literal (unqualified `both_go`, or a typo'd
//! `file.both_gooo`) cannot be discharged: the claim evaluates to
//! `unknown` with a labeled reason naming the literal. Before this
//! rule, such a literal compared as an opaque string that matched
//! nothing, and `¬(∃ t ∈ Transition: t.to == both_go)` evaluated
//! `verified` — a vacuous pass over a claim that was never checked.
//!
//! Responsibilities: pin the three-valued verdicts for unresolved
//! literals (unknown — never verified, never counterexample), pin the
//! labeled reason (it names the literal), pin the unchanged verdicts
//! of RESOLVED literals (the regression guard), pin `reachable`'s
//! unresolvable target (unknown, not counterexample), and pin the
//! Kleene discipline around the new unknown (¬unknown stays unknown; a
//! counterexample conjunct still dominates ∧).
//!
//! Rationale: the kernel's `unknown` is honest only if a claim that
//! cannot be checked says so. A literal that resolves to nothing is
//! precisely such a claim, and the corpus already promises the honest
//! status (`kernel.kernel_status_three_valued` /
//! `kernel.kernel_decidable`). These tests live out-of-module as an
//! integration file — `src/kernel.rs` has no in-module test block for
//! this area and the ticket prefers the integration split.

use specodelic::compile::ThreeValued;
use specodelic::kernel::{KernelEnv, parse_kernel_str};
use specodelic::spec::Spec;

#[path = "common/mod.rs"]
mod common;

use common::UnwrapPanic;
use common::kernel_corpora::TRAFFIC;

/// The evaluation environment over the traffic corpus — the corpus
/// whose declared ids are file-qualified.
fn traffic_env() -> KernelEnv {
    let specs: Vec<Spec> = TRAFFIC
        .iter()
        .map(|f| specodelic::spec::parse_str(f).unwrap_p())
        .collect();
    KernelEnv::from_specs(&specs)
}

/// Evaluate one kernel expression over the traffic corpus, returning
/// the status and the labeled reason.
fn eval(expr: &str) -> (ThreeValued, Option<String>) {
    let expr = parse_kernel_str(expr).unwrap_p().unwrap_p();
    traffic_env().evaluate_reasoned(&expr)
}

// --- the honesty rule: an unresolvable literal is unknown ---

#[test]
fn unqualified_id_literal_is_unknown_never_a_vacuous_pass() {
    // The E2 probe shape (F1): `both_go` names no declared id — the
    // literal resolves to nothing, so the claim cannot be discharged.
    // It is `unknown` with a labeled reason — NEVER `verified` (the
    // vacuous pass the finding measured) and never `counterexample`.
    let (status, reason) = eval("∃ t ∈ Transition: t.to == both_go");
    assert_eq!(
        status,
        ThreeValued::Unknown,
        "an unresolvable id literal cannot be discharged — never a vacuous verdict"
    );
    let reason = reason.unwrap_p();
    assert!(
        reason.contains("`both_go`"),
        "the reason must name the offending literal: {reason}"
    );
}

#[test]
fn unqualified_id_literal_under_negation_stays_unknown() {
    // Kleene ¬: unknown stays unknown. The E2 probe's exact shape —
    // `¬(∃ t ∈ Transition: t.to == both_go)` — must NOT verify.
    let (status, reason) = eval("¬(∃ t ∈ Transition: t.to == both_go)");
    assert_eq!(
        status,
        ThreeValued::Unknown,
        "¬unknown is unknown — never a vacuous verified"
    );
    let reason = reason.unwrap_p();
    assert!(
        reason.contains("`both_go`"),
        "the reason must name the offending literal: {reason}"
    );
}

#[test]
fn typoed_qualified_id_literal_is_unknown_with_a_named_reason() {
    // A file-qualified literal that names no declared row is just as
    // unresolvable as an unqualified one — a typo must not read as a
    // refutation either.
    let (status, reason) = eval("∃ t ∈ Transition: t.to == demo.traf.both_gooo");
    assert_eq!(
        status,
        ThreeValued::Unknown,
        "a typo'd id literal cannot be discharged — unknown, never counterexample"
    );
    let reason = reason.unwrap_p();
    assert!(
        reason.contains("`demo.traf.both_gooo`"),
        "the reason must name the offending literal: {reason}"
    );
}

// --- the regression guard: resolved literals keep their verdicts ---

#[test]
fn resolved_id_literal_keeps_its_verdict() {
    // `demo.traf.both_go` IS a declared id (the state the transition
    // reaches) — the claim discharges exactly as before the resolution
    // rule.
    let (status, reason) = eval("∃ t ∈ Transition: t.to == demo.traf.both_go");
    assert_eq!(
        status,
        ThreeValued::Verified,
        "a resolved literal discharges"
    );
    assert_eq!(reason, None, "a verified status carries no reason");

    // A resolved literal that does not match is still a refutation —
    // the resolution rule never turns a mismatch into an unknown.
    let (status, _) = eval("∃ t ∈ Transition: t.to == demo.traf.start");
    assert_eq!(
        status,
        ThreeValued::Counterexample,
        "a resolved-but-mismatched literal still refutes"
    );
}

// --- reachable: the target id is resolve-checked too ---

#[test]
fn reachable_with_an_unresolvable_target_is_unknown() {
    // The target is a set-membership test over the forward closure; an
    // unresolvable target could never match, and reading that as a
    // refutation would report a claim it never checked. It is unknown
    // with a labeled reason naming the target.
    let (status, reason) = eval("reachable(demo.traf.go, demo.traf.nope, guard)");
    assert_eq!(
        status,
        ThreeValued::Unknown,
        "an unresolvable reachable target cannot be discharged"
    );
    let reason = reason.unwrap_p();
    assert!(
        reason.contains("`demo.traf.nope`"),
        "the reason must name the offending target: {reason}"
    );
}

#[test]
fn reachable_with_a_resolved_target_keeps_its_verdict() {
    // The guard morphism carries the transition to the constraint it
    // cites — the target resolves and the reachability holds.
    let (status, _) = eval("reachable(demo.traf.go, demo.traf.t1, guard)");
    assert_eq!(
        status,
        ThreeValued::Verified,
        "a resolved target discharges"
    );

    // A resolved target outside the closure is still a refutation.
    let (status, _) = eval("reachable(demo.traf.go, demo.traf.start, guard)");
    assert_eq!(
        status,
        ThreeValued::Counterexample,
        "a resolved-but-unreachable target still refutes"
    );
}

// --- Kleene discipline around the new unknown ---

#[test]
fn a_counterexample_conjunct_still_dominates_an_unknown_one() {
    // Kleene ∧: a refuted conjunct dominates the unknown one — the
    // new unknown never swallows a refutation.
    let (status, _) =
        eval("(∃ t ∈ Transition: t.to == demo.traf.start) ∧ (∃ t ∈ Transition: t.to == both_go)");
    assert_eq!(
        status,
        ThreeValued::Counterexample,
        "a counterexample conjunct dominates an unknown conjunct"
    );
}

#[test]
fn a_verified_conjunct_beside_an_unknown_one_is_unknown() {
    // Kleene ∧: verified ∧ unknown is unknown — never a pass.
    let (status, reason) =
        eval("(∃ t ∈ Transition: t.to == demo.traf.both_go) ∧ (∃ t ∈ Transition: t.to == both_go)");
    assert_eq!(
        status,
        ThreeValued::Unknown,
        "verified ∧ unknown is unknown — never coerced to pass"
    );
    let reason = reason.unwrap_p();
    assert!(
        reason.contains("`both_go`"),
        "the absorbed unknown keeps its labeled reason: {reason}"
    );
}

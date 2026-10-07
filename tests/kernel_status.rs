//! The three-valued status chain end-to-end — Tier B core of
//! add-min-expr-kernel (specodelic-7ga, tasks.md §3.4 GREEN; design
//! D1/D8).
//!
//! The full chain: a Constraints expr cell opts in with `**kernel:**`,
//! compile extracts the claim into `ModelIr.guard_kernel`, and
//! `KernelEnv` evaluates it over the acset instances — reporting
//! exactly one of `verified`/`counterexample`/`unknown`. `unknown` is
//! honest and propagates under Kleene rules; no path coerces unknown
//! to pass (dl/1 Kleene absorb). The statuses ARE the slice-1 citation
//! algebra's `ThreeValued`, never a re-invention.
//!
//! These live out-of-module as new integration tests (the ticket's
//! preferred split; src/model_check.rs is pinned by the pretender
//! file_lines ratchet — shrink-only).

use specodelic::compile::{ThreeValued, extract_model_ir};
use specodelic::kernel::{KernelEnv, parse_kernel_str};
use specodelic::spec::parse_str;

#[path = "common/mod.rs"]
mod common;

use common::kernel_corpora::{ABSORBING, BROKEN, CLEAN, UNKNOWABLE};

/// The end-to-end chain under test: parse → compile extraction →
/// evaluate each extracted claim over the acset instance. Returns
/// (constraint id, status) pairs, sorted.
fn chain(text: &str) -> Vec<(String, ThreeValued)> {
    let spec = parse_str(text).expect("fixture parses");
    let ir = extract_model_ir(&spec);
    let env = KernelEnv::from_specs(std::slice::from_ref(&spec));
    let mut out: Vec<(String, ThreeValued)> = ir
        .guard_kernel
        .iter()
        .filter_map(|(id, text)| {
            parse_kernel_str(text)
                .expect("extracted claims re-parse")
                .map(|expr| (id.clone(), env.evaluate(&expr)))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

// --- the chain end-to-end ---

#[test]
fn verified_claim_travels_the_whole_chain() {
    // Scenario: Verified and counterexample remain first-class — a
    // discharged claim reports verified, persisted in the extraction
    // the run output reads.
    let statuses = chain(CLEAN);
    assert_eq!(statuses.len(), 2, "both kernel cells extract");
    assert_eq!(statuses[0], ("a1".to_string(), ThreeValued::Verified));
    assert_eq!(statuses[1], ("a2".to_string(), ThreeValued::Verified));
}

#[test]
fn counterexample_claim_travels_the_whole_chain() {
    let statuses = chain(BROKEN);
    assert_eq!(
        statuses,
        vec![("b1".to_string(), ThreeValued::Counterexample)]
    );
}

#[test]
fn unknown_claim_travels_the_whole_chain_honestly() {
    // Scenario: Honest unknown propagates — a claim whose subclaim
    // cannot be discharged over the instance is unknown at the end of
    // the chain, never pass.
    let statuses = chain(UNKNOWABLE);
    assert_eq!(statuses, vec![("u1".to_string(), ThreeValued::Unknown)]);
}

#[test]
fn unknown_absorbs_in_composites_never_coerced_to_pass() {
    // dl/1 Kleene absorb: a verified conjunct plus an undischargable
    // one is UNKNOWN — no evaluation path coerces it to pass or to
    // counterexample.
    let statuses = chain(ABSORBING);
    assert_eq!(statuses, vec![("a1".to_string(), ThreeValued::Unknown)]);
}

#[test]
fn counterexample_dominates_unknown_in_composites() {
    // Kleene ∧: a counterexample conjunct dominates even alongside an
    // unknown — the honest reading of a refuted subclaim. The dangling
    // corpus refutes resolves(traces_to) while the reachability stays
    // unknown.
    let dangling = BROKEN.replace(
        "`**kernel:** resolves(traces_to)`",
        "`**kernel:** reachable(demo.ghost.nowhere, demo.st.broken.b1, supersedes) ∧ resolves(traces_to)`",
    );
    let statuses = chain(&dangling);
    assert_eq!(
        statuses,
        vec![("b1".to_string(), ThreeValued::Counterexample)],
        "a refuted subclaim dominates an unknown conjunct — never absorbed into unknown"
    );
}

#[test]
fn negation_swaps_verified_and_counterexample_keeps_unknown() {
    // Kleene ¬: verified ⇄ counterexample; unknown stays unknown —
    // `¬claim` on an undischargable claim is still honest unknown.
    let text = CLEAN.replace(
        "`**kernel:** acyclic(supersedes)`",
        "`**kernel:** ¬acyclic(supersedes)`",
    );
    let statuses = chain(&text);
    assert_eq!(statuses[0], ("a1".to_string(), ThreeValued::Counterexample));
    let text = UNKNOWABLE.replace(
        "reachable(demo.ghost.nowhere, demo.st.unk.u1, supersedes)",
        "¬reachable(demo.ghost.nowhere, demo.st.unk.u1, supersedes)",
    );
    let statuses = chain(&text);
    assert_eq!(statuses[0], ("u1".to_string(), ThreeValued::Unknown));
}

#[test]
fn empty_domain_quantifiers_decide_honestly() {
    // Bounded ∀ over an empty instance is verified (vacuous, finite);
    // bounded ∃ over an empty instance is counterexample — both
    // decided, never unknown.
    let spec = parse_str(
        "---\nid: demo.st.empty\nkind: intent\nstatement: \"THE empty SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| e1 | invariant | `**kernel:** ∀ t ∈ Transition: |Transition| == 0` | [[demo.st.empty]] |\n| e2 | invariant | `**kernel:** ∃ t ∈ Transition: |Transition| == 0` | [[demo.st.empty]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.st.empty.e1]] | `arb()` | `true` |\n",
    )
    .expect("parses");
    let ir = extract_model_ir(&spec);
    let env = KernelEnv::from_specs(std::slice::from_ref(&spec));
    assert!(env.domain("Transition").is_empty());
    for (id, expected) in [
        ("e1", ThreeValued::Verified),
        ("e2", ThreeValued::Counterexample),
    ] {
        let text = ir.guard_kernel.get(id).expect("claim extracted");
        let expr = parse_kernel_str(text)
            .expect("re-parses")
            .expect("an expression");
        assert_eq!(env.evaluate(&expr), expected, "claim {id}");
    }
}

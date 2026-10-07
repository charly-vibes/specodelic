//! Per-atomic grounding agreement — Tier B core of add-min-expr-kernel
//! (specodelic-7ga, tasks.md §3.3 GREEN; design D1).
//!
//! Each v0 atomic must AGREE with its grounding machinery on shared
//! fixtures: `acyclic`/`reachable` on graph traversal
//! (`acset::query::cyclic_nodes` / `forward_closure`), `unique`/
//! `resolves` on acset traversal (the instance's morphism vectors and
//! dangling report), `==`/comparisons/bounded ∀/∃ on bounded
//! evaluation over the finite instances I(k). An atomic without a
//! grounding entry cannot ship — the grounding table is the D1 table
//! as data.
//!
//! These live out-of-module as new integration tests (the ticket's
//! preferred split; src/model_check.rs is pinned by the pretender
//! file_lines ratchet — shrink-only).

use specodelic::acset::query;
use specodelic::compile::ThreeValued;
use specodelic::kernel::{
    Atomic, Grounding, KernelEnv, KernelExpr, PredicateRegistry, Term, parse_kernel_str,
};
use specodelic::spec::{Spec, parse_str};

#[path = "common/mod.rs"]
mod common;

use common::kernel_corpora::{CHAIN, CYCLIC, DANGLING, DUPLICATES};

/// Parse a per-file fixture list into a corpus (each chunk is one
/// complete spec file, parsed through the real parser).
fn corpus(files: &[&str]) -> Vec<Spec> {
    files
        .iter()
        .map(|f| parse_str(f).expect("fixture file parses"))
        .collect()
}

// --- acyclic: graph traversal agreement ---

#[test]
fn acyclic_agrees_with_graph_cycle_detection() {
    // Scenario: Per-atomic grounding holds — acyclic's result agrees
    // with the machinery's result (query::cyclic_nodes) on the same
    // instance, cyclic and acyclic alike.
    let cyclic_specs = corpus(CYCLIC);
    let cyclic_env = KernelEnv::from_specs(&cyclic_specs);
    let machinery_cyclic =
        query::cyclic_nodes(cyclic_env.instance(), &["supersedes"]).expect("machinery runs");
    assert!(!machinery_cyclic.is_empty(), "fixture must have a cycle");
    let expr = parse_kernel_str("acyclic(supersedes)")
        .expect("parses")
        .expect("an expression");
    assert_eq!(cyclic_env.evaluate(&expr), ThreeValued::Counterexample);

    let chain_specs = corpus(CHAIN);
    let chain_env = KernelEnv::from_specs(&chain_specs);
    let machinery_chain =
        query::cyclic_nodes(chain_env.instance(), &["supersedes"]).expect("machinery runs");
    assert!(machinery_chain.is_empty(), "fixture must be acyclic");
    assert_eq!(chain_env.evaluate(&expr), ThreeValued::Verified);
}

// --- reachable: graph traversal agreement ---

#[test]
fn reachable_agrees_with_forward_closure() {
    // Scenario: Per-atomic grounding holds — reachable's result agrees
    // with query::forward_closure on the same instance.
    let specs = corpus(CHAIN);
    let env = KernelEnv::from_specs(&specs);
    let instance = env.instance();
    // Machinery: nc reachable from na through supersedes (na → nb →
    // nc); nc's forward closure contains only itself.
    let machinery =
        query::forward_closure(instance, &["demo.chn.a.na"], &["supersedes"]).expect("machinery");
    assert!(machinery.contains("demo.chn.c.nc"));
    let yes = parse_kernel_str("reachable(demo.chn.a.na, demo.chn.c.nc, supersedes)")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&yes), ThreeValued::Verified);
    // The negative case: a row outside the closure entirely.
    let outside = parse_kernel_str("reachable(demo.chn.c.nc, demo.chn.a.na, supersedes)")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&outside), ThreeValued::Counterexample);
    // And the machinery agrees with that negative, too.
    let machinery_outside =
        query::forward_closure(instance, &["demo.chn.c.nc"], &["supersedes"]).expect("machinery");
    assert!(!machinery_outside.contains("demo.chn.a.na"));
}

#[test]
fn unknown_seed_is_unknown_never_a_fabricated_verdict() {
    // reachable's grounding (forward_closure) rejects unknown seeds —
    // the kernel reports unknown, honestly, never verified.
    let specs = corpus(CHAIN);
    let env = KernelEnv::from_specs(&specs);
    let ghost = parse_kernel_str("reachable(demo.ghost.nowhere, demo.chn.c.nc, supersedes)")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&ghost), ThreeValued::Unknown);
}

// --- resolves: acset traversal agreement ---

#[test]
fn resolves_agrees_with_the_dangling_report() {
    // Scenario: Per-atomic grounding holds — resolves agrees with the
    // instance's dangling report (acset traversal).
    let env = KernelEnv::from_specs(&corpus(DANGLING));
    let machinery_dangling: Vec<_> = env
        .instance()
        .dangling()
        .iter()
        .filter(|d| d.kind == "traces_to")
        .collect();
    assert_eq!(machinery_dangling.len(), 1, "fixture must dangle once");
    let bad = parse_kernel_str("resolves(traces_to)")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&bad), ThreeValued::Counterexample);

    let clean_env = KernelEnv::from_specs(&corpus(CHAIN));
    assert!(clean_env.instance().dangling().is_empty());
    let good = parse_kernel_str("resolves(traces_to)")
        .expect("parses")
        .expect("an expression");
    assert_eq!(clean_env.evaluate(&good), ThreeValued::Verified);
}

// --- unique: acset traversal agreement ---

#[test]
fn unique_agrees_with_injectivity_of_defined_values() {
    // Scenario: Per-atomic grounding holds — unique agrees with the
    // machinery: two constraints resolving to the same intent refute
    // injectivity; distinct targets verify.
    let env = KernelEnv::from_specs(&corpus(DUPLICATES));
    let dup = parse_kernel_str("unique(traces_to)")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&dup), ThreeValued::Counterexample);

    let env = KernelEnv::from_specs(&corpus(CHAIN));
    let good = parse_kernel_str("unique(traces_to)")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&good), ThreeValued::Verified);
}

// --- ==, comparisons, ∀/∃: bounded evaluation agreement ---

#[test]
fn comparisons_agree_with_bounded_evaluation() {
    let env = KernelEnv::from_specs(&corpus(CHAIN));
    // The chain corpus has 3 intents and 3 constraints.
    let card = parse_kernel_str("|Intent| == 3")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&card), ThreeValued::Verified);
    let wrong = parse_kernel_str("|Intent| == 4")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&wrong), ThreeValued::Counterexample);
    let le = parse_kernel_str("|Constraint| <= 3")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&le), ThreeValued::Verified);
    let gt = parse_kernel_str("|Constraint| > 3")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&gt), ThreeValued::Counterexample);
}

#[test]
fn bounded_quantifiers_agree_over_the_instance() {
    let env = KernelEnv::from_specs(&corpus(CHAIN));
    // ∀ over I(Constraint): finite, terminating, vacuous-truth-free.
    let all = parse_kernel_str("∀ c ∈ Constraint: |Constraint| >= 1")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&all), ThreeValued::Verified);
    // ∃ with a witness.
    let some = parse_kernel_str("∃ c ∈ Constraint: |Intent| == 3")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&some), ThreeValued::Verified);
    // ∃ refuted when no witness exists.
    let none = parse_kernel_str("∃ c ∈ Constraint: |Intent| == 99")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&none), ThreeValued::Counterexample);
    // Projection row-typing: c.supersedes resolves to an id or ⊥.
    let proj = parse_kernel_str("∃ c ∈ Constraint: c.supersedes == demo.chn.b.nb")
        .expect("parses")
        .expect("an expression");
    assert_eq!(env.evaluate(&proj), ThreeValued::Verified);
}

#[test]
fn every_v0_atomic_has_a_grounding_entry() {
    // Scenario: Ungrounded atomic cannot ship — the D1 grounding table
    // as data: every v0 atomic names its machinery; an atomic outside
    // the registry has none and cannot ship.
    let reg = PredicateRegistry::v0();
    for atomic in [
        "resolves",
        "unique",
        "acyclic",
        "reachable",
        "==",
        "<",
        "<=",
        ">",
        ">=",
        "∀",
        "∃",
    ] {
        assert!(
            reg.grounding(atomic).is_some(),
            "v0 atomic `{atomic}` must carry a grounding entry (D1)"
        );
    }
    assert_eq!(reg.grounding("acyclic"), Some(Grounding::GraphTraversal));
    assert_eq!(reg.grounding("reachable"), Some(Grounding::GraphTraversal));
    assert_eq!(reg.grounding("resolves"), Some(Grounding::AcsetTraversal));
    assert_eq!(reg.grounding("unique"), Some(Grounding::AcsetTraversal));
    assert_eq!(reg.grounding("=="), Some(Grounding::BoundedEvaluation));
    assert_eq!(reg.grounding("∀"), Some(Grounding::BoundedEvaluation));
    assert_eq!(reg.grounding("frobnicate"), None);
    // The parsed shape names the atomics too (the enum is closed).
    let _ = (
        KernelExpr::Atomic(Atomic::Resolves("traces_to".into())),
        Term::Dangling,
    );
}

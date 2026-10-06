//! The predicate-registry seam — Tier B tidy of add-min-expr-kernel
//! (specodelic-7ga, tasks.md §3.5 TIDY; design D8).
//!
//! The seam later pack registration rides: `PredicateRegistry` holds
//! the grounding table (D1 — no registration without a grounding
//! entry) behind D8's decidability gate (a predicate that is not
//! decidable over finite instances cannot register — it stays
//! pack-side, checked by contract-TOML runners). Registration code
//! exists here; NO pack predicate registers in this phase — the
//! shipped registry is exactly the v0 set, and grammar-level widening
//! rides the widening law in a later change.
//!
//! These live out-of-module as new integration tests (the ticket's
//! preferred split; src/model_check.rs is pinned by the pretender
//! file_lines ratchet — shrink-only).

use specodelic::compile::ThreeValued;
use specodelic::kernel::{
    Grounding, KernelEnv, PredicateRegistry, RegisterError, parse_kernel_str,
};
use specodelic::spec::parse_str;

/// The v0 registry is exactly the closed set — no pack predicate has
/// registered (D8 gate: nothing registers in this phase).
#[test]
fn shipped_registry_is_exactly_the_v0_closed_set() {
    let reg = PredicateRegistry::default();
    assert_eq!(
        reg.names(),
        vec![
            "<",
            "<=",
            "==",
            ">",
            ">=",
            "acyclic",
            "reachable",
            "resolves",
            "unique",
            "∀",
            "∃",
        ],
        "the shipped registry is the v0 grounding table — nothing outside it registers"
    );
    assert_eq!(reg.grounding("frobnicate"), None);
}

/// D8's gate: registration without a grounding entry is refused (D1),
/// without a decidability certificate is refused (D8), duplicates are
/// refused; a grounded, decidable predicate registers.
#[test]
fn registration_gate_refuses_ungrounded_and_undecidable() {
    let mut reg = PredicateRegistry::v0();
    // No grounding entry — an atomic without a grounding entry cannot
    // ship (kernel_grounded_in_machinery).
    assert_eq!(
        reg.try_register("pack_only", None, true),
        Err(RegisterError::NoGrounding("pack_only".into()))
    );
    // Grounded but undecidable — stays pack-side, checked by
    // contract-TOML runners (kernel_decidable).
    assert_eq!(
        reg.try_register("pack_only", Some(Grounding::BoundedEvaluation), false),
        Err(RegisterError::Undecidable("pack_only".into()))
    );
    // Neither attempt left an entry behind.
    assert_eq!(reg.grounding("pack_only"), None);
    // A grounded, decidable predicate registers through the seam.
    assert!(
        reg.try_register("pack_only", Some(Grounding::BoundedEvaluation), true)
            .is_ok()
    );
    assert_eq!(
        reg.grounding("pack_only"),
        Some(Grounding::BoundedEvaluation)
    );
    // A duplicate is refused.
    assert_eq!(
        reg.try_register("pack_only", Some(Grounding::BoundedEvaluation), true),
        Err(RegisterError::Duplicate("pack_only".into()))
    );
}

/// Widening purity (the D8 property, pinned as a test): after a
/// decidable registration, every expression valid before evaluates
/// identically after it.
#[test]
fn decidable_registration_widens_purely() {
    let spec = parse_str(
        "---\nid: demo.reg\nkind: intent\nstatement: \"THE reg SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| r1 | invariant | `holds r1` | [[demo.reg]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|-----------|\n| p1 | unit | [[demo.reg.r1]] | `arb()` | `true` |\n",
    )
    .expect("parses");
    let env = KernelEnv::from_specs(std::slice::from_ref(&spec));
    // A battery of v0 expressions — the pre-registration behavior.
    let exprs: Vec<_> = [
        "acyclic(supersedes)",
        "resolves(traces_to)",
        "unique(traces_to)",
        "|Intent| == 1",
        "∀ c ∈ Constraint: |Intent| == 1",
        "∃ c ∈ Constraint: c.traces_to == demo.reg",
    ]
    .iter()
    .map(|s| {
        (
            *s,
            parse_kernel_str(s).expect("parses").expect("an expression"),
        )
    })
    .collect();
    let before: Vec<ThreeValued> = exprs.iter().map(|(_, e)| env.evaluate(e)).collect();
    // Register a decidable predicate (grounding entry + certificate).
    let mut reg = PredicateRegistry::v0();
    reg.try_register("pack_only", Some(Grounding::AcsetTraversal), true)
        .expect("the decidable predicate registers");
    // Every v0 expression evaluates identically after the registration.
    let after: Vec<ThreeValued> = exprs.iter().map(|(_, e)| env.evaluate(e)).collect();
    assert_eq!(before, after, "widening is pure — nothing valid changed");
    // And the grammar did not widen with the registry entry: the new
    // predicate is not parseable this Revision (grammar-level widening
    // rides the widening law in a later change).
    assert!(
        parse_kernel_str("pack_only(x)").is_err(),
        "a registered pack predicate does not enter this Revision's closed grammar"
    );
}

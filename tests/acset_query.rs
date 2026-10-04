//! RED unit tests for the closure primitive (openspec change
//! `add-acset-core`, tasks 4.1): forward/backward closure over a seed set
//! and a morphism set M — the only traversal primitives (`single_traversal_primitive`),
//! shared by graph, merge, and refactor (tasks 4.4–4.6).
//!
//! Named after the spec's property rows: `unknown_seed_rejected`
//! (seeds_exist), `supersedes_scope_exact` (scope_by_morphism_set),
//! `closure_on_cycle_terminates` (closure_terminates),
//! `dangling_not_followed_in_traversal` (dangling_not_followed),
//! `closure_laws_hold` (contains / idempotence / monotonicity), and
//! `results_sorted_by_id` (results_ordered).
//!
//! RED: `specodelic::acset::query` does not exist yet — the compile error is
//! the observed red (same discipline as the 3.x parity property before
//! `from_specs` landed).

use std::collections::BTreeSet;

use proptest::prelude::*;
use specodelic::acset::instance::Instance;
use specodelic::acset::query::{self, Direction, Query, QueryError};
use specodelic::spec::{self, Spec};

/// A small two-file corpus exercising every closure-relevant shape: a
/// supersedes cycle (r1 → r2 → r1), a traces_to edge to an intent, a
/// derives_from edge to another file's constraint, and one dangling
/// traces_to — parsed through the real parser.
fn corpus() -> Vec<Spec> {
    let c1 = spec::parse_str(
        r#"---
id: c1
kind: intent
statement: "closure corpus: cycle, cross-file edge, dangling"
---

## Constraints

| id | kind | expr | traces_to | supersedes |
|----|------|------|-----------|------------|
| r1 | invariant | `true` | [[c2]] | [[c1.r2]] |
| r2 | invariant | `true` |  | [[c1.r1]] |
| r3 | invariant | `true` | [[c1.ghost_row]] |  |
| r4 | invariant | `true` |  |  |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[c2.r1]] | `arb()` | `true` |
"#,
    )
    .expect("c1 parses");
    let c2 = spec::parse_str(
        r#"---
id: c2
kind: intent
statement: "the rows c1's links resolve to"
---

## Constraints

| id | kind | expr |
|----|------|------|
| r1 | invariant | `true` |
"#,
    )
    .expect("c2 parses");
    vec![c1, c2]
}

/// The corpus's node set — deterministic (sorted-order interning,
/// `build_deterministic`), so the test can name every id.
const IDS: [&str; 7] = ["c1", "c1.p1", "c1.r1", "c1.r2", "c1.r3", "c1.r4", "c2.r1"];

/// The morphisms the corpus actually carries values under.
const MORPHS: [&str; 3] = ["traces_to", "supersedes", "derives_from"];

/// `unknown_seed_rejected` (spec.seeds_exist): a seed that is not an id of
/// the instance is rejected, not ignored — `check` fails naming it, and
/// `run` refuses to traverse.
#[test]
fn unknown_seed_rejected() {
    let instance = Instance::from_specs(&corpus());
    let q = Query {
        direction: Direction::Forward,
        seeds: BTreeSet::from(["c1.r1".to_string(), "c1.nobody".to_string()]),
        morphisms: BTreeSet::from(["supersedes".to_string()]),
    };
    assert_eq!(
        query::check(&q, &instance),
        Err(vec![QueryError::UnknownSeed("c1.nobody".to_string())]),
        "the unknown seed is named, the known one is not blamed"
    );
    assert!(query::run(&q, &instance).is_err(), "run refuses too");
}

/// `supersedes_scope_exact` (spec.scope_by_morphism_set): restricting M to
/// supersedes yields exactly the supersedes subgraph — the traces_to and
/// derives_from edges out of the same rows contribute nothing, and vice
/// versa.
#[test]
fn supersedes_scope_exact() {
    let instance = Instance::from_specs(&corpus());
    let sup = query::forward_closure(&instance, &["c1.r1"], &["supersedes"]).expect("seed exists");
    assert_eq!(
        sup,
        BTreeSet::from(["c1.r1".to_string(), "c1.r2".to_string()]),
        "the supersedes cycle is the whole result — no traces edge followed"
    );
    let traces =
        query::forward_closure(&instance, &["c1.r1"], &["traces_to"]).expect("seeds exist");
    assert_eq!(
        traces,
        BTreeSet::from(["c1.r1".to_string(), "c2".to_string()]),
        "M = traces_to reaches the intent, not the supersedes cycle"
    );
}

/// `closure_on_cycle_terminates` (spec.closure_terminates): closure over a
/// cyclic M terminates and visits each node at most once — a set-valued
/// result with the expected membership, not a hang or a stack overflow.
#[test]
fn closure_on_cycle_terminates() {
    let instance = Instance::from_specs(&corpus());
    let out = query::forward_closure(&instance, &["c1.r1", "c1.r2"], &["supersedes"])
        .expect("seeds exist");
    assert_eq!(
        out,
        BTreeSet::from(["c1.r1".to_string(), "c1.r2".to_string()]),
        "each node visited at most once — the cycle adds nothing twice"
    );
}

/// `dangling_not_followed_in_traversal` (spec.dangling_not_followed): the
/// unresolved reference is not an edge — traversal skips it, and the
/// dangling report lists it.
#[test]
fn dangling_not_followed_in_traversal() {
    let instance = Instance::from_specs(&corpus());
    let out = query::forward_closure(&instance, &["c1.r3"], &["traces_to"]).expect("seed exists");
    assert_eq!(
        out,
        BTreeSet::from(["c1.r3".to_string()]),
        "the dangling endpoint contributes no reachability"
    );
    assert!(
        instance
            .dangling()
            .iter()
            .any(|d| d.from == "c1.r3" && d.target == "c1.ghost_row"),
        "the dangling report lists the unresolved reference"
    );
}

// `closure_laws_hold` (spec.single_traversal_primitive): contains
// (S ⊆ closure(S)), idempotence (closure(closure(S)) == closure(S)),
// monotonicity (S ⊆ T implies closure(S) ⊆ closure(T)) — for both
// directions, over arbitrary seed and morphism sets. RED: the primitive
// does not exist yet.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn closure_laws(
        seeds_t in prop::collection::vec(any::<bool>(), IDS.len()),
        seeds_s in prop::collection::vec(any::<bool>(), IDS.len()),
        morphs in prop::collection::vec(any::<bool>(), MORPHS.len()),
        forward in any::<bool>(),
    ) {
        let instance = Instance::from_specs(&corpus());
        let morphs: Vec<&str> = MORPHS
            .iter()
            .zip(&morphs)
            .filter(|(_, on)| **on)
            .map(|(m, _)| *m)
            .collect();
        let t: Vec<&str> = IDS
            .iter()
            .zip(&seeds_t)
            .filter(|(_, on)| **on)
            .map(|(id, _)| *id)
            .collect();
        // S ⊆ T: S is the mask restricted to T's members.
        let s: Vec<&str> = t
            .iter()
            .zip(&seeds_s)
            .filter(|(_, on)| **on)
            .map(|(id, _)| *id)
            .collect();
        let direction = if forward { Direction::Forward } else { Direction::Backward };
        let q = |seeds: &[&str]| Query {
            direction,
            seeds: seeds.iter().map(|s| s.to_string()).collect(),
            morphisms: morphs.iter().map(|m| m.to_string()).collect(),
        };
        let run = |seeds: &[&str]| query::run(&q(seeds), &instance).expect("seeds exist");

        let closure_s = run(&s);
        // contains: S ⊆ closure(S)
        prop_assert!(s.iter().all(|seed| closure_s.contains(*seed)));
        // idempotence: closure(closure(S)) == closure(S)
        let again: Vec<&str> = closure_s.iter().map(|s| s.as_str()).collect();
        prop_assert_eq!(run(&again), closure_s.clone());
        // monotonicity: S ⊆ T implies closure(S) ⊆ closure(T)
        let closure_t = run(&t);
        prop_assert!(closure_s.is_subset(&closure_t));
    }
}

/// `results_sorted_by_id` (spec.results_ordered): query results are sorted
/// by file-qualified id — the same query over a corpus whose files were
/// built in reverse order returns the identical, sorted set.
#[test]
fn results_sorted_by_id() {
    let seeds = ["c1.r4", "c1.r1", "c2.r1", "c1.p1"];
    let morphs = ["traces_to", "supersedes", "derives_from"];
    let forward = Instance::from_specs(&corpus());
    let reversed = {
        let mut specs = corpus();
        specs.reverse();
        Instance::from_specs(&specs)
    };
    let out = query::forward_closure(&forward, &seeds, &morphs).expect("seeds exist");
    let out_reversed = query::forward_closure(&reversed, &seeds, &morphs).expect("seeds exist");
    assert_eq!(out, out_reversed, "file order cannot reach the result");
    let as_vec: Vec<&str> = out.iter().map(|s| s.as_str()).collect();
    let mut sorted = as_vec.clone();
    sorted.sort_unstable();
    assert_eq!(as_vec, sorted, "results are ordered by id");
}

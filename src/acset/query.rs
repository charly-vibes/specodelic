//! The query primitive (openspec change `add-acset-core`, tasks 4.x): the
//! single traversal primitive — forward/backward closure over a seed set
//! and a morphism set M — through which graph, merge, and refactor are
//! expressed (`single_traversal_primitive`).
//!
//! Purpose: answer reachability questions over a typed instance. A query
//! names its seeds (validated: an unknown seed is rejected, not ignored —
//! `seeds_exist`) and the schema morphisms to follow (`scope_by_morphism_set`:
//! restricting M to `supersedes` yields exactly the supersedes subgraph).
//! Closure terminates on any finite instance including cyclic M
//! (`closure_terminates` — one worklist, a visited set, each node visited
//! at most once); an unresolved reference is `None` in the morphism vector
//! and is never followed (`dangling_not_followed`); results are a
//! `BTreeSet` of file-qualified ids, so they are sorted by construction
//! (`results_ordered`). Rationale: graph's duplicated edge loops and
//! merge's private adjacency map are two renderings of this one walk;
//! both migrate onto it behind parity gates (tasks 4.3–4.5).

use std::collections::{BTreeMap, BTreeSet};

use crate::acset::instance::Instance;

/// Which way the closure walks the morphism values: forward follows
/// `from -> to` (what a definition reaches), backward follows `to -> from`
/// (what depends on it — merge's dependents half).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

/// One reachability question: walk the instance from `seeds` along the
/// named schema morphisms, in `direction`. `run` rejects unknown seeds
/// (`seeds_exist` — rejected, not ignored).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub direction: Direction,
    pub seeds: BTreeSet<String>,
    /// The set M of schema morphisms to follow, by name — rows sharing a
    /// name (supersedes on constraints and intents) are all followed.
    pub morphisms: BTreeSet<String>,
}

/// Why a query was rejected: a seed that is not an id of the instance,
/// named — never silently dropped from the traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    UnknownSeed(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSeed(seed) => {
                write!(f, "unknown seed `{seed}` — not an id of this instance")
            }
        }
    }
}

/// Check a query against an instance without running it: every seed must
/// be an id of the instance (`seeds_exist`). All violations are reported,
/// sorted.
pub fn check(query: &Query, instance: &Instance) -> Result<(), Vec<QueryError>> {
    let unknown: Vec<QueryError> = query
        .seeds
        .iter()
        .filter(|seed| instance.index_of(seed).is_none())
        .map(|seed| QueryError::UnknownSeed(seed.clone()))
        .collect();
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(unknown)
    }
}

/// Run the closure: the seeds plus everything reachable from them by
/// following the morphism set M (in the query's direction), each node
/// visited at most once (`closure_terminates`), dangling values never
/// followed (`dangling_not_followed` — a `None` morphism value is no
/// edge), results sorted by file-qualified id (`results_ordered`).
pub fn run(query: &Query, instance: &Instance) -> Result<BTreeSet<String>, Vec<QueryError>> {
    check(query, instance)?;
    // The adjacency over the named morphisms, in the query's direction —
    // built from the instance's stored values; `None` (dangling) values
    // contribute no edge.
    let mut adjacency: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for name in &query.morphisms {
        for (from, to) in instance.morphism_values(name) {
            if let Some(to) = to {
                let (from, to) = match query.direction {
                    Direction::Forward => (from, to),
                    Direction::Backward => (to, from),
                };
                adjacency.entry(from).or_default().push(to);
            }
        }
    }
    // Worklist closure over the interned indices — each node enters the
    // queue at most once (`closure_terminates`, including cyclic M).
    let mut seen: BTreeSet<usize> = query
        .seeds
        .iter()
        .filter_map(|seed| instance.index_of(seed))
        .collect();
    let mut queue: Vec<usize> = seen.iter().copied().collect();
    while let Some(node) = queue.pop() {
        for next in adjacency.get(&node).into_iter().flatten().copied() {
            if seen.insert(next) {
                queue.push(next);
            }
        }
    }
    Ok(seen
        .iter()
        .map(|&i| instance.id_of(i).to_string())
        .collect())
}

/// Forward closure — what the seeds reach.
pub fn forward_closure(
    instance: &Instance,
    seeds: &[&str],
    morphisms: &[&str],
) -> Result<BTreeSet<String>, Vec<QueryError>> {
    run(
        &Query {
            direction: Direction::Forward,
            seeds: seeds.iter().map(|s| (*s).to_string()).collect(),
            morphisms: morphisms.iter().map(|m| (*m).to_string()).collect(),
        },
        instance,
    )
}

/// Backward closure — what reaches the seeds (the dependents half of a
/// blast radius).
pub fn backward_closure(
    instance: &Instance,
    seeds: &[&str],
    morphisms: &[&str],
) -> Result<BTreeSet<String>, Vec<QueryError>> {
    run(
        &Query {
            direction: Direction::Backward,
            seeds: seeds.iter().map(|s| (*s).to_string()).collect(),
            morphisms: morphisms.iter().map(|m| (*m).to_string()).collect(),
        },
        instance,
    )
}

/// Closure over an already-validated seed set — the internal step the
/// mixed blast radius iterates (`seeds_exist` is pre-checked, so the
/// seeds are all instance ids).
fn closure(
    direction: Direction,
    seeds: &BTreeSet<String>,
    morphisms: &[&str],
    instance: &Instance,
) -> Result<BTreeSet<String>, Vec<QueryError>> {
    run(
        &Query {
            direction,
            seeds: seeds.clone(),
            morphisms: morphisms.iter().map(|m| (*m).to_string()).collect(),
        },
        instance,
    )
}

/// The blast radius of touching `touched` (`blast_radius_defined`, tracing
/// to [[spec.blast_radius_defined]]): the mixed transitive closure — from
/// every reached node, BOTH directions are followed, so a dependent's own
/// forward reach joins the radius. Expressed through the two primitives
/// alone (`single_traversal_primitive`): the fixpoint of
/// `X ↦ backward_closure(forward_closure(X))`, which is closed under both
/// directions and contains the seeds — hence exactly the mixed closure.
/// The parity property against `merge`'s pre-migration walk pinned this
/// definition: the naive reading (backward of touched union forward of
/// touched) misses the forward reach of backward-reached dependents.
pub fn blast_radius(
    instance: &Instance,
    touched: &[&str],
    morphisms: &[&str],
) -> Result<BTreeSet<String>, Vec<QueryError>> {
    let seeds: BTreeSet<String> = touched.iter().map(|t| (*t).to_string()).collect();
    let mut current = closure(Direction::Backward, &seeds, morphisms, instance)?;
    loop {
        let reached = closure(Direction::Forward, &current, morphisms, instance)?;
        let closed = closure(Direction::Backward, &reached, morphisms, instance)?;
        if closed == current {
            return Ok(current);
        }
        current = closed;
    }
}

/// The nodes on at least one cycle through the morphism set M (`
/// closure_terminates`'s counterpart for cycle membership, and the
/// closure-based reading of specs/specodelic.md's supersedes_acyclic): a
/// node is on a cycle through M iff some morphism value `s → t` exists
/// with `s` reachable forward from `t` — including the trivial `s == t`
/// self-edge. Unresolved references are never followed
/// (`dangling_not_followed`).
pub fn cyclic_nodes(
    instance: &Instance,
    morphisms: &[&str],
) -> Result<BTreeSet<String>, Vec<QueryError>> {
    let mut cyclic = BTreeSet::new();
    for name in morphisms {
        for (from, to) in instance.morphism_values(name) {
            let Some(to) = to else { continue };
            let seed = BTreeSet::from([instance.id_of(to).to_string()]);
            let reached = run(
                &Query {
                    direction: Direction::Forward,
                    seeds: seed,
                    morphisms: morphisms.iter().map(|m| (*m).to_string()).collect(),
                },
                instance,
            )?;
            if reached.contains(instance.id_of(from)) {
                cyclic.insert(instance.id_of(from).to_string());
            }
        }
    }
    Ok(cyclic)
}

/// `fan_in` — the number of pairs (source, morphism) whose defined value
/// is the node (`fan_in_is_preimage_size`), keyed by file-qualified id.
pub fn fan_in(instance: &Instance) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for edge in instance.edges() {
        *counts.entry(edge.to.clone()).or_default() += 1;
    }
    counts
}

/// `fan_out` — the number of defined morphism values whose source is the
/// node, keyed by file-qualified id.
pub fn fan_out(instance: &Instance) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for edge in instance.edges() {
        *counts.entry(edge.from.clone()).or_default() += 1;
    }
    counts
}

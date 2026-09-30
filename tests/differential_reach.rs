//! Purpose: differential reachability harness — beads specodelic-vv8 S4.
//!
//! Responsibilities:
//! - Generate random small valid models (1..=8 states, every state in ≥1
//!   transition, guard cells resolving to a real constraint — the shape
//!   `compile_spec` accepts and lint passes) as pure functions of a seed.
//! - NATIVE LEG (ungated, cheap): run spk's native model-check backend
//!   (`compile_spec` → `model_check::run`) over each generated model and
//!   compare its exploration against an INDEPENDENT BFS closure computed
//!   from the generator's ground-truth edge list — never from spk's parse.
//!   A completed run must report `exploration_only` (the native backend
//!   executes no predicates — never `no_counterexample`), carry
//!   `invariants_checked: []`, and `states_explored` must equal the BFS
//!   closure size exactly — including models with unreachable states,
//!   which pin that exploration is a reachability closure, not a state
//!   census.
//! - TLC LEG (`tlc_differential`, `#[ignore]`d + env-gated): the same
//!   differential against real TLC (`run_tlc`) — only when
//!   `SPECODELIC_TLC_JAR` points at a tla2tools.jar (java binary via
//!   `SPK_TLC_JAVA`, default `java`). Absent jar = skip with a note; a
//!   present jar that misbehaves = failure. Coordinated with ug3's TLC
//!   backend work — when ug3 lands its fake-JVM seam, the leg can extend
//!   to it.
//!
//! Rationale: the 2026-09-28 adversarial review's differential harness
//! (60/60 vs BFS, 25/25 vs real TLC) existed only outside the repo; this
//! files the BFS leg in ungated and keeps the TLC leg opt-in (a JVM is an
//! environment property, not a test property). Anti-goal: the harness never
//! adapts the BFS to spk's behavior — the generator's edge list IS the
//! ground truth, and any disagreement is a bug in spk or in the harness,
//! to be diagnosed, not absorbed.

use specodelic::compile;
use specodelic::lint;
use specodelic::model_check::{self, Outcome};
use specodelic::spec::parse_str;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Base seed — same discipline as tests/fuzz.rs: any disagreeing model
/// reproduces from the seed alone.
const BASE_SEED: u64 = 0x0DEF_EE10_C1A0;

// splitmix64 — identical construction to tests/fuzz.rs (no shared test
// helper module; duplication over cross-test coupling).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed ^ BASE_SEED)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// One generated model: the corpus text plus the ground-truth transition
/// edge list the differential BFS runs over.
struct GeneratedModel {
    text: String,
    /// Ground-truth edges (from-state name, to-state name).
    edges: Vec<(String, String)>,
    states: Vec<String>,
}

/// A generated model: `n` states each with 1..=2 outgoing transitions
/// (so `every_transition_valid` passes) plus a few extra random edges —
/// cycles and unreachable states arise naturally. Guards resolve to c1;
/// the constraint/property furniture keeps the corpus lint-clean.
fn random_model(seed: u64) -> GeneratedModel {
    let mut rng = Rng::new(seed);
    let n = 1 + rng.below(8);
    let id = "f0".to_string();
    let states: Vec<String> = (0..n).map(|i| format!("st{i}")).collect();

    let mut edges: Vec<(String, String)> = Vec::new();
    // Every state gets ≥1 outgoing edge → appears in a transition.
    for (i, s) in states.iter().enumerate() {
        let targets: Vec<usize> = (0..n).filter(|j| *j != i).collect();
        let to = if targets.is_empty() {
            i // single-state model: self-loop keeps it transition-valid
        } else {
            targets[rng.below(targets.len())]
        };
        edges.push((s.clone(), states[to].clone()));
        if !targets.is_empty() && rng.below(2) == 0 {
            let to2 = targets[rng.below(targets.len())];
            edges.push((s.clone(), states[to2].clone()));
        }
    }

    let state_bullets: String = states
        .iter()
        .map(|s| format!("- {s}\n"))
        .collect::<Vec<_>>()
        .join("");
    let transition_rows: String = edges
        .iter()
        .enumerate()
        .map(|(i, (from, to))| format!("| t{i} | {from} | {to} | [[{id}.c1]] |\n"))
        .collect::<Vec<_>>()
        .join("");

    let text = format!(
        "---\n\
         id: {id}\n\
         kind: intent\n\
         statement: \"THE {id} SHALL be a differential fixture\"\n\
         ---\n\
         \n## Constraints\n\
         \n| id | kind | expr | traces_to |\n\
         |----|------|------|-----------|\n\
         | c1 | invariant | `holds` | [[{id}]] |\n\
         \n## Model\n\
         \n### States\n\
         \n{state_bullets}\
         \n### Transitions\n\
         \n| id | from | to | guard |\n\
         |----|------|----|-------|\n\
         {transition_rows}\
         \n## Properties\n\
         \n| id | kind | derives_from | generator | predicate |\n\
         |----|------|--------------|-----------|------------|\n\
         | p1 | unit | [[{id}.c1]] | `g()` | `x` |\n"
    );

    GeneratedModel {
        text,
        edges,
        states,
    }
}

/// Independent BFS closure over the generator's ground-truth edges from the
/// first listed state (the pc-automaton's init). Returns the reachable set.
fn bfs_closure(model: &GeneratedModel) -> BTreeSet<String> {
    let init = model
        .states
        .first()
        .expect("generator always emits ≥1 state")
        .clone();
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (from, to) in &model.edges {
        adj.entry(from.as_str()).or_default().push(to.as_str());
    }
    let mut seen: BTreeSet<String> = BTreeSet::from([init.clone()]);
    let mut queue = vec![init];
    while let Some(node) = queue.pop() {
        for next in adj.get(node.as_str()).into_iter().flatten() {
            if seen.insert((*next).to_string()) {
                queue.push((*next).to_string());
            }
        }
    }
    seen
}

/// Generous bound: a ≤8-state automaton always ends within it, so any
/// `timed_out` would be a spk bug, not a bound interplay to tolerate.
fn generous_bound() -> model_check::Bound {
    model_check::Bound {
        max_depth: 10_000,
        max_states: Some(1_000_000),
        timeout_secs: Some(10),
    }
}

/// Run spk's native backend over the generated model; panic on anything
/// but a completed honest exploration.
fn native_report(model: &GeneratedModel) -> model_check::RunReport {
    let mut spec = parse_str(&model.text).expect("generated model parses");
    spec.path = Some(PathBuf::from("f0.md"));
    let report = lint::lint_all(&[spec.clone()], &[]);
    assert!(
        report.issues.is_empty() && report.warnings.is_empty(),
        "generated model must lint clean: {:?}",
        report.issues
    );
    let compiled = compile::compile_spec(&spec).expect("lint-clean model compiles");
    let tla = compiled.tla.clone().into_bytes();
    model_check::run(&compiled.model_ir, &tla, &generous_bound())
        .expect("native run succeeds on a generated model")
}

// ===========================================================================
// Pinned discriminator — unreachable states must not be explored.
// ===========================================================================

/// A fixed 3-state model: st0 ⇄ st1 reachable, st2 self-loops but is
/// unreachable from init. Ground-truth closure = {st0, st1}; if spk
/// explores st2, its exploration is a state census, not reachability.
fn pinned_model() -> GeneratedModel {
    GeneratedModel {
        text: "---\n\
               id: f0\n\
               kind: intent\n\
               statement: \"THE f0 SHALL be a differential fixture\"\n\
               ---\n\
               \n## Constraints\n\
               \n| id | kind | expr | traces_to |\n\
               |----|------|------|-----------|\n\
               | c1 | invariant | `holds` | [[f0]] |\n\
               \n## Model\n\
               \n### States\n\
               \n- st0\n\
               - st1\n\
               - st2\n\
               \n### Transitions\n\
               \n| id | from | to | guard |\n\
               |----|------|----|-------|\n\
               | t0 | st0 | st1 | [[f0.c1]] |\n\
               | t1 | st1 | st0 | [[f0.c1]] |\n\
               | t2 | st2 | st2 | [[f0.c1]] |\n\
               \n## Properties\n\
               \n| id | kind | derives_from | generator | predicate |\n\
               |----|------|--------------|-----------|------------|\n\
               | p1 | unit | [[f0.c1]] | `g()` | `x` |\n"
            .to_string(),
        edges: vec![
            ("st0".into(), "st1".into()),
            ("st1".into(), "st0".into()),
            ("st2".into(), "st2".into()),
        ],
        states: vec!["st0".into(), "st1".into(), "st2".into()],
    }
}

// ===========================================================================
// The differential — native leg ungated, TLC leg ignored+env-gated.
// ===========================================================================

#[test]
fn differential_on_pinned_model() {
    let model = pinned_model();
    let report = native_report(&model);
    assert_eq!(
        report.outcome,
        Outcome::ExplorationOnly,
        "a completed native run is exploration_only — never no_counterexample"
    );
    assert!(
        report.invariants_checked.is_empty(),
        "the native backend executes no predicates — honesty invariant"
    );
    assert_eq!(
        report.states_explored as usize,
        bfs_closure(&model).len(),
        "st2 self-loops but is unreachable from init — exploration is a \
         reachability closure, not a state census"
    );
}

#[test]
fn differential_on_random_models() {
    for seed in 0..60u64 {
        let model = random_model(seed);
        let report = native_report(&model);
        assert_eq!(
            report.outcome,
            Outcome::ExplorationOnly,
            "seed {seed}: a ≤8-state automaton always ends within the bound"
        );
        assert_eq!(
            report.states_explored as usize,
            bfs_closure(&model).len(),
            "seed {seed}: native exploration must equal the independent BFS closure"
        );
    }
}

#[test]
fn random_models_vary_with_seed() {
    let a = random_model(0);
    let b = random_model(1);
    let c = random_model(42);
    assert!(
        a.text != b.text || b.text != c.text,
        "the generator is a stub or degenerate — different seeds must \
         produce different models"
    );
}

// ===========================================================================
// TLC leg — #[ignore]d + env-gated (real JVM + tla2tools.jar required).
// ===========================================================================

#[test]
#[ignore = "needs a real JVM + tla2tools.jar — set SPECODELIC_TLC_JAR and run with -- --ignored"]
fn tlc_differential() {
    let Some(jar) = std::env::var("SPECODELIC_TLC_JAR").ok().map(PathBuf::from) else {
        eprintln!(
            "skipped: set SPECODELIC_TLC_JAR to a tla2tools.jar to run the \
             TLC differential leg"
        );
        return;
    };
    let paths = model_check::TlcPaths {
        java: std::env::var("SPK_TLC_JAVA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("java")),
        jar,
    };

    for seed in 0..25u64 {
        let model = random_model(seed);
        let mut spec = parse_str(&model.text).expect("generated model parses");
        spec.path = Some(PathBuf::from("f0.md"));
        let compiled = compile::compile_spec(&spec).expect("lint-clean model compiles");
        let tla = compiled.tla.clone().into_bytes();
        let bound = model_check::Bound {
            max_depth: 10_000,
            max_states: None,
            timeout_secs: Some(120),
        };
        let report = model_check::run_tlc(
            &compiled.model_ir,
            Path::new("f0.tla"),
            &tla,
            &bound,
            &paths,
        )
        .expect("TLC run succeeds on a generated model");
        assert_eq!(
            report.outcome,
            Outcome::ExplorationOnly,
            "seed {seed}: a ≤8-state automaton must end within the bound"
        );
        assert_eq!(
            report.states_explored as usize,
            bfs_closure(&model).len(),
            "seed {seed}: TLC's distinct-state count must equal the BFS closure"
        );
    }
}

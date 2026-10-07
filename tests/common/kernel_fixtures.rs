//! Shared fixture corpus for kernel backend agreement — phase 4 of
//! add-min-expr-kernel (specodelic-36n, tasks.md §4.3; design D2).
//!
//! This module is the single home of the agreement fixture corpus: the
//! same files, the same per-cell expected statuses, for every backend.
//! The rust side consumes it from `tests/kernel_agreement.rs`; the py
//! side (specodelic-l8l) imports this exact module for its parity
//! tests, so backend divergence cannot hide behind divergent fixtures.
//!
//! Promotion path (deferred of record — proposal Deferred): once both
//! backends exist, the agreement property promotes to a
//! `law_requires_cases`-shaped Property row in the l8l/aby change —
//! the property needs both backends before it can be law-shaped.

use specodelic::compile::{ModelIr, ThreeValued, extract_model_ir};
use specodelic::kernel::KernelEnv;
use specodelic::spec::{Spec, parse_str};

// --- the backend registry ---

/// A kernel backend: evaluates an extracted claim over the fixture's
/// corpus and reports either a three-valued status or labeled
/// `Unsupported`. Incapable backends never fabricate a verdict.
pub enum Backend {
    Rust,
    Py,
}

pub enum Outcome {
    Status(ThreeValued),
    Unsupported(String),
}

impl Backend {
    pub fn name(&self) -> &'static str {
        match self {
            Backend::Rust => "rust",
            Backend::Py => "py",
        }
    }
}

/// The seats the agreement property covers. `py` is the l8l activation
/// seat: present so the cross-backend assertion is exercised on every
/// run, reporting `Unsupported` until l8l's emitter lands. l8l's change
/// edits `evaluate` (and this registry if needed) — it must never
/// delete the seat or comment out the assertions that read it.
pub const BACKENDS: &[Backend] = &[Backend::Rust, Backend::Py];

pub fn evaluate(backend: &Backend, specs: &[Spec], expr: &str) -> Outcome {
    match backend {
        Backend::Rust => {
            let env = KernelEnv::from_specs(specs);
            match specodelic::kernel::parse_kernel_str(expr) {
                Ok(Some(parsed)) => Outcome::Status(env.evaluate(&parsed)),
                Ok(None) => Outcome::Unsupported(format!(
                    "rust backend cannot evaluate non-kernel cell: {expr}"
                )),
                Err(e) => Outcome::Unsupported(format!("rust backend parse failure: {e:?}")),
            }
        }
        Backend::Py => Outcome::Unsupported(
            "py emitter not landed — activation seat for specodelic-l8l".to_string(),
        ),
    }
}

// --- the shared fixture corpus ---

/// One fixture: corpus files plus the expected rust status per kernel
/// claim (constraint row id). The py column is implied until l8l: the
/// seat must report `Unsupported` for every cell; l8l's change adds the
/// py expectations here when it activates.
pub struct Fixture {
    pub name: &'static str,
    pub files: &'static [&'static str],
    pub expected: &'static [(&'static str, ThreeValued)],
}

/// Three intent files chained na → nb → nc through supersedes — the
/// clean shape: every v0 atomic verifies over it. Auxiliary claim rows
/// carry empty traces_to so the injectivity fixture stays clean.
const CHAIN: &[&str] = &[
    "---\nid: demo.agr.chain.a\nkind: intent\nstatement: \"THE a SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| na | invariant | `**kernel:** acyclic(supersedes)` | [[demo.agr.chain.a]] | [[demo.agr.chain.b.nb]] |\n| na2 | invariant | `**kernel:** |Intent| == 3` |  |  |\n",
    "---\nid: demo.agr.chain.b\nkind: intent\nstatement: \"THE b SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| nb | invariant | `**kernel:** resolves(traces_to)` | [[demo.agr.chain.b]] | [[demo.agr.chain.c.nc]] |\n| nb2 | invariant | `**kernel:** unique(traces_to)` |  |  |\n",
    "---\nid: demo.agr.chain.c\nkind: intent\nstatement: \"THE c SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| nc | invariant | `**kernel:** ∀ c ∈ Constraint: |Constraint| >= 1` |  |  |\n| nc2 | invariant | `**kernel:** ∃ c ∈ Constraint: c.supersedes == demo.agr.chain.b.nb` |  |  |\n",
];

/// Two intent files with a supersedes cycle between their constraints —
/// `acyclic` counterexample.
const CYCLIC: &[&str] = &[
    "---\nid: demo.agr.cyc.a\nkind: intent\nstatement: \"THE a SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| ca | invariant | `**kernel:** acyclic(supersedes)` | [[demo.agr.cyc.a]] | [[demo.agr.cyc.b.cb]] |\n",
    "---\nid: demo.agr.cyc.b\nkind: intent\nstatement: \"THE b SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | supersedes |\n|----|------|------|-----------|------------|\n| cb | invariant | `**kernel:** resolves(traces_to)` | [[demo.agr.cyc.b]] | [[demo.agr.cyc.a.ca]] |\n",
];

/// One intent file with a dangling traces_to — `resolves` counterexample.
const DANGLING: &[&str] = &[
    "---\nid: demo.agr.dng\nkind: intent\nstatement: \"THE dng SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| da | invariant | `**kernel:** resolves(traces_to)` | [[demo.ghost.missing]] |\n",
];

/// One intent file whose two constraints resolve to the same target —
/// `unique` counterexample (injectivity refuted).
const DUPLICATES: &[&str] = &[
    "---\nid: demo.agr.dup\nkind: intent\nstatement: \"THE dup SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| d1 | invariant | `**kernel:** unique(traces_to)` | [[demo.agr.dup]] |\n| d2 | invariant | `**kernel:** resolves(traces_to)` | [[demo.agr.dup]] |\n",
];

/// One intent file whose claim seeds reachability from an id that is
/// not an instance row — the honest `unknown` shape.
const GHOST_SEED: &[&str] = &[
    "---\nid: demo.agr.ghost\nkind: intent\nstatement: \"THE ghost SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| g1 | invariant | `**kernel:** reachable(demo.ghost.nowhere, demo.agr.ghost.g1, supersedes)` | [[demo.agr.ghost]] |\n",
];

/// One intent file with a verified conjunct and an undischargable one —
/// Kleene absorb: the composite is unknown, never pass.
const ABSORBING: &[&str] = &[
    "---\nid: demo.agr.abs\nkind: intent\nstatement: \"THE abs SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a1 | invariant | `**kernel:** acyclic(supersedes) ∧ reachable(demo.ghost.nowhere, demo.agr.abs.a1, supersedes)` | [[demo.agr.abs]] |\n",
];

pub const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "clean_chain",
        files: CHAIN,
        expected: &[
            ("na", ThreeValued::Verified),
            ("na2", ThreeValued::Verified),
            ("nb", ThreeValued::Verified),
            ("nb2", ThreeValued::Verified),
            ("nc", ThreeValued::Verified),
            ("nc2", ThreeValued::Verified),
        ],
    },
    Fixture {
        name: "cyclic_pair",
        files: CYCLIC,
        expected: &[
            // ca refutes acyclic; cb's traces_to is clean so resolves verifies.
            ("ca", ThreeValued::Counterexample),
            ("cb", ThreeValued::Verified),
        ],
    },
    Fixture {
        name: "dangling_ref",
        files: DANGLING,
        expected: &[("da", ThreeValued::Counterexample)],
    },
    Fixture {
        name: "duplicate_targets",
        files: DUPLICATES,
        expected: &[
            ("d1", ThreeValued::Counterexample),
            ("d2", ThreeValued::Verified),
        ],
    },
    Fixture {
        name: "ghost_seed",
        files: GHOST_SEED,
        expected: &[("g1", ThreeValued::Unknown)],
    },
    Fixture {
        name: "absorbing_composite",
        files: ABSORBING,
        expected: &[("a1", ThreeValued::Unknown)],
    },
];

// --- the harness ---

/// Parse each fixture file through the real parser and extract the
/// kernel claims (the compile step the CLI run reads). Returns the
/// corpus specs and (claim id, kernel text) pairs sorted by id.
pub fn extract(files: &[&str]) -> (Vec<Spec>, Vec<(String, String)>) {
    let specs: Vec<Spec> = files
        .iter()
        .map(|f| parse_str(f).expect("fixture file parses"))
        .collect();
    let mut claims: Vec<(String, String)> = specs
        .iter()
        .map(|spec| {
            let ir: ModelIr = extract_model_ir(spec);
            ir.guard_kernel
        })
        .flat_map(|map| map.into_iter())
        .collect();
    claims.sort_by(|a, b| a.0.cmp(&b.0));
    (specs, claims)
}

//! The tidy-first refactor advisor (`specs/refactor.md`).
//!
//! Purpose: mechanize Kent Beck's "separate the structural change from the
//! behavioral one" as a graph query — surface a non-gating advisory to split
//! a node BEFORE the behavioral edit lands. Responsibilities: attribute every
//! incoming edge the derived graph already extracted to its owning file,
//! count unrelated-namespace dependents (nearest common ancestor in the id
//! namespace is the root — decision of record 2026-09-30, see the file's
//! Notes), and flag narrow-diff changesets that touch a strict subset of a
//! node's owned rows while depending on none of its others. Rationale: every
//! count is a `graph` query, never an independent markdown walk
//! (`fan_in_read_from_graph`) — the advisor is a pure consumer of the
//! derived `GraphReport`, which is what makes its counts trustworthy.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::graph::GraphReport;
use crate::spec::Spec;

/// The fan-in count that counts as "high" when the invocation doesn't
/// configure one (`threshold_is_per_repo_setting`: the SPEC never fixes
/// this number — the tool's default is a per-release choice, and the CLI's
/// `--high-fan-in` makes it a per-repo/per-invocation setting).
pub const DEFAULT_HIGH_FAN_IN: usize = 3;

/// One advisory finding (`advisory_finding_emitted`): exactly the four
/// fields the spec's effect constraint names — nothing more.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Finding {
    /// The flagged node's file id, e.g. `alpha.hub`.
    pub node_id: String,
    /// Incoming cross-file edges to the node's intent and all its owned
    /// rows — `graph`'s fan-in restricted to incoming edges.
    pub dependent_count: usize,
    /// How many of those dependents share no namespace segment with the
    /// node below the root (`unrelated_fan_in_defined`).
    pub unrelated_namespace_count: usize,
    /// Always true on a finding — the presence of a finding IS the split
    /// suggestion; the field exists so consumers can pattern-match the
    /// spec'd shape without special-casing.
    pub suggested_split: bool,
}

/// The advisor report: advisory only, never gating (exit 0 either way).
#[derive(Debug, Clone, Serialize, Default)]
pub struct RefactorReport {
    pub files: usize,
    pub findings: Vec<Finding>,
}

/// The file id that owns a graph node id: rows are stored file-qualified
/// (`file.row`, the file's dots are the leading segments), intent nodes are
/// their own file. `n.c1` → `n`; `alpha.hub` → `alpha.hub`.
fn file_of(node: &str) -> &str {
    match node.rsplit_once('.') {
        Some((file, _row)) => file,
        // A dotless id is a file's intent node (or a bare metasyntactic
        // spelling — the graph never records unresolvable ids as edges).
        None => node,
    }
}

/// `unrelated_fan_in_defined` as decided of record (2026-09-30): unrelated
/// ⇔ the nearest common ancestor in the id namespace is the root ⇔ the two
/// ids' first (top-level) segments differ. A flat-namespace repo makes
/// almost every cross-file dependent unrelated — the documented, contained
/// failure mode (the finding is non-gating and the threshold is configurable).
fn unrelated(a: &str, b: &str) -> bool {
    match (a.split_once('.'), b.split_once('.')) {
        (Some(x), Some(y)) => x.0 != y.0,
        // One side flat, one nested: no shared top-level segment below root.
        _ => a != b,
    }
}

/// Run the advisor over the already-derived graph.
///
/// `threshold` is `threshold_is_per_repo_setting`: the fan-in count that
/// counts as "high", configured per invocation (the CLI exposes
/// `--high-fan-in`), never fixed here. `changeset` is the optional set of
/// row ids a proposed edit would touch (`narrow_diff_heuristic`).
pub fn analyze(
    report: &GraphReport,
    specs: &[Spec],
    threshold: usize,
    changeset: &BTreeSet<String>,
) -> RefactorReport {
    // Owned rows per candidate file, file-qualified — Constraints and
    // Properties are the rows `narrow_diff_heuristic` speaks of.
    let mut owned: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for spec in specs {
        let file = spec.id();
        let entry = owned.entry(file).or_default();
        for id in spec.constraints.iter().map(|r| &r.id) {
            entry.insert(format!("{file}.{id}"));
        }
        for id in spec.properties.iter().map(|r| &r.id) {
            entry.insert(format!("{file}.{id}"));
        }
    }

    // Fan-in aggregation: incoming cross-file edges to the file's intent
    // node or any of its owned rows. Every edge comes from `report` —
    // derived once by graph::build, never re-walked here.
    let mut dependent_count: BTreeMap<&str, usize> = BTreeMap::new();
    let mut unrelated_count: BTreeMap<&str, usize> = BTreeMap::new();
    for edge in &report.edges {
        let target_file = owned
            .keys()
            .copied()
            .find(|f| edge.to == *f || file_of(&edge.to) == *f);
        let Some(target_file) = target_file else {
            continue;
        };
        let from_file = file_of(&edge.from);
        if from_file == target_file {
            // A file's own rows referencing itself is cohesion, not fan-in.
            continue;
        }
        *dependent_count.entry(target_file).or_default() += 1;
        if unrelated(from_file, target_file) {
            *unrelated_count.entry(target_file).or_default() += 1;
        }
    }

    let mut out = RefactorReport {
        files: specs.len(),
        findings: vec![],
    };
    let mut flagged: BTreeSet<&str> = BTreeSet::new();
    // `flag.guard`: unrelated_fan_in_defined (high unrelated fan-in, per the
    // configured threshold) ∨ narrow_diff_heuristic.
    for (file, &unrelated_n) in &unrelated_count {
        if unrelated_n >= threshold {
            flagged.insert(file);
        }
    }
    if !changeset.is_empty() {
        for (file, rows) in &owned {
            let touched: BTreeSet<&String> = changeset.iter().collect();
            let touched_owned: Vec<&String> = touched
                .iter()
                .filter(|t| rows.contains(t.as_str()))
                .copied()
                .collect();
            // A strict subset of N's owned rows, depending on none of N's
            // other owned rows → flag regardless of fan-in. An edge from a
            // changeset row to an untouched owned row means the changeset
            // needs the node whole — coherent, no flag.
            if touched_owned.len() < rows.len()
                && !report.edges.iter().any(|e| {
                    touched.contains(&e.from)
                        && rows.contains(&e.to)
                        && !touched_owned.contains(&&e.to)
                })
            {
                flagged.insert(file);
            }
        }
    }
    for file in flagged {
        out.findings.push(Finding {
            node_id: file.to_string(),
            dependent_count: dependent_count.get(file).copied().unwrap_or(0),
            unrelated_namespace_count: unrelated_count.get(file).copied().unwrap_or(0),
            suggested_split: true,
        });
    }
    // BTreeSet iteration is already deterministic; findings are per-file
    // so the report is byte-stable across reruns (deterministic_rerun).
    out
}

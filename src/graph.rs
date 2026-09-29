//! The typed reference graph — derived, never authored (`specs/graph.md`).
//!
//! Purpose: turn every typed reference field in the corpus into one
//! adjacency structure over the format's kinds. Responsibilities: extract
//! edges from structured cells only, resolve them corpus-wide, and report
//! dangling references plus per-node fan-in/fan-out. Rationale: the graph
//! is the substrate rename/merge/refactor all build on — deriving it once,
//! deterministically, is what makes those tools safe.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::spec::Spec;

/// One directed, typed edge.
#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    /// Source id (the row or intent the link is anchored to).
    pub from: String,
    /// Resolved target id.
    pub to: String,
    /// Typed reference field, e.g. `constraints.traces_to`,
    /// `properties.derives_from`, `transitions.guard`.
    pub kind: String,
}

/// The derived graph report.
#[derive(Debug, Clone, Serialize, Default)]
pub struct GraphReport {
    pub files: usize,
    pub nodes: usize,
    pub edges: Vec<Edge>,
    /// Links that resolve to nothing in the corpus.
    pub dangling: Vec<String>,
    /// Per-node fan-in counts (how many edges point at the node).
    pub fan_in: BTreeMap<String, usize>,
    /// Per-node fan-out counts.
    pub fan_out: BTreeMap<String, usize>,
}

/// Build the graph for a corpus of parsed specs.
pub fn build(specs: &[Spec]) -> GraphReport {
    // Resolution index: file id -> defined ids (intent + rows).
    // Same-id files (openspec `spec.md` → `id: spec`) aggregate their
    // row sets so a file's OWN rows are never erased by a same-id file
    // (#37) — but each `id: spec` file resolves against its own row set
    // only (self-contained deltas, mirroring the linter's file-scoped
    // total_refs semantics, #42); other file ids resolve corpus-wide.
    let mut file_rows: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for spec in specs {
        file_rows
            .entry(spec.intent.id.clone())
            .or_default()
            .extend(spec.defined_ids());
    }
    let mut report = GraphReport {
        files: specs.len(),
        ..Default::default()
    };

    let mut nodes: BTreeMap<String, ()> = BTreeMap::new();
    for spec in specs {
        nodes.insert(spec.intent.id.clone(), ());
        for id in spec.defined_ids() {
            nodes.insert(id, ());
        }
    }

    for spec in specs {
        let file_id = &spec.intent.id;
        // `id: spec` files resolve file-scoped (self-contained deltas);
        // others against the corpus-wide map.
        let scoped_rows: BTreeMap<String, Vec<String>> = if file_id == "spec" {
            BTreeMap::from([(
                file_id.clone(),
                spec.defined_ids().into_iter().collect::<Vec<_>>(),
            )])
        } else {
            file_rows.clone()
        };
        for link in &spec.links {
            let resolved = resolve(&scoped_rows, file_id, &link.target);
            // Metasyntactic example links (`[[old_id]]`, `[[...]]`) are
            // format documentation inside expr cells — not graph edges.
            let metasyn = !link.target.contains('.') && !scoped_rows.contains_key(&link.target)
                || link.target == "..."
                || link.target == "…";
            if metasyn {
                continue;
            }
            match resolved {
                Some(target) => {
                    let kind = if link.column.is_empty() {
                        link.field.clone()
                    } else {
                        format!("{}.{}", link.field, link.column)
                    };
                    let from = if link.source.as_str() == file_id.as_str() {
                        // frontmatter links are anchored on the intent row
                        format!("{file_id} (intent)")
                    } else {
                        format!("{file_id}.{}", link.source)
                    };
                    let edge = Edge {
                        from,
                        to: target,
                        kind,
                    };
                    *report.fan_out.entry(edge.from.clone()).or_default() += 1;
                    *report.fan_in.entry(edge.to.clone()).or_default() += 1;
                    report.edges.push(edge);
                }
                None => report
                    .dangling
                    .push(format!("{} → [[{}]]", file_id, link.target)),
            }
        }
    }

    report.nodes = nodes.len();
    report
}

/// Resolve a target relative to a source file. Returns the canonical
/// resolved id when found.
fn resolve(
    file_rows: &BTreeMap<String, Vec<String>>,
    source_file: &str,
    target: &str,
) -> Option<String> {
    if target == "model.state" || target == "model.transition" {
        return Some(format!("{source_file}.{target}"));
    }
    if file_rows.contains_key(target) {
        return Some(target.to_string());
    }
    // Try last-dot split (handles dotted file ids like linter.frontmatter).
    if let Some((file_id, rest)) = target.rsplit_once('.')
        && let Some(rows) = file_rows.get(file_id)
    {
        if rows.iter().any(|r| r == rest) || rest == "model.state" || rest == "model.transition" {
            return Some(target.to_string());
        }
        if let Some((row_id, member)) = rest.split_once('.')
            && rows.iter().any(|r| r == row_id)
        {
            return Some(format!("{file_id}.{row_id} ({member})"));
        }
    }
    // Retry with first-dot split for dotted-file-id targets like
    // `specodelic.model.state` (file `specodelic`, anchor `model.state`).
    if let Some((file_id, rest)) = target.split_once('.')
        && let Some(rows) = file_rows.get(file_id)
        && (rows.iter().any(|r| r == rest) || rest == "model.state" || rest == "model.transition")
    {
        return Some(target.to_string());
    }
    None
}

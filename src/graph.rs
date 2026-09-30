//! The typed reference graph — derived, never authored (`specs/graph.md`).
//!
//! Purpose: turn every typed reference field in the corpus into one
//! adjacency structure over the format's kinds. Responsibilities: extract
//! edges from structured cells only (including a Transition's `from`/`to`
//! state edges), resolve them corpus-wide, enforce the Reference Typing
//! table (forbidden edges are reported as labeled violations, never
//! recorded — `edge_kind_matches_typing`), flag `supersedes` cycles, and
//! report dangling references plus per-node fan-in/fan-out. Rationale: the
//! graph is the substrate rename/merge/refactor all build on — deriving it
//! once, deterministically, is what makes those tools safe.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::spec::Spec;

/// One directed, typed edge.
#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    /// Source id (the row or intent the link is anchored to).
    pub from: String,
    /// Resolved target id.
    pub to: String,
    /// Typed reference field, e.g. `constraints.traces_to`,
    /// `properties.derives_from`, `transitions.guard`, `transitions.from`.
    pub kind: String,
}

/// A typing-forbidden edge (the Reference Typing table in
/// `specs/specodelic.md`). Reported as a labeled violation, never recorded
/// as an edge — `specs/graph.md`'s `edge_kind_matches_typing`.
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    /// Source node the forbidden edge would have started at.
    pub from: String,
    /// Resolved target the forbidden edge would have pointed at.
    pub to: String,
    /// The typed reference field, e.g. `constraints.traces_to`.
    pub edge_kind: String,
    /// Why the Reference Typing table forbids this edge.
    pub reason: String,
}

/// The derived graph report.
#[derive(Debug, Clone, Serialize, Default)]
pub struct GraphReport {
    pub files: usize,
    pub nodes: usize,
    pub edges: Vec<Edge>,
    /// Links that resolve to nothing in the corpus.
    pub dangling: Vec<String>,
    /// Typing-forbidden edges — labeled, never recorded as edges.
    pub violations: Vec<Violation>,
    /// Cycles in the `supersedes` edge set (`supersedes_dag`,
    /// `specs/linter-graph_shape.md`), rendered as `a → b → a` paths.
    pub supersedes_cycles: Vec<String>,
    /// Per-node fan-in counts (how many edges point at the node).
    pub fan_in: BTreeMap<String, usize>,
    /// Per-node fan-out counts.
    pub fan_out: BTreeMap<String, usize>,
    /// Files classified as external boundaries (specs/graph.md
    /// `external_boundary_derived`): each hosts ≥1 `extension_point`
    /// Constraint — pure derivation, never an authored tag.
    pub external_boundaries: Vec<String>,
}

/// What kind of node a resolved id addresses — the target side of the
/// Reference Typing checks.
#[derive(Debug, Clone, PartialEq)]
enum NodeKind {
    Intent,
    /// The row's own `kind` cell (`invariant`/`advisory`/`effect`), empty
    /// when absent.
    Constraint(String),
    /// The row's own `kind` cell (`unit`/`law`), empty when absent.
    Property(String),
    State,
    Transition,
}

impl NodeKind {
    /// Human phrase for violation reasons, e.g. `an Intent`.
    fn describe(&self) -> String {
        match self {
            NodeKind::Intent => "an Intent".into(),
            NodeKind::Constraint(k) if k.is_empty() => "a Constraint".into(),
            NodeKind::Constraint(k) => format!("a Constraint (kind `{k}`)"),
            NodeKind::Property(k) if k.is_empty() => "a Property".into(),
            NodeKind::Property(k) => format!("a Property (kind `{k}`)"),
            NodeKind::State => "a State".into(),
            NodeKind::Transition => "a Transition".into(),
        }
    }

    /// Do two node kinds belong to the same class (the `supersedes` same-kind
    /// rule: Constraint→Constraint, Property→Property)?
    fn same_class(&self, other: &NodeKind) -> bool {
        matches!(
            (self, other),
            (NodeKind::Constraint(_), NodeKind::Constraint(_))
                | (NodeKind::Property(_), NodeKind::Property(_))
                | (NodeKind::Intent, NodeKind::Intent)
                | (NodeKind::State, NodeKind::State)
                | (NodeKind::Transition, NodeKind::Transition)
        )
    }
}

/// Index every resolvable id in the corpus to its node kind: file ids to
/// their Intent, row ids (file-qualified `file.row`) to the row's layer.
fn kind_index(specs: &[Spec]) -> BTreeMap<String, NodeKind> {
    let mut idx = BTreeMap::new();
    for spec in specs {
        let file = &spec.intent.id;
        idx.entry(file.clone()).or_insert(NodeKind::Intent);
        for r in &spec.constraints {
            idx.entry(format!("{file}.{}", r.id))
                .or_insert(NodeKind::Constraint(r.kind.clone().unwrap_or_default()));
        }
        for r in &spec.properties {
            idx.entry(format!("{file}.{}", r.id))
                .or_insert(NodeKind::Property(r.kind.clone().unwrap_or_default()));
        }
        for r in &spec.states {
            idx.entry(format!("{file}.{}", r.id))
                .or_insert(NodeKind::State);
        }
        for t in &spec.transitions {
            idx.entry(format!("{file}.{}", t.id))
                .or_insert(NodeKind::Transition);
        }
    }
    idx
}

/// The Reference Typing check for one resolved link: `None` when the edge
/// is allowed (or the column is not a typed reference field), `Some(reason)`
/// when the typing table forbids it.
fn typing_violation(
    column: &str,
    source_kind: Option<&NodeKind>,
    target_kind: Option<&NodeKind>,
) -> Option<String> {
    let target = target_kind?;
    let tk = target.describe();
    match column {
        "traces_to" if *target != NodeKind::Intent => Some(format!(
            "traces_to must resolve to an Intent (Reference Typing); target is {tk}"
        )),
        "derives_from" if !matches!(target, NodeKind::Constraint(_)) => Some(format!(
            "derives_from must resolve to a Constraint (Reference Typing); target is {tk}"
        )),
        "guard" => match target {
            NodeKind::Constraint(k) if k == "invariant" => None,
            _ => Some(format!(
                "guard must resolve to an invariant Constraint (Reference Typing); target is {tk}"
            )),
        },
        "emits" => match target {
            NodeKind::Constraint(k) if k == "effect" => None,
            _ => Some(format!(
                "emits must resolve to an effect Constraint (Reference Typing); target is {tk}"
            )),
        },
        "satisfies" => match target {
            NodeKind::Constraint(k) if k == "extension_point" => None,
            _ => Some(format!(
                "satisfies must resolve to an extension_point Constraint (Reference Typing); target is {tk}"
            )),
        },
        "observes" => match target {
            NodeKind::Constraint(k) if k == "effect" => None,
            _ => Some(format!(
                "observes must resolve to an effect Constraint (Reference Typing); target is {tk}"
            )),
        },
        "supersedes" => {
            let source = source_kind?;
            if source.same_class(target) {
                None
            } else {
                Some(format!(
                    "supersedes must target the same kind as the row it appears on \
                     (Reference Typing); source is {}, target is {tk}",
                    source.describe()
                ))
            }
        }
        _ => None,
    }
}

/// Cycles in the `supersedes` edge set (`supersedes_dag`,
/// `specs/linter-graph_shape.md`). Deterministic: BTreeMap iteration plus
/// rotation-normalized cycle paths (a cycle found from any of its nodes
/// reports once, smallest node first).
fn find_supersedes_cycles(edges: &[Edge]) -> Vec<String> {
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for e in edges {
        if e.kind.ends_with(".supersedes") {
            adj.entry(e.from.as_str()).or_default().push(e.to.as_str());
        }
    }
    let mut found: BTreeSet<String> = BTreeSet::new();
    for start in adj.keys() {
        let mut path = vec![*start];
        let mut on_path: BTreeSet<&str> = BTreeSet::from([*start]);
        dfs_supersedes(start, start, &adj, &mut path, &mut on_path, &mut found);
    }
    found.into_iter().collect()
}

fn dfs_supersedes<'a>(
    cur: &'a str,
    start: &'a str,
    adj: &BTreeMap<&str, Vec<&'a str>>,
    path: &mut Vec<&'a str>,
    on_path: &mut BTreeSet<&'a str>,
    found: &mut BTreeSet<String>,
) {
    for next in adj.get(cur).into_iter().flatten() {
        if *next == start {
            // Rotate the cycle so it starts at its smallest node — the same
            // cycle is discovered from each of its members. The path's first
            // node closes the cycle (it equals `start`).
            let min = *path.iter().min().unwrap();
            let rotation = path.iter().position(|n| *n == min).unwrap_or(0);
            let mut nodes: Vec<&str> = path[rotation..].to_vec();
            nodes.extend_from_slice(&path[..rotation]);
            nodes.push(nodes[0]);
            found.insert(nodes.join(" → "));
        } else if !on_path.contains(next) {
            path.push(next);
            on_path.insert(next);
            dfs_supersedes(next, start, adj, path, on_path, found);
            on_path.remove(next);
            path.pop();
        }
    }
}

/// Build the graph for a corpus of parsed specs.
pub fn build(specs: &[Spec]) -> GraphReport {
    // Node-kind index for the Reference Typing checks.
    let kinds = kind_index(specs);
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
        // A Transition's `from`/`to` cells are typed reference fields (→
        // State, same file): one edge each, per `specs/graph.md`'s
        // total_extraction. An unknown state dangles — never dropped.
        for t in &spec.transitions {
            let from_node = format!("{file_id}.{}", t.id);
            for (edge_kind, cell) in [("transitions.from", &t.from), ("transitions.to", &t.to)] {
                let state = cell.trim().trim_matches('`').trim();
                if state.is_empty() {
                    // A null from/to cell is a Model-shape problem
                    // (linter-graph_shape), not a graph edge.
                    continue;
                }
                if spec.states.iter().any(|s| s.id == state) {
                    let edge = Edge {
                        from: from_node.clone(),
                        to: format!("{file_id}.{state}"),
                        kind: edge_kind.into(),
                    };
                    *report.fan_out.entry(edge.from.clone()).or_default() += 1;
                    *report.fan_in.entry(edge.to.clone()).or_default() += 1;
                    report.edges.push(edge);
                } else {
                    report.dangling.push(format!(
                        "{from_node} ({edge_kind}) → state `{state}` is not defined in {file_id}"
                    ));
                }
            }
        }
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
            // Bare-local rows (specodelic-15g, Option A): in an id:spec
            // file a dotless target naming one of the file's own rows has
            // exactly one possible meaning — the local row — so it
            // resolves instead of vanishing into the metasyntactic skip.
            // Dotful spellings keep the skip (ambiguous with `file.row`);
            // other files keep corpus-wide behavior.
            let bare_local = file_id == "spec"
                && !link.target.contains('.')
                && scoped_rows
                    .get(file_id)
                    .is_some_and(|rows| rows.contains(&link.target));
            // Metasyntactic example links (`[[old_id]]`, `[[...]]`) are
            // format documentation inside expr cells — not graph edges.
            let metasyn = !bare_local
                && (!link.target.contains('.') && !scoped_rows.contains_key(&link.target)
                    || link.target == "..."
                    || link.target == "…");
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
                    // Reference Typing (specs/specodelic.md): a typed
                    // reference column whose target kind is forbidden is
                    // reported as a labeled violation, never recorded as an
                    // edge (edge_kind_matches_typing, specs/graph.md).
                    let source_kind = match (link.field.as_str(), link.source.as_str()) {
                        ("constraints", _)
                        | ("properties", _)
                        | ("transitions", _)
                        | ("states", _) => {
                            let own = format!("{file_id}.{}", link.source);
                            kinds.get(&own)
                        }
                        _ => None,
                    };
                    if let Some(reason) =
                        typing_violation(&link.column, source_kind, kinds.get(&target))
                    {
                        report.violations.push(Violation {
                            from,
                            to: target,
                            edge_kind: kind,
                            reason,
                        });
                        continue;
                    }
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

    report.supersedes_cycles = find_supersedes_cycles(&report.edges);
    // External boundaries (specs/graph.md `external_boundary_derived`):
    // derived from published contracts alone — a stale tag cannot exist
    // because there is no tag.
    report.external_boundaries = specs
        .iter()
        .filter(|s| {
            s.constraints
                .iter()
                .any(|r| r.kind.as_deref() == Some("extension_point"))
        })
        .map(|s| s.intent.id.clone())
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect();
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
    // Bare-local row (specodelic-15g, Option A): the source file's own
    // rows resolve in bare spelling (canonical `file.row`). For non-dual
    // files the metasyntactic skip masks bare targets, so this arm only
    // fires for id:spec files.
    if !target.contains('.')
        && file_rows
            .get(source_file)
            .is_some_and(|rows| rows.iter().any(|r| r == target))
    {
        return Some(format!("{source_file}.{target}"));
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

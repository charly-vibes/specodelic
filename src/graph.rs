//! The typed reference graph — derived, never authored (`specs/graph.md`).
//!
//! Purpose: turn every typed reference field in the corpus into one
//! adjacency structure over the format's kinds. Responsibilities: extract
//! edges from structured cells only (including a Transition's `from`/`to`
//! state edges), resolve them corpus-wide, enforce the Reference Typing
//! table (forbidden edges are reported as labeled violations, never
//! recorded — `edge_kind_matches_typing`), flag `supersedes` cycles, and
//! report dangling references, per-node fan-in/fan-out, and the corpus's
//! sorted intent ids (the view layer's owning-file collapse key). Rationale: the
//! graph is the substrate rename/merge/refactor all build on — deriving it
//! once, deterministically, is what makes those tools safe. The raw edge
//! projection (`spk graph --format edges`, add-graph-views D2/D3) is the
//! parseable view of that substrate: canonical node ids only, every
//! violation riding along as an annotation row — a view is never cleaner
//! than the artifact. The dot/mermaid projections (`--format dot|mermaid`,
//! add-graph-views task 1.5/D8) render the same projection rows as
//! plain-text graph templates — zero crates, byte-stable re-runs, the D8
//! visual grammar (solid = state machine, dashed = guards, bold = `emits`,
//! dotted = traceability, red dashed = dangling/violations), rendering
//! always external. The wiring projection (`--view wiring`, task 1.6,
//! specodelic-5qj) collapses `constraints.satisfies` edges to the file
//! level — owning intents, self-loops dropped, the remaining pairs
//! aggregated with instance counts — exposing the corpus's declared
//! inter-file producer/consumer wiring, labeled `no_wiring` when none.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::acset::schema::{self, Endpoint, Schema};
use crate::spec::{Link, Spec};

/// One directed, typed edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    /// The corpus's intent (file) ids, sorted — the owning-file
    /// collapse key for the view layer (add-graph-views 2.4: the edges
    /// TSV alone cannot recover it — an intent without typed-reference
    /// endpoints never appears there, and dotted intent ids make prefix
    /// splitting ambiguous without the declared set).
    pub intents: Vec<String>,
}

/// What kind of node a resolved id addresses — the target side of the
/// Reference Typing checks.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodeKind {
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
    pub(crate) fn describe(&self) -> String {
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

    /// The schema object this node kind belongs to (the Reference Typing
    /// table's Appears-on / Must-resolve-to vocabulary).
    pub(crate) fn object(&self) -> &'static str {
        match self {
            NodeKind::Intent => "Intent",
            NodeKind::Constraint(_) => "Constraint",
            NodeKind::Property(_) => "Property",
            NodeKind::State => "State",
            NodeKind::Transition => "Transition",
        }
    }

    /// The row's own kind cell — empty when the object carries none
    /// (Intent/State/Transition have no kind column of their own).
    pub(crate) fn kind_cell(&self) -> &str {
        match self {
            NodeKind::Constraint(k) | NodeKind::Property(k) => k,
            NodeKind::Intent | NodeKind::State | NodeKind::Transition => "",
        }
    }
}

/// Index every resolvable id in the corpus to its node kind: file ids to
/// their Intent, row ids (file-qualified `file.row`) to the row's layer.
pub(crate) fn kind_index(specs: &[Spec]) -> BTreeMap<String, NodeKind> {
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

/// The closed union of typed reference columns the graph extracts edges
/// from — `specs/graph.md`'s `total_extraction`, whose authority is the
/// Reference Typing table in `specs/specodelic.md`. `satisfies` IS a typed
/// field (Revision 7) even though graph.md's parenthetical enumeration
/// predates it — `errors.md`'s `contract_satisfied_from_consumer` pins
/// satisfies edges as extracted. A link in any OTHER column — `expr`,
/// `predicate`, `generator`, frontmatter — is format documentation or a
/// prose citation, not a reference field: it never becomes an edge, never
/// dangles in the graph report, and never trips typing. Its resolution is
/// `total_refs`' beat in the linter, which scopes over every link in the
/// file (beads specodelic-mlg). `from`/`to` are absent here by design: the
/// transitions walk below owns those fields (one edge per well-formed
/// cell), so the links loop must not double-record them.
pub(crate) const TYPED_REFERENCE_COLUMNS: [&str; 7] = [
    "traces_to",
    "derives_from",
    "guard",
    "supersedes",
    "emits",
    "satisfies",
    "observes",
];

/// The Reference Typing check for one resolved link: `None` when the edge
/// is allowed (or the column is not a typed reference field), `Some(reason)`
/// when the typing table forbids it. The decision is the Schema's — this
/// adapter only maps `NodeKind` onto the schema's endpoint vocabulary; the
/// per-field match arms this function used to carry are retired
/// (`typing_table_is_data`, add-acset-core task 2.3).
fn typing_violation(
    schema: &Schema,
    column: &str,
    source_kind: Option<&NodeKind>,
    target_kind: Option<&NodeKind>,
) -> Option<String> {
    // Dangling references are the caller's beat (`let target = ...?`).
    let target = target_kind?;
    let endpoint = |k: &NodeKind| Endpoint {
        object: k.object().to_string(),
        kind: k.kind_cell().to_string(),
        describe: k.describe(),
    };
    schema::typing_violation(schema, column, source_kind.map(endpoint), endpoint(target))
}

/// Record one resolved edge — the single place an edge enters the report
/// (task 2.5 TIDY: both derivation sites — the transitions' from/to cells
/// and the typed reference links — carried the same three-line fan/edge
/// dance; now they share one entry point). The fan counts are no longer
/// accumulated here: task 4.4 derives them once, after the walk, through
/// `acset::query`'s primitives (`fan_in_is_preimage_size`), so edge
/// accounting and fan accounting cannot drift apart.
fn record_edge(report: &mut GraphReport, edge: Edge) {
    report.edges.push(edge);
}

/// One typed reference link's shared resolution (task 3.4 TIDY): the
/// outcome both `graph::build`'s report and the acset builder consume —
/// the typed-column filter, `resolve`, the bare-local and metasyntactic
/// skips, the anchoring, and the source-kind lookup are derived exactly
/// once here, so the two walks cannot drift apart.
pub(crate) enum ResolvedLink {
    /// The link resolved to a node — classify it against the Reference
    /// Typing schema before storing.
    Stored {
        /// The anchor the graph reports the edge from: `file (intent)` for
        /// frontmatter links, `file.row` for table links.
        anchor: String,
        /// The source node the instance's morphism vector keys on.
        source_node: String,
        /// The Reference Typing column the link sat in.
        column: String,
        /// The graph's edge-kind spelling (`field.column`).
        kind: String,
        /// The resolved target id.
        target: String,
        /// The source node's kind, for the typing check.
        source_kind: Option<NodeKind>,
    },
    /// The link resolved to nothing — the dangling value, with the pieces
    /// both the message shapes and the instance's stored `None` derive
    /// from.
    Dangling {
        file_id: String,
        anchor: String,
        source_node: String,
        column: String,
        target: String,
    },
}

/// Resolve one parsed link against the walk's resolution index — `None`
/// when the link is not graph structure at all (an untyped column, a
/// metasyntactic example). The one derivation `graph::build` and the
/// acset builder share: whatever it computes, both paths see identically.
pub(crate) fn resolve_link(
    link: &Link,
    file_id: &str,
    scoped_rows: &BTreeMap<String, Vec<String>>,
    kinds: &BTreeMap<String, NodeKind>,
) -> Option<ResolvedLink> {
    // total_extraction: only typed reference columns produce graph
    // structure. Links elsewhere (expr/predicate cells, frontmatter) are
    // the linter's `total_refs` beat — skipped entirely here.
    if !TYPED_REFERENCE_COLUMNS.contains(&link.column.as_str()) {
        return None;
    }
    let resolved = resolve(scoped_rows, file_id, &link.target);
    // Bare-local rows (specodelic-15g, Option A): in an id:spec file a
    // dotless target naming one of the file's own rows has exactly one
    // possible meaning — the local row — so it resolves instead of
    // vanishing into the metasyntactic skip. Dotful spellings keep the
    // skip (ambiguous with `file.row`); other files keep corpus-wide
    // behavior.
    let bare_local = file_id == "spec"
        && !link.target.contains('.')
        && scoped_rows
            .get(file_id)
            .is_some_and(|rows| rows.contains(&link.target));
    // Metasyntactic example links (`[[old_id]]`, `[[...]]`) are format
    // documentation inside expr cells — not graph edges.
    let metasyn = !bare_local
        && (!link.target.contains('.') && !scoped_rows.contains_key(&link.target)
            || link.target == "..."
            || link.target == "\u{2026}");
    if metasyn {
        return None;
    }
    // Edge anchoring (shared by the recorded edge and the interface-shaped
    // dangling message below): frontmatter links anchor on the intent row,
    // table links on `file.row`.
    let anchor = if link.source == *file_id {
        format!("{file_id} (intent)")
    } else {
        format!("{file_id}.{}", link.source)
    };
    let source_node = if link.source == *file_id {
        file_id.to_string()
    } else {
        format!("{file_id}.{}", link.source)
    };
    Some(match resolved {
        Some(target) => {
            let kind = if link.column.is_empty() {
                link.field.clone()
            } else {
                format!("{}.{}", link.field, link.column)
            };
            let source_kind = match (link.field.as_str(), link.source.as_str()) {
                ("constraints", _) | ("properties", _) | ("transitions", _) | ("states", _) => {
                    let own = format!("{file_id}.{}", link.source);
                    kinds.get(&own)
                }
                _ => None,
            };
            ResolvedLink::Stored {
                anchor,
                source_node,
                column: link.column.clone(),
                kind,
                target,
                source_kind: source_kind.cloned(),
            }
        }
        None => ResolvedLink::Dangling {
            file_id: file_id.to_string(),
            anchor,
            source_node,
            column: link.column.clone(),
            target: link.target.clone(),
        },
    })
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

/// Strip a display qualifier — `id (intent)`, `file.row (member)` — down
/// to the canonical node id (intent ids and qualified row ids only in
/// projections; add-graph-views D2). Ids without a qualifier pass through.
pub(crate) fn canonical_id(id: &str) -> &str {
    match id.strip_suffix(')').and_then(|s| s.rsplit_once(" (")) {
        Some((base, _)) => base,
        None => id,
    }
}

/// Pin the violation annotation column tab-free so the six-column TSV
/// contract holds regardless of reason prose (task 1.2: full reason text,
/// escaped).
pub(crate) fn escape_tab_free(text: &str) -> String {
    text.replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

/// One edge-projection row in structured form — the six TSV columns
/// without their separators (add-graph-views task 1.4 TIDY). The pure
/// projection core (`edge_projection`) produces these; the TSV renderer
/// and the task 1.5/1.6 dot/mermaid renderers both consume the same rows,
/// so normalization and column semantics cannot drift between formats.
pub(crate) struct ProjectionRow {
    /// Canonical source node id (D2: display labels normalized away);
    /// empty on violation annotation rows.
    pub from: String,
    /// The source node's schema object label (`Intent`, `Constraint`, …);
    /// empty when the id is not in the corpus's kind index.
    pub from_kind: String,
    /// The edge kind (`field.column`), or `violation:<edge_kind>` on an
    /// annotation row.
    pub field: String,
    /// Canonical target node id.
    pub to: String,
    /// The target node's schema object label.
    pub to_kind: String,
    /// The finding text on annotation rows (tab-escaped, D3); empty on
    /// recorded-edge rows.
    pub annotation: String,
    /// The forbidden edge's source id (canonical, D2) on violation
    /// annotation rows — the TSV contract drops it (empty source columns,
    /// task 1.2) but the task 1.5 dot/mermaid renderers draw the forbidden
    /// edge red dashed from it; empty on recorded-edge rows.
    pub violation_from: String,
}

/// The row's byte-exact TSV rendering — six tab-separated columns. Every
/// row carries the trailing separator of the empty annotation column, so
/// the six-column contract holds line-wise.
pub(crate) fn render_tsv_row(row: &ProjectionRow) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        row.from, row.from_kind, row.field, row.to, row.to_kind, row.annotation
    )
}

/// The pure projection core (`spk graph --format edges`, add-graph-views
/// D2/D3; task 1.4 TIDY): normalizes label-qualified endpoints to
/// canonical node ids (`canonical_id`, D2), maps endpoints to their
/// schema object labels through the corpus kind index, maps every typing
/// violation to an annotation row (empty source id/kind,
/// `violation:<edge_kind>` in the field column, the full reason text —
/// tab-escaped — in the annotation column, D3), preserves multiplicity
/// (one row per reference instance), and emits the rows sorted by their
/// TSV rendering. Pure: a function over the report and the kind index —
/// no I/O, no CLI state, no parsing — so the format renderers (edges,
/// dot, mermaid) reuse one normalization path.
pub(crate) fn edge_projection(
    report: &GraphReport,
    kinds: &BTreeMap<String, NodeKind>,
) -> Vec<ProjectionRow> {
    let kind_of = |id: &str| {
        kinds
            .get(id)
            .map(|k| k.object().to_string())
            .unwrap_or_default()
    };
    let mut rows: Vec<ProjectionRow> =
        Vec::with_capacity(report.edges.len() + report.violations.len());
    for e in &report.edges {
        let from = canonical_id(&e.from);
        let to = canonical_id(&e.to);
        rows.push(ProjectionRow {
            from: from.into(),
            from_kind: kind_of(from),
            field: e.kind.clone(),
            to: to.into(),
            to_kind: kind_of(to),
            annotation: String::new(),
            violation_from: String::new(),
        });
    }
    for v in &report.violations {
        let to = canonical_id(&v.to);
        rows.push(ProjectionRow {
            from: String::new(),
            from_kind: String::new(),
            field: format!("violation:{}", v.edge_kind),
            to: to.into(),
            to_kind: kind_of(to),
            annotation: escape_tab_free(&v.reason),
            violation_from: canonical_id(&v.from).into(),
        });
    }
    rows.sort_by_cached_key(render_tsv_row);
    rows
}

/// The raw six-column TSV edge projection (`spk graph --format edges`,
/// add-graph-views D2/D3). Derives the report and kind index from the
/// corpus, then delegates to the pure projection core and renders one TSV
/// line per row: source_id, source_kind, field, target_id, target_kind,
/// annotation. Sorted; an empty corpus yields an empty TSV.
pub fn edges_tsv(specs: &[Spec]) -> String {
    let report = build(specs);
    let kinds = kind_index(specs);
    let mut tsv = String::new();
    for row in edge_projection(&report, &kinds) {
        tsv.push_str(&render_tsv_row(&row));
        tsv.push('\n');
    }
    tsv
}

/// DOT string escaping — ids and labels are always double-quoted, so the
/// only characters that can break out are backslash and quote.
fn dot_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The D8 visual grammar as DOT edge attributes: dashed = guards, bold =
/// `emits`, dotted = traceability, solid (no attributes) = the state
/// machine. Unknown kinds default to solid (the retired jq bridge's
/// `// ""` fallback).
fn dot_edge_style(field: &str) -> &'static str {
    match field.rsplit_once('.').map(|(_, kind)| kind) {
        Some("guard") => ", style=dashed",
        Some("emits") => ", penwidth=2",
        Some("traces_to") => ", style=dotted",
        Some("derives_from") => ", style=dotted, color=gray50",
        _ => "",
    }
}

/// The DOT projection (add-graph-views task 1.5, D8) — pure renderer over
/// the projection rows plus the dangling messages. Shape parity with the
/// retired `scripts/graph_to_dot.jq`: `digraph spec {` header,
/// `rankdir=LR`, ellipse default node, two-space indentation,
/// `"<from>" -> "<to>" [label="<kind>"<style>];` edge lines, and jq
/// `unique`-style sorted dedup (a duplicated reference instance renders
/// one line — multiplicity is the TSV's contract, not the drawing's).
/// Typing violations render the forbidden edge red dashed with the full
/// reason in the label (D3: never silently clean); dangling references
/// render as red dashed note nodes carrying the message verbatim — no
/// prose parsing, the message shapes differ per remediation.
pub(crate) fn render_dot(rows: &[ProjectionRow], dangling: &[String]) -> String {
    let mut lines: BTreeSet<String> = BTreeSet::new();
    for row in rows {
        if row.field.starts_with("violation:") {
            let label = format!("{}: {}", row.field, row.annotation);
            if row.violation_from.is_empty() {
                lines.insert(format!(
                    "  \"{}\" [shape=box, color=red, style=dashed, label=\"{}\"];",
                    dot_escape(&row.to),
                    dot_escape(&label)
                ));
            } else {
                lines.insert(format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\", style=dashed, color=red];",
                    dot_escape(&row.violation_from),
                    dot_escape(&row.to),
                    dot_escape(&label)
                ));
            }
        } else {
            lines.insert(format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"{}];",
                dot_escape(&row.from),
                dot_escape(&row.to),
                dot_escape(&row.field),
                dot_edge_style(&row.field)
            ));
        }
    }
    for message in dangling {
        lines.insert(format!(
            "  \"{}\" [shape=box, color=red, style=dashed];",
            dot_escape(message)
        ));
    }
    let mut out =
        String::from("digraph spec {\n  rankdir=LR;\n  node [shape=ellipse, fontsize=10];\n");
    for line in &lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

/// The DOT projection of a corpus (`spk graph --format dot`, task 1.5):
/// derive the report and kind index, delegate to the pure projection
/// core, render. Rendering stays 100 % external (D8) — the tool never
/// shells out to a renderer.
pub fn dot_projection(specs: &[Spec]) -> String {
    let report = build(specs);
    let kinds = kind_index(specs);
    render_dot(&edge_projection(&report, &kinds), &report.dangling)
}

/// Mermaid label/id escaping — quoted labels, so only the quote itself
/// breaks out (HTML-escaped, mermaid's own convention).
fn mermaid_escape(text: &str) -> String {
    text.replace('"', "&quot;")
}

/// Mermaid ids reject the dots canonical ids carry (`two.c1`), so ids
/// sanitize to `[A-Za-z0-9_]` and the original id rides as the quoted
/// node label. Sanitization is made injective with `_2`, `_3` suffixes —
/// `a.b` and `a_b` must not merge into one drawn node. `reserved` names
/// (the dangling notes) are never assigned to regular nodes.
fn mermaid_node_ids(
    ids: &BTreeSet<String>,
    reserved: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    let mut used: BTreeSet<String> = reserved.clone();
    for id in ids {
        let mut base: String = id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        if base.is_empty() {
            base = "n".into();
        }
        let mut name = base.clone();
        let mut n = 2;
        while used.contains(&name) {
            name = format!("{base}_{n}");
            n += 1;
        }
        used.insert(name.clone());
        names.insert(id.clone(), name);
    }
    names
}

/// One projection row's mermaid rendering: a link (with the CSS its
/// grammar role needs via `linkStyle`) or, for a from-less violation, an
/// annotated node. Endpoints resolve through the node-name map; a
/// degenerate row with an unmapped endpoint is skipped, never a panic —
/// and a skipped violation still arrived through the rows, so a clean
/// view can only mean the artifact was clean (D3).
enum MermaidElement {
    /// A link line plus the `linkStyle` CSS it needs (None = the arrow
    /// type alone carries the grammar).
    Link(String, Option<&'static str>),
    /// A from-less violation's annotated node line.
    ViolationNode(String),
}

fn mermaid_row_element(
    row: &ProjectionRow,
    names: &BTreeMap<String, String>,
) -> Option<MermaidElement> {
    let node_of = |id: &str| names.get(id);
    if row.field.starts_with("violation:") {
        // `to` is always resolved (typing_violation's `target_kind?`) but
        // the renderer is pure — skip instead of panicking.
        let to = node_of(&row.to)?;
        let label = format!("{}: {}", row.field, row.annotation);
        Some(match node_of(&row.violation_from) {
            Some(from) => MermaidElement::Link(
                format!("  {from} -->|\"{}\"| {to}", mermaid_escape(&label)),
                Some("stroke:red,stroke-dasharray:5 5"),
            ),
            None => MermaidElement::ViolationNode(format!(
                "  {to}[\"{}\"]:::violation",
                mermaid_escape(&label)
            )),
        })
    } else {
        let from = node_of(&row.from)?;
        let to = node_of(&row.to)?;
        let (arrow, style): (&str, Option<&'static str>) =
            match row.field.rsplit_once('.').map(|(_, kind)| kind) {
                Some("guard") => ("-.->", None),
                Some("emits") => ("==>", None),
                Some("traces_to" | "derives_from") => ("-->", Some("stroke-dasharray:2 2")),
                _ => ("-->", None),
            };
        Some(MermaidElement::Link(
            format!("  {from} {arrow}|\"{}\"| {to}", mermaid_escape(&row.field)),
            style,
        ))
    }
}

/// The mermaid projection (add-graph-views task 1.5, D8) — pure renderer
/// over the projection rows plus the dangling messages. Structure: nodes
/// declared first (sorted, quoted labels), then links (labeled with the
/// edge kind, sorted + deduped like the dot renderer), then `linkStyle`
/// lines in link order, then classDefs — emitted only when used — and
/// the annotated red elements (from-less violations, dangling notes).
/// Visual grammar: `-->` solid = state machine, `-.->` dashed = guards,
/// `==>` bold = `emits`, `linkStyle … stroke-dasharray:2 2` dotted =
/// traceability, `stroke:red,stroke-dasharray:5 5` red dashed =
/// violations/dangling (D3: never silently clean).
pub(crate) fn render_mermaid(rows: &[ProjectionRow], dangling: &[String]) -> String {
    // The dangling notes reserve `dangling_<n>` names (sorted messages,
    // 1-based) before regular ids are assigned, so a corpus node literally
    // named `dangling_1` cannot swallow a note.
    let mut sorted_dangling: BTreeSet<&str> = BTreeSet::new();
    for message in dangling {
        sorted_dangling.insert(message);
    }
    let reserved: BTreeSet<String> = (1..=dangling.len())
        .map(|n| format!("dangling_{n}"))
        .collect();
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for row in rows {
        for id in [&row.from, &row.to, &row.violation_from] {
            if !id.is_empty() {
                ids.insert(id.clone());
            }
        }
    }
    let names = mermaid_node_ids(&ids, &reserved);
    let mut links: BTreeMap<String, Option<&'static str>> = BTreeMap::new();
    let mut violation_nodes: BTreeSet<String> = BTreeSet::new();
    for row in rows {
        match mermaid_row_element(row, &names) {
            Some(MermaidElement::Link(line, style)) => {
                links.insert(line, style);
            }
            Some(MermaidElement::ViolationNode(line)) => {
                violation_nodes.insert(line);
            }
            None => {}
        }
    }
    let mut out = String::from("flowchart LR\n");
    for (id, name) in &names {
        out.push_str(&format!("  {name}[\"{}\"]\n", mermaid_escape(id)));
    }
    let mut styles: Vec<String> = Vec::new();
    for (index, (line, style)) in links.iter().enumerate() {
        out.push_str(line);
        out.push('\n');
        if let Some(css) = style {
            styles.push(format!("  linkStyle {index} {css}"));
        }
    }
    for line in styles {
        out.push_str(&line);
        out.push('\n');
    }
    if links
        .values()
        .any(|s| *s == Some("stroke:red,stroke-dasharray:5 5"))
        || !violation_nodes.is_empty()
    {
        out.push_str("  classDef violation stroke:red,stroke-dasharray:5 5\n");
    }
    if !sorted_dangling.is_empty() {
        out.push_str("  classDef dangling stroke:red,stroke-dasharray:5 5\n");
    }
    for line in &violation_nodes {
        out.push_str(line);
        out.push('\n');
    }
    for (index, message) in sorted_dangling.iter().enumerate() {
        out.push_str(&format!(
            "  dangling_{}[\"{}\"]:::dangling\n",
            index + 1,
            mermaid_escape(message)
        ));
    }
    out
}

/// The mermaid projection of a corpus (`spk graph --format mermaid`,
/// task 1.5): derive the report and kind index, delegate to the pure
/// projection core, render. Rendering stays 100 % external (D8).
pub fn mermaid_projection(specs: &[Spec]) -> String {
    let report = build(specs);
    let kinds = kind_index(specs);
    render_mermaid(&edge_projection(&report, &kinds), &report.dangling)
}

/// The wiring view's empty-state label — the annotated element an
/// otherwise empty view must carry instead of rendering silently clean
/// (task 1.6, specodelic-5qj's exploration_only ≠ clean principle).
pub(crate) const NO_WIRING_NOTE: &str =
    "no constraints.satisfies edges — this corpus declares no inter-file wiring";

/// One file-level wiring row (add-graph-views task 1.6, specodelic-5qj):
/// a distinct consumer-file → producer-file pair and how many
/// `constraints.satisfies` instances collapsed onto it. The drawn arrow
/// follows the satisfies edge direction — consumer file → the producer's
/// published contract — matching the decision record's sample outputs.
pub(crate) struct WiringRow {
    /// The file whose constraint declares the satisfies edge.
    pub consumer: String,
    /// The file publishing the extension_point contract row the edge
    /// points at.
    pub producer: String,
    /// Collapsed satisfies instances (view-layer aggregation; multiplicity
    /// is the raw TSV's contract, not the drawing's — task 1.7).
    pub instances: usize,
}

/// Map a canonical node id to its owning file: intent ids are their own
/// file; qualified row ids split at the last dot (the resolution
/// convention of record, specodelic-njh — row ids are single-segment so
/// the file prefix is everything before it). An id attributable to no
/// known file falls back to itself — the wiring view never silently drops
/// an edge it cannot attribute (D3).
fn owning_file(id: &str, kinds: &BTreeMap<String, NodeKind>) -> String {
    if kinds.get(id) == Some(&NodeKind::Intent) {
        return id.to_string();
    }
    match id.rsplit_once('.') {
        Some((file, _)) if kinds.get(file) == Some(&NodeKind::Intent) => file.to_string(),
        _ => id.to_string(),
    }
}

/// The pure wiring core (`spk graph --view wiring`, task 1.6): filter the
/// report's edges to `constraints.satisfies`, collapse each endpoint to
/// its owning file (`owning_file`), drop self-loops (a file satisfying
/// its own contract carries no inter-file wiring), and aggregate the
/// remaining instances per distinct consumer→producer pair, sorted by the
/// pair. Pure over the report and kind index — no I/O, no CLI state — so
/// the three format renderers reuse one collapse path.
pub(crate) fn wiring_projection(
    report: &GraphReport,
    kinds: &BTreeMap<String, NodeKind>,
) -> Vec<WiringRow> {
    let mut pairs: BTreeMap<(String, String), usize> = BTreeMap::new();
    for e in &report.edges {
        if e.kind != "constraints.satisfies" {
            continue;
        }
        let consumer = owning_file(&e.from, kinds);
        let producer = owning_file(&e.to, kinds);
        if consumer == producer {
            continue;
        }
        *pairs.entry((consumer, producer)).or_insert(0) += 1;
    }
    pairs
        .into_iter()
        .map(|((consumer, producer), instances)| WiringRow {
            consumer,
            producer,
            instances,
        })
        .collect()
}

/// The wiring TSV (`--view wiring --format edges`): one row per distinct
/// pair — consumer_id, consumer_kind, `constraints.satisfies`,
/// producer_id, producer_kind, collapsed instance count — same six-column
/// shape as the raw projection, with the owning-intent ids in the id
/// columns and the aggregation count in the annotation column. An empty
/// view emits the labeled `no_wiring` row (marker in the field column,
/// the explanation in the annotation column — the violation-row
/// convention), never a silently clean empty TSV.
fn render_wiring_tsv(rows: &[WiringRow], kinds: &BTreeMap<String, NodeKind>) -> String {
    let kind_of = |id: &str| {
        kinds
            .get(id)
            .map(|k| k.object().to_string())
            .unwrap_or_default()
    };
    if rows.is_empty() {
        return format!("\t\tno_wiring\t\t\t{NO_WIRING_NOTE}\n");
    }
    let mut out = String::new();
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\tconstraints.satisfies\t{}\t{}\t{}\n",
            row.consumer,
            kind_of(&row.consumer),
            row.producer,
            kind_of(&row.producer),
            row.instances
        ));
    }
    out
}

/// The wiring DOT projection (`--view wiring --format dot`): shape parity
/// with the specodelic-5qj exploration sample — `digraph wiring {` header,
/// `rankdir=LR`, box/rounded nodes, gray thin edge defaults, and one
/// `"<consumer>" -> "<producer>" [label="<n> satisfies"];` line per pair.
/// An empty view renders the red dashed `no_wiring` note node (D3: never
/// silently clean).
fn render_wiring_dot(rows: &[WiringRow]) -> String {
    let mut out = String::from(
        "digraph wiring {\n  rankdir=LR;\n  node [shape=box, style=rounded, fontsize=11];\n  edge [color=gray30, arrowsize=0.7];\n",
    );
    if rows.is_empty() {
        out.push_str(&format!(
            "  \"no_wiring\" [shape=box, color=red, style=dashed, label=\"no_wiring: {}\"];\n",
            dot_escape(NO_WIRING_NOTE)
        ));
    }
    for row in rows {
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{} satisfies\"];\n",
            dot_escape(&row.consumer),
            dot_escape(&row.producer),
            row.instances
        ));
    }
    out.push_str("}\n");
    out
}

/// The wiring mermaid projection (`--view wiring --format mermaid`): file
/// nodes declared first (sorted, quoted labels), then the aggregated
/// `-->` links labeled `<n> satisfies`. An empty view renders the red
/// `no_wiring` node with the violation classDef (D3: never silently
/// clean).
fn render_wiring_mermaid(rows: &[WiringRow]) -> String {
    let mut out = String::from("flowchart LR\n");
    if rows.is_empty() {
        out.push_str(&format!(
            "  no_wiring[\"no_wiring: {}\"]:::violation\n",
            mermaid_escape(NO_WIRING_NOTE)
        ));
        out.push_str("  classDef violation stroke:red,stroke-dasharray:5 5\n");
        return out;
    }
    let ids: BTreeSet<String> = rows
        .iter()
        .flat_map(|r| [r.consumer.clone(), r.producer.clone()])
        .collect();
    let names = mermaid_node_ids(&ids, &BTreeSet::new());
    for (id, name) in &names {
        out.push_str(&format!("  {name}[\"{}\"]\n", mermaid_escape(id)));
    }
    for row in rows {
        out.push_str(&format!(
            "  {} -->|\"{} satisfies\"| {}\n",
            names[&row.consumer], row.instances, names[&row.producer]
        ));
    }
    out
}

/// The wiring TSV of a corpus (`spk graph --view wiring --format edges`,
/// task 1.6): derive the report and kind index, delegate to the pure
/// wiring core, render.
pub fn wiring_tsv(specs: &[Spec]) -> String {
    let report = build(specs);
    let kinds = kind_index(specs);
    render_wiring_tsv(&wiring_projection(&report, &kinds), &kinds)
}

/// The wiring DOT projection of a corpus (`spk graph --view wiring
/// --format dot`, task 1.6).
pub fn wiring_dot(specs: &[Spec]) -> String {
    let report = build(specs);
    let kinds = kind_index(specs);
    render_wiring_dot(&wiring_projection(&report, &kinds))
}

/// The wiring mermaid projection of a corpus (`spk graph --view wiring
/// --format mermaid`, task 1.6).
pub fn wiring_mermaid(specs: &[Spec]) -> String {
    let report = build(specs);
    let kinds = kind_index(specs);
    render_wiring_mermaid(&wiring_projection(&report, &kinds))
}

/// Build the graph for a corpus of parsed specs.
pub fn build(specs: &[Spec]) -> GraphReport {
    // The Reference Typing table as data — the schema the typing check
    // reads (`typing_table_is_data`), built once per report.
    let schema = schema::canonical();
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
        intents: specs
            .iter()
            .map(|s| s.intent.id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
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
                    record_edge(
                        &mut report,
                        Edge {
                            from: from_node.clone(),
                            to: format!("{file_id}.{state}"),
                            kind: edge_kind.into(),
                        },
                    );
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
            // The shared edge derivation (task 3.4 TIDY): the typed-column
            // filter, resolution, bare-local and metasyntactic skips,
            // anchoring, and source-kind lookup are derived once — the
            // graph report and the acset builder consume the same result.
            let Some(outcome) = resolve_link(link, file_id, &scoped_rows, &kinds) else {
                continue;
            };
            match outcome {
                ResolvedLink::Stored {
                    anchor,
                    column,
                    kind,
                    target,
                    source_kind,
                    ..
                } => {
                    // Reference Typing (specs/specodelic.md): a typed
                    // reference column whose target kind is forbidden is
                    // reported as a labeled violation, never recorded as an
                    // edge (edge_kind_matches_typing, specs/graph.md).
                    if let Some(reason) =
                        typing_violation(&schema, &column, source_kind.as_ref(), kinds.get(&target))
                    {
                        report.violations.push(Violation {
                            from: anchor,
                            to: target,
                            edge_kind: kind,
                            reason,
                        });
                        continue;
                    }
                    record_edge(
                        &mut report,
                        Edge {
                            from: anchor,
                            to: target,
                            kind,
                        },
                    );
                }
                ResolvedLink::Dangling {
                    file_id,
                    anchor,
                    column,
                    target,
                    ..
                } => {
                    // specodelic-2q8: consumption edges get an
                    // interface-shaped dangling message — a reader cannot
                    // tell "typo'd row id" from "consuming a contract
                    // nobody published" from the generic shape, and the
                    // remediation differs. Honesty rule (ticket scope):
                    // name BOTH remediations; the message must not claim
                    // to know which applies. Non-consumption columns keep
                    // the generic shape (other tooling/tests may pin it).
                    if matches!(column.as_str(), "satisfies" | "observes") {
                        report.dangling.push(format!(
                            "{anchor} ({column}) → [[{target}]]: no published \
                             contract row {target} exists — publish it in the \
                             producer's file or fix the id"
                        ));
                    } else {
                        report.dangling.push(format!("{file_id} → [[{target}]]"));
                    }
                }
            }
        }
    }

    report.supersedes_cycles = find_supersedes_cycles(&report.edges);
    // Fan counts derive from the same instance the parity property pins
    // (`fan_in_is_preimage_size`, task 4.4): one derivation, through the
    // acset primitives, after the walk — never accumulated per site.
    let instance = crate::acset::instance::Instance::from_specs(specs);
    report.fan_in = crate::acset::query::fan_in(&instance);
    report.fan_out = crate::acset::query::fan_out(&instance);
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
pub(crate) fn resolve(
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
    // Every split point, last dot first (algorithm: specodelic-njh):
    // dotted file ids are the common case, dotted row ids resolve via
    // the member arm at any split point.
    let mut dots: Vec<usize> = target.match_indices('.').map(|(i, _)| i).collect();
    dots.reverse();
    for i in dots {
        let (file_id, rest) = (&target[..i], &target[i + 1..]);
        if let Some(rows) = file_rows.get(file_id) {
            if rows.iter().any(|r| r == rest) || rest == "model.state" || rest == "model.transition"
            {
                return Some(target.to_string());
            }
            if let Some((row_id, member)) = rest.split_once('.')
                && rows.iter().any(|r| r == row_id)
            {
                return Some(format!("{file_id}.{row_id} ({member})"));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pinned (task 1.4): the pure projection core over a
    /// hand-constructed report — no I/O, no CLI state. Label-qualified
    /// endpoints normalize to canonical ids (D2), kinds resolve through
    /// the kind index, multiplicity is preserved, rows come out sorted by
    /// their TSV rendering.
    #[test]
    fn edge_projection_normalizes_sorts_and_preserves_multiplicity() {
        let report = GraphReport {
            edges: vec![
                Edge {
                    from: "b.c1 (intent)".into(),
                    to: "a".into(),
                    kind: "constraints.traces_to".into(),
                },
                Edge {
                    from: "b.c1".into(),
                    to: "a".into(),
                    kind: "constraints.traces_to".into(),
                },
            ],
            violations: vec![Violation {
                from: "b.c1".into(),
                to: "b.c2".into(),
                edge_kind: "constraints.traces_to".into(),
                reason: "must resolve\tto an Intent".into(),
            }],
            ..Default::default()
        };
        let kinds = BTreeMap::from([
            ("a".to_string(), NodeKind::Intent),
            ("b.c1".to_string(), NodeKind::Constraint("invariant".into())),
            ("b.c2".to_string(), NodeKind::Constraint(String::new())),
        ]);
        let rows = edge_projection(&report, &kinds);
        let rendered: Vec<String> = rows.iter().map(render_tsv_row).collect();
        assert_eq!(
            rendered,
            vec![
                "\t\tviolation:constraints.traces_to\tb.c2\tConstraint\tmust resolve\\tto an Intent".to_string(),
                "b.c1\tConstraint\tconstraints.traces_to\ta\tIntent\t".to_string(),
                "b.c1\tConstraint\tconstraints.traces_to\ta\tIntent\t".to_string(),
            ],
            "annotation row first (empty source sorts before ids), \"b.c1 (intent)\" \
             normalized to the canonical id, duplicate instances preserved, reason tab-escaped"
        );
    }

    /// Pinned (task 1.4): unknown ids project an empty kind column rather
    /// than panicking or guessing — a violation may target a node the
    /// kind index addresses only through its canonical id.
    #[test]
    fn edge_projection_unknown_ids_carry_empty_kind() {
        let report = GraphReport {
            edges: vec![Edge {
                from: "x.t".into(),
                to: "x.s1".into(),
                kind: "transitions.from".into(),
            }],
            ..Default::default()
        };
        let rows = edge_projection(&report, &BTreeMap::new());
        assert_eq!(
            rows.iter().map(render_tsv_row).collect::<Vec<_>>(),
            vec!["x.t\t\ttransitions.from\tx.s1\t\t"],
        );
    }

    /// Rows map fixture from (file id, row ids).
    fn rows_from(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(f, rows)| (f.to_string(), rows.iter().map(|r| r.to_string()).collect()))
            .collect()
    }

    /// RED (specodelic-njh): multi-segment MEMBER path via the first-dot
    /// fallback arm — `a.b.c.d` = file `a`, row `b`, member path `c.d`.
    #[test]
    fn member_path_resolves_in_first_dot_arm() {
        let rows = rows_from(&[("a", &["b"])]);
        assert_eq!(
            resolve(&rows, "a", "a.b.c.d"),
            Some("a.b (c.d)".to_string())
        );
    }

    /// Pinned (specodelic-njh): row ids are single-segment — a dotted
    /// "row" tail means row.member, never a dotted row.
    #[test]
    fn dotted_row_ids_are_unaddressable() {
        let rows = rows_from(&[("a", &["b.c"])]);
        assert_eq!(resolve(&rows, "a", "a.b.c.d"), None);
    }

    /// Pinned (specodelic-njh): dotted file id + row/member via last-dot
    /// split — the shape gh#1 named (intent id contains a dot).
    #[test]
    fn dotted_file_id_resolves_last_dot_first() {
        let rows = rows_from(&[("extraction.claims", &["span"])]);
        assert_eq!(
            resolve(&rows, "other", "extraction.claims.span"),
            Some("extraction.claims.span".to_string())
        );
        let rows2 = rows_from(&[("a.b", &["c"])]);
        assert_eq!(
            resolve(&rows2, "a.b", "a.b.c.d"),
            Some("a.b.c (d)".to_string())
        );
    }

    /// Pinned (specodelic-njh): anchors and bare-local rows.
    #[test]
    fn anchors_and_bare_local_rows() {
        let rows = rows_from(&[("a", &["r"])]);
        assert_eq!(
            resolve(&rows, "a", "model.state"),
            Some("a.model.state".to_string())
        );
        assert_eq!(
            resolve(&rows, "a", "a.model.state"),
            Some("a.model.state".to_string())
        );
        assert_eq!(resolve(&rows, "a", "r"), Some("a.r".to_string()));
        assert_eq!(resolve(&rows, "a", "missing"), None);
    }

    /// Pinned (task 1.6): the pure wiring core over a hand-built report —
    /// satisfies edges collapse to owning files (dotted file ids split at
    /// the last dot), self-loops drop, cross-file instances aggregate per
    /// distinct pair sorted by the pair, and non-satisfies edges never
    /// leak into the view.
    #[test]
    fn wiring_projection_collapses_drops_and_aggregates() {
        let report = GraphReport {
            edges: vec![
                Edge {
                    from: "cons.c1".into(),
                    to: "prod.contract".into(),
                    kind: "constraints.satisfies".into(),
                },
                Edge {
                    from: "cons.c2".into(),
                    to: "prod.contract".into(),
                    kind: "constraints.satisfies".into(),
                },
                Edge {
                    from: "cons.c2".into(),
                    to: "cons.contract".into(),
                    kind: "constraints.satisfies".into(),
                },
                Edge {
                    from: "cons.c1".into(),
                    to: "cons".into(),
                    kind: "constraints.traces_to".into(),
                },
                Edge {
                    from: "extraction.claims.span".into(),
                    to: "prod.contract".into(),
                    kind: "constraints.satisfies".into(),
                },
            ],
            ..Default::default()
        };
        let kinds = BTreeMap::from([
            ("cons".to_string(), NodeKind::Intent),
            ("prod".to_string(), NodeKind::Intent),
            ("extraction.claims".to_string(), NodeKind::Intent),
        ]);
        let rows = wiring_projection(&report, &kinds);
        assert_eq!(
            rows.iter()
                .map(|r| (r.consumer.as_str(), r.producer.as_str(), r.instances))
                .collect::<Vec<_>>(),
            vec![("cons", "prod", 2), ("extraction.claims", "prod", 1),],
            "self-loop dropped, instances aggregated per distinct pair, dotted file id \
             owns its row (last-dot split), non-satisfies edges filtered"
        );
    }

    /// Pinned (task 1.6): an endpoint attributable to no known file falls
    /// back to itself rather than vanishing — the view never silently
    /// drops an edge it cannot file (D3).
    #[test]
    fn wiring_projection_unattributable_endpoint_falls_back_to_itself() {
        let report = GraphReport {
            edges: vec![Edge {
                from: "ghost.row".into(),
                to: "prod.contract".into(),
                kind: "constraints.satisfies".into(),
            }],
            ..Default::default()
        };
        let kinds = BTreeMap::from([("prod".to_string(), NodeKind::Intent)]);
        let rows = wiring_projection(&report, &kinds);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].consumer, "ghost.row");
        assert_eq!(rows[0].producer, "prod");
    }

    fn wiring_row(consumer: &str, producer: &str, instances: usize) -> WiringRow {
        WiringRow {
            consumer: consumer.into(),
            producer: producer.into(),
            instances,
        }
    }

    /// Pinned (task 1.6): the wiring dot renderer emits the specodelic-5qj
    /// exploration sample shape — `digraph wiring {`, box/rounded nodes,
    /// gray thin edge defaults, one `"consumer" -> "producer"
    /// [label="n satisfies"];` line per pair.
    #[test]
    fn render_wiring_dot_pins_sample_shape() {
        let rows = vec![wiring_row("cons", "prod", 2)];
        assert_eq!(
            render_wiring_dot(&rows),
            "digraph wiring {\n  rankdir=LR;\n  node [shape=box, style=rounded, fontsize=11];\n  edge [color=gray30, arrowsize=0.7];\n  \"cons\" -> \"prod\" [label=\"2 satisfies\"];\n}\n"
        );
    }

    /// Pinned (task 1.6): the wiring mermaid renderer — file nodes first
    /// (sorted, quoted labels), then the aggregated `<n> satisfies` links.
    #[test]
    fn render_wiring_mermaid_pins_exact_bytes() {
        let rows = vec![wiring_row("cons", "prod", 2)];
        assert_eq!(
            render_wiring_mermaid(&rows),
            "flowchart LR\n  cons[\"cons\"]\n  prod[\"prod\"]\n  cons -->|\"2 satisfies\"| prod\n"
        );
    }

    /// Pinned (task 1.6): an empty wiring view is labeled in every format
    /// — marker row in the TSV (violation-row convention), red dashed
    /// note node in dot, red violation node + classDef in mermaid — never
    /// silently clean.
    #[test]
    fn render_wiring_labels_empty_views() {
        let note = NO_WIRING_NOTE;
        assert_eq!(
            render_wiring_tsv(&[], &BTreeMap::new()),
            format!("\t\tno_wiring\t\t\t{note}\n")
        );
        let dot = render_wiring_dot(&[]);
        assert!(dot.contains(
            "\"no_wiring\" [shape=box, color=red, style=dashed, label=\"no_wiring: no constraints.satisfies edges — this corpus declares no inter-file wiring\"];"
        ));
        let mermaid = render_wiring_mermaid(&[]);
        assert!(mermaid
            .contains("no_wiring[\"no_wiring: no constraints.satisfies edges — this corpus declares no inter-file wiring\"]:::violation"));
        assert!(mermaid.contains("classDef violation stroke:red,stroke-dasharray:5 5"));
    }
}

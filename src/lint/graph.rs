//! Graph-shape beat of the linter (split from mod.rs — specodelic-g17
//! file_lines ratchet): edge resolution, no_self_ref, acyclic, and the
//! tiered single_root_reachable reachability, plus the shared Index.

use std::collections::{BTreeMap, BTreeSet};

use super::super::spec::Spec;
use super::{Issue, Report};

pub(crate) fn resolves_row(index: &Index, target: &str) -> bool {
    for split in [target.rsplit_once('.'), target.split_once('.')] {
        let Some((file_id, rest)) = split else {
            continue;
        };
        if rest == "model.state" || rest == "model.transition" {
            continue;
        }
        if let Some(rows) = index.files.get(file_id) {
            if rows.contains(rest) {
                return true;
            }
            // member of a row: file_id.row.member
            if let Some((row_id, _)) = rest.split_once('.')
                && rows.contains(row_id)
            {
                return true;
            }
        }
    }
    false
}

/// External completeness (specs/linter-external_completeness.md): diff
/// each declared `*.checklist.md` against the corpus. Optional and
/// non-gating — it runs only for repos that declare a checklist, never
/// touches any file's `linted` state, and its findings never block a
/// lifecycle stage (an orchestrator may choose to require the pass, a
/// policy layered on top, not a lifecycle fact). Findings are issues:
/// the checker's model ends in `failed`, not a warning.
pub(crate) fn lint_graph_shape(specs: &[Spec], report: &mut Report) {
    let g = graph_edges(specs);
    self_ref_findings(&g.edges, report);
    cycle_findings(&g.edges, report);
    reachability_findings(specs, &g, report);
}

/// The resolved edge material for the graph-shape checks: typed links
/// (source, target, field, column), structural transition→state edges
/// (connectivity only), and the intent-node set.
pub(crate) struct GraphEdges {
    edges: Vec<(String, String, String, String)>,
    conn_edges: Vec<(String, String)>,
    intent_nodes: BTreeSet<String>,
}

/// Resolve every spec's links against the corpus index into graph edges.
/// Skips metasyntactic targets and dangling refs (total_refs's beat), and
/// resolves bare-local rows in id:spec files (specodelic-15g Option A).
pub(crate) fn graph_edges(specs: &[Spec]) -> GraphEdges {
    let full = Index::build(specs);
    // (source node, target node, field, column) per resolved link.
    let mut edges: Vec<(String, String, String, String)> = vec![];
    // transition → state edges (structural: from/to are plain row ids,
    // not [[wiki-links]]) for the connectivity graph only.
    let mut conn_edges: Vec<(String, String)> = vec![];
    let mut intent_nodes: BTreeSet<String> = BTreeSet::new();
    for spec in specs {
        let scoped;
        let index = if spec.intent.id == "spec" {
            scoped = Index::build(std::slice::from_ref(spec));
            &scoped
        } else {
            &full
        };
        let file_id = spec.intent.id.clone();
        intent_nodes.insert(file_id.clone());
        for t in &spec.transitions {
            let t_node = format!("{file_id}.{}", t.id);
            // from/to resolve within the file's own states (validated
            // separately by every_transition_valid — a ghost endpoint
            // creates no edge here).
            for endpoint in [&t.from, &t.to] {
                if spec.states.iter().any(|s| s.id == *endpoint) {
                    conn_edges.push((t_node.clone(), format!("{file_id}.{endpoint}")));
                }
            }
        }
        for link in &spec.links {
            // Same skip rules as total_refs (specodelic-15g Option A:
            // bare-local rows in id:spec files resolve).
            let bare_local = file_id == "spec"
                && !link.target.contains('.')
                && index
                    .files
                    .get(&file_id)
                    .is_some_and(|rows| rows.contains(&link.target));
            if !bare_local && is_metasyntactic(&link.target, index) {
                continue;
            }
            let Some(target) = resolve_node(index, &file_id, &link.target) else {
                continue; // dangling — total_refs already reported it
            };
            // Frontmatter links anchor on the intent (no row source);
            // they join the connectivity graph but never the typed
            // acyclic edge set.
            let source = if link.source == file_id {
                file_id.clone()
            } else {
                format!("{file_id}.{}", link.source)
            };
            edges.push((
                source,
                target,
                link.field.to_string(),
                link.column.to_string(),
            ));
        }
    }
    GraphEdges {
        edges,
        conn_edges,
        intent_nodes,
    }
}

/// no_self_ref — a row referencing itself via traces_to or
/// derives_from traces to nothing that owns it
/// (specs/linter-graph_shape.md no_self_ref).
pub(crate) fn self_ref_findings(edges: &[(String, String, String, String)], report: &mut Report) {
    for (source, target, field, column) in edges {
        if ((*field == "constraints" && *column == "traces_to")
            || (*field == "properties" && *column == "derives_from"))
            && source == target
        {
            report.issues.push(Issue::new(
                "no_self_ref",
                source.clone(),
                format!(
                    "row `{source}` references itself via {column} — a self-tracing row has no owning purpose"
                ),
            ));
        }
    }
}

/// acyclic — the directed graph formed by traces_to ∪ derives_from ∪
/// guard-as-edge has no cycle (specs/linter-graph_shape.md). Self-loops
/// are `no_self_ref`'s beat and excluded here.
pub(crate) fn cycle_findings(edges: &[(String, String, String, String)], report: &mut Report) {
    let mut ref_edges: BTreeSet<(String, String)> = BTreeSet::new();
    for (source, target, field, column) in edges {
        let is_ref_edge = matches!(
            (field.as_str(), column.as_str()),
            ("constraints", "traces_to")
                | ("properties", "derives_from")
                | ("transitions", "guard")
        );
        if is_ref_edge && source != target {
            ref_edges.insert((source.clone(), target.clone()));
        }
    }
    for cycle in find_cycles(&ref_edges) {
        // Close the walk for display: the last element steps back to the first.
        let mut display = cycle.clone();
        if let Some(first) = cycle.first() {
            display.push(first.clone());
        }
        let path = display.join(" → ");
        report.issues.push(Issue::new(
            "acyclic",
            cycle[0].clone(),
            format!(
                "reference cycle: {path} (the traces_to/derives_from/guard graph must stay a DAG)"
            ),
        ));
    }
}

/// single_root_reachable — tiered own-file reachability
/// (specodelic.md Revision 10, HITL mp1 row 8): every row reaches the
/// file's OWN intent through own-file primary linkage — the edge set
/// is the file's own-file resolved references (traces_to,
/// derives_from, guard, satisfies, observes, emits, frontmatter) plus
/// the model's from/to edges, connectivity not outbound-only (an
/// outbound-only reading would flag every non-emitting state, which
/// no corpus satisfies). Cross-file typed edges (guard citations of
/// foreign constraints, satisfies, observes) are outbound leaves,
/// NEVER reachability paths — they cannot carry a row to an intent.
/// Tiered enforcement: a row with no own-file path whose component in
/// the FULL graph still contains some intent row is advisory (warnings
/// channel, exit 0 — its only ties are cross-file, possibly a
/// cross-feature reference filed under the wrong id); a row with no
/// path to ANY intent at all is an orphaned island and hard-fails.
pub(crate) fn reachability_findings(specs: &[Spec], g: &GraphEdges, report: &mut Report) {
    // node → owning file id, so own-file vs cross-file edges split
    // without re-parsing node names (intent nodes are bare file ids).
    let node_file = node_ownership(specs);
    let file_of = |node: &str| -> Option<String> { node_file.get(node).cloned() };
    // Own-file adjacency: both endpoints in the same file. The full
    // adjacency (any file) is kept for the advisory tier's "still
    // connected to SOME intent" escape hatch.
    let mut own_adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut link = |own: bool, a: String, b: String| {
        if a == b {
            return;
        }
        adj.entry(a.clone()).or_default().insert(b.clone());
        adj.entry(b.clone()).or_default().insert(a.clone());
        if own {
            own_adj.entry(a.clone()).or_default().insert(b.clone());
            own_adj.entry(b.clone()).or_default().insert(a.clone());
        }
    };
    for (source, target, _, _) in &g.edges {
        let own = file_of(source) == file_of(target) && file_of(source).is_some();
        link(own, source.clone(), target.clone());
    }
    for (a, b) in &g.conn_edges {
        // from/to are structural and always own-file by construction.
        link(true, a.clone(), b.clone());
    }
    let component_has_intent = |start: &str| -> bool {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut stack = vec![start.to_string()];
        seen.insert(start.to_string());
        while let Some(n) = stack.pop() {
            if g.intent_nodes.contains(&n) {
                return true;
            }
            for m in adj.get(&n).into_iter().flatten() {
                if seen.insert(m.clone()) {
                    stack.push(m.clone());
                }
            }
        }
        false
    };
    let mut advisory: Vec<(String, Vec<String>)> = vec![];
    let mut islands: Vec<(String, Vec<String>)> = vec![];
    for spec in specs {
        let file_id = spec.intent.id.clone();
        // BFS from the file's own intent over own-file edges only; any
        // row left unvisited has no own-file primary linkage.
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut stack = vec![file_id.clone()];
        seen.insert(file_id.clone());
        while let Some(n) = stack.pop() {
            for m in own_adj.get(&n).into_iter().flatten() {
                if seen.insert(m.clone()) {
                    stack.push(m.clone());
                }
            }
        }
        // This spec's rows only — never another same-id (id:spec)
        // file's rows (the file-scope self-containment law).
        let mut unanchored: Vec<String> = rows(spec)
            .iter()
            .map(|(_, r)| format!("{file_id}.{}", r.id))
            .chain(
                spec.transitions
                    .iter()
                    .map(|t| format!("{file_id}.{}", t.id)),
            )
            .filter(|r| !seen.contains(r.as_str()))
            .collect();
        if unanchored.is_empty() {
            continue;
        }
        unanchored.sort();
        // Per-row tier (Revision 10): a row still connected to SOME
        // intent through the full graph (its only ties are cross-file)
        // is advisory; a row connected to no intent at all hard-fails.
        let (adv, hard): (Vec<_>, Vec<_>) = unanchored
            .into_iter()
            .partition(|r| component_has_intent(r));
        if !adv.is_empty() {
            advisory.push((file_id.clone(), adv));
        }
        if !hard.is_empty() {
            islands.push((file_id, hard));
        }
    }
    emit_reachability(advisory, islands, report);
}

/// node → owning file id for every node in the reachability graph
/// (intent nodes are bare file ids; rows and transitions are
/// `file.row`-qualified).
pub(crate) fn node_ownership(specs: &[Spec]) -> BTreeMap<String, String> {
    let mut node_file: BTreeMap<String, String> = BTreeMap::new();
    for spec in specs {
        let file_id = spec.intent.id.clone();
        node_file.insert(file_id.clone(), file_id.clone());
        for r in rows(spec) {
            node_file.insert(format!("{file_id}.{}", r.1.id), file_id.clone());
        }
        for t in &spec.transitions {
            node_file.insert(format!("{file_id}.{}", t.id), file_id.clone());
        }
    }
    node_file
}

/// Emit the reachability tiers: cross-file-only rows are advisory
/// (warnings channel, never gating); rows with no path to ANY intent
/// hard-fail as orphaned islands.
pub(crate) fn emit_reachability(
    advisory: Vec<(String, Vec<String>)>,
    islands: Vec<(String, Vec<String>)>,
    report: &mut Report,
) {
    for (file, rows) in advisory {
        let shown = shown_rows(&rows);
        report.warnings.push(Issue::new(
            "single_root_reachable",
            file,
            format!(
                "{} row(s) have no own-file path to this file's intent row — their only ties are cross-file references (guard/satisfies/observes are outbound leaves, never reachability paths): {shown} — advisory: anchor them to this file's intent, or they may be filed under the wrong id",
                rows.len()
            ),
        ));
    }
    for (file, rows) in islands {
        let shown = shown_rows(&rows);
        report.issues.push(Issue::new(
            "single_root_reachable",
            file,
            format!(
                "{} row(s) unreachable from any intent row — an orphaned island (traces_to/derives_from/guard/from-to/emits): {shown}",
                rows.len()
            ),
        ));
    }
}

/// The first five rows of a reachability finding's row list, with an
/// "(and N more)" suffix when truncated.
pub(crate) fn shown_rows(rows: &[String]) -> String {
    let shown: Vec<String> = rows.iter().take(5).cloned().collect();
    let more = if rows.len() > shown.len() {
        format!(" (and {} more)", rows.len() - shown.len())
    } else {
        String::new()
    };
    let shown = shown.join(", ");
    format!("{shown}{more}")
}

/// Observability pass (specs/linter-observability.md): every effect
/// Constraint in the invocation's file set must be the target of ≥1
/// `observes` edge sourced at a *different* row — a row does not observe
/// itself. The finding is advisory: it lands in [`Report::warnings`]
/// (rendered onto the success envelope's warnings channel, exit 0),
/// never in `issues` — the Issue model carries no severity, and
/// [`Report::failures`] counts issues, so an issues-channel finding
/// would gate the run, exactly what the check's `advisory_severity`
/// forbids. Dangling `observes` targets resolve to no effect row and
/// stay `total_refs`' beat, so the two checks compose without
/// double-reporting the same row.
pub(crate) fn resolve_node(index: &Index, source_file: &str, target: &str) -> Option<String> {
    // Section anchors name no row — no edge.
    if target == "model.state" || target == "model.transition" {
        return None;
    }
    if index.files.contains_key(target) {
        return Some(target.to_string());
    }
    if !target.contains('.')
        && index
            .files
            .get(source_file)
            .is_some_and(|rows| rows.contains(target))
    {
        return Some(format!("{source_file}.{target}"));
    }
    // Every split point, last dot first (algorithm: specodelic-njh).
    // Section anchors name no row at ANY split point — no edge.
    let mut dots: Vec<usize> = target.match_indices('.').map(|(i, _)| i).collect();
    dots.reverse();
    for i in dots {
        let (file_id, rest) = (&target[..i], &target[i + 1..]);
        if rest == "model.state" || rest == "model.transition" {
            continue;
        }
        if let Some(rows) = index.files.get(file_id) {
            if rows.contains(rest) {
                return Some(target.to_string());
            }
            if let Some((row_id, _member)) = rest.split_once('.')
                && rows.contains(row_id)
            {
                return Some(format!("{file_id}.{row_id}"));
            }
        }
    }
    None
}

/// Rotation-normalized directed cycles over an edge set: each distinct
/// cycle is reported once, starting at its smallest node (the same
/// approach as the supersedes cycle finder in `crate::graph`).
pub(crate) fn find_cycles(edges: &BTreeSet<(String, String)>) -> Vec<Vec<String>> {
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (a, b) in edges {
        adj.entry(a.as_str()).or_default().push(b.as_str());
    }
    let mut found: Vec<Vec<String>> = vec![];
    let mut seen: BTreeSet<Vec<String>> = BTreeSet::new();
    let starts: Vec<&str> = adj.keys().copied().collect();
    for start in starts {
        let mut path: Vec<String> = vec![start.to_string()];
        let mut on_path: BTreeSet<String> = BTreeSet::from([start.to_string()]);
        dfs_cycles(
            start,
            start,
            &adj,
            &mut path,
            &mut on_path,
            &mut found,
            &mut seen,
        );
    }
    found
}

pub(crate) fn rows(spec: &Spec) -> Vec<(&'static str, &crate::spec::Row)> {
    let mut out: Vec<(&'static str, &crate::spec::Row)> = vec![];
    for r in &spec.constraints {
        out.push(("constraints", r));
    }
    for r in &spec.properties {
        out.push(("properties", r));
    }
    for r in &spec.states {
        out.push(("states", r));
    }
    out
}

/// Corpus-wide id index for reference resolution.
pub(crate) struct Index {
    /// file id -> set of local row ids
    pub(crate) files: BTreeMap<String, BTreeSet<String>>,
}

impl Index {
    /// Aggregate row sets per file id: several files may legally share an
    /// id (openspec naming law forces every dual-format file to be
    /// `spec.md` → `id: spec`), so same-id row sets merge instead of
    /// overwriting (the #37 bug). How the merged set is USED depends on
    /// the caller: `lint_references` resolves non-`spec` ids corpus-wide
    /// but scopes `id: spec` files to their own rows (self-contained
    /// deltas — #42).
    pub(crate) fn build(specs: &[Spec]) -> Index {
        let mut files: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for spec in specs {
            files
                .entry(spec.intent.id.clone())
                .or_default()
                .extend(spec.defined_ids());
        }
        Index { files }
    }

    /// Resolve a link target: a file id, `file_id.row_id`, a
    /// `model.state`/`model.transition` section anchor (bare or
    /// `file_id.model.…`), or a member of a row (`file_id.row.member`).
    /// `source_file` enables the bare-local row arm (specodelic-15g,
    /// Option A): the source file's own rows resolve in bare spelling —
    /// only id:spec files reach this arm bare (the metasyntactic skip
    /// masks bare targets elsewhere); the scoped index makes it the
    /// file's own rows.
    ///
    /// Algorithm (specodelic-njh, gh#1): try every split point from the
    /// LAST dot to the FIRST; at each, `file_id` must be a known file and
    /// the remainder must be a row, a section anchor, or
    /// `row_id.member` (row = first segment of the remainder, so dotted
    /// row ids work). Last-dot wins: a dotted file id is the common case.
    pub(crate) fn resolves(&self, source_file: &str, target: &str) -> bool {
        // Bare section anchors.
        if target == "model.state" || target == "model.transition" {
            return true;
        }
        // Exact file id.
        if self.files.contains_key(target) {
            return true;
        }
        // Bare-local row: the source file's own rows in bare spelling
        // (canonical `file_id.row_id` is the dotted form's job below).
        if !target.contains('.')
            && self
                .files
                .get(source_file)
                .is_some_and(|rows| rows.contains(target))
        {
            return true;
        }
        // Every split point, last dot first: dotted file ids are the
        // common case, dotted row ids still resolve via the member arm.
        let mut dots: Vec<usize> = target.match_indices('.').map(|(i, _)| i).collect();
        dots.reverse();
        for i in dots {
            let (file_id, rest) = (&target[..i], &target[i + 1..]);
            if let Some(rows) = self.files.get(file_id) {
                if rows.contains(rest) || rest == "model.state" || rest == "model.transition" {
                    return true;
                }
                // member of a row: file_id.row.member (row = first
                // segment of rest, so dotted row ids resolve too).
                if let Some((row_id, _member)) = rest.split_once('.')
                    && rows.contains(row_id)
                {
                    return true;
                }
            }
        }
        false
    }
}

/// Metasyntactic link targets — format documentation inside expr cells
/// (`∀ ref ∈ file: resolves(ref) — no dangling [[...]]`, `[[old_id]]`,
/// `[[x]]`). Real cross-file references are file ids (single segment, must
/// resolve) or row references (always dotted). A single-segment target that
/// is not a file id is therefore metasyntactic, and so is pure ellipsis.
pub(crate) fn is_metasyntactic(target: &str, index: &Index) -> bool {
    if target == "..." || target == "…" {
        return true;
    }
    !target.contains('.') && !index.files.contains_key(target)
}

/// total_refs — every structured-field `[[link]]` resolves somewhere in the
/// corpus.
pub(crate) fn lint_references(specs: &[Spec], report: &mut Report) {
    let full = Index::build(specs);
    for spec in specs {
        // Dual-format self-containment (spec-integration law): `id: spec`
        // files resolve against their OWN rows only — the corpus-wide
        // union would silently false-resolve any ref that collides with
        // a row in another dual-format file. Other file ids keep
        // corpus-wide resolution.
        let scoped;
        let index = if spec.intent.id == "spec" {
            scoped = Index::build(std::slice::from_ref(spec));
            &scoped
        } else {
            &full
        };
        let file = spec
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<{}>", spec.intent.id));
        for link in &spec.links {
            // Bare-local rows (specodelic-15g, Option A): in an id:spec
            // file a dotless target naming one of the file's own rows has
            // exactly one possible meaning — the local row — so it
            // resolves instead of vanishing into the metasyntactic skip.
            // Dotful spellings keep the skip/hint (ambiguous with
            // `file.row`); other files keep corpus-wide behavior.
            let bare_local = spec.intent.id == "spec"
                && !link.target.contains('.')
                && index
                    .files
                    .get(&spec.intent.id)
                    .is_some_and(|rows| rows.contains(&link.target));
            // Dotless unknown targets skip as metasyntactic (e.g. `[[id]]`
            // used as format documentation).
            if !bare_local && is_metasyntactic(&link.target, index) {
                continue;
            }
            if !index.resolves(&spec.intent.id, &link.target) {
                let mut msg = format!(
                    "dangling reference `[[{}]]` from {}{} — target not defined in any spec file",
                    link.target,
                    link.field,
                    if link.column.is_empty() {
                        String::new()
                    } else {
                        format!(".{}", link.column)
                    }
                );
                // gh#5: when the unresolved target names a row that lives
                // in this very file, the fix is local — say so and show
                // the file-qualified form instead of sending the author
                // corpus-hunting.
                if let Some(rows) = index.files.get(&spec.intent.id)
                    && rows.contains(&link.target)
                {
                    msg.push_str(&format!(
                        " — hint: row `{}` is defined in this file; refs must be file-qualified: `[[{}.{}]]`",
                        link.target, spec.intent.id, link.target
                    ));
                }
                report
                    .issues
                    .push(Issue::new("total_refs", file.clone(), msg));
            }
        }
    }
}

/// coverage — every constraint has a deriving property.
/// The row kind a resolved derives_from target landed on (rk3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowKind {
    Constraint,
    Property,
    LawProperty,
}

pub(crate) fn dfs_cycles(
    start: &str,
    current: &str,
    adj: &BTreeMap<&str, Vec<&str>>,
    path: &mut Vec<String>,
    on_path: &mut BTreeSet<String>,
    found: &mut Vec<Vec<String>>,
    seen: &mut BTreeSet<Vec<String>>,
) {
    let Some(nexts) = adj.get(current) else {
        return;
    };
    for &next in nexts {
        if next == start {
            // Canonicalize: rotate so the smallest member leads — the
            // same cycle is discovered from each of its members
            // (graph.rs rotation pattern). The walk holds distinct
            // nodes; the closing step back to `start` is implied. The
            // live DFS path is left untouched.
            let mut cycle = path.clone();
            if let Some(pos) = cycle
                .iter()
                .position(|p| p == &cycle.iter().min().cloned().unwrap_or_default())
            {
                cycle.rotate_left(pos);
            }
            if seen.insert(cycle.clone()) {
                found.push(cycle);
            }
            continue;
        }
        if on_path.contains(next) {
            continue; // an inner cycle is found from its own smallest node
        }
        path.push(next.to_string());
        on_path.insert(next.to_string());
        dfs_cycles(start, next, adj, path, on_path, found, seen);
        path.pop();
        on_path.remove(next);
    }
}

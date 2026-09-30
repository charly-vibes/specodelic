//! The Specodelic linter — per-file invariants plus corpus-wide reference
//! resolution.
//!
//! Purpose: mechanize the invariants in `specs/specodelic.md` and its
//! checker files (`linter-*.md`). Responsibilities: check frontmatter
//! validity, the filename↔id mapping, id uniqueness, guard presence, the
//! EARS grammar, coverage, and total reference resolution. Rationale: only
//! structured fields (frontmatter + fixed-schema tables) are ever
//! inspected — `prose_untouched` is itself one of the invariants.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::checklist::{self, Checklist};
use crate::ears;
use crate::guide;
use crate::spec::Spec;

/// One lint finding.
///
/// Self-describing (add-embedded-aix-guide task 3.x): every finding
/// carries the stable `linter.<name>` rule id and a one-line semantics
/// string from [`RULE_TABLE`], in both JSON and human output.
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    /// Stable rule identifier: `linter.<name>` (review CORR: the bare
    /// `rule` field was dropped — `rule_id` is the single naming).
    pub rule_id: String,
    /// One-line semantics: what the rule requires (from [`RULE_TABLE`]).
    pub rule_semantics: String,
    /// File the finding is anchored to.
    pub file: String,
    /// Human-readable message with a suggested fix.
    pub message: String,
}

impl Issue {
    /// Build a finding from the bare rule name. Panics on a rule name
    /// that is not in [`RULE_TABLE`] — a rule the catalog cannot describe
    /// must not exist (task 3.3's coverage guarantee).
    pub fn new(rule: &str, file: impl Into<String>, message: impl Into<String>) -> Issue {
        let semantics = rule_semantics(rule)
            .unwrap_or_else(|| panic!("rule `{rule}` is not in lint::RULE_TABLE"));
        Issue {
            rule_id: rule_id(rule),
            rule_semantics: semantics.to_string(),
            file: file.into(),
            message: message.into(),
        }
    }
}

/// The lint rule catalog: `(bare rule name, one-line semantics)` pairs.
/// Single source of truth for the emitted `rule_id`/`rule_semantics`
/// fields, the `explain lint-rules` topic body, and the fixture-coverage
/// unit tests. Append-only — never rename or remove (rule ids are
/// stable identifiers agents rely on).
pub const RULE_TABLE: &[(&str, &str)] = &[
    (
        "frontmatter_valid",
        "frontmatter `kind` must be `intent` — the only top-level intent kind",
    ),
    (
        "id_matches_file",
        "frontmatter `id` must equal the filename stem with `-` mapped to `.` (`_` is literal)",
    ),
    (
        "unique_id",
        "every row id in a file must be unique across all of the file's tables",
    ),
    (
        "guard_required",
        "every transition must carry a non-null guard that cites an invariant Constraint",
    ),
    (
        "model_present",
        "the Model section must contain both a States list and a Transitions table (empty-but-present beats absent)",
    ),
    (
        "ears_syntax",
        "the intent statement must contain an imperative `SHALL` and match one of the five EARS patterns",
    ),
    (
        "no_conjoined_id",
        "an id must not encode two capabilities joined by `and`/`or`",
    ),
    (
        "no_universal_in_id",
        "an id must not contain a universal token (all/every/any/always/never)",
    ),
    (
        "total_refs",
        "every structured-field [[link]] must resolve to a definition somewhere in the corpus — dual-format `id: spec` files are self-contained: their refs must resolve within the file itself",
    ),
    (
        "coverage",
        "every constraint must have a deriving property (`∃ property.derives_from == <constraint>`)",
    ),
    (
        "no_orphan_property",
        "every property must derive from at least one constraint",
    ),
    (
        "requirement_drift",
        "a dual-format file's ## Requirements mirror must hold the same requirement text as ## ADDED Requirements (blank lines and trailing space ignored)",
    ),
    (
        "dual_format_valid",
        "a file carrying `## ADDED Requirements` must be a dual-format file — declare `id: spec` and pair it with a sibling `## Requirements` section",
    ),
    (
        "every_state_used",
        "every declared state must appear as from or to in at least one transition — a state no transition reaches is machinery the model can never enter or leave",
    ),
    (
        "every_transition_valid",
        "every transition's from and to must name states declared in the same file's States section",
    ),
    (
        "no_self_ref",
        "a row must not reference itself via traces_to or derives_from — a self-tracing row has no owning purpose",
    ),
    (
        "acyclic",
        "the directed graph formed by traces_to ∪ derives_from ∪ guard-as-edge must contain no cycle (a reference cycle has no derivation order)",
    ),
    (
        "single_root_reachable",
        "every constraint/property/state/transition row must be connected to some intent row through the reference graph (traces_to, derives_from, guard, from/to, emits) — no orphaned islands",
    ),
    (
        "observability",
        "every effect Constraint must be the target of ≥1 `observes` reference from a different row — advisory: warned on the warnings channel (exit 0), never a failure",
    ),
    (
        "checklist_well_formed",
        "a declared checklist manifest (`*.checklist.md`) must be a flat item list with stable ids plus a mapping table with exactly item/status/mapped_ids/rationale columns — a manifest the linter cannot read is a checklist going silently unconsulted",
    ),
    (
        "every_item_accounted",
        "every checklist item must have exactly one mapping row with status `covered` or `waived` — an unconsulted item is the failure this checker exists to prevent",
    ),
    (
        "covered_maps_resolve",
        "a `covered` mapping row must name a non-empty mapped_ids list whose ids resolve to real constraint or property rows — a claim resting on nothing is not a claim",
    ),
    (
        "waiver_has_rationale",
        "a `waived` mapping row must carry non-empty rationale prose — an unexplained waiver is an unconsulted item with extra steps",
    ),
    (
        "no_duplicate_claim",
        "no two mapping rows may target the same checklist item — one claim per item, on the record",
    ),
];

/// The stable rule identifier for a bare rule name: `linter.<name>`.
pub fn rule_id(rule: &str) -> String {
    format!("linter.{rule}")
}

/// Look up a rule's one-line semantics by bare rule name.
pub fn rule_semantics(rule: &str) -> Option<&'static str> {
    RULE_TABLE
        .iter()
        .find(|(name, _)| *name == rule)
        .map(|(_, semantics)| *semantics)
}

/// Corpus lint report.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Report {
    pub files_linted: usize,
    pub issues: Vec<Issue>,
    /// Advisory findings (specs/linter-observability.md): rendered onto
    /// the success envelope's warnings channel, exit 0 — never counted
    /// by [`Report::failures`]. The `Issue` model carries no severity,
    /// so advisory findings must not ride the issues channel.
    pub warnings: Vec<Issue>,
}

impl Report {
    pub fn failures(&self) -> usize {
        self.issues.len()
    }
}

/// The non-spec files the corpus intentionally exempts from frontmatter.
pub const EXEMPT_FILES: &[&str] = &[
    "AGENTS.md",
    "STATUS.md",
    "USAGE.md",
    "CHANGELOG.md",
    "theory.md",
];

/// Lint a whole corpus (all specs together) and return the report.
pub fn lint_corpus(specs: &[Spec]) -> Report {
    let mut report = Report {
        files_linted: specs.len(),
        issues: vec![],
        warnings: vec![],
    };
    for spec in specs {
        lint_one(spec, &mut report);
    }
    lint_references(specs, &mut report);
    lint_coverage(specs, &mut report);
    lint_graph_shape(specs, &mut report);
    lint_observability(specs, &mut report);
    report
}

/// The full pass: internal-consistency checks over the specs plus the
/// external-completeness check over any declared `*.checklist.md`
/// manifests. The CLI lint command runs this; `lint_corpus` alone stays
/// checklist-free for callers whose gate must not see the optional
/// checker (compile's precondition — external_completeness never gates
/// a stage, specs/linter-external_completeness.md Notes).
pub fn lint_all(specs: &[Spec], checklists: &[Checklist]) -> Report {
    let mut report = lint_corpus(specs);
    lint_checklists(specs, checklists, &mut report);
    report
}

/// Can `target` resolve to a CONSTRAINT or PROPERTY row? Stricter than
/// [`Index::resolves`]: a bare file id or a `model.state`/`model.transition`
/// section anchor is machinery, not a claim to rest a checklist item on.
fn resolves_row(index: &Index, target: &str) -> bool {
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
pub fn lint_checklists(specs: &[Spec], checklists: &[Checklist], report: &mut Report) {
    if checklists.is_empty() {
        return; // not_applicable — nothing external to be incomplete against
    }
    let index = Index::build(specs);
    for cl in checklists {
        let file = cl.path.display().to_string();
        // checklist_well_formed — the parser's defect list, verbatim.
        for defect in &cl.defects {
            report
                .issues
                .push(Issue::new("checklist_well_formed", &file, defect.clone()));
        }
        // Group rows by claimed item for the exactly-one checks.
        let mut by_item: BTreeMap<&str, Vec<&checklist::MappingRow>> = BTreeMap::new();
        for row in &cl.mapping {
            by_item.entry(row.item.as_str()).or_default().push(row);
        }
        for item in &cl.items {
            match by_item.get(item.id.as_str()).map(Vec::as_slice) {
                // every_item_accounted: ∃ exactly one row with a legal
                // status. Zero rows → unconsulted item. One row with a
                // status outside {covered, waived} → same failure.
                None => report.issues.push(Issue::new(
                    "every_item_accounted",
                    &file,
                    format!(
                        "checklist item `{}` has no mapping row — nothing was consulted for it",
                        item.id
                    ),
                )),
                Some([row]) if row.status != "covered" && row.status != "waived" => {
                    report.issues.push(Issue::new(
                        "every_item_accounted",
                        &file,
                        format!(
                            "mapping row for item `{}` has status `{}` — must be `covered` or `waived`",
                            item.id, row.status
                        ),
                    ));
                }
                Some([row]) => {
                    if row.status == "covered" {
                        // covered_maps_resolve: non-empty, row-resolving ids.
                        if row.mapped_ids.is_empty() {
                            report.issues.push(Issue::new(
                                "covered_maps_resolve",
                                &file,
                                format!(
                                    "covered mapping row for item `{}` has an empty mapped_ids list",
                                    item.id
                                ),
                            ));
                        }
                        for id in &row.mapped_ids {
                            if !resolves_row(&index, id) {
                                report.issues.push(Issue::new(
                                    "covered_maps_resolve",
                                    &file,
                                    format!(
                                        "covered mapping row for item `{}` maps to `{id}`, which does not resolve to a constraint or property row",
                                        item.id
                                    ),
                                ));
                            }
                        }
                    } else {
                        // waiver_has_rationale: non-empty rationale prose.
                        if row.rationale.trim().is_empty() {
                            report.issues.push(Issue::new(
                                "waiver_has_rationale",
                                &file,
                                format!(
                                    "waived mapping row for item `{}` carries no rationale",
                                    item.id
                                ),
                            ));
                        }
                    }
                }
                // >1 rows: no_duplicate_claim's beat — do NOT also fire
                // every_item_accounted (the item IS claimed, just twice).
                Some(_) => {
                    report.issues.push(Issue::new(
                        "no_duplicate_claim",
                        &file,
                        format!(
                            "{} mapping rows target checklist item `{}` — one claim per item",
                            by_item[item.id.as_str()].len(),
                            item.id
                        ),
                    ));
                }
            }
        }
    }
}

/// Corpus-wide graph-shape pass (specs/linter-graph_shape.md): build the
/// resolved reference graph once, then check `no_self_ref`, `acyclic`
/// (over traces_to ∪ derives_from ∪ guard-as-edge — a self-loop is
/// `no_self_ref`'s beat, never double-reported as a cycle), and
/// `single_root_reachable` (every row connected to some intent row
/// through the reference graph: traces_to, derives_from, guard,
/// from/to, emits). Resolution reuses [`Index`] with the same
/// metasyntactic skip as [`lint_references`]; dangling targets are
/// `total_refs`'s job, not ours — the two checks compose without
/// double-reporting the same row.
fn lint_graph_shape(specs: &[Spec], report: &mut Report) {
    let full = Index::build(specs);
    // (source node, target node, field, column) per resolved link.
    let mut edges: Vec<(String, String, &str, &str)> = vec![];
    // transition → state edges (structural: from/to are plain row ids,
    // not [[wiki-links]]) for the connectivity graph only.
    let mut conn_edges: Vec<(String, String)> = vec![];
    let mut intent_nodes: BTreeSet<String> = BTreeSet::new();
    let mut all_rows: BTreeSet<String> = BTreeSet::new();
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
        for r in rows(spec) {
            all_rows.insert(format!("{file_id}.{}", r.1.id));
        }
        for t in &spec.transitions {
            let t_node = format!("{file_id}.{}", t.id);
            all_rows.insert(t_node.clone());
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
            edges.push((source, target, link.field.as_str(), link.column.as_str()));
        }
    }

    // no_self_ref — a row referencing itself via traces_to or
    // derives_from traces to nothing that owns it
    // (specs/linter-graph_shape.md no_self_ref).
    for (source, target, field, column) in &edges {
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

    // acyclic — the directed graph formed by traces_to ∪ derives_from ∪
    // guard-as-edge has no cycle (specs/linter-graph_shape.md). Self-loops
    // are `no_self_ref`'s beat and excluded here.
    let mut ref_edges: BTreeSet<(String, String)> = BTreeSet::new();
    for (source, target, field, column) in &edges {
        let is_ref_edge = matches!(
            (*field, *column),
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

    // single_root_reachable — undirected connectivity: every row's
    // component contains some intent row. The edge set is the full
    // resolved reference graph (any typed column, incl. frontmatter)
    // plus the model's own from/to edges — a state reaches its intent
    // through the transitions that reference it, so the check is
    // connectivity, not outbound-only reachability (an outbound-only
    // reading would flag every state that does not emit, which no
    // corpus satisfies). Which intents count (any vs the file's own)
    // is specodelic-mp1 row 8's open question — this implements the
    // checker spec text as written ("reachable(row, some intent row)").
    let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (source, target, _, _) in &edges {
        if source != target {
            adj.entry(source.clone())
                .or_default()
                .insert(target.clone());
            adj.entry(target.clone())
                .or_default()
                .insert(source.clone());
        }
    }
    for (a, b) in &conn_edges {
        adj.entry(a.clone()).or_default().insert(b.clone());
        adj.entry(b.clone()).or_default().insert(a.clone());
    }
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut islands: Vec<(String, Vec<String>)> = vec![];
    for node in all_rows.iter().cloned().collect::<Vec<_>>() {
        if visited.contains(&node) {
            continue;
        }
        let mut comp: Vec<String> = vec![];
        let mut stack = vec![node.clone()];
        visited.insert(node.clone());
        while let Some(n) = stack.pop() {
            comp.push(n.clone());
            for m in adj.get(&n).into_iter().flatten() {
                if visited.insert(m.clone()) {
                    stack.push(m.clone());
                }
            }
        }
        if !comp.iter().any(|n| intent_nodes.contains(n)) {
            let anchor = comp.iter().min().cloned().unwrap_or_default();
            comp.sort();
            islands.push((anchor, comp));
        }
    }
    for (_, rows) in islands {
        let shown: Vec<String> = rows.iter().take(5).cloned().collect();
        let more = if rows.len() > shown.len() {
            format!(" (and {} more)", rows.len() - shown.len())
        } else {
            String::new()
        };
        let shown = shown.join(", ");
        let file = rows[0]
            .rsplit_once('.')
            .map(|(f, _)| f.to_string())
            .unwrap_or_else(|| rows[0].clone());
        report.issues.push(Issue::new(
            "single_root_reachable",
            file,
            format!(
                "{} row(s) unreachable from any intent row — an orphaned island (traces_to/derives_from/guard/from-to/emits): {shown}{more}",
                rows.len()
            ),
        ));
    }
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
fn lint_observability(specs: &[Spec], report: &mut Report) {
    let mut observed: BTreeSet<String> = BTreeSet::new();
    for spec in specs {
        let file_id = spec.intent.id.clone();
        for link in &spec.links {
            if link.column != "observes" {
                continue;
            }
            // Resolve the target to a row key within this invocation:
            // dotted `file.row` as written; a dotless target is bare-local
            // and only id:spec files resolve their own rows bare
            // (specodelic-15g Option A) — anything else is metasyntactic
            // here, exactly as in [`lint_graph_shape`].
            let key = if link.target.contains('.') {
                link.target.clone()
            } else if file_id == "spec" {
                format!("{file_id}.{}", link.target)
            } else {
                continue;
            };
            let source = format!("{file_id}.{}", link.source);
            // Self-observation is vacuous (the checker spec's
            // self_observation_not_counted) — it never counts.
            if key != source {
                observed.insert(key);
            }
        }
    }
    for spec in specs {
        let file_id = spec.intent.id.clone();
        for r in &spec.constraints {
            if r.kind.as_deref() != Some("effect") {
                continue;
            }
            let key = format!("{file_id}.{}", r.id);
            if observed.contains(&key) {
                continue;
            }
            report.warnings.push(Issue::new(
                "observability",
                file_id.clone(),
                format!(
                    "effect `{key}` has no `observes` reference targeting it — a declared output nobody observes (advisory, exit 0) [hint: add an `observes` column on the row that consumes this output, or remove the effect if it is unintentional]"
                ),
            ));
        }
    }
}

/// Resolve a link target to its canonical graph node: the file id for an
/// intent target, `file_id.row_id` for a row (a `file.row.member` anchor
/// resolves to the row). Mirrors [`Index::resolves`]'s arms; keep in sync.
fn resolve_node(index: &Index, source_file: &str, target: &str) -> Option<String> {
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
    if let Some((file_id, rest)) = target.rsplit_once('.') {
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
        if let Some((file_id, rest)) = target.split_once('.')
            && let Some(rows) = index.files.get(file_id)
            && rows.contains(rest)
        {
            return Some(target.to_string());
        }
    }
    None
}

/// Rotation-normalized directed cycles over an edge set: each distinct
/// cycle is reported once, starting at its smallest node (the same
/// approach as the supersedes cycle finder in `crate::graph`).
fn find_cycles(edges: &BTreeSet<(String, String)>) -> Vec<Vec<String>> {
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

fn dfs_cycles(
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

/// Per-file invariants.
fn lint_one(spec: &Spec, report: &mut Report) {
    let file = spec
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| format!("<{}>", spec.intent.id));

    // frontmatter_valid — kind must be in the closed intent-kind set
    // (parse already required the fields).
    if !guide::INTENT_KINDS.contains(&spec.intent.kind.as_str()) {
        report.issues.push(Issue::new(
            "frontmatter_valid",
            file.clone(),
            format!(
                "frontmatter kind is `{}`, must be `intent`",
                spec.intent.kind
            ),
        ));
    }

    // id_matches_file — frontmatter.id == replace(stem(path), "-", ".").
    if let Some(path) = &spec.path {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let expected = stem.replace('-', ".");
        if spec.intent.id != expected {
            report.issues.push(Issue::new("id_matches_file", file.clone(), format!(
                    "frontmatter id `{}` does not match filename (expected `{}` — `-` in filename maps to `.` in id; `_` is literal)",
                    spec.intent.id, expected
                )));
        }
    }

    // dual_format_valid — a file carrying `## ADDED Requirements` (the
    // openspec delta half) must be a dual-format file: declare `id: spec`
    // (openspec hard-requires the spec.md filename) and pair the ADDED
    // section with a sibling `## Requirements` section (the capability
    // half that survives archiving). Plain corpus specs (no ADDED
    // section) are exempt.
    if spec.has_added_requirements {
        if spec.intent.id != "spec" {
            report.issues.push(Issue::new(
                "dual_format_valid",
                file.clone(),
                format!(
                    "file carries `## ADDED Requirements` but declares id `{}` — dual-format files must declare `id: spec` (openspec requires the spec.md filename)",
                    spec.intent.id
                ),
            ));
        }
        if !spec.has_requirements_section {
            report.issues.push(Issue::new(
                "dual_format_valid",
                file.clone(),
                "file carries `## ADDED Requirements` without a sibling `## Requirements` section — not a dual-format file (the capability half is missing; migrate per the recipe in openspec/project.md: mirror the requirement content into ## Requirements, keep the specodelic tables alongside, then gates: spk lint + openspec validate + scripts/check_section_sync.py for drift)".to_string(),
            ));
        }
    }

    // requirement_drift — when both halves of a dual-format file are
    // present, the mirror must match (gh#4: the migration recipe has
    // agents hand-create the mirror, so drift is easy and was previously
    // only caught by this repo's local section-sync script, never by
    // `spk lint`). Normalization mirrors scripts/check_section_sync.py:
    // per-line trailing space and blank lines are ignored.
    if spec.has_added_requirements && spec.has_requirements_section {
        let norm = |body: &str| -> String {
            body.lines()
                .map(str::trim_end)
                .filter(|l| !l.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        };
        if norm(&spec.added_requirements_body) != norm(&spec.requirements_body) {
            report.issues.push(Issue::new(
                "requirement_drift",
                file.clone(),
                "## Requirements does not match ## ADDED Requirements — the mirror must hold identical requirement text (blank lines and trailing space ignored); regenerate it from the ADDED section".to_string(),
            ));
        }
    }

    // unique_id — every row id in the file is unique.
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for (table, row) in rows(spec) {
        if let Some(first_table) = seen.insert(row.id.as_str(), table) {
            report.issues.push(Issue::new(
                "unique_id",
                file.clone(),
                format!("duplicate id `{}` (in {first_table} and {table})", row.id),
            ));
        }
    }

    // guard_required — every transition's guard is present.
    for t in &spec.transitions {
        if t.guard.is_none() {
            report.issues.push(Issue::new(
                "guard_required",
                file.clone(),
                format!(
                    "transition `{}` has an empty guard — every guard must be non-null",
                    t.id
                ),
            ));
        }
    }

    // model_present — States list and Transitions table both exist.
    if spec.states.is_empty() || spec.transitions.is_empty() {
        report.issues.push(Issue::new("model_present", file.clone(), format!(
                "Model section incomplete: {} states, {} transitions — both sections must exist (empty-but-present beats absent)",
                spec.states.len(),
                spec.transitions.len()
            )));
    }

    // every_state_used — a declared state no transition reaches is
    // machinery the model can never enter or leave (specs/
    // linter-model_shape.md).
    let state_ids: BTreeSet<&str> = spec.states.iter().map(|s| s.id.as_str()).collect();
    let used: BTreeSet<&str> = spec
        .transitions
        .iter()
        .flat_map(|t| [t.from.as_str(), t.to.as_str()])
        .collect();
    for sid in &spec.states {
        if !spec.transitions.is_empty() && !used.contains(sid.id.as_str()) {
            report.issues.push(Issue::new(
                "every_state_used",
                file.clone(),
                format!(
                    "state `{}` is declared but no transition enters or leaves it — every state must appear as from or to in at least one transition",
                    sid.id
                ),
            ));
        }
    }

    // every_transition_valid — from/to must name declared states; a
    // dangling `to` makes the model-checker simulate a different graph
    // than the author wrote.
    for t in &spec.transitions {
        for (label, endpoint) in [("from", &t.from), ("to", &t.to)] {
            if !state_ids.contains(endpoint.as_str()) {
                report.issues.push(Issue::new(
                    "every_transition_valid",
                    file.clone(),
                    format!(
                        "transition `{}` has {label} `{}` — not a declared state (states: {:?})",
                        t.id,
                        endpoint,
                        state_ids.iter().copied().collect::<Vec<_>>()
                    ),
                ));
            }
        }
    }

    // ears_syntax — the intent statement matches one of the 5 EARS patterns.
    if ears::classify(&spec.intent.statement).is_none() {
        let hint = if !ears::has_shall(&spec.intent.statement) {
            "no imperative `SHALL` found (`has_shall`)"
        } else {
            "does not match any of: Ubiquitous / Event-Driven / State-Driven / Unwanted-Behavior / Optional-Feature"
        };
        report.issues.push(Issue::new(
            "ears_syntax",
            file.clone(),
            format!("intent statement {hint}"),
        ));
    }

    // no_conjoined_id / no_universal_in_id — checked on the id token.
    // Scope: capability ids (the Intent id). The rules' own examples are
    // intent-style ids (`order.cancel_and_refund`, `order.always_validate`);
    // checker row ids like `every_state_used` describe checks, not
    // capabilities, and the corpus relies on that distinction.
    {
        let (label, id) = ("intent", spec.intent.id.as_str());
        let tokens: Vec<String> = id
            .split(['.', '_', '-'])
            .map(|t| t.to_lowercase())
            .collect();
        if tokens.iter().any(|t| t == "and" || t == "or") {
            report.issues.push(Issue::new(
                "no_conjoined_id",
                file.clone(),
                format!("{label} id `{id}` encodes two capabilities joined by and/or"),
            ));
        }
        if tokens
            .iter()
            .any(|t| matches!(t.as_str(), "all" | "every" | "any" | "always" | "never"))
        {
            report.issues.push(Issue::new(
                "no_universal_in_id",
                file.clone(),
                format!(
                    "{label} id `{id}` contains a universal token (all/every/any/always/never)"
                ),
            ));
        }
    }
}

/// All rows in a file as (table-name, row) pairs.
fn rows(spec: &Spec) -> Vec<(&'static str, &crate::spec::Row)> {
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
struct Index {
    /// file id -> set of local row ids
    files: BTreeMap<String, BTreeSet<String>>,
}

impl Index {
    /// Aggregate row sets per file id: several files may legally share an
    /// id (openspec naming law forces every dual-format file to be
    /// `spec.md` → `id: spec`), so same-id row sets merge instead of
    /// overwriting (the #37 bug). How the merged set is USED depends on
    /// the caller: `lint_references` resolves non-`spec` ids corpus-wide
    /// but scopes `id: spec` files to their own rows (self-contained
    /// deltas — #42).
    fn build(specs: &[Spec]) -> Index {
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
    fn resolves(&self, source_file: &str, target: &str) -> bool {
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
        // file_id + "." + rest (split at the LAST dot so file ids with dots,
        // e.g. `linter.frontmatter`, resolve too).
        if let Some((file_id, rest)) = target.rsplit_once('.') {
            if let Some(rows) = self.files.get(file_id) {
                if rows.contains(rest) || rest == "model.state" || rest == "model.transition" {
                    return true;
                }
                // member of a row: file_id.row.member
                if let Some((row_id, _member)) = rest.split_once('.')
                    && rows.contains(row_id)
                {
                    return true;
                }
            }
            // The file id itself might carry the dot (e.g. target
            // `linter.frontmatter.has_id` → file `linter.frontmatter`,
            // row `has_id`) — handled above. But a target like
            // `specodelic.model.state` splits to (`specodelic.model`,
            // `state`) first; retry with the first dot as the split point
            // only when the last-dot split found nothing.
            if let Some((file_id, rest)) = target.split_once('.')
                && let Some(rows) = self.files.get(file_id)
                && (rows.contains(rest) || rest == "model.state" || rest == "model.transition")
            {
                return true;
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
fn is_metasyntactic(target: &str, index: &Index) -> bool {
    if target == "..." || target == "…" {
        return true;
    }
    !target.contains('.') && !index.files.contains_key(target)
}

/// total_refs — every structured-field `[[link]]` resolves somewhere in the
/// corpus.
fn lint_references(specs: &[Spec], report: &mut Report) {
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
fn lint_coverage(specs: &[Spec], report: &mut Report) {
    for spec in specs {
        let file_id = &spec.intent.id;
        // Constraint ids this file defines (local row ids).
        let constraint_ids: BTreeSet<&str> =
            spec.constraints.iter().map(|c| c.id.as_str()).collect();
        // Property ids this file defines (local row ids).
        let property_ids: BTreeSet<&str> = spec.properties.iter().map(|p| p.id.as_str()).collect();
        // Coverage targets claimed by this file's properties.
        let mut derived: BTreeSet<String> = BTreeSet::new();
        for p in &spec.properties {
            for link in &spec.links {
                if link.source == p.id
                    && link.field == "properties"
                    && link.column == "derives_from"
                {
                    derived.insert(link.target.clone());
                }
            }
        }
        for cid in &constraint_ids {
            let full = format!("{file_id}.{cid}");
            if !derived.contains(&full) && !derived.contains(*cid) {
                report.issues.push(Issue::new("coverage", file_id.clone(), format!(
                        "constraint `{cid}` has no deriving property — ∃ property.derives_from == `{full}` is required — write `[[{full}]]` in the property's derives_from cell"
                    )));
            }
        }
        // no_orphan_property — every property derives from something.
        for pid in &property_ids {
            let has_source = spec
                .links
                .iter()
                .any(|l| l.source == *pid && l.field == "properties" && l.column == "derives_from");
            if !has_source {
                report.issues.push(Issue::new(
                    "no_orphan_property",
                    file_id.clone(),
                    format!(
                        "property `{pid}` derives from nothing — derives_from takes a wiki-link like `[[{file_id}.<constraint-id>]]` (in an id:spec file the bare row form `[[<constraint-id>]]` resolves too)",
                    ),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_at(text: &str, path: &str) -> Spec {
        let mut spec = crate::spec::parse_str(text).expect("fixture parses");
        spec.path = Some(std::path::PathBuf::from(path));
        spec
    }

    /// Fixture corpus that triggers every rule in the catalog, so the
    /// test can assert the catalog covers exactly the rule ids the
    /// linter can emit (task 3.3).
    fn fixture_corpus() -> Vec<Spec> {
        vec![
            // fires frontmatter_valid (kind), no_conjoined_id,
            // no_universal_in_id — intent id must match its filename
            spec_at(
                "---\nid: bad.kind_and_every\nkind: feature\nstatement: \"THE system SHALL behave\"\n---\n",
                "bad-kind_and_every.md",
            ),
            // fires id_matches_file, unique_id, guard_required,
            // ears_syntax (no SHALL), coverage (constraint `a` has no
            // deriving property)
            spec_at(
                "---\nid: b.spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | |\n| a | invariant | `y` | |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | |\n",
                "b-spec.md",
            ),
            // fires total_refs (dangling dotted target),
            // no_orphan_property (property with empty derives_from)
            spec_at(
                "---\nid: c.ghost\nkind: intent\nstatement: \"THE system SHALL resolve\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `[[ghost.file.c]]` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit |  | `g()` | `x` |\n",
                "c-ghost.md",
            ),
            // fires model_present (no model section at all)
            spec_at(
                "---\nid: bare_spec\nkind: intent\nstatement: \"THE system SHALL exist\"\n---\n",
                "bare-spec.md",
            ),
            // fires dual_format_valid (## ADDED Requirements without the
            // sibling ## Requirements section)
            spec_at(
                "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## Purpose\nHalf a dual-format file.\n\n## ADDED Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n",
                "spec.md",
            ),
            // fires requirement_drift (mirrored ## Requirements drifted
            // from ## ADDED Requirements — gh#4)
            spec_at(
                "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds BUT THE MIRROR DRIFTED\n",
                "spec.md",
            ),
            // fires every_state_used (s2 unused) AND single_root_reachable
            // (s2 is an island — no transition, no link touches it)
            spec_at(
                "---\nid: b.spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | |\n| a | invariant | `y` | |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | |\n",
                "b-spec.md",
            ),
            // fires every_transition_valid (to a state that was never
            // declared) and no_self_ref (row tracing to itself)
            spec_at(
                "---\nid: b.spec\nkind: intent\nstatement: \"THE b SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| self_ref | invariant | `[[b.spec.self_ref]]` | [[b.spec.self_ref]] |\n| other | invariant | `y` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | ghost | [[b.spec.other]] |\n",
                "b-spec.md",
            ),
            // fires acyclic (two constraints mutually tracing via
            // traces_to — the checker spec's edge set)
            spec_at(
                "---\nid: b.cycle\nkind: intent\nstatement: \"THE b SHALL cycle\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| x | invariant | `x` | [[b.cycle.y]] |\n| y | invariant | `y` | [[b.cycle.x]] |\n",
                "b-cycle.md",
            ),
            // fires observability (warning, never an issue): an effect
            // Constraint with no observes edge — the advisory-severity
            // fixture. Adding it to the corpus keeps
            // catalog_covers_every_rule_the_linter_can_emit true after
            // the rule joined RULE_TABLE.
            spec_at(
                "---\nid: obs.unwatched\nkind: intent\nstatement: \"THE watcher SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `v finite` | [[obs.unwatched]] |\n| eff | effect | `output == {v}` | [[obs.unwatched]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.unwatched.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.unwatched.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.unwatched.eff]] | `g()` | `x` |\n",
                "obs-unwatched.md",
            ),
        ]
    }

    /// Checklist fixtures that trigger every external_completeness rule
    /// (mp1 row 10's manifest format), so the catalog-coverage test
    /// sees them alongside the spec-side fixtures.
    fn checklist_fixtures() -> Vec<crate::checklist::Checklist> {
        use crate::checklist::parse_str;
        vec![
            // One manifest, five beats: the nested item fires
            // checklist_well_formed; `unmapped` fires every_item_accounted;
            // `dangling` fires covered_maps_resolve (id resolves to
            // nothing); `norationale` fires waiver_has_rationale; the two
            // rows targeting `dup.target` fire no_duplicate_claim.
            parse_str(
                "dup.checklist.md".into(),
                "## Items\n\
                 \n- **unmapped**: never claimed\n\
                 - **dangling**: claimed against nothing\n\
                 - **norationale**: waived in silence\n\
                 - **dup.target**: claimed twice\n  - **nested.child**: indented under another item\n\
                 \n## Mapping\n\
                 \n| item | status | mapped_ids | rationale |\n\
                 |------|--------|------------|-----------|\n\
                 | dangling | covered | [[absent.row]] | |\n\
                 | norationale | waived | | |\n\
                 | dup.target | covered | [[absent.row]] | |\n\
                 | dup.target | waived | | fine on its own |\n",
            ),
        ]
    }

    // ---- external_completeness (specs/linter-external_completeness.md) ----

    use crate::checklist::parse_str;

    /// One spec whose constraint row a checklist can legitimately map to.
    fn mapped_spec() -> Spec {
        spec_at(
            "---\nid: x.file\nkind: intent\nstatement: \"THE x SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[x.file]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[x.file.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[x.file.c1]] | `g()` | `x` |\n",
            "x-file.md",
        )
    }

    fn checklist(text: &str) -> crate::checklist::Checklist {
        parse_str("ship.checklist.md".into(), text)
    }

    /// A fully mapped checklist passes: `covered` rows resolve, `waived`
    /// rows carry rationale — zero issues, zero warnings.
    #[test]
    fn fully_mapped_checklist_passes() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n- **a.second**: out of scope here\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[x.file.c1]], [[x.file.p1]] | |\n| a.second | waived | | tracked in the other repo |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        assert!(report.issues.is_empty(), "{:?}", report.issues);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    }

    /// No declared checklist → not_applicable: the pass runs but emits
    /// nothing — a repo with no checklist is out of scope, not vacuous.
    #[test]
    fn no_checklist_is_not_applicable() {
        let report = lint_all(&[mapped_spec()], &[]);
        assert!(report.issues.is_empty(), "{:?}", report.issues);
    }

    /// every_item_accounted: an item with no mapping row at all.
    #[test]
    fn unmapped_item_rejected() {
        let cl = checklist(
            "## Items\n\n- **a.lost**: never claimed\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.every_item_accounted")
            .collect();
        assert_eq!(hits.len(), 1, "{:?}", report.issues);
        assert!(hits[0].message.contains("a.lost"), "{:?}", hits);
        assert!(hits[0].file.contains("ship.checklist.md"), "{:?}", hits);
    }

    /// every_item_accounted also owns the status set: a row whose
    /// status is neither `covered` nor `waived` leaves the item
    /// unaccounted-for.
    #[test]
    fn unknown_status_leaves_item_unaccounted() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | pending | [[x.file.c1]] | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.every_item_accounted"),
            "{:?}",
            report.issues
        );
    }

    /// covered_maps_resolve: an empty mapped_ids list and a dangling id
    /// both fail the covered claim.
    #[test]
    fn dangling_mapped_id_rejected() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[absent.row]] | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.covered_maps_resolve")
            .collect();
        assert_eq!(hits.len(), 1, "{:?}", report.issues);
        assert!(hits[0].message.contains("absent.row"), "{:?}", hits);
        // The empty-list variant.
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.covered_maps_resolve"),
            "empty mapped_ids on a covered row: {:?}",
            report.issues
        );
    }

    /// covered_maps_resolve must reject a target that resolves to a
    /// file id or section anchor but NOT to a constraint/property row —
    /// the claim rests on a row, not on a file existing.
    #[test]
    fn covered_target_must_be_a_row_not_a_file_or_anchor() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[x.file]], [[x.file.model.state]] | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.covered_maps_resolve")
            .count();
        assert_eq!(
            hits, 2,
            "file-id and anchor targets are not rows: {:?}",
            report.issues
        );
    }

    /// waiver_has_rationale: an empty rationale is an unexplained waiver.
    #[test]
    fn unrationalized_waiver_rejected() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | waived | |   |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.waiver_has_rationale")
            .collect();
        assert_eq!(hits.len(), 1, "{:?}", report.issues);
        assert!(hits[0].message.contains("a.first"), "{:?}", hits);
    }

    /// no_duplicate_claim: two rows targeting one item — and the checks
    /// compose without double-reporting it as every_item_accounted too.
    #[test]
    fn duplicate_claim_rejected() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[x.file.c1]] | |\n| a.first | waived | | contradicts the first row |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let dup: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.no_duplicate_claim")
            .collect();
        assert_eq!(dup.len(), 1, "{:?}", report.issues);
        assert!(dup[0].message.contains("a.first"), "{:?}", dup);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.every_item_accounted"),
            "the duplication is no_duplicate_claim's beat — no double-report: {:?}",
            report.issues
        );
    }

    /// checklist_well_formed: parser defects surface as findings — a
    /// manifest the linter cannot read must never silently vanish.
    #[test]
    fn malformed_checklist_rejected() {
        let cl = checklist("# just a title\n");
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.checklist_well_formed")
            .count();
        assert_eq!(
            hits, 2,
            "missing Items + Mapping sections: {:?}",
            report.issues
        );
        // Orphan mapping row references an undeclared item.
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| ghost.item | covered | [[x.file.c1]] | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.checklist_well_formed"
                    && i.message.contains("ghost.item")),
            "{:?}",
            report.issues
        );
    }

    #[test]
    fn catalog_covers_every_rule_the_linter_can_emit() {
        let report = lint_all(&fixture_corpus(), &checklist_fixtures());
        // Warnings count as emittable rule ids too — the observability
        // rule emits on the warnings channel, never as an Issue.
        let emitted: BTreeSet<String> = report
            .issues
            .iter()
            .chain(report.warnings.iter())
            .map(|i| {
                i.rule_id
                    .strip_prefix("linter.")
                    .expect("rule_id prefix")
                    .to_string()
            })
            .collect();
        let catalog: BTreeSet<&str> = RULE_TABLE.iter().map(|(name, _)| *name).collect();
        let catalog_set: BTreeSet<String> = catalog.into_iter().map(String::from).collect();
        assert_eq!(
            emitted, catalog_set,
            "the rule table and the emittable rule ids must coincide — \
             a rule missing from the table panics at construction, one \
             missing from the corpus means the fixture stopped covering it"
        );
    }

    /// Observability semantics (specs/linter-observability.md): the
    /// warning channel, not the issues channel; self-observation does
    /// not count; observed effects are silent; failures() is untouched.
    #[test]
    fn unobserved_effect_warns_but_never_fails() {
        let spec = spec_at(
            "---\nid: obs.solo\nkind: intent\nstatement: \"THE solo SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `v finite` | [[obs.solo]] |\n| eff | effect | `output == {v}` | [[obs.solo]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.solo.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.solo.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.solo.eff]] | `g()` | `x` |\n",
            "obs-solo.md",
        );
        let report = lint_corpus(&[spec]);
        assert!(
            report.issues.is_empty(),
            "advisory must never be an issue: {:?}",
            report.issues
        );
        assert_eq!(report.failures(), 0, "failures() counts issues only");
        let w = report
            .warnings
            .iter()
            .find(|w| w.rule_id == "linter.observability")
            .expect("the unobserved effect must be warned");
        assert!(
            w.message.contains("obs.solo.eff"),
            "warning names the row: {}",
            w.message
        );
        assert_eq!(w.rule_semantics, rule_semantics("observability").unwrap());
    }

    #[test]
    fn observed_effect_is_silent_and_self_observation_does_not_count() {
        let make = |observes: &str| {
            spec_at(
                &format!(
                    "---\nid: obs.pair\nkind: intent\nstatement: \"THE pair SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| inv | invariant | `v finite` | [[obs.pair]] | |\n| eff | effect | `output == {{v}}` | [[obs.pair]] | {observes} |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.pair.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.pair.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.pair.eff]] | `g()` | `x` |\n"
                ),
                "obs-pair.md",
            )
        };
        // A DIFFERENT row in the same file observing counts.
        let observing = spec_at(
            "---\nid: obs.pair\nkind: intent\nstatement: \"THE pair SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| inv | invariant | `v finite` | [[obs.pair]] | |\n| eff | effect | `output == {{v}}` | [[obs.pair]] | |\n| watch | invariant | `seen` | [[obs.pair]] | [[obs.pair.eff]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.pair.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.pair.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.pair.eff]] | `g()` | `x` |\n| p_w | unit | [[obs.pair.eff]] | `g()` | `x` |\n",
            "obs-pair.md",
        );
        let report = lint_corpus(&[observing]);
        assert!(
            report
                .warnings
                .iter()
                .all(|w| w.rule_id != "linter.observability"),
            "a different-row observation counts: {:?}",
            report.warnings
        );
        // The effect row observing ITSELF does not count.
        let self_observing = make("[[obs.pair.eff]]");
        let report = lint_corpus(&[self_observing]);
        assert!(
            report
                .warnings
                .iter()
                .any(|w| w.rule_id == "linter.observability"),
            "self-observation is vacuous — still warned: {:?}",
            report.warnings
        );
    }

    #[test]
    fn dangling_observes_is_total_refs_beat_not_observability() {
        // The effect is observed by a resolving edge; a second row's
        // observes dangles. The dangling target must produce exactly one
        // total_refs finding and zero observability warnings — the
        // checks compose without double-reporting the same row (the
        // effect itself is genuinely observed, so it is silent too).
        let spec = spec_at(
            "---\nid: obs.ghost\nkind: intent\nstatement: \"THE ghost SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| inv | invariant | `v finite` | [[obs.ghost]] | |\n| eff | effect | `output == {{v}}` | [[obs.ghost]] | |\n| watch | invariant | `seen` | [[obs.ghost]] | [[obs.ghost.eff]] |\n| lost | invariant | `other` | [[obs.ghost]] | [[obs.absent.effect]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.ghost.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.ghost.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.ghost.eff]] | `g()` | `x` |\n| p_watch | unit | [[obs.ghost.eff]] | `g()` | `x` |\n| p_lost | unit | [[obs.ghost.inv]] | `g()` | `x` |\n",
            "obs-ghost.md",
        );
        let report = lint_corpus(&[spec]);
        let dangling_refs: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.total_refs")
            .collect();
        assert_eq!(
            dangling_refs.len(),
            1,
            "exactly one total_refs finding for the dangling target: {:?}",
            report.issues
        );
        assert!(
            report
                .warnings
                .iter()
                .all(|w| w.rule_id != "linter.observability"),
            "zero observability warnings — the checks compose without double-reporting: {:?}",
            report.warnings
        );
    }

    #[test]
    fn no_orphan_property_hint_teaches_wiki_link_syntax() {
        // gh#3: a bare id (or bare text) in derives_from is invisible to
        // the parser — the orphan finding must teach the file-qualified
        // wiki-link form, concretely for the file it fires in (for a
        // dual-format delta that is `[[spec.<constraint-id>]]`).
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | c | `g()` | `x` |\n",
            "spec.md",
        );
        let report = lint_corpus(&[spec]);
        let orphan = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.no_orphan_property")
            .expect("orphan finding fires for the bare-text cell");
        assert!(
            orphan.message.contains("[[spec."),
            "hint must show the self-file wiki-link form: {}",
            orphan.message
        );
        assert!(
            orphan.message.to_lowercase().contains("bare"),
            "hint must state that bare ids / bare text do not resolve: {}",
            orphan.message
        );
    }

    #[test]
    fn coverage_finding_suggests_the_wiki_link_form() {
        // gh#3: the coverage message states the required target as plain
        // `file-id.constraint-id`; the fix the author must TYPE is the
        // wiki-link — say so.
        let spec = spec_at(
            "---\nid: t\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | `[[t.other]]` | `g()` | `x` |\n",
            "t.md",
        );
        let report = lint_corpus(&[spec]);
        let cov = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.coverage")
            .expect("coverage finding fires — wrong target");
        assert!(
            cov.message.contains("[[spec.c]]") || cov.message.contains("[[t.c]]"),
            "coverage hint must show the wiki-link form of the required target: {}",
            cov.message
        );
    }

    #[test]
    fn requirement_drift_fires_on_mirrored_section_drift() {
        // gh#4: explain dual-format promises a drift check; the mirror
        // must match the ADDED section (normalized like
        // scripts/check_section_sync.py: per-line trailing space and
        // blank lines ignored).
        let drifted = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds BUT THE MIRROR DRIFTED\n",
            "spec.md",
        );
        let report = lint_corpus(&[drifted]);
        let drift = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.requirement_drift")
            .expect("drift finding fires on mismatched mirrors");
        assert!(
            drift.message.contains("## Requirements"),
            "finding must name the mirror section: {}",
            drift.message
        );
        // Identical mirrors (differing only in blank lines / trailing
        // space) stay clean.
        let clean = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds  \n\n\n",
            "spec.md",
        );
        let report = lint_corpus(&[clean]);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.requirement_drift"),
            "blank-line/trailing-space-only differences must not fire: {:?}",
            report.issues
        );
    }

    #[test]
    fn total_refs_hint_points_at_the_file_qualified_form() {
        // gh#5: a bare dotted row ref in a self-contained dual-format file
        // dangles with 'not defined in any spec file' — the finding must
        // hint that the row exists right here and show the fixed form.
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| thing.one | invariant | `x` | [[spec]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s1 | [[thing.one]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[spec.thing.one]] | `g()` | `x` |\n",
            "spec.md",
        );
        let report = lint_corpus(&[spec]);
        let dangling = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.total_refs")
            .expect("bare dotted row ref dangles");
        assert!(
            dangling.message.contains("[[spec.thing.one]]"),
            "hint must show the file-qualified fix: {}",
            dangling.message
        );
        assert!(
            dangling.message.contains("defined in this file"),
            "hint must say the row is local: {}",
            dangling.message
        );
    }

    #[test]
    fn total_refs_hint_absent_for_genuinely_unknown_targets() {
        // gh#5: the hint only fires when the row actually exists in the
        // same file — unknown targets keep the plain message.
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[spec]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s1 | [[spec.nope]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[spec.c]] | `g()` | `x` |\n",
            "spec.md",
        );
        let report = lint_corpus(&[spec]);
        let dangling = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.total_refs")
            .expect("unknown row dangles");
        assert!(
            !dangling.message.contains("defined in this file"),
            "no hint for a target that exists nowhere: {}",
            dangling.message
        );
    }

    #[test]
    fn dual_format_file_with_both_halves_is_clean() {
        // A complete dual-format file — frontmatter id: spec +
        // ## ADDED Requirements + ## Requirements — must not emit
        // dual_format_valid.
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## Purpose\nBoth halves present.\n\n## ADDED Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n\n## Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n",
            "spec.md",
        );
        let report = lint_corpus(&[spec]);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.dual_format_valid"),
            "complete dual-format file must be clean: {:?}",
            report.issues
        );
    }

    #[test]
    fn dual_format_added_without_requirements_section_fires() {
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n",
            "spec.md",
        );
        let report = lint_corpus(&[spec]);
        let issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.dual_format_valid")
            .collect();
        assert_eq!(issues.len(), 1, "missing ## Requirements: {:?}", issues);
        assert!(
            issues[0].message.contains("## Requirements"),
            "message names the missing half: {}",
            issues[0].message
        );
    }

    #[test]
    fn dual_format_file_with_wrong_id_fires() {
        // id: spec is the naming law for openspec-housed dual-format
        // files (openspec hard-requires the spec.md filename).
        let spec = spec_at(
            "---\nid: other.thing\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n\n## Requirements\n\n### Requirement: Something\nThe system SHALL do the thing.\n",
            "other-thing.md",
        );
        let report = lint_corpus(&[spec]);
        let issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.dual_format_valid")
            .collect();
        assert_eq!(issues.len(), 1, "wrong id: {:?}", issues);
        assert!(
            issues[0].message.contains("id: spec"),
            "message names the required id: {}",
            issues[0].message
        );
    }

    #[test]
    fn plain_spec_without_added_section_is_exempt() {
        // Corpus files (no ## ADDED Requirements) must not be touched
        // by the dual-format rule.
        let spec = spec_at(
            "---\nid: plain.spec\nkind: intent\nstatement: \"THE system SHALL behave\"\n---\n\n## Requirements\n\nSome requirements.\n",
            "plain-spec.md",
        );
        let report = lint_corpus(&[spec]);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.dual_format_valid"),
            "plain spec is exempt: {:?}",
            report.issues
        );
    }

    #[test]
    fn every_finding_carries_rule_id_and_semantics() {
        let report = lint_corpus(&fixture_corpus());
        assert!(!report.issues.is_empty());
        for issue in &report.issues {
            assert_eq!(
                issue.rule_id,
                rule_id(issue.rule_id.strip_prefix("linter.").unwrap()),
                "rule_id must be the stable linter.<name> identifier"
            );
            assert!(
                !issue.rule_semantics.is_empty(),
                "rule_semantics must be non-empty for {}",
                issue.rule_id
            );
            assert_eq!(
                issue.rule_semantics,
                rule_semantics(issue.rule_id.strip_prefix("linter.").unwrap()).unwrap(),
                "rule_semantics must come from the rule table"
            );
        }
    }

    #[test]
    fn rule_table_keys_are_unique() {
        let names: BTreeSet<&str> = RULE_TABLE.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            names.len(),
            RULE_TABLE.len(),
            "duplicate rule names in table"
        );
    }

    /// Two files sharing the same id (openspec naming law forces every
    /// dual-format delta/capability file to be `spec.md` → `id: spec`)
    /// must not erase each other from the reference index: every file's
    /// OWN rows stay resolvable (the #37 bug was an overwrite erasing
    /// them). Since #42's self-containment law they must also NOT see
    /// each other's rows — see the next test.
    #[test]
    fn same_id_files_do_not_collide_in_reference_resolution() {
        let make = |c: &str, p: &str, path: &str| {
            spec_at(
                &format!(
                    "---\nid: spec\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| {c} | invariant | `x` | [[spec]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[spec.{c}]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| {p} | unit | [[spec.{c}]] | `g()` | `x` |\n"
                ),
                path,
            )
        };
        let a = make("ca", "pa", "a/spec.md");
        let b = make("cb", "pb", "b/spec.md");
        let report = lint_corpus(&[a, b]);
        assert!(
            report.issues.is_empty(),
            "same-id files must not dangle each other's rows: {:?}",
            report.issues
        );
    }

    /// The dual-format self-containment law (spec-integration: "deltas
    /// stay self-contained — wiki-refs resolve only within the file"):
    /// for `id: spec` files a dotted ref must resolve against the file's
    /// OWN rows only. The corpus-wide union false-resolves any typo that
    /// collides with a row in another dual-format file (Rule-of-5
    /// CORR-001, demonstrated empirically).
    #[test]
    fn same_id_files_resolve_file_scoped_self_containment() {
        let a = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE a SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| local_a | invariant | `x` | |\n| cross | invariant | `y` | [[spec.row_in_b]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[spec.local_a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pa | unit | [[spec.local_a]] | `g()` | `x` |\n| pc | unit | [[spec.cross]] | `g()` | `y` |\n\n## Requirements\n\n### Requirement: A\nThe system SHALL hold.\n",
            "a/spec.md",
        );
        let b = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE b SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| row_in_b | invariant | `z` | |\n\n## Model\n\n### States\n\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s2 | s2 | [[spec.row_in_b]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| pb | unit | [[spec.row_in_b]] | `g()` | `z` |\n\n## Requirements\n\n### Requirement: B\nThe system SHALL hold.\n",
            "b/spec.md",
        );
        let report = lint_corpus(&[a, b]);
        let refs: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.total_refs")
            .collect();
        assert_eq!(refs.len(), 1, "cross-file ref must dangle: {:?}", refs);
        assert!(
            refs[0].message.contains("spec.row_in_b"),
            "names the self-containment violation: {}",
            refs[0].message
        );
    }

    // --- specodelic-b15: model_shape remainder + graph_shape ---

    /// model_shape.every_state_used: a declared state that no transition
    /// references is a mode the machine can never enter or leave.
    #[test]
    fn unused_state_fails_every_state_used() {
        let spec = spec_at(
            "---\nid: d.shape\nkind: intent\nstatement: \"THE system SHALL shape\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[d.shape]] |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `g` |\n",
            "d-shape.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.every_state_used")
            .collect();
        assert_eq!(hits.len(), 1, "s2 is declared but never used: {:?}", hits);
        assert!(
            hits[0].message.contains("s2"),
            "finding must name the unused state: {}",
            hits[0].message
        );
    }

    #[test]
    fn every_state_in_a_transition_passes() {
        let spec = spec_at(
            "---\nid: d.shape\nkind: intent\nstatement: \"THE system SHALL shape\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[d.shape]] |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | `g` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.shape.c]] | `g()` | `x` |\n",
            "d-shape.md",
        );
        let report = lint_corpus(&[spec]);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.every_state_used"),
            "all states used — no finding: {:?}",
            report.issues
        );
    }

    /// model_shape.every_transition_valid: from/to must name declared
    /// states — a dangling `to` would make the model-checker simulate a
    /// different graph than the author wrote.
    #[test]
    fn transition_to_undeclared_state_fails_every_transition_valid() {
        let spec = spec_at(
            "---\nid: d.shape\nkind: intent\nstatement: \"THE system SHALL shape\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[d.shape]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | ghost | `g` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.shape.c]] | `g()` | `x` |\n",
            "d-shape.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.every_transition_valid")
            .collect();
        assert_eq!(hits.len(), 1, "`ghost` is not a declared state: {:?}", hits);
        assert!(
            hits[0].message.contains("ghost"),
            "finding must name the undeclared state: {}",
            hits[0].message
        );
    }

    /// graph_shape.no_self_ref: a row referencing itself traces to
    /// nothing that owns it.
    #[test]
    fn self_referencing_row_fails_no_self_ref() {
        let spec = spec_at(
            "---\nid: d.self\nkind: intent\nstatement: \"THE system SHALL not self-reference\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[d.self.a]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[d.self.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.self.a]] | `g()` | `x` |\n",
            "d-self.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.no_self_ref")
            .collect();
        assert_eq!(hits.len(), 1, "a traces_to itself: {:?}", hits);
        assert!(
            hits[0].message.contains("traces_to"),
            "finding names the offending column: {}",
            hits[0].message
        );
    }

    /// graph_shape.acyclic: a traces_to cycle is rejected. The edge set
    /// is traces_to ∪ derives_from ∪ guard-as-edge (spec text) — a
    /// two-row mutual trace is the minimal cycle.
    #[test]
    fn mutual_traces_cycle_fails_acyclic() {
        let spec = spec_at(
            "---\nid: d.cycle\nkind: intent\nstatement: \"THE system SHALL stay acyclic\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[d.cycle.b]] |\n| b | invariant | `y` | [[d.cycle.a]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[d.cycle.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.cycle.a]] | `g()` | `x` |\n",
            "d-cycle.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.acyclic")
            .collect();
        assert_eq!(hits.len(), 1, "one 2-cycle, one finding: {:?}", hits);
        assert!(
            hits[0].message.contains("a") && hits[0].message.contains("b"),
            "finding names the cycle members: {}",
            hits[0].message
        );
    }

    /// graph_shape.single_root_reachable: every row must be connected to
    /// some intent row through the reference graph (traces_to, derives_from,
    /// guard-as-edge, from/to, emits) — an island of rows tracing only to
    /// each other has no owning purpose.
    #[test]
    fn orphan_cluster_fails_single_root_reachable() {
        let spec = spec_at(
            "---\nid: d.island\nkind: intent\nstatement: \"THE system SHALL anchor every row\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| anchored | invariant | `x` | [[d.island]] |\n| lost | invariant | `y` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[d.island.anchored]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.island.lost]] | `g()` | `y` |\n",
            "d-island.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.single_root_reachable")
            .collect();
        assert_eq!(
            hits.len(),
            1,
            "`lost` has no path to any intent: {:?}",
            hits
        );
        assert!(
            hits[0].message.contains("lost"),
            "finding names the disconnected row: {}",
            hits[0].message
        );
    }

    #[test]
    fn well_formed_file_passes_graph_and_model_shape() {
        // A well-formed file (every state used, valid transitions, no
        // self-refs, everything anchored) emits none of the new rules.
        let spec = spec_at(
            "---\nid: ok.shape\nkind: intent\nstatement: \"THE system SHALL shape\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[ok.shape]] |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[ok.shape.c]] |\n| t2 | s2 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[ok.shape.c]] | `g()` | `x` |\n",
            "ok-shape.md",
        );
        let report = lint_corpus(&[spec]);
        for rule in [
            "every_state_used",
            "every_transition_valid",
            "no_self_ref",
            "acyclic",
            "single_root_reachable",
        ] {
            assert!(
                !report
                    .issues
                    .iter()
                    .any(|i| i.rule_id == format!("linter.{rule}")),
                "well-formed file must not fire {rule}: {:?}",
                report.issues
            );
        }
    }
}

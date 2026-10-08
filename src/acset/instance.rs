//! The typed instance builder (openspec change `add-acset-core`, tasks
//! 3.x): a parsed corpus loaded into a typed instance — interned, dense
//! id-sets per schema object and a partial morphism vector per Reference
//! Typing row — in which an unresolved reference is a representable value
//! (`None`) that is never dropped and never a parse error.
//!
//! Parity context (design.md decision 2): the old `graph::build` path stays
//! authoritative until `adapter_graph_equivalent` holds over the whole
//! corpus, including its tolerated dirty shapes — duplicate ids resolve
//! first-wins (`or_insert`) and surface only as a named collision report,
//! never the hard failure the old path never emitted (Rule-of-5 CORR-001).

use std::collections::{BTreeMap, BTreeSet};

use crate::acset::schema::{self, Endpoint, Morphism, Schema, Typing, classify};
use crate::graph::{self, Edge, NodeKind, canonical_id};
use crate::spec::Spec;

/// A link that resolved to no id — stored as a value (`None` in the
/// morphism vector), never dropped, never a parse error
/// (`dangling_is_a_value`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dangling {
    /// The anchor the link was emitted from: `file (intent)` for
    /// frontmatter links, `file.row` for table links, the transition row
    /// for the from/to walk.
    pub from: String,
    /// The typed reference column the link sat in (`transitions.from` for
    /// the transitions walk).
    pub kind: String,
    /// The unresolved target spelling.
    pub target: String,
}

/// A Reference Typing violation: a link whose resolved target object is
/// outside the morphism's allowed targets — reported, never stored as a
/// morphism value (`forbidden_edge_not_stored`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    /// The anchor the forbidden edge would have started at.
    pub from: String,
    /// The resolved target the forbidden edge would have pointed at.
    pub to: String,
    /// The typed reference field, e.g. `constraints.traces_to`.
    pub edge_kind: String,
    /// The labeled reason — the Schema's violation prose.
    pub reason: String,
}

/// One morphism value: the source row it sits on, the resolved target
/// (`None` when the reference dangles — the value is stored as `None`),
/// the graph's edge-kind spelling, and the report anchor.
struct Value {
    /// The schema.morphisms index the value is stored under.
    morph: usize,
    /// Interned source node (the row the morphism sits on).
    from: usize,
    /// Interned target node — `None` for the dangling value.
    to: Option<usize>,
    /// The graph's edge-kind spelling (`field.column`).
    kind: String,
    /// The unresolved target spelling (present when `to` is `None`).
    raw: String,
}

/// The typed instance over a parsed corpus: one interned id-set per schema
/// object, one partial morphism vector per Reference Typing row, plus the
/// build-time reports (dangling values, typing violations, duplicate-id
/// collisions).
pub struct Instance {
    /// The schema the instance is typed against — the canonical Reference
    /// Typing value, built once per build.
    schema: Schema,
    /// Dense, bidirectional interning of the corpus's node set with
    /// sorted-order assignment (`ids[i]` is the i-th id in sorted order).
    intern: Intern,
    /// One partial morphism vector per Reference Typing row (parallel to
    /// `schema.morphisms`), walk-ordered.
    vectors: Vec<Vec<Value>>,
    /// Attribute cells carried untouched: node id -> column -> cell.
    cells: BTreeMap<String, BTreeMap<String, String>>,
    /// The stored edges in walk order — `graph::build`'s recording order
    /// (per spec: the transitions walk, then the links loop), which is
    /// what the parity property compares edge for edge.
    walk: Vec<Edge>,
    /// The dangling values, in walk order.
    dangling: Vec<Dangling>,
    /// The Reference Typing violations, in walk order.
    violations: Vec<Violation>,
    /// The named duplicate-id collision report, sorted.
    collisions: Vec<String>,
}

/// Dense, bidirectional interning with sorted-order assignment: ids are
/// assigned indices over the corpus's node set in sorted order, with the
/// reverse map kept alongside.
struct Intern {
    ids: Vec<String>,
    index: BTreeMap<String, usize>,
}

impl Intern {
    fn build(nodes: &BTreeSet<String>) -> Self {
        Self {
            ids: nodes.iter().cloned().collect(),
            index: nodes
                .iter()
                .enumerate()
                .map(|(i, id)| (id.clone(), i))
                .collect(),
        }
    }

    fn id(&self, i: usize) -> &str {
        &self.ids[i]
    }
}

/// Intern one node id — the walk only addresses defined ids, so a miss is
/// a build bug, not a corpus shape.
fn intern_of(intern: &Intern, id: &str) -> usize {
    intern
        .index
        .get(id)
        .copied()
        .unwrap_or_else(|| panic!("uninterned node id: {id}"))
}

/// The schema.morphisms index of the row with this column and source
/// object — how the transitions walk finds the from/to morphisms.
fn morph_by(schema: &Schema, column: &str, source: &str) -> usize {
    schema
        .morphisms
        .iter()
        .position(|m| m.column == column && m.source.0 == source)
        .unwrap_or_else(|| panic!("no `{column}` morphism on {source}"))
}

/// The first row of a column, regardless of source object — the fallback
/// when no row's predicates were consulted (the unresolvable-target beat,
/// the old `let target = target_kind?` path, which still stores).
fn morph_by_column(schema: &Schema, column: &str) -> Option<usize> {
    schema.morphisms.iter().position(|m| m.column == column)
}

/// The schema.morphisms index of a matched row — it is a borrow into
/// `schema.morphisms`, so pointer identity finds it.
fn morph_index(schema: &Schema, row: &Morphism) -> usize {
    schema
        .morphisms
        .iter()
        .position(|m| std::ptr::eq(m, row))
        .expect("the matched row is a schema row")
}

/// Node set, attribute cells, and duplicate-row collisions in one
/// pass over the corpus (see `from_specs` for the collision law).
fn collect_nodes_and_cells(specs: &[Spec]) -> CorpusCells {
    let mut nodes: BTreeSet<String> = BTreeSet::new();
    let mut cells: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut collisions: BTreeSet<String> = BTreeSet::new();
    for spec in specs {
        let file_id = &spec.intent.id;
        nodes.insert(file_id.clone());
        // Qualified row nodes — the edge endpoints address rows as
        // `file.row`, never the bare id.
        for r in &spec.constraints {
            nodes.insert(format!("{file_id}.{}", r.id));
        }
        for r in &spec.properties {
            nodes.insert(format!("{file_id}.{}", r.id));
        }
        for r in &spec.states {
            nodes.insert(format!("{file_id}.{}", r.id));
        }
        for t in &spec.transitions {
            nodes.insert(format!("{file_id}.{}", t.id));
        }
        for r in &spec.constraints {
            note_row(
                file_id,
                &r.id,
                &r.cells,
                &mut seen,
                &mut collisions,
                &mut cells,
            );
        }
        for r in &spec.properties {
            note_row(
                file_id,
                &r.id,
                &r.cells,
                &mut seen,
                &mut collisions,
                &mut cells,
            );
        }
        for r in &spec.states {
            note_row(
                file_id,
                &r.id,
                &r.cells,
                &mut seen,
                &mut collisions,
                &mut cells,
            );
        }
        for t in &spec.transitions {
            let mut row_cells = BTreeMap::new();
            row_cells.insert("from".to_string(), t.from.clone());
            row_cells.insert("to".to_string(), t.to.clone());
            if let Some(g) = &t.guard {
                row_cells.insert("guard".into(), g.clone());
            }
            note_row(
                file_id,
                &t.id,
                &row_cells,
                &mut seen,
                &mut collisions,
                &mut cells,
            );
        }
    }
    (nodes, cells, collisions)
}

/// The walk state: read-only context (schema, intern, kinds, resolution
/// index) plus the three output buckets (stored values, dangling,
/// typing violations) accumulated in walk order.
struct Walk<'a> {
    schema: &'a Schema,
    intern: &'a Intern,
    kinds: &'a BTreeMap<String, NodeKind>,
    file_rows: &'a BTreeMap<String, Vec<String>>,
    from_morph: usize,
    stored: Vec<Value>,
    dangling: Vec<Dangling>,
    violations: Vec<Violation>,
}

impl Walk<'_> {
    /// A Transition's from/to cells are typed reference fields (→
    /// State, same file): one morphism value each, per
    /// specs/graph.md's total_extraction. An unknown state dangles
    /// — never dropped. A null from/to cell is a Model-shape
    /// problem (linter-graph_shape), not a graph edge.
    fn transitions(&mut self, spec: &Spec, file_id: &str, to_morph: usize) {
        for t in &spec.transitions {
            let anchor = format!("{file_id}.{}", t.id);
            for (col, cell, morph) in [("from", &t.from, self.from_morph), ("to", &t.to, to_morph)]
            {
                let state = cell.trim().trim_matches('`').trim();
                if state.is_empty() {
                    continue;
                }
                let from_idx = intern_of(self.intern, &anchor);
                if spec.states.iter().any(|s| s.id == state) {
                    self.stored.push(Value {
                        morph,
                        from: from_idx,
                        to: Some(intern_of(self.intern, &format!("{file_id}.{state}"))),
                        kind: format!("transitions.{col}"),
                        raw: String::new(),
                    });
                } else {
                    self.stored.push(Value {
                        morph,
                        from: from_idx,
                        to: None,
                        kind: format!("transitions.{col}"),
                        raw: state.to_string(),
                    });
                    self.dangling.push(Dangling {
                        from: anchor.clone(),
                        kind: format!("transitions.{col}"),
                        target: state.to_string(),
                    });
                }
            }
        }
    }

    /// The typed reference links, in link order, against the
    /// corpus-wide map (Revision 18: the former `id: spec` file-scoped
    /// resolution retired with `id: spec` itself).
    fn links(&mut self, spec: &Spec, file_id: &str) {
        let scoped_rows = self.file_rows.clone();
        for link in &spec.links {
            // The shared edge derivation (task 3.4 TIDY): the
            // typed-column filter, resolution, bare-local and
            // metasyntactic skips, anchoring, and source-kind lookup
            // are graph.rs's `resolve_link` — one walk, two consumers.
            let Some(outcome) = graph::resolve_link(link, file_id, &scoped_rows, self.kinds) else {
                continue;
            };
            match outcome {
                graph::ResolvedLink::Stored {
                    anchor,
                    source_node,
                    column,
                    kind,
                    target,
                    source_kind,
                } => self.stored_link(anchor, source_node, column, kind, target, source_kind),
                graph::ResolvedLink::Dangling {
                    file_id: _,
                    anchor,
                    source_node,
                    column,
                    target,
                } => self.dangling_link(anchor, source_node, column, target),
            }
        }
    }

    /// Reference Typing: a typed reference column whose target kind is
    /// forbidden is reported as a labeled violation, never stored as a
    /// value (`edge_kind_matches_typing`, specs/graph.md).
    #[allow(clippy::too_many_arguments)]
    fn stored_link(
        &mut self,
        anchor: String,
        source_node: String,
        column: String,
        kind: String,
        target: String,
        source_kind: Option<NodeKind>,
    ) {
        let endpoint = |k: &NodeKind| Endpoint {
            object: k.object().to_string(),
            kind: k.kind_cell().to_string(),
            describe: k.describe(),
        };
        let morph = match classify(
            self.schema,
            &column,
            source_kind.as_ref().map(endpoint),
            self.kinds.get(&target).map(endpoint),
        ) {
            Typing::Allowed(Some(row)) => morph_index(self.schema, row),
            // No row consulted (unresolvable target kind) —
            // the column's first row, as the reason
            // composition does.
            Typing::Allowed(None) => {
                morph_by_column(self.schema, &column).unwrap_or(self.from_morph)
            }
            Typing::Forbidden(reason) => {
                self.violations.push(Violation {
                    from: anchor,
                    to: target,
                    edge_kind: kind,
                    reason,
                });
                return;
            }
        };
        // specodelic-efb: resolve()'s member arm can return a
        // label-qualified member target (`v.row (deep)`) — a display
        // qualifier over a defined row id, not a node of its own. Intern
        // the canonical id; the qualifier is a projection-time annotation
        // (add-graph-views D2), never an internable node. The typing
        // check above deliberately ran on the raw spelling: the qualifier
        // is not a kind, so member-path links keep their resolved-clean
        // semantics (specodelic-njh) — lint and graph must not diverge.
        self.stored.push(Value {
            morph,
            from: intern_of(self.intern, &source_node),
            to: Some(intern_of(self.intern, canonical_id(&target))),
            kind,
            raw: String::new(),
        });
    }

    /// Dangling is a value — stored, never dropped.
    fn dangling_link(
        &mut self,
        anchor: String,
        source_node: String,
        column: String,
        target: String,
    ) {
        let morph = morph_by_column(self.schema, &column).unwrap_or(self.from_morph);
        self.stored.push(Value {
            morph,
            from: intern_of(self.intern, &source_node),
            to: None,
            kind: column.clone(),
            raw: target.clone(),
        });
        self.dangling.push(Dangling {
            from: anchor,
            kind: column,
            target,
        });
    }
}

/// The collected node/cell state from one corpus pass: node set,
/// attribute cells per row, and duplicate-row collisions.
type CorpusCells = (
    BTreeSet<String>,
    BTreeMap<String, BTreeMap<String, String>>,
    BTreeSet<String>,
);

impl Instance {
    /// Build the typed instance for a parsed corpus. Infallible: every
    /// shape the old path accepts — dangling links, duplicate ids — is a
    /// value here, not a failure.
    pub fn from_specs(specs: &[Spec]) -> Self {
        let schema = schema::canonical();
        // First-wins kind index — the same `or_insert` result the old path
        // resolves duplicate-id corpora with
        // (`duplicate_id_first_wins_parity`).
        let kinds = graph::kind_index(specs);

        // Resolution index — the same aggregation graph::build runs (#37:
        // same-id files aggregate their row sets so a file's OWN rows are
        // never erased; Revision 18: resolution is corpus-wide, the
        // former `id: spec` file-scoping retired with `id: spec` itself).
        let mut file_rows: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for spec in specs {
            file_rows
                .entry(spec.intent.id.clone())
                .or_default()
                .extend(spec.defined_ids());
        }

        // Node set, attribute cells, and collision detection in one pass.
        // A collision is a file-qualified row id defined twice: the old
        // path silently keeps the first (`or_insert`); the builder
        // resolves identically and names the collision instead. The
        // intent-id aggregation itself is by design (#37), not a collision.
        let (nodes, cells, collisions) = collect_nodes_and_cells(specs);
        let intern = Intern::build(&nodes);

        // The walk — the same shape graph::build walks: per spec, the
        // transitions' from/to cells first, then the typed reference
        // links in link order. Walk order is what `edges()` returns, so
        // the parity comparison is edge for edge, in the builder's order.
        let from_morph = morph_by(&schema, "from", "Transition");
        let to_morph = morph_by(&schema, "to", "Transition");
        let mut walk = Walk {
            schema: &schema,
            intern: &intern,
            kinds: &kinds,
            file_rows: &file_rows,
            from_morph,
            stored: Vec::new(),
            dangling: Vec::new(),
            violations: Vec::new(),
        };
        for spec in specs {
            let file_id = spec.intent.id.as_str();
            walk.transitions(spec, file_id, to_morph);
            walk.links(spec, file_id);
        }

        // Destructure the walk (ends its borrows) before building the instance.
        let Walk {
            stored,
            dangling,
            violations,
            ..
        } = walk;

        // The edges, in walk order — the parity property's byte (the same
        // recording order graph::build pushes: per spec, the transitions
        // walk, then the links loop).
        let walk_edges = stored
            .iter()
            .filter(|v| v.to.is_some())
            .map(|v| Edge {
                from: intern.id(v.from).to_string(),
                to: intern.id(v.to.expect("filtered")).to_string(),
                kind: v.kind.clone(),
            })
            .collect();
        // Morphism vectors: group the walk by schema row (partial vector
        // per morphism, walk-ordered — the acset semantics).
        let mut vectors: Vec<Vec<Value>> =
            (0..schema.morphisms.len()).map(|_| Vec::new()).collect();
        for v in stored {
            vectors[v.morph].push(v);
        }

        Instance {
            schema,
            intern,
            vectors,
            cells,
            walk: walk_edges,
            dangling,
            violations,
            collisions: collisions.into_iter().collect(),
        }
    }

    /// Links that resolved AND passed Reference Typing — the stored
    /// morphism values (`no_link_dropped`'s first bucket).
    pub fn stored(&self) -> usize {
        self.vectors
            .iter()
            .flatten()
            .filter(|v| v.to.is_some())
            .count()
    }

    /// The unresolved links, stored as values (`dangling_is_a_value`).
    pub fn dangling(&self) -> &[Dangling] {
        &self.dangling
    }

    /// The Reference Typing violations — labeled, never stored as values
    /// (`forbidden_edge_not_stored`).
    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    /// The named duplicate-id collision report — surfaced, never a hard
    /// failure the old path did not emit
    /// (`duplicate_id_first_wins_parity`).
    pub fn collisions(&self) -> &[String] {
        &self.collisions
    }

    /// The corpus's edge set, derived from the instance — the parity
    /// property compares it, edge for edge, with `graph::build`'s
    /// (`adapter_graph_equivalent`, task 3.3).
    pub fn edges(&self) -> Vec<Edge> {
        self.walk.clone()
    }

    /// The canonical serialization — byte-identical for the same corpus
    /// regardless of input file order (`rebuild_is_byte_stable`). Every
    /// section is emitted in sorted order, so input file order cannot
    /// reach the bytes.
    pub fn serialize(&self) -> String {
        let mut out = String::from("instance\n");
        out.push_str(&format!("nodes: {}\n", self.intern.ids.len()));
        for id in &self.intern.ids {
            out.push_str(&format!("node: {id}\n"));
        }
        for (m, values) in self.schema.morphisms.iter().zip(&self.vectors) {
            let mut entries: Vec<&Value> = values.iter().collect();
            entries.sort_by(|a, b| {
                (&self.intern.ids[a.from], &a.kind).cmp(&(&self.intern.ids[b.from], &b.kind))
            });
            for v in entries {
                let from = &self.intern.ids[v.from];
                match &v.to {
                    Some(to) => out.push_str(&format!(
                        "morph {}: {} -> {}\n",
                        m.name, from, self.intern.ids[*to]
                    )),
                    None => {
                        out.push_str(&format!("morph {}: {} -> None ({})\n", m.name, from, v.raw))
                    }
                }
            }
        }
        for (node, cells) in &self.cells {
            for (col, cell) in cells {
                out.push_str(&format!("cell: {node} {col} = {cell}\n"));
            }
        }
        let mut dangling: Vec<&Dangling> = self.dangling.iter().collect();
        dangling.sort();
        for d in dangling {
            out.push_str(&format!(
                "dangling: {} ({}) -> {}\n",
                d.from, d.kind, d.target
            ));
        }
        let mut violations: Vec<&Violation> = self.violations.iter().collect();
        violations.sort();
        for v in violations {
            out.push_str(&format!(
                "violation: {} -[{}]-> {}: {}\n",
                v.from, v.edge_kind, v.to, v.reason
            ));
        }
        for c in &self.collisions {
            out.push_str(&format!("collision: {c}\n"));
        }
        out
    }

    /// The attribute cells carried untouched for one node id
    /// (`cells_carried`) — `None` when the id addresses no row this build
    /// saw.
    pub fn cells(&self, id: &str) -> Option<&BTreeMap<String, String>> {
        self.cells.get(id)
    }

    /// The interned index for a file-qualified id — the query module's
    /// seed-existence check (`seeds_exist`); `None` when the id addresses
    /// no node this build saw.
    pub(crate) fn index_of(&self, id: &str) -> Option<usize> {
        self.intern.index.get(id).copied()
    }

    /// The interned id for a node index — the inverse of `index_of`.
    pub(crate) fn id_of(&self, i: usize) -> &str {
        self.intern.id(i)
    }

    /// The stored morphism values under one schema morphism name, across
    /// every row carrying it — `(source index, resolved target index)`;
    /// a `None` target is the dangling value, which traversal never
    /// follows (`dangling_not_followed`). Walk order is irrelevant to the
    /// closure (set semantics), so the raw vector order is fine.
    pub(crate) fn morphism_values(&self, name: &str) -> Vec<(usize, Option<usize>)> {
        self.schema
            .morphisms
            .iter()
            .zip(&self.vectors)
            .filter(|(m, _)| m.name == name)
            .flat_map(|(_, values)| values.iter().map(|v| (v.from, v.to)))
            .collect()
    }

    /// The interned node ids, in canonical sorted order — the seed space
    /// every query ranges over (`seeds_exist`).
    pub fn ids(&self) -> &[String] {
        &self.intern.ids
    }

    /// The distinct schema morphism names this instance carries values
    /// under — the full morphism set M (the default scope when a consumer
    /// means "every reference field", as `merge`'s blast radius does).
    pub fn morphism_names(&self) -> Vec<&'static str> {
        self.schema
            .morphisms
            .iter()
            .map(|m| m.name)
            .collect::<BTreeSet<&'static str>>()
            .into_iter()
            .collect()
    }
}

/// Record one row definition: node membership (already collected), the
/// attribute cells carried untouched, and collision detection — a second
/// definition of the same file-qualified row id is the collision the old
/// path swallowed (`or_insert`); it is named here, never a hard failure.
fn note_row(
    file_id: &str,
    id: &str,
    row_cells: &BTreeMap<String, String>,
    seen: &mut BTreeSet<(String, String)>,
    collisions: &mut BTreeSet<String>,
    cells: &mut BTreeMap<String, BTreeMap<String, String>>,
) {
    if !seen.insert((file_id.to_string(), id.to_string())) {
        collisions.insert(format!("{file_id}.{id}"));
    }
    for (col, cell) in row_cells {
        cells
            .entry(format!("{file_id}.{id}"))
            .or_default()
            .insert(col.clone(), cell.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::parse_str;

    /// Parse an inline corpus text (the same parser `Spec::from_file` runs).
    fn parse(text: &str) -> Spec {
        parse_str(text).expect("test corpus parses")
    }

    /// `attribute_cells_carried_untouched` (task 3.2): the builder carries
    /// each row's attribute cells through as parsed — no normalization,
    /// no re-typing — so later phases (and the writer change) see the
    /// corpus exactly as it was.
    #[test]
    fn attribute_cells_carried_untouched() {
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "rows carry their cells"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `x > 0` | [[c1]] |
"#,
        );
        let inst = Instance::from_specs(&[c1]);
        let cells = inst.cells("c1.r").expect("the row's cells are carried");
        assert_eq!(cells.get("kind").map(String::as_str), Some("invariant"));
        assert_eq!(cells.get("expr").map(String::as_str), Some("`x > 0`"));
    }

    /// `dangling_is_a_value` (acset-core properties): one link resolving to
    /// no id — the morphism value is None, the build succeeds.
    #[test]
    fn dangling_is_a_value() {
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "points at a ghost"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `true` | [[c1.ghost_row]] |
"#,
        );
        let c2 = parse("---\nid: c2\nkind: intent\nstatement: \"another intent\"\n---\n");
        // The build succeeds — a dangling reference is a value, not an error.
        let inst = Instance::from_specs(&[c1, c2]);
        assert_eq!(inst.dangling().len(), 1, "the ghost link dangles");
        assert!(
            inst.dangling()[0].target.contains("ghost_row"),
            "the dangling value names the unresolved target"
        );
        assert_eq!(inst.stored(), 0, "nothing stored — the only link dangles");
    }

    /// `no_link_dropped` (delta scenario): stored + dangling + violations
    /// == links — every typed-reference link lands in exactly one bucket.
    #[test]
    fn no_link_dropped() {
        // Three typed-reference links, one per bucket: `[[c1.other]]` is a
        // typing violation (traces_to must reach an Intent), `[[c2]]` stores
        // (Constraint → Intent allowed), `[[c1.ghost_row]]` dangles.
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "one link of each outcome"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `true` | [[c1.other]] |
| other | invariant | `true` | [[c1.ghost_row]] |
| src | invariant | `true` | [[c2]] |
"#,
        );
        let c2 = parse("---\nid: c2\nkind: intent\nstatement: \"the intent target\"\n---\n");
        let inst = Instance::from_specs(&[c1, c2]);
        assert_eq!(inst.violations().len(), 1, "the constraint target violates");
        assert_eq!(inst.dangling().len(), 1, "the ghost link dangles");
        assert_eq!(inst.stored(), 1, "the Intent-targeted edge stores");
        assert_eq!(
            inst.stored() + inst.dangling().len() + inst.violations().len(),
            3,
            "stored + dangling + violations == links — none dropped"
        );
    }

    /// `forbidden_edge_not_stored` (delta scenario): a reference whose
    /// target object is outside the morphism's allowed targets is recorded
    /// as a typing violation and stores no morphism value.
    #[test]
    fn forbidden_edge_not_stored() {
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "traces_to must reach an Intent, not a constraint row"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `true` | [[c1.other]] |
| other | invariant | `true` |  |
"#,
        );
        let inst = Instance::from_specs(&[c1]);
        assert_eq!(inst.violations().len(), 1, "the forbidden link is reported");
        assert!(
            inst.violations()[0].reason.contains("traces_to"),
            "the violation names the morphism: {:?}",
            inst.violations()[0]
        );
        assert_eq!(inst.stored(), 0, "no morphism value stored for it");
        assert!(
            inst.dangling().is_empty(),
            "the target resolves — it is a violation, not a dangling link"
        );
    }

    /// `rebuild_is_byte_stable` (delta scenario): the same corpus built
    /// with input files in two different orders serializes byte-identically.
    #[test]
    fn rebuild_is_byte_stable() {
        let c1 = parse(
            r#"---
id: c1
kind: intent
statement: "clean baseline, first file"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| r | invariant | `true` | [[c2]] |
"#,
        );
        let c2 = parse(
            r#"---
id: c2
kind: intent
statement: "the other file, pointing back"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| s | invariant | `true` | [[c1]] |
"#,
        );
        let forward = Instance::from_specs(&[c1.clone(), c2.clone()]);
        let shuffled = Instance::from_specs(&[c2, c1]);
        assert_eq!(
            forward.serialize(),
            shuffled.serialize(),
            "shuffled input order rebuilds byte-identically"
        );
    }

    /// `duplicate_id_first_wins_parity` (delta scenario): a corpus with two
    /// rows sharing the file-qualified id — which the existing builder
    /// accepts first-wins via `kind_index`'s `or_insert` — resolves the
    /// same way in the builder, with the collision NAMED in the collision
    /// report, never a hard failure the old path did not emit.
    #[test]
    fn duplicate_id_first_wins_parity() {
        // Two files with the SAME intent id `c`: the first defines row `r`
        // as an invariant Constraint, the second as a unit Property.
        // kind_index keeps the FIRST (`Constraint("invariant")`), so a guard
        // link to `c.r` is allowed — the builder must resolve identically.
        let first = parse(
            r#"---
id: c
kind: intent
statement: "first file — the first-wins winner"
---

## Constraints

| id | kind | expr |
|----|------|------|
| r | invariant | `true` |
"#,
        );
        let second = parse(
            r#"---
id: c
kind: intent
statement: "duplicate id — must not erase or fail"
---

## Properties

| id | kind | generator | predicate |
|----|------|-----------|-----------|
| r | unit | `arb()` | `true` |
"#,
        );
        let user = parse(
            r#"---
id: u
kind: intent
statement: "guards the duplicated id's row"
---

## Model

### States
- `alive`
- `dead`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | alive | dead | [[c.r]] |
"#,
        );
        let specs = [first, second, user];
        let inst = Instance::from_specs(&specs);
        // First-wins parity: the guard link resolves against the FIRST
        // file's invariant constraint, so the guard edge stores — exactly
        // what graph::build's kind_index or_insert yields.
        assert_eq!(
            inst.edges(),
            crate::graph::build(&specs).edges,
            "the builder resolves the duplicate-id corpus identically to the old path"
        );
        // … and the collision is NAMED — never silently swallowed.
        assert_eq!(
            inst.collisions().len(),
            1,
            "one collision report for the duplicated id: {:?}",
            inst.collisions()
        );
        assert!(
            inst.collisions()[0].contains("c.r"),
            "the collision names the duplicated file-qualified id"
        );
    }
}

//! The format's fixed finite schema as one data value (`add-acset-core`
//! tasks 2.x; the acset-core capability's Schema-as-data typing).
//!
//! Purpose: hold the five objects and the typed reference morphisms of
//! `specs/specodelic.md`'s Reference Typing table as a single value that a
//! generic checker evaluates — instead of the per-field match arms in
//! `src/graph.rs::typing_violation`. Responsibilities: declare the closed
//! object set (a sixth object exists only under a new Revision heading),
//! the morphism rows with their declared refinement predicates and
//! endo-acyclicity flags, the canonical sorted order every derived artifact
//! inherits, and the schema-level `check` (objects closed, morphisms typed,
//! flags consistent). Rationale: adding a reference field must be adding
//! one row — `typing_read_from_schema` — and the document vs code drift is
//! lint-visible (`schema_matches_typing_table`, task 2.4).
//!
//! Morphism modeling notes (the Reference Typing table read as rows):
//!
//! - A table row is one morphism per (source object, target object) it
//!   admits. A column resolving to more than one object splits into rows
//!   sharing the `column` but distinguished by `name` — the schema-unique
//!   identity is the pair (source, name) (`morphisms_typed`). `guard`
//!   splits into `guard` (→ invariant Constraint) and `guard_state`
//!   (→ State, Revision 12); `derives_from` splits into `derives_from`
//!   (→ Constraint) and `derives_from_law` (→ the same-kind law case).
//! - Refinements are declared predicates over a row's own kind column
//!   (`refinement_declared`): `emits`/`observes` target effect Constraints
//!   only, `satisfies` targets extension_point only, `guard` targets an
//!   invariant Constraint, the `derives_from` law case is law-to-law, and
//!   `uses` targets a `kind: profile` Intent — never inlined into a
//!   checker.
//! - `endo_acyclicity_flagged`: a structurally endo morphism (source ==
//!   target) carries an explicit flag. `Some(true)` — cycles through it
//!   are forbidden and checked generically (supersedes, both rows:
//!   `supersedes_acyclic`). `Some(false)` — the endo case exists but cycle
//!   policing lives elsewhere or the cycle is well-formed (the
//!   `derives_from` law case: the linter's `acyclic_traces` owns it;
//!   `satisfies`/`observes`: mutual cross-file claims are well-formed).
//!   `None` — not an endo morphism (source != target); `check` rejects a
//!   missing flag on an endo row and a flag on a non-endo row.

use std::collections::{BTreeMap, BTreeSet};

/// One object of the schema — the closed five of
/// `specs/specodelic.md` (`objects_closed`). The id is open data: `check`
/// rejects anything outside {Intent, Constraint, State, Transition,
/// Property} so the sixth object can be named, tested, and rejected.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(pub String);

impl ObjectId {
    pub fn new(id: &str) -> Self {
        ObjectId(id.to_string())
    }
}

/// The objects of the schema, in canonical sorted order (`canonical_order`):
/// every artifact derived from the Schema enumerates them this way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema {
    pub objects: BTreeSet<ObjectId>,
    /// The typed reference morphisms, canonical order: sorted by
    /// (source, name). `(source, name)` is unique across the schema
    /// (`morphisms_typed`) — a column split across target objects is
    /// distinguished by its `name`, never a duplicate pair.
    pub morphisms: Vec<Morphism>,
}

/// One Reference Typing row, as data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Morphism {
    /// The reference field whose values fill this morphism (the surface
    /// column name). Equal to `name` except where the column splits across
    /// target objects.
    pub column: &'static str,
    /// The schema-unique morphism identity; the pair (source, name) is
    /// unique across the schema.
    pub name: &'static str,
    /// The object this morphism's rows appear on (the table's Appears-on
    /// column).
    pub source: ObjectId,
    /// The single object this morphism resolves to (the table's Must-
    /// resolve-to column, one object per row — a compound cell splits).
    pub target: ObjectId,
    /// Declared refinement predicates over a row's own kind column
    /// (`refinement_declared`), evaluated at build time.
    pub refinements: Vec<Refinement>,
    /// The source-side rule this row enforces (see `SourceRule`).
    pub source_rule: SourceRule,
    /// The labeled reason when no row of this column accepts the edge —
    /// the exact prose the graph fixtures pin, carried as data.
    /// `{tk}` interpolates the target's description. For a column split
    /// across rows, every row carries the same template.
    pub target_violation: &'static str,
    /// The appears-on reason (`AppearsOn` rows only), fired before the
    /// target is considered; `{src}` interpolates the source's
    /// description (or `an untyped row`).
    pub source_violation: Option<&'static str>,
    /// Endo-acyclicity flag (`endo_acyclicity_flagged`): `Some(flag)` when
    /// the morphism is structurally endo (source == target), `None` when
    /// not.
    pub endo_acyclic: Option<bool>,
}

/// Which row's kind column a refinement predicate reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The row carrying the reference (the Appears-on side).
    Source,
    /// The row the reference resolves to.
    Target,
}

/// A refinement predicate: `side`'s own kind column equals `kind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refinement {
    pub side: Side,
    pub kind: &'static str,
}

/// The source-side rule a morphism row enforces, as data. Three generic
/// rules cover every Reference Typing row; adding a reference field is
/// adding one row, never checker code (`typing_table_is_data`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRule {
    /// The source side is not consulted — target-side typing alone
    /// (traces_to, guard, emits, satisfies, observes, uses, from/to).
    Unchecked,
    /// The source must be a row of the morphism's source object — the
    /// Appears-on column read as normative (specodelic-huf); an untyped
    /// source row violates, and the rule fires before the target is
    /// considered.
    AppearsOn,
    /// The source must be of the same class as the target (the
    /// Constraint→Constraint / Property→Property rule). An untyped source
    /// row is not policed by it — a frontmatter supersedes link stays
    /// allowed (the current `let source = source_kind?` behaviour).
    SameKind,
}

/// One endpoint of a typed reference — the row's schema object, its own
/// kind cell (empty when the object carries none), and the human
/// description the violation reasons interpolate (`{tk}` / `{src}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub object: String,
    pub kind: String,
    pub describe: String,
}

/// A schema-level failure, named after the invariant it violates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    /// `objects_closed`: an object outside the closed five.
    ObjectOutsideClosedSet(ObjectId),
    /// `objects_closed`: one of the five objects is missing.
    MissingClosedObject(ObjectId),
    /// `morphisms_typed`: two rows share the (source, name) identity.
    DuplicateMorphism { source: ObjectId, name: String },
    /// `morphisms_typed`: a morphism with an empty name or an object
    /// reference outside the schema's objects.
    MalformedMorphism { name: String, reason: String },
    /// `endo_acyclicity_flagged`: a structurally endo morphism without a
    /// flag, or a flagged morphism that is not endo.
    InconsistentEndoFlag { name: String },
}

/// A cycle found through a flagged endo morphism: the rotation-normalized
/// node path (smallest id first, closing repeat omitted) — a cycle found
/// from any of its nodes reports once, matching `find_supersedes_cycles`'s
/// normalization discipline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    pub nodes: Vec<String>,
}

/// The canonical schema — the Reference Typing table of
/// `specs/specodelic.md` as data. The value `check` accepts and the
/// `schema_matches_typing_table` lint gate compares against the document.
pub fn canonical() -> Schema {
    let objects: BTreeSet<ObjectId> = ["Constraint", "Intent", "Property", "State", "Transition"]
        .iter()
        .map(|s| ObjectId::new(s))
        .collect();
    let o = ObjectId::new;
    // One row per (source object, target object) a column admits — the
    // Reference Typing table read as data. Authored in table order; the
    // canonical (source, name) sort below makes declaration order
    // irrelevant (`order_is_canonical`).
    //
    // The violation templates are the exact prose the graph fixtures pin
    // (byte parity with the retired match arms). Split columns share a
    // constant so their rows cannot drift apart.
    const DERIVES_FROM_TARGET_VIOLATION: &str = "derives_from must resolve to a Constraint, or to a Property when the source is a law (Reference Typing); target is {tk}";
    const DERIVES_FROM_SOURCE_VIOLATION: &str =
        "derives_from appears on Property rows only (Reference Typing); source is {src}";
    const GUARD_TARGET_VIOLATION: &str = "guard must resolve to an invariant Constraint or a State (Reference Typing); target is {tk}";
    const SUPERSEDES_TARGET_VIOLATION: &str = "supersedes must target the same kind as the row it appears on (Reference Typing); source is {src}, target is {tk}";
    let mut morphisms = vec![
        // traces_to | Constraint | Intent
        Morphism {
            column: "traces_to",
            name: "traces_to",
            source: o("Constraint"),
            target: o("Intent"),
            refinements: vec![],
            source_rule: SourceRule::Unchecked,
            target_violation: "traces_to must resolve to an Intent (Reference Typing); target is {tk}",
            source_violation: None,
            endo_acyclic: None,
        },
        // derives_from | Property | Constraint
        Morphism {
            column: "derives_from",
            name: "derives_from",
            source: o("Property"),
            target: o("Constraint"),
            refinements: vec![],
            source_rule: SourceRule::AppearsOn,
            target_violation: DERIVES_FROM_TARGET_VIOLATION,
            source_violation: Some(DERIVES_FROM_SOURCE_VIOLATION),
            endo_acyclic: None,
        },
        // derives_from | Property | the same Property when the deriving
        // row is itself a law — the endo case; the source-side law
        // refinement is the whole rule (Revision 10: any Property may be
        // restated); cycle policing stays with the linter's
        // acyclic_traces (flag false).
        Morphism {
            column: "derives_from",
            name: "derives_from_law",
            source: o("Property"),
            target: o("Property"),
            source_rule: SourceRule::AppearsOn,
            target_violation: DERIVES_FROM_TARGET_VIOLATION,
            source_violation: Some(DERIVES_FROM_SOURCE_VIOLATION),
            endo_acyclic: Some(false),
            refinements: vec![Refinement {
                side: Side::Source,
                kind: "law",
            }],
        },
        // guard | Transition | an invariant Constraint
        Morphism {
            column: "guard",
            name: "guard",
            source: o("Transition"),
            target: o("Constraint"),
            refinements: vec![Refinement {
                side: Side::Target,
                kind: "invariant",
            }],
            source_rule: SourceRule::Unchecked,
            target_violation: GUARD_TARGET_VIOLATION,
            source_violation: None,
            endo_acyclic: None,
        },
        // guard | Transition | a State — the "has reached state X"
        // pattern (Revision 12)
        Morphism {
            column: "guard",
            name: "guard_state",
            source: o("Transition"),
            target: o("State"),
            refinements: vec![],
            source_rule: SourceRule::Unchecked,
            target_violation: GUARD_TARGET_VIOLATION,
            source_violation: None,
            endo_acyclic: None,
        },
        // supersedes | Constraint | Constraint — flagged: cycles forbidden
        // (supersedes_acyclic)
        Morphism {
            column: "supersedes",
            name: "supersedes",
            source: o("Constraint"),
            target: o("Constraint"),
            refinements: vec![],
            source_rule: SourceRule::SameKind,
            target_violation: SUPERSEDES_TARGET_VIOLATION,
            source_violation: None,
            endo_acyclic: Some(true),
        },
        // supersedes | Property | Property — flagged the same way
        Morphism {
            column: "supersedes",
            name: "supersedes",
            source: o("Property"),
            target: o("Property"),
            refinements: vec![],
            source_rule: SourceRule::SameKind,
            target_violation: SUPERSEDES_TARGET_VIOLATION,
            source_violation: None,
            endo_acyclic: Some(true),
        },
        // emits | State | Constraint, kind == effect only
        Morphism {
            column: "emits",
            name: "emits",
            source: o("State"),
            target: o("Constraint"),
            refinements: vec![Refinement {
                side: Side::Target,
                kind: "effect",
            }],
            source_rule: SourceRule::Unchecked,
            target_violation: "emits must resolve to an effect Constraint (Reference Typing); target is {tk}",
            source_violation: None,
            endo_acyclic: None,
        },
        // satisfies | Constraint | Constraint, kind == extension_point
        // only — structurally endo; mutual cross-file claims are
        // well-formed, no acyclicity check at this layer.
        Morphism {
            column: "satisfies",
            name: "satisfies",
            source: o("Constraint"),
            target: o("Constraint"),
            refinements: vec![Refinement {
                side: Side::Target,
                kind: "extension_point",
            }],
            source_rule: SourceRule::Unchecked,
            target_violation: "satisfies must resolve to an extension_point Constraint (Reference Typing); target is {tk}",
            source_violation: None,
            endo_acyclic: Some(false),
        },
        // observes | Constraint | Constraint, kind == effect only —
        // structurally endo; mutual cross-file observation is
        // well-formed (the table says so of record).
        Morphism {
            column: "observes",
            name: "observes",
            source: o("Constraint"),
            target: o("Constraint"),
            refinements: vec![Refinement {
                side: Side::Target,
                kind: "effect",
            }],
            source_rule: SourceRule::Unchecked,
            target_violation: "observes must resolve to an effect Constraint (Reference Typing); target is {tk}",
            source_violation: None,
            endo_acyclic: Some(false),
        },
        // uses | Constraint | Intent of a kind: profile file (Revision 14)
        Morphism {
            column: "uses",
            name: "uses",
            source: o("Constraint"),
            target: o("Intent"),
            refinements: vec![Refinement {
                side: Side::Target,
                kind: "profile",
            }],
            source_rule: SourceRule::Unchecked,
            target_violation: "uses must resolve to a profile Intent (Reference Typing); target is {tk}",
            source_violation: None,
            endo_acyclic: None,
        },
        // from / to | Transition | State — the transitions walk owns these
        // fields (graph.rs does not read them as links; the rows exist for
        // table equality and the instance builder's from/to edges).
        Morphism {
            column: "from",
            name: "from",
            source: o("Transition"),
            target: o("State"),
            refinements: vec![],
            source_rule: SourceRule::Unchecked,
            target_violation: "from must resolve to a State (Reference Typing); target is {tk}",
            source_violation: None,
            endo_acyclic: None,
        },
        Morphism {
            column: "to",
            name: "to",
            source: o("Transition"),
            target: o("State"),
            refinements: vec![],
            source_rule: SourceRule::Unchecked,
            target_violation: "to must resolve to a State (Reference Typing); target is {tk}",
            source_violation: None,
            endo_acyclic: None,
        },
    ];
    morphisms.sort_by(|a, b| (&a.source, a.name).cmp(&(&b.source, b.name)));
    Schema { objects, morphisms }
}

/// The schema-level check: objects closed to the exact five, every morphism
/// named and pointing at declared objects, (source, name) unique, endo
/// flags consistent with structure. `Ok(())` for the canonical schema.
pub fn check(schema: &Schema) -> Result<(), Vec<SchemaError>> {
    let mut errs = Vec::new();
    // objects_closed: the objects are exactly the closed five.
    const CLOSED: [&str; 5] = ["Constraint", "Intent", "Property", "State", "Transition"];
    for id in CLOSED {
        if !schema.objects.contains(&ObjectId::new(id)) {
            errs.push(SchemaError::MissingClosedObject(ObjectId::new(id)));
        }
    }
    for o in &schema.objects {
        if !CLOSED.contains(&o.0.as_str()) {
            errs.push(SchemaError::ObjectOutsideClosedSet(o.clone()));
        }
    }
    // morphisms_typed: named, pointing at declared objects, (source, name)
    // unique; endo_acyclicity_flagged: the flag matches the structure.
    let mut seen: BTreeSet<(&ObjectId, &str)> = BTreeSet::new();
    for m in &schema.morphisms {
        if m.name.is_empty() {
            errs.push(SchemaError::MalformedMorphism {
                name: m.name.to_string(),
                reason: "empty name".to_string(),
            });
        }
        for (role, obj) in [("source", &m.source), ("target", &m.target)] {
            if !schema.objects.contains(obj) {
                errs.push(SchemaError::MalformedMorphism {
                    name: m.name.to_string(),
                    reason: format!("{} object {} is not a declared object", role, obj.0),
                });
            }
        }
        if !seen.insert((&m.source, m.name)) {
            errs.push(SchemaError::DuplicateMorphism {
                source: m.source.clone(),
                name: m.name.to_string(),
            });
        }
        let is_endo = m.source == m.target;
        if is_endo != m.endo_acyclic.is_some() {
            errs.push(SchemaError::InconsistentEndoFlag {
                name: m.name.to_string(),
            });
        }
    }
    if errs.is_empty() { Ok(()) } else { Err(errs) }
}

/// Generic acyclicity over a flagged endo morphism: `Err` only when the
/// named morphism carries `Some(true)` and the edge set contains a cycle —
/// an unflagged (or flag-absent) morphism is never cycle-checked
/// (`unflagged_endo_cycle_tolerated`).
pub fn check_endo_acyclicity(
    schema: &Schema,
    morphism_name: &str,
    edges: &[(ObjectId, ObjectId)],
) -> Result<(), Vec<Cycle>> {
    // Only flagged morphisms are cycle-checked — an unflagged endo case
    // (or a name matching no row) is never a cycle report.
    let flagged = schema
        .morphisms
        .iter()
        .any(|m| m.name == morphism_name && m.endo_acyclic == Some(true));
    if !flagged {
        return Ok(());
    }
    // Adjacency over the morphism's defined values, deterministic.
    let mut adj: BTreeMap<&ObjectId, BTreeSet<&ObjectId>> = BTreeMap::new();
    for (from, to) in edges {
        adj.entry(from).or_default().insert(to);
    }
    // Depth-first search with Gray/Black marking: an edge back into the
    // current path (Gray) closes a cycle — extracted from the path,
    // rotation-normalized smallest-node-first, deduplicated. Visits each
    // node at most once per start, so termination is guaranteed.
    #[derive(PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color: BTreeMap<&ObjectId, Color> = adj.keys().map(|k| (*k, Color::White)).collect();
    let mut found: BTreeSet<Vec<String>> = BTreeSet::new();
    fn dfs<'a>(
        node: &'a ObjectId,
        path: &mut Vec<&'a ObjectId>,
        color: &mut BTreeMap<&'a ObjectId, Color>,
        adj: &BTreeMap<&'a ObjectId, BTreeSet<&'a ObjectId>>,
        found: &mut BTreeSet<Vec<String>>,
    ) {
        color.insert(node, Color::Gray);
        path.push(node);
        if let Some(nexts) = adj.get(node) {
            for next in nexts {
                match color.get(*next) {
                    None | Some(Color::Black) => {}
                    Some(Color::White) => dfs(next, path, color, adj, found),
                    Some(Color::Gray) => {
                        // Cycle: the path segment from `next` to the
                        // current node.
                        let idx = path
                            .iter()
                            .position(|n| *n == *next)
                            .expect("gray is on path");
                        let cyc: Vec<String> = path[idx..].iter().map(|n| n.0.clone()).collect();
                        // Rotation-normalize: smallest node first.
                        let min = cyc
                            .iter()
                            .enumerate()
                            .min_by(|(_, a), (_, b)| a.cmp(b))
                            .map(|(i, _)| i)
                            .expect("cycle is non-empty");
                        found.insert(
                            cyc.iter()
                                .cycle()
                                .skip(min)
                                .take(cyc.len())
                                .cloned()
                                .collect(),
                        );
                    }
                }
            }
        }
        path.pop();
        color.insert(node, Color::Black);
    }
    for start in adj.keys().copied().collect::<Vec<_>>() {
        if color.get(start) == Some(&Color::White) {
            let mut path = Vec::new();
            dfs(start, &mut path, &mut color, &adj, &mut found);
        }
    }
    if found.is_empty() {
        Ok(())
    } else {
        Err(found.into_iter().map(|nodes| Cycle { nodes }).collect())
    }
}

/// The Reference Typing check for one resolved reference, decided from the
/// schema's rows — the generic checker the retired per-field match arms in
/// `graph.rs::typing_violation` are replaced by (`typing_table_is_data`):
/// adding a reference field is adding one row, never checker code.
///
/// Evaluation order (pinned byte-for-byte by the graph fixtures):
///
/// 1. A column with no rows is not a typed reference field — allowed.
/// 2. `AppearsOn` columns police the source before the target is even
///    considered (specodelic-huf): a source that cannot sit on any of the
///    column's source objects — including an untyped source row — fires
///    the column's source-side reason.
/// 3. The rows are consulted in canonical order: a row accepts the edge
///    when the target's object and the row's target-side refinements and
///    source-side refinements all hold, and the row's `SourceRule` is
///    satisfied. No row accepting composes the column's target-side
///    reason from its template.
pub fn typing_violation(
    schema: &Schema,
    column: &str,
    source: Option<Endpoint>,
    target: Endpoint,
) -> Option<String> {
    let rows: Vec<&Morphism> = schema
        .morphisms
        .iter()
        .filter(|m| m.column == column)
        .collect();
    if rows.is_empty() {
        return None;
    }
    if rows.iter().any(|r| r.source_rule == SourceRule::AppearsOn) {
        let source_ok = source.as_ref().is_some_and(|s| {
            rows.iter()
                .any(|r| r.source_rule == SourceRule::AppearsOn && s.object == r.source.0)
        });
        if !source_ok {
            let src = source
                .map(|s| s.describe)
                .unwrap_or_else(|| "an untyped row".to_string());
            let template = rows
                .iter()
                .find_map(|r| r.source_violation)
                .expect("AppearsOn rows carry a source template");
            return Some(template.replace("{src}", &src));
        }
    }
    for row in &rows {
        // Target object + target-side refinements.
        if target.object != row.target.0 {
            continue;
        }
        if !row
            .refinements
            .iter()
            .filter(|r| r.side == Side::Target)
            .all(|r| target.kind == r.kind)
        {
            continue;
        }
        // Source-side refinements (the derives_from law case).
        if !row
            .refinements
            .iter()
            .filter(|r| r.side == Side::Source)
            .all(|r| source.as_ref().is_some_and(|s| s.kind == r.kind))
        {
            continue;
        }
        match row.source_rule {
            SourceRule::Unchecked => return None,
            SourceRule::AppearsOn => {
                if source.as_ref().is_some_and(|s| s.object == row.source.0) {
                    return None;
                }
                continue;
            }
            SourceRule::SameKind => match source {
                // An untyped source row is not policed by the same-kind
                // rule (current `let source = source_kind?` behaviour).
                None => return None,
                Some(s) if s.object == target.object => return None,
                Some(_) => continue,
            },
        }
    }
    let reason = rows[0]
        .target_violation
        .replace(
            "{src}",
            &source.map_or_else(|| "an untyped row".to_string(), |s| s.describe),
        )
        .replace("{tk}", &target.describe);
    Some(reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `sixth_object_rejected` (acset-core properties table): a schema
    /// carrying a sixth object fails `check`, naming the closed-set
    /// violation — the object set grows only under a new Revision heading.
    #[test]
    fn sixth_object_rejected() {
        let mut s = canonical();
        s.objects.insert(ObjectId::new("Action"));
        let errs = check(&s).expect_err("a sixth object must fail check");
        assert!(
            errs.iter()
                .any(|e| matches!(e, SchemaError::ObjectOutsideClosedSet(id) if id.0 == "Action")),
            "expected ObjectOutsideClosedSet(Action), got {errs:?}"
        );
    }

    /// `duplicate_morphism_rejected`: two rows with the same (source, name)
    /// identity fail `check` — the pair is unique across the schema.
    #[test]
    fn duplicate_morphism_rejected() {
        let mut s = canonical();
        let dup = s.morphisms[0].clone();
        s.morphisms.push(dup);
        let errs = check(&s).expect_err("a duplicated (source, name) must fail check");
        assert!(
            errs.iter()
                .any(|e| matches!(e, SchemaError::DuplicateMorphism { .. })),
            "expected DuplicateMorphism, got {errs:?}"
        );
    }

    /// `endo_cycle_detected`: a cycle through a flagged endo morphism
    /// (supersedes a→b→a) is reported — acyclicity is checked generically
    /// for flagged morphisms.
    #[test]
    fn endo_cycle_detected() {
        let s = canonical();
        let edges = vec![
            (ObjectId::new("a"), ObjectId::new("b")),
            (ObjectId::new("b"), ObjectId::new("a")),
        ];
        let cycles =
            check_endo_acyclicity(&s, "supersedes", &edges).expect_err("a→b→a must be a cycle");
        assert_eq!(cycles.len(), 1, "one cycle, reported once: {cycles:?}");
        let path = &cycles[0].nodes;
        assert_eq!(path, &["a".to_string(), "b".to_string()]);
    }

    /// `unflagged_endo_cycle_tolerated`: a cycle through an unflagged endo
    /// morphism (a law deriving from a law — even a self-edge) is NOT
    /// reported; only flagged morphisms are cycle-checked.
    #[test]
    fn unflagged_endo_cycle_tolerated() {
        let s = canonical();
        let self_edge = [(ObjectId::new("l1"), ObjectId::new("l1"))];
        check_endo_acyclicity(&s, "derives_from_law", &self_edge)
            .expect("the derives_from law case is unflagged — cycles tolerated");
    }

    /// The canonical schema itself is valid — the precondition every fixture
    /// (a canonical clone) inherits, and the value the lint drift gate
    /// (task 2.4) compares against the document.
    #[test]
    fn canonical_schema_is_valid() {
        let s = canonical();
        check(&s).expect("the canonical schema must pass check");
        // Canonical order: morphisms sorted by (source, name).
        let mut sorted = s.morphisms.clone();
        sorted.sort_by(|a, b| (&a.source, a.name).cmp(&(&b.source, b.name)));
        assert_eq!(s.morphisms, sorted, "morphisms must be in canonical order");
        // The five objects, exactly.
        let ids: Vec<String> = s.objects.iter().map(|o| o.0.clone()).collect();
        assert_eq!(
            ids,
            ["Constraint", "Intent", "Property", "State", "Transition"],
            "the closed five, canonically sorted"
        );
    }

    // ---------------------------------------------------------------------------
    // Typing read from the Schema (task 2.3 — RED first). The generic
    // checker evaluates the schema's rows; the per-column match arms in
    // graph.rs's typing_violation are retired.
    // ---------------------------------------------------------------------------

    fn endpoint(object: &str, kind: &str, describe: &str) -> Endpoint {
        Endpoint {
            object: object.to_string(),
            kind: kind.to_string(),
            describe: describe.to_string(),
        }
    }

    /// `typing_read_from_schema`: a new reference field added as one schema
    /// row decides typing for that field with no change to checker code —
    /// and a column with no rows is not a typed reference field.
    #[test]
    fn typing_read_from_schema() {
        let mut s = canonical();
        s.morphisms.push(Morphism {
            column: "backs",
            name: "backs",
            source: ObjectId::new("State"),
            target: ObjectId::new("Intent"),
            refinements: vec![],
            endo_acyclic: None,
            source_rule: SourceRule::Unchecked,
            target_violation: "backs must resolve to an Intent (Reference Typing); target is {tk}",
            source_violation: None,
        });
        let source_state = Some(endpoint("State", "", "a State"));
        assert_eq!(
            typing_violation(
                &s,
                "backs",
                source_state,
                endpoint("Intent", "", "an Intent")
            ),
            None,
            "the added row admits State→Intent with no checker change"
        );
        assert_eq!(
            typing_violation(&s, "backs", None, endpoint("State", "", "a State")),
            Some(
                "backs must resolve to an Intent (Reference Typing); target is a State".to_string()
            ),
            "the added row's own template composes the reason"
        );
        assert_eq!(
            typing_violation(&s, "no_such_column", None, endpoint("State", "", "a State")),
            None,
            "a column with no rows is not a typed reference field"
        );
    }

    /// `emits_refinement_enforced`: the effect-only rule comes from the
    /// declared refinement on the emits row — an emits edge to an invariant
    /// Constraint fails, to an effect Constraint passes.
    #[test]
    fn emits_refinement_enforced() {
        let s = canonical();
        let source_state = Some(endpoint("State", "", "a State"));
        assert_eq!(
            typing_violation(
                &s,
                "emits",
                source_state.clone(),
                endpoint("Constraint", "invariant", "a Constraint (kind `invariant`)"),
            ),
            Some(
                "emits must resolve to an effect Constraint (Reference Typing); target is a Constraint (kind `invariant`)"
                    .to_string()
            ),
        );
        assert_eq!(
            typing_violation(
                &s,
                "emits",
                source_state,
                endpoint("Constraint", "effect", "a Constraint (kind `effect`)"),
            ),
            None,
        );
    }

    /// Byte-parity pins: the generic checker composes the exact reasons the
    /// graph fixtures pin (tests/snapshots/typing_violations.txt), including
    /// the huf order (derives_from's appears-on rule fires before the
    /// target is considered) and the supersedes same-kind rule.
    #[test]
    fn typing_reasons_match_pinned_prose() {
        let s = canonical();
        let constraint = endpoint("Constraint", "", "a Constraint");
        let property = endpoint("Property", "", "a Property");
        let law = endpoint("Property", "law", "a Property (kind `law`)");
        // traces_to: the source side is not consulted (a properties row
        // carrying traces_to is typed on the target alone).
        assert_eq!(
            typing_violation(&s, "traces_to", Some(property.clone()), constraint.clone()),
            Some(
                "traces_to must resolve to an Intent (Reference Typing); target is a Constraint"
                    .to_string()
            ),
        );
        assert_eq!(
            typing_violation(
                &s,
                "traces_to",
                Some(property),
                endpoint("Intent", "", "an Intent")
            ),
            None,
        );
        // derives_from: appears-on fires first, even over a matching target.
        assert_eq!(
            typing_violation(&s, "derives_from", Some(constraint.clone()), constraint.clone()),
            Some("derives_from appears on Property rows only (Reference Typing); source is a Constraint".to_string()),
        );
        assert_eq!(
            typing_violation(&s, "derives_from", None, constraint),
            Some("derives_from appears on Property rows only (Reference Typing); source is an untyped row".to_string()),
        );
        // The law case: law→law allowed, and (Revision 10) a law may
        // restate any Property — the source-side refinement is the whole
        // rule; a non-law source over a Property target is rejected.
        assert_eq!(
            typing_violation(&s, "derives_from", Some(law.clone()), law.clone()),
            None,
            "a law derives from a law"
        );
        let unit_law = endpoint("Property", "unit", "a Property (kind `unit`)");
        assert_eq!(
            typing_violation(&s, "derives_from", Some(law), unit_law),
            None,
            "Revision 10: a law restates any Property"
        );
        // guard: the split column admits a State (Revision 12).
        let source_transition = Some(endpoint("Transition", "", "a Transition"));
        assert_eq!(
            typing_violation(
                &s,
                "guard",
                source_transition.clone(),
                endpoint("State", "", "a State")
            ),
            None,
        );
        assert_eq!(
            typing_violation(
                &s,
                "guard",
                source_transition,
                endpoint("Constraint", "advisory", "a Constraint (kind `advisory`)"),
            ),
            Some("guard must resolve to an invariant Constraint or a State (Reference Typing); target is a Constraint (kind `advisory`)".to_string()),
        );
        // supersedes: the same-kind rule, with the exact combined reason.
        assert_eq!(
            typing_violation(
                &s,
                "supersedes",
                Some(endpoint("Constraint", "", "a Constraint")),
                endpoint("Constraint", "", "a Constraint")
            ),
            None,
        );
        assert_eq!(
            typing_violation(&s, "supersedes", Some(endpoint("Constraint", "", "a Constraint")), endpoint("Property", "", "a Property")),
            Some("supersedes must target the same kind as the row it appears on (Reference Typing); source is a Constraint, target is a Property".to_string()),
        );
        // An untyped source row is not policed by the same-kind rule
        // (frontmatter supersedes links stay allowed — current behaviour).
        assert_eq!(
            typing_violation(
                &s,
                "supersedes",
                None,
                endpoint("Constraint", "", "a Constraint")
            ),
            None,
        );
        // satisfies/observes: extension_point-only / effect-only.
        assert_eq!(
            typing_violation(&s, "satisfies", Some(endpoint("Constraint", "", "a Constraint")), endpoint("Constraint", "effect", "a Constraint (kind `effect`)")),
            Some("satisfies must resolve to an extension_point Constraint (Reference Typing); target is a Constraint (kind `effect`)".to_string()),
        );
        assert_eq!(
            typing_violation(&s, "observes", Some(endpoint("Constraint", "", "a Constraint")), endpoint("Constraint", "", "a Constraint")),
            Some("observes must resolve to an effect Constraint (Reference Typing); target is a Constraint".to_string()),
        );
    }
}

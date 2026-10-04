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
    let mut morphisms = vec![
        // traces_to | Constraint | Intent
        Morphism {
            column: "traces_to",
            name: "traces_to",
            source: o("Constraint"),
            target: o("Intent"),
            refinements: vec![],
            endo_acyclic: None,
        },
        // derives_from | Property | Constraint
        Morphism {
            column: "derives_from",
            name: "derives_from",
            source: o("Property"),
            target: o("Constraint"),
            refinements: vec![],
            endo_acyclic: None,
        },
        // derives_from | Property | the same Property when the deriving
        // row is itself a law — the endo case; cycle policing stays with
        // the linter's acyclic_traces (flag false).
        Morphism {
            column: "derives_from",
            name: "derives_from_law",
            source: o("Property"),
            target: o("Property"),
            refinements: vec![
                Refinement {
                    side: Side::Source,
                    kind: "law",
                },
                Refinement {
                    side: Side::Target,
                    kind: "law",
                },
            ],
            endo_acyclic: Some(false),
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
            endo_acyclic: Some(true),
        },
        // supersedes | Property | Property — flagged the same way
        Morphism {
            column: "supersedes",
            name: "supersedes",
            source: o("Property"),
            target: o("Property"),
            refinements: vec![],
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
            endo_acyclic: None,
        },
        Morphism {
            column: "to",
            name: "to",
            source: o("Transition"),
            target: o("State"),
            refinements: vec![],
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
}

//! The row-shape rule (`linter.row_shape_closed`) — a 𝒦 row's field set
//! is closed.
//!
//! Purpose: decide the shape claims `openspec/specs/kinds/spec.md` makes
//! about the five kinds' rows — a Constraint row's fields are exactly
//! {id, kind, expr} plus the typed reference columns the Reference
//! Typing table declares, a State row's are exactly {id, emits?}, and
//! so on — as a lint finding instead of an unchecked prose claim.
//! Responsibilities: compute each kind's allowed field set and report
//! every column a row carries outside it. Rationale: the reference
//! columns are read from the Schema-as-data (the format's own
//! Reference Typing table as one value), so a new reference field
//! widens the allowed set with no change to this checker — a hardcoded
//! per-field list would drift from the table it claims to mirror.
//!
//! The base (non-reference) fields are the per-kind constants below —
//! the same shape the corpus states in `kinds.md`'s four `*_row_shape`
//! constraints, and the same precedent `guide::CONSTRAINT_KINDS` sets
//! for the closed kind sets: a small, explicit table beside the
//! checker that reads it.

use std::collections::{BTreeMap, BTreeSet};

use super::super::spec::Spec;
use super::{Issue, Report};

/// The base (non-reference) fields of each schema object — the fields
/// a row carries besides the typed reference columns the Reference
/// Typing table declares.
///
/// `Constraint`: {id, kind, expr} — `traces_to` and the other typed
/// reference columns come from the Schema-as-data.
/// `Property`: {id, kind, generator, predicate} — `derives_from` comes
/// from the Schema-as-data.
/// `State`: {id} — `emits` comes from the Schema-as-data.
/// `Transition`: {id} — `guard`, `from`, `to` come from the
/// Schema-as-data.
///
/// The Intent (the frontmatter block) is NOT in this table: its field
/// set is {id, kind, statement} plus the optionally-declared
/// frontmatter keys `kinds.md` allows (`checked_against_core`), which
/// are per-file conventions rather than a closed kind shape.
const BASE_ROW_FIELDS: &[(&str, &[&str])] = &[
    ("Constraint", &["id", "kind", "expr"]),
    ("Property", &["id", "kind", "generator", "predicate"]),
    ("State", &["id"]),
    ("Transition", &["id"]),
];

/// The allowed field set of one schema object: its base fields plus
/// the typed reference columns the Schema-as-data declares for it.
///
/// The reference half is read live from
/// [`crate::acset::schema::canonical`] — the Reference Typing table of
/// `openspec/specs/specodelic/spec.md` as one value — so a new
/// reference field (a `uses` edge, a `satisfies` edge) widens the
/// allowed set by adding one Schema row, never by editing this
/// checker.
pub fn allowed_fields(object: &str) -> BTreeSet<String> {
    let mut fields: BTreeSet<String> = BASE_ROW_FIELDS
        .iter()
        .find(|(name, _)| *name == object)
        .map(|(_, base)| base.iter().map(|f| f.to_string()).collect())
        .unwrap_or_default();
    for morphism in &crate::acset::schema::canonical().morphisms {
        if morphism.source.0 == object {
            fields.insert(morphism.column.to_string());
        }
    }
    fields
}

/// The namespaces of the discovered packs — a pack's namespace is its
/// id's first segment (a dotless id names its namespace directly).
fn discovered_pack_namespaces(corpus: &[Spec]) -> BTreeSet<String> {
    crate::packs::discover(corpus)
        .iter()
        .map(|p| p.id.split('.').next().unwrap_or(&p.id).to_string())
        .collect()
}

/// Is this column pack-qualified vocabulary whose namespace has a
/// discovered pack? The kernel pack's `kernel.binding` on the
/// Constraints table is the working example: the pack owns the column,
/// the namespace is the pack's, and the orphan-vocabulary walk treats
/// the same token as declared rather than orphaned.
fn is_pack_qualified(column: &str, pack_namespaces: &BTreeSet<String>) -> bool {
    crate::packs::namespace_of(column).is_some_and(|ns| pack_namespaces.contains(&ns))
}

/// The corpus constraint that states one kind's row shape — the
/// `kinds.md` row the finding cites.
fn shape_constraint(object: &str) -> &'static str {
    match object {
        "Constraint" => "constraint_row_shape",
        "Property" => "property_row_shape",
        "State" => "state_row_shape",
        "Transition" => "transition_row_shape",
        _ => "kind_enum_closed",
    }
}

/// The row-shape family: every row of the file carries only the fields
/// its kind declares.
pub(crate) fn lint_row_shape(spec: &Spec, corpus: &[Spec], report: &mut Report) {
    let pack_namespaces = discovered_pack_namespaces(corpus);
    let file = spec
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| spec.intent.id.clone());

    // Constraint rows: every table column lands in the row's cells.
    let constraints: Vec<(String, BTreeSet<String>)> = spec
        .constraints
        .iter()
        .map(|r| (r.id.clone(), r.cells.keys().cloned().collect()))
        .collect();
    check_kind(&file, "Constraint", &constraints, &pack_namespaces, report);

    // Property rows: same shape.
    let properties: Vec<(String, BTreeSet<String>)> = spec
        .properties
        .iter()
        .map(|r| (r.id.clone(), r.cells.keys().cloned().collect()))
        .collect();
    check_kind(&file, "Property", &properties, &pack_namespaces, report);

    // State rows: the bullet's fields (id, the optional emits, and any
    // other named field the bullet carries).
    let states: Vec<(String, BTreeSet<String>)> = spec
        .states
        .iter()
        .map(|r| (r.id.clone(), r.cells.keys().cloned().collect()))
        .collect();
    check_kind(&file, "State", &states, &pack_namespaces, report);

    // Transition rows: a Transition row keeps no cell map — its
    // columns are the keys of its cell spans.
    let transitions: Vec<(String, BTreeSet<String>)> = spec
        .transitions
        .iter()
        .map(|t| (t.id.clone(), t.cell_spans.keys().cloned().collect()))
        .collect();
    check_kind(&file, "Transition", &transitions, &pack_namespaces, report);
}

/// Report every field the rows of one kind carry outside its allowed
/// set — one finding per undeclared field, naming the first row that
/// carries it (the table's column set, read as the union of its rows'
/// columns).
fn check_kind(
    file: &str,
    object: &str,
    rows: &[(String, BTreeSet<String>)],
    pack_namespaces: &BTreeSet<String>,
    report: &mut Report,
) {
    if rows.is_empty() {
        return;
    }
    let allowed = allowed_fields(object);
    let mut carriers: BTreeMap<String, String> = BTreeMap::new();
    for (row_id, columns) in rows {
        for column in columns {
            carriers
                .entry(column.clone())
                .or_insert_with(|| row_id.clone());
        }
    }
    for (column, row_id) in carriers {
        if allowed.contains(&column) || is_pack_qualified(&column, pack_namespaces) {
            continue;
        }
        let allowed_list = allowed
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        report.issues.push(Issue::new(
            "row_shape_closed",
            file.to_string(),
            format!(
                "{object} row `{row_id}` carries the field `{column}`, which the {object} row shape does not declare — a {object} row's fields are exactly {{{allowed_list}}} (openspec/specs/kinds/spec.md {})",
                shape_constraint(object)
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::test_panic::UnwrapPanic;

    use super::*;

    /// The reference half of the allowed set comes from the
    /// Schema-as-data: a Constraint row may carry every typed reference
    /// column the Reference Typing table declares for it.
    #[test]
    fn the_allowed_fields_follow_the_schema_as_data() {
        let constraint = allowed_fields("Constraint");
        for column in ["traces_to", "supersedes", "satisfies", "observes", "uses"] {
            assert!(
                constraint.contains(column),
                "the Constraint row shape must allow the `{column}` reference column: {constraint:?}"
            );
        }
        assert!(constraint.contains("expr"));
        assert!(!constraint.contains("guard"), "guard is a Transition field");
        assert!(!allowed_fields("State").contains("kind"));
        assert!(allowed_fields("Property").contains("predicate"));
        assert!(allowed_fields("Transition").contains("from"));
    }

    /// An extra Constraint column is a finding; the clean shape is not.
    #[test]
    fn an_extra_constraint_column_is_rejected() {
        let text = "---\nid: fx\nkind: intent\nstatement: \"THE fx SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | guard |\n|----|------|------|-----------|-------|\n| c1 | invariant | `holds` | [[fx]] | |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s1 | [[fx.c1]] |\n";
        let spec = crate::spec::parse_str(text).unwrap_p();
        let mut report = Report {
            files_linted: 1,
            issues: vec![],
            warnings: vec![],
            checklists_declared: 0,
            packs: vec![],
        };
        lint_row_shape(&spec, &[], &mut report);
        assert_eq!(report.issues.len(), 1, "{:?}", report.issues);
        assert!(
            report.issues[0].message.contains("guard"),
            "{:?}",
            report.issues
        );
    }

    /// A State bullet carrying an undeclared named field is a finding.
    #[test]
    fn an_extra_state_field_is_rejected() {
        let text = "---\nid: fx\nkind: intent\nstatement: \"THE fx SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[fx]] |\n\n## Model\n\n### States\n\n- s1\n- s2 (kind: moore)\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[fx.c1]] |\n";
        let spec = crate::spec::parse_str(text).unwrap_p();
        let mut report = Report {
            files_linted: 1,
            issues: vec![],
            warnings: vec![],
            checklists_declared: 0,
            packs: vec![],
        };
        lint_row_shape(&spec, &[], &mut report);
        assert_eq!(report.issues.len(), 1, "{:?}", report.issues);
        assert!(
            report.issues[0].message.contains("kind"),
            "{:?}",
            report.issues
        );
    }
}

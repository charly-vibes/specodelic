//! Observability and coverage beats of the linter (split from mod.rs —
//! specodelic-g17 file_lines ratchet).

use std::collections::BTreeSet;

use super::super::spec::{Link, Spec};
use super::graph::RowKind;
use super::{Issue, Report};

pub(crate) fn lint_observability(specs: &[Spec], report: &mut Report) {
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
pub(crate) fn lint_coverage(specs: &[Spec], report: &mut Report) {
    // Target-kind resolution (specodelic-rk3): linter-coverage.md's
    // no_orphan_property invariant reads "p.derives_from resolves to a
    // real constraint" — a link's PRESENCE is not enough. Map
    // (file_id, row_id) -> row kind so a derives_from target that
    // resolves to a Property row can be told from one that resolves to
    // a Constraint. Unresolved targets stay total_refs' beat.
    let mut row_kinds: std::collections::BTreeMap<(String, String), RowKind> =
        std::collections::BTreeMap::new();
    for spec in specs {
        let file_id = &spec.intent.id;
        for c in &spec.constraints {
            row_kinds.insert((file_id.clone(), c.id.clone()), RowKind::Constraint);
        }
        for p in &spec.properties {
            row_kinds.insert(
                (file_id.clone(), p.id.clone()),
                if p.kind.as_deref() == Some("law") {
                    RowKind::LawProperty
                } else {
                    RowKind::Property
                },
            );
        }
    }
    // Resolve a derives_from target the way total_refs does: `file.row`
    // looks up that file; a bare `row` tries the own file first, then the
    // corpus (same leniency as lint_references for non-`spec` ids).
    let resolves_to_property =
        |file_id: &str,
         target: &str,
         row_kinds: &std::collections::BTreeMap<(String, String), RowKind>|
         -> Option<bool> {
            let kind = if let Some((f, r)) = target.rsplit_once('.') {
                row_kinds.get(&(f.to_string(), r.to_string()))
            } else {
                row_kinds
                    .get(&(file_id.to_string(), target.to_string()))
                    .or_else(|| {
                        row_kinds
                            .iter()
                            .find(|((_, rid), _)| rid == target)
                            .map(|(_, k)| k)
                    })
            }?;
            Some(matches!(kind, RowKind::Property | RowKind::LawProperty))
        };
    for spec in specs {
        let file_id = &spec.intent.id;
        // Constraint ids this file defines (local row ids).
        let constraint_ids: BTreeSet<&str> =
            spec.constraints.iter().map(|c| c.id.as_str()).collect();
        // Property ids this file defines (local row ids).
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
        for p in &spec.properties {
            let links: Vec<&Link> = spec
                .links
                .iter()
                .filter(|l| {
                    l.source == p.id && l.field == "properties" && l.column == "derives_from"
                })
                .collect();
            if links.is_empty() {
                report.issues.push(Issue::new(
                    "no_orphan_property",
                    file_id.clone(),
                    format!(
                        "property `{}` derives from nothing — derives_from takes a wiki-link like `[[{file_id}.<constraint-id>]]` (in an id:spec file the bare row form `[[<constraint-id>]]` resolves too)",
                        p.id
                    ),
                ));
                continue;
            }
            // The presence check passed; now the target kind (rk3). A
            // law property may derive from another Property (the cxq
            // law-restates-law edge); any other property deriving from a
            // Property is not coverage — the invariant requires a real
            // constraint.
            if p.kind.as_deref() != Some("law") {
                for link in links {
                    if resolves_to_property(file_id, &link.target, &row_kinds) == Some(true) {
                        report.issues.push(Issue::new(
                            "no_orphan_property",
                            file_id.clone(),
                            format!(
                                "property `{}`, deriving from `{}`, a property — no_orphan_property requires a real constraint (only a `law` property may derive from a property, the law-restates-law edge); write `[[{file_id}.<constraint-id>]]`",
                                p.id,
                                link.target
                            ),
                        ));
                    }
                }
            }
        }

        // every_law_has_cases (specodelic.md Revision 13) — a law-kind
        // row's required cases are whatever its predicate enumerates as
        // `**name:**` case labels, parsed by spec::law_case_labels (the
        // same helper compile's block expansion consumes — the compiler
        // and the linter cannot disagree on what a case is). The
        // identity and associativity floor is mandatory; extra labels
        // are first-class checkable declarations. A prose mention of a
        // case name is not an enumeration.
        for p in &spec.properties {
            if p.kind.as_deref() != Some("law") {
                continue;
            }
            let predicate = p.cells.get("predicate").cloned().unwrap_or_default();
            let labels: BTreeSet<String> = crate::spec::law_case_labels(&predicate)
                .into_iter()
                .map(|c| c.to_lowercase())
                .collect();
            let missing: Vec<&str> = ["identity", "associativity"]
                .iter()
                .filter(|c| !labels.contains(**c))
                .copied()
                .collect();
            if !missing.is_empty() {
                report.issues.push(Issue::new(
                    "law_cases",
                    file_id.clone(),
                    format!(
                        "law property `{}` does not enumerate its required cases in machine-findable form — missing floor case(s) {missing:?}: write `**identity:** … **associativity:** …` case labels in the predicate (specodelic.md Revision 13; a prose mention of a case name is not an enumeration)",
                        p.id
                    ),
                ));
            }
        }
    }
}

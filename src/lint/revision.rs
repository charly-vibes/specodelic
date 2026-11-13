//! The revision-pair seam — `append_only_variants` read across two
//! revisions of one file.
//!
//! Purpose: decide the claim `openspec/specs/specodelic/spec.md`
//! `append_only_variants` makes — across revisions r < r', a 𝒦-governed
//! id-set grows only under a new `## Revision N` heading and never
//! loses a member — as a callable seam over a (prior, current) text
//! pair. Responsibilities: parse both revisions, compute the governed
//! id-sets of each, diff them, and report the violations. Rationale:
//! the claim is about two revisions of one file, so it cannot be a
//! per-file lint rule (the linter sees one file at a time); it is a
//! seam the property rows call, exactly like the row-shape rule is a
//! seam the same rows call.
//!
//! The governed id-sets are the ones `append_only_variants` names:
//! a variant table's ids (chiefly the States), a Constraint row's own
//! `kind` value-set, a Property row's own `kind` value-set, and the
//! Reference Typing table's field set.

use std::collections::{BTreeMap, BTreeSet};

use crate::spec::{Spec, parse_str};

/// What happened to a governed id-set's member between two revisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// The member was removed — a governed id-set never shrinks
    /// (`S(r) ⊆ S(r')`, "no member removed or renumbered").
    Removed,
    /// The member was added with no new `## Revision N` heading in the
    /// current revision — a governed id-set grows only under a new
    /// Revision heading, never silently.
    GrownSilently,
}

/// One violation of `append_only_variants` between two revisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionFinding {
    /// The governed id-set the change happened in.
    pub set: String,
    /// The member that changed.
    pub member: String,
    /// What happened to it.
    pub change: ChangeKind,
}

/// The governed id-sets of one revision: the States variant table's
/// ids, the Constraint rows' kind value-set, and the Property rows'
/// kind value-set. The Reference Typing table's field set is the
/// fourth governed id-set, but it is carried by the format doc alone —
/// [`reference_typing_fields`] reads it, and [`check_pair`] diffs it.
pub fn governed_id_sets(spec: &Spec) -> BTreeMap<String, BTreeSet<String>> {
    let mut sets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let mut states = BTreeSet::new();
    for state in &spec.states {
        states.insert(state.id.clone());
    }
    sets.insert("states".to_string(), states);

    let mut constraint_kinds = BTreeSet::new();
    for row in &spec.constraints {
        if let Some(kind) = &row.kind {
            constraint_kinds.insert(kind.clone());
        }
    }
    sets.insert("constraint kind values".to_string(), constraint_kinds);

    let mut property_kinds = BTreeSet::new();
    for row in &spec.properties {
        if let Some(kind) = &row.kind {
            property_kinds.insert(kind.clone());
        }
    }
    sets.insert("property kind values".to_string(), property_kinds);

    sets
}

/// The Reference Typing table's field set — the third governed id-set
/// `append_only_variants` names. Only a file that carries the table
/// has one; `None` for every other file (an absent table is an empty
/// set, not an unreadable one).
pub fn reference_typing_fields(spec: &Spec) -> Result<Option<BTreeSet<String>>, String> {
    if spec.reference_typing_body.trim().is_empty() {
        return Ok(None);
    }
    let rows = crate::acset::doc::parse_typing_table(&spec.reference_typing_body)
        .map_err(|errors| format!("the Reference Typing table does not parse: {errors:?}"))?;
    Ok(Some(rows.into_iter().map(|r| r.column).collect()))
}

/// Did the current revision carry a `## Revision N` heading the prior one
/// did not? A governed id-set may grow only under such a heading.
pub fn has_new_revision_heading(prior: &Spec, current: &Spec) -> bool {
    current
        .revision_headings
        .iter()
        .any(|n| !prior.revision_headings.contains(n))
}

/// The diff between two revisions' governed id-sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdSetDiff {
    /// The members a governed id-set lost, `(id-set, member)`.
    pub removed: Vec<(String, String)>,
    /// The members a governed id-set gained, `(id-set, member)`.
    pub added: Vec<(String, String)>,
    /// Did the current revision carry a `## Revision N` heading the
    /// prior one did not?
    pub new_revision_heading: bool,
}

/// The governed id-sets of two revisions, diffed.
pub fn diff_id_sets(prior: &Spec, current: &Spec) -> IdSetDiff {
    let before = governed_id_sets(prior);
    let after = governed_id_sets(current);
    let mut removed = Vec::new();
    let mut added = Vec::new();
    for (set, members) in &after {
        let old = before.get(set).cloned().unwrap_or_default();
        for member in members.difference(&old) {
            added.push((set.clone(), member.clone()));
        }
    }
    for (set, members) in &before {
        let new = after.get(set).cloned().unwrap_or_default();
        for member in members.difference(&new) {
            removed.push((set.clone(), member.clone()));
        }
    }
    let new_revision_heading = has_new_revision_heading(prior, current);
    IdSetDiff {
        removed,
        added,
        new_revision_heading,
    }
}

/// The revision-pair check: the violations of `append_only_variants`
/// between the prior and the current revision of a file.
///
/// A member removed from a governed id-set is always a finding — the
/// set never shrinks. A member added is a finding only when the
/// current revision carries no new `## Revision N` heading: a growth
/// the revision records is allowed, a silent one is not.
pub fn check_pair(prior_text: &str, current_text: &str) -> Result<Vec<RevisionFinding>, String> {
    let prior =
        parse_str(prior_text).map_err(|e| format!("the prior revision does not parse: {e}"))?;
    let current =
        parse_str(current_text).map_err(|e| format!("the current revision does not parse: {e}"))?;
    let mut findings = Vec::new();

    // The Reference Typing table's field set — the third governed
    // id-set, carried only by the format doc.
    let before_fields = reference_typing_fields(&prior)?;
    let after_fields = reference_typing_fields(&current)?;
    if let (Some(before), Some(after)) = (&before_fields, &after_fields) {
        if !has_new_revision_heading(&prior, &current) {
            for member in after.difference(before) {
                findings.push(RevisionFinding {
                    set: "reference typing fields".to_string(),
                    member: member.clone(),
                    change: ChangeKind::GrownSilently,
                });
            }
        }
        for member in before.difference(after) {
            findings.push(RevisionFinding {
                set: "reference typing fields".to_string(),
                member: member.clone(),
                change: ChangeKind::Removed,
            });
        }
    }

    let diff = diff_id_sets(&prior, &current);
    for (set, member) in diff.removed {
        findings.push(RevisionFinding {
            set,
            member,
            change: ChangeKind::Removed,
        });
    }
    if !diff.new_revision_heading {
        for (set, member) in diff.added {
            findings.push(RevisionFinding {
                set,
                member,
                change: ChangeKind::GrownSilently,
            });
        }
    }
    Ok(findings)
}

#[cfg(test)]
mod tests {
    use crate::test_panic::UnwrapPanic;

    use super::*;

    fn spec_with(states: &[&str], kinds: &[&str], revisions: &[u32]) -> String {
        let mut text = String::from(
            "---\nid: rv\nkind: intent\nstatement: \"THE rv SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n",
        );
        for (i, kind) in kinds.iter().enumerate() {
            text.push_str(&format!("| c{i} | {kind} | `holds` | [[rv]] |\n"));
        }
        text.push_str("\n## Model\n\n### States\n\n");
        for state in states {
            text.push_str(&format!("- {state}\n"));
        }
        text.push_str(
            "\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s0 | s0 | `holds` |\n",
        );
        for n in revisions {
            text.push_str(&format!("\n## Revision {n}\n\nA revision.\n"));
        }
        text
    }

    /// A member removed from a governed id-set is a finding.
    #[test]
    fn a_removed_member_is_found() {
        let before = spec_with(&["s1", "s2"], &["invariant"], &[]);
        let after = spec_with(&["s1"], &["invariant"], &[]);
        let findings = check_pair(&before, &after).unwrap_p();
        assert!(
            findings
                .iter()
                .any(|f| f.change == ChangeKind::Removed && f.member == "s2"),
            "{findings:?}"
        );
    }

    /// A kind value added with no new Revision heading is a finding; the
    /// same addition carried by a new Revision heading is allowed.
    #[test]
    fn a_silent_growth_is_found_and_a_declared_one_is_not() {
        let before = spec_with(&["s1"], &["invariant"], &[]);
        let silent = spec_with(&["s1"], &["invariant", "advisory"], &[]);
        let findings = check_pair(&before, &silent).unwrap_p();
        assert!(
            findings
                .iter()
                .any(|f| f.change == ChangeKind::GrownSilently && f.member == "advisory"),
            "{findings:?}"
        );
        let declared = spec_with(&["s1"], &["invariant", "advisory"], &[2]);
        let findings = check_pair(&before, &declared).unwrap_p();
        assert!(findings.is_empty(), "{findings:?}");
    }
}

//! The pre-merge semantic check (`specs/merge.md`).
//!
//! Purpose: detect what a textually clean 3-way merge cannot see — id
//! collisions minted independently on two branches, and renames left
//! dangling by references the other branch minted — BEFORE the merge is
//! reported complete. Responsibilities: index the ids each tree defines,
//! flag new-id collisions against the common ancestor
//! (`no_new_id_collision`), record per-branch blast radii as queries over
//! each branch's own [[graph]] artifact (never a local re-walk —
//! `graph_reused_not_rederived`), flag the rename-replay case
//! (`rename_replayed_onto_foreign_edits`, flag only — the replay itself
//! is `rename.md`'s mechanism, `spk rename`), and re-run the linters over
//! the would-be merged tree before any `merged` verdict
//! (`post_merge_relint_required`). Rationale: the tool never runs git
//! and never writes — the caller supplies base/current/incoming as
//! (relative path, text) sets, so the check is read-only by construction
//! and a flagged merge leaves every tree byte-identical.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Serialize;

use crate::graph;
use crate::lint;
use crate::spec::{Spec, parse_str};

/// One semantic finding the textual merge cannot see.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finding {
    /// `id_collision` | `rename_replay` | `blast_radius_intersection` |
    /// `textual_conflict` | `relint_failure` | `unparsable`
    pub kind: String,
    /// Description naming the ids and files involved.
    pub message: String,
}

/// The merge check's verdict over the two branch tips.
#[derive(Debug, Clone, Serialize)]
pub struct MergeReport {
    /// `merged` (no findings), `needs_review` (blast radii intersected or
    /// a rename replay is owed), or `failed` (collision, textual
    /// conflict, unparsable input, or the merged tree fails the linters).
    pub verdict: String,
    pub findings: Vec<Finding>,
    pub branch_files: usize,
    pub incoming_files: usize,
}

/// One branch tip: parsed specs plus the id index. Paths are relative
/// within the tree so the three trees are comparable.
struct Tip {
    specs: Vec<Spec>,
    /// id (intent id, or `intent.row` for rows) -> (rel path, raw text).
    ids: BTreeMap<String, (String, String)>,
    /// rel path -> raw text (parsed files only participate in edits).
    texts: BTreeMap<String, String>,
    /// rel paths that failed to parse, with the error.
    broken: Vec<(String, String)>,
}

fn load_tip(files: &[(String, String)]) -> Tip {
    let mut tip = Tip {
        specs: vec![],
        ids: BTreeMap::new(),
        texts: BTreeMap::new(),
        broken: vec![],
    };
    for (rel, raw) in files {
        match parse_str(raw) {
            Ok(mut s) => {
                s.path = Some(PathBuf::from(rel));
                for id in s.defined_ids() {
                    let qualified = if id == s.intent.id {
                        id
                    } else {
                        format!("{}.{}", s.intent.id, id)
                    };
                    tip.ids.insert(qualified, (rel.clone(), raw.clone()));
                }
                tip.specs.push(s);
            }
            Err(e) => tip.broken.push((rel.clone(), e.to_string())),
        }
        tip.texts.insert(rel.clone(), raw.clone());
    }
    tip
}

/// Ids defined in files this branch changed relative to the ancestor
/// (content differs, or the file is new) — the `touched` set
/// (`blast_radii_recorded_pre_merge` records these per branch).
fn touched_ids(tip: &Tip, base: &Tip) -> Vec<String> {
    let mut touched = vec![];
    for (id, (rel, raw)) in &tip.ids {
        let changed = match base.texts.get(rel) {
            Some(base_text) => base_text != raw,
            None => true, // new file on this branch
        };
        if changed {
            touched.push(id.clone());
        }
    }
    // Deletions are touches too: an id the ancestor defined that this
    // branch no longer defines. Its dependents (fan-in) belong in the
    // blast radius exactly as if the definition had been edited.
    for id in base.ids.keys() {
        if !tip.ids.contains_key(id) {
            touched.push(id.clone());
        }
    }
    touched
}

/// The blast radius of touching `touched`: the transitive fan-in closure
/// over the branch's OWN graph artifact (`graph::build` — the reachability
/// query is a graph query, never a fresh markdown walk), plus the targets
/// the touched definitions reach (a new reference reaches its target too).
fn blast_radius(tip: &Tip, touched: &[String]) -> BTreeSet<String> {
    let g = graph::build(&tip.specs);
    // Reverse adjacency (to -> froms): dependents of a touched node.
    let mut dependents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    // Forward adjacency from touched definitions to their targets.
    let mut reaches: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for e in &g.edges {
        dependents
            .entry(e.to.as_str())
            .or_default()
            .push(e.from.as_str());
        reaches
            .entry(e.from.as_str())
            .or_default()
            .push(e.to.as_str());
    }
    let mut affected: BTreeSet<String> = touched.iter().cloned().collect();
    let mut queue: Vec<String> = touched.to_vec();
    while let Some(node) = queue.pop() {
        for dep in dependents.get(node.as_str()).into_iter().flatten() {
            if affected.insert((*dep).to_string()) {
                queue.push((*dep).to_string());
            }
        }
        for target in reaches.get(node.as_str()).into_iter().flatten() {
            if affected.insert((*target).to_string()) {
                queue.push((*target).to_string());
            }
        }
    }
    affected
}

/// Run the merge check. `base`, `a` (current), `b` (incoming) are
/// (relative path, text) sets. Read-only: nothing is ever written.
pub fn run(
    base: &[(String, String)],
    a: &[(String, String)],
    b: &[(String, String)],
) -> MergeReport {
    let tip_base = load_tip(base);
    let tip_a = load_tip(a);
    let tip_b = load_tip(b);
    let mut findings: Vec<Finding> = vec![];

    for (label, tip) in [("A", &tip_a), ("B", &tip_b)] {
        for (path, err) in &tip.broken {
            findings.push(Finding {
                kind: "unparsable".into(),
                message: format!("branch {label}: {path} does not parse: {err}"),
            });
        }
    }

    // --- no_new_id_collision: an id defined on both tips is a collision
    // unless the common ancestor defined it and at most one branch
    // changed its definition (both-changed-differently is the same
    // independent-mint hazard at the content level).
    for (id, (a_path, a_text)) in &tip_a.ids {
        let Some((b_path, b_text)) = tip_b.ids.get(id) else {
            continue;
        };
        let base_text = tip_base.ids.get(id).map(|(_, t)| t.as_str());
        let newly_minted = base_text.is_none();
        let both_edited = match base_text {
            Some(base_text) => a_text != base_text && b_text != base_text,
            None => false,
        };
        if newly_minted || both_edited {
            let why = if newly_minted {
                "minted independently on both branches"
            } else if a_text == b_text {
                "edited on both branches to identical definitions"
            } else {
                "edited on both branches with different definitions"
            };
            findings.push(Finding {
                kind: "id_collision".into(),
                message: format!(
                    "id {id} is {why} — branch A: {a_path}, branch B: {b_path}; \
                     an id minted (or divergently edited) by both branches is a \
                     collision, not a merge"
                ),
            });
        }
    }

    // --- textual conflicts: git marks these; surfaced for completeness.
    for (path, a_text) in &tip_a.texts {
        if let (Some(b_text), Some(base_text)) = (
            tip_b.texts.get(path.as_str()),
            tip_base.texts.get(path.as_str()),
        ) && a_text.as_str() != base_text.as_str()
            && b_text.as_str() != base_text.as_str()
            && a_text != b_text
        {
            findings.push(Finding {
                kind: "textual_conflict".into(),
                message: format!(
                    "{path} was edited on both branches — git will mark this \
                     conflict; resolve it, then re-run the check"
                ),
            });
        }
    }

    // --- blast_radii_recorded_pre_merge + intersection → needs_review --
    let touched_a = touched_ids(&tip_a, &tip_base);
    let touched_b = touched_ids(&tip_b, &tip_base);
    let radius_a = blast_radius(&tip_a, &touched_a);
    let radius_b = blast_radius(&tip_b, &touched_b);
    let intersect: Vec<String> = radius_a.intersection(&radius_b).cloned().collect();
    if !intersect.is_empty() {
        findings.push(Finding {
            kind: "blast_radius_intersection".into(),
            message: format!(
                "the branches' blast radii intersect at {} — the union needs \
                 human review even where the file diffs don't overlap",
                intersect.join(", ")
            ),
        });
    }

    // --- rename_replayed_onto_foreign_edits (flag only; the replay
    // mechanism is rename.md's own `spk rename`, never a fresh rewrite) -
    for (renamed, other, label) in [(&tip_a, &tip_b, "A"), (&tip_b, &tip_a, "B")] {
        // Ids the ancestor defined that this branch renamed away.
        let renamed_away: Vec<&String> = tip_base
            .ids
            .keys()
            .filter(|id| !renamed.ids.contains_key(*id))
            .collect();
        if renamed_away.is_empty() {
            continue;
        }
        for spec in &other.specs {
            for link in &spec.links {
                for old in &renamed_away {
                    let t = &link.target;
                    if t.as_str() == old.as_str() || t.starts_with(&format!("{old}.")) {
                        let file = spec
                            .path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "<unknown>".into());
                        findings.push(Finding {
                            kind: "rename_replay".into(),
                            message: format!(
                                "branch {label} removed {old} (a rename or a deletion), but branch {}'s files still reference [[{t}]] in {file} — if the removal was a rename, replay it onto that reference (specodelic rename); if a deletion, drop the stale reference",
                                if label == "A" { "B" } else { "A" }
                            ),
                        });
                    }
                }
            }
        }
    }

    // --- post_merge_relint_required: the union tree must re-lint clean
    // (linter.referential_integrity via zero dangling + full corpus lint)
    // before merge is reported passed. Union rule: a branch's deletion
    // wins when the other branch left the file untouched; an edit beats
    // an untouched counterpart; edited-on-one-side + deleted-on-the-other
    // is a modify/delete conflict (git marks it too) — the edited version
    // stays in the merged tree for the relint and the conflict is flagged
    // so the verdict can never be a false clean `merged`.
    let mut merged: BTreeMap<String, String> = BTreeMap::new();
    for (rel, text) in a {
        merged.insert(rel.clone(), text.clone());
    }
    for (rel, text) in b {
        match a.iter().find(|(p, _)| p == rel) {
            None => {
                // Not in A: either B-only (take it) or deleted by A (a
                // delete wins unless B changed the file away from base).
                let unchanged_in_b =
                    tip_base.texts.get(rel).map(String::as_str) == Some(text.as_str());
                if !tip_base.texts.contains_key(rel) || !unchanged_in_b {
                    merged.insert(rel.clone(), text.clone());
                }
            }
            Some((_, a_text)) => {
                // A has it: keep A's version only if A actually edited it;
                // if A left it untouched, B's edit (if any) survives.
                let a_untouched =
                    tip_base.texts.get(rel).map(String::as_str) == Some(a_text.as_str());
                if a_untouched {
                    merged.insert(rel.clone(), text.clone());
                }
            }
        }
    }
    // Symmetric deletion handling: every base file missing from one tip
    // is a deletion by that branch. When the other branch edited it, the
    // modify/delete conflict is flagged (the edited version stays so the
    // relint sees the tree the merge would actually produce); when the
    // other branch left it untouched, the deletion wins.
    for rel in tip_base.texts.keys() {
        let in_a = a.iter().any(|(p, _)| p == rel);
        let in_b = b.iter().any(|(p, _)| p == rel);
        if in_a && in_b {
            continue; // present on both sides — no deletion involved
        }
        let (edited_tip, deleted_tip) = if in_a { ("A", "B") } else { ("B", "A") };
        let edited = if edited_tip == "A" {
            a.iter()
                .find(|(p, _)| p == rel)
                .map(|(_, t)| t.as_str())
                .is_some_and(|t| tip_base.texts.get(rel).map(String::as_str) != Some(t))
        } else {
            tip_b
                .texts
                .get(rel)
                .is_some_and(|t| tip_base.texts.get(rel).map(String::as_str) != Some(t.as_str()))
        };
        if edited {
            findings.push(Finding {
                kind: "textual_conflict".into(),
                message: format!(
                    "{rel} was edited on branch {edited_tip} but deleted on branch \
                     {deleted_tip} — modify/delete conflict; git will mark it — \
                     resolve it, then re-run the check"
                ),
            });
        } else {
            merged.remove(rel);
        }
    }
    let mut details: Vec<String> = vec![];
    let mut merged_specs: Vec<Spec> = vec![];
    for (rel, text) in &merged {
        match parse_str(text) {
            Ok(mut s) => {
                s.path = Some(PathBuf::from(rel));
                merged_specs.push(s);
            }
            Err(e) => details.push(format!("{rel}: merged tree does not parse: {e}")),
        }
    }
    if details.is_empty() && !merged.is_empty() {
        let report = lint::lint_corpus(&merged_specs);
        for issue in &report.issues {
            details.push(format!(
                "merged tree fails {}: {}",
                issue.rule_id, issue.message
            ));
        }
        let g = graph::build(&merged_specs);
        for d in &g.dangling {
            details.push(format!("dangling reference in merged tree: {d}"));
        }
    }
    for d in &details {
        findings.push(Finding {
            kind: "relint_failure".into(),
            message: d.clone(),
        });
    }

    let verdict = if findings.iter().any(|f| {
        matches!(
            f.kind.as_str(),
            "id_collision" | "relint_failure" | "unparsable" | "textual_conflict"
        )
    }) {
        "failed"
    } else if findings.is_empty() {
        "merged"
    } else {
        "needs_review"
    };

    MergeReport {
        verdict: verdict.to_string(),
        findings,
        branch_files: tip_a.specs.len(),
        incoming_files: tip_b.specs.len(),
    }
}

#[cfg(test)]
mod blast_radius_parity {
    //! Parity for the closure migration (tasks 4.3–4.5): the pre-migration
    //! walk here is the authority `acset::query::blast_radius` must match —
    //! the property decides the DEFINITION of blast radius before the walk
    //! is retired (design.md decision 2).

    use std::collections::BTreeMap;

    use crate::acset::instance::Instance;
    use crate::acset::query;
    use crate::spec::parse_str;

    use super::Tip;

    /// Build a tip the way `load_tip` does, without touching the disk.
    fn tip_of(files: &[(&str, &str)]) -> Tip {
        let mut tip = Tip {
            specs: vec![],
            ids: BTreeMap::new(),
            texts: BTreeMap::new(),
            broken: vec![],
        };
        for (rel, raw) in files {
            match parse_str(raw) {
                Ok(mut s) => {
                    s.path = Some(std::path::PathBuf::from(rel));
                    for id in s.defined_ids() {
                        let qualified = if id == s.intent.id {
                            id
                        } else {
                            format!("{}.{}", s.intent.id, id)
                        };
                        tip.ids
                            .insert(qualified, (rel.to_string(), raw.to_string()));
                    }
                    tip.specs.push(s);
                }
                Err(e) => tip.broken.push(((*rel).to_string(), e.to_string())),
            }
            tip.texts.insert((*rel).to_string(), (*raw).to_string());
        }
        tip
    }

    /// A dependent's own forward reach joins the blast radius: touching
    /// the observed effect row `a.x` pulls in `a.r1` (its dependent) AND
    /// `a.r1`'s own traces_to target `b` — reachable only by following a
    /// forward edge FROM a node first reached BACKWARD.
    const OBSERVER_A: &str = r#"---
id: a
kind: intent
statement: "observes the shared effect and traces to b"
---

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| r1 | invariant | `true` | [[b]] | [[a.x]] |
| x | effect | `true` |  |  |
"#;

    #[test]
    fn dependents_forward_reach_joins_the_radius() {
        let tip = tip_of(&[
            ("a.md", OBSERVER_A),
            ("b.md", "---\nid: b\nkind: intent\nstatement: \"target\"\n"),
        ]);
        let touched = vec!["a.x".to_string()];
        let expected = super::blast_radius(&tip, &touched);
        let instance = Instance::from_specs(&tip.specs);
        let all = instance.morphism_names();
        let queried = query::blast_radius(&instance, &["a.x"], &all).expect("seed exists");
        assert_eq!(
            queried, expected,
            "the query-derived blast radius must reproduce the pre-migration walk"
        );
    }
}

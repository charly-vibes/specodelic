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
                                "branch {label} renamed {old} away, but the other branch \
                                 mints a reference to [[{t}]] in {file} — after joining, \
                                 replay the rename onto that reference (specodelic rename) \
                                 so it is neither dangling nor dropped"
                            ),
                        });
                    }
                }
            }
        }
    }

    // --- post_merge_relint_required: the union tree must re-lint clean
    // (linter.referential_integrity via zero dangling + full corpus lint)
    // before merge is reported passed. Union rule: A wins deletions; a
    // file A left untouched takes B's version.
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

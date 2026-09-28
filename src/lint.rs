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

use crate::ears;
use crate::spec::Spec;

/// One lint finding.
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    /// Which invariant fired, e.g. `guard_required`.
    pub rule: String,
    /// File the finding is anchored to.
    pub file: String,
    /// Human-readable message with a suggested fix.
    pub message: String,
}

/// Corpus lint report.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Report {
    pub files_linted: usize,
    pub issues: Vec<Issue>,
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
    };
    for spec in specs {
        lint_one(spec, &mut report);
    }
    lint_references(specs, &mut report);
    lint_coverage(specs, &mut report);
    report
}

/// Per-file invariants.
fn lint_one(spec: &Spec, report: &mut Report) {
    let file = spec
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| format!("<{}>", spec.intent.id));

    // frontmatter_valid — kind must be `intent` (parse already required the fields).
    if spec.intent.kind != "intent" {
        report.issues.push(Issue {
            rule: "frontmatter_valid".into(),
            file: file.clone(),
            message: format!(
                "frontmatter kind is `{}`, must be `intent`",
                spec.intent.kind
            ),
        });
    }

    // id_matches_file — frontmatter.id == replace(stem(path), "-", ".").
    if let Some(path) = &spec.path {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let expected = stem.replace('-', ".");
        if spec.intent.id != expected {
            report.issues.push(Issue {
                rule: "id_matches_file".into(),
                file: file.clone(),
                message: format!(
                    "frontmatter id `{}` does not match filename (expected `{}` — `-` in filename maps to `.` in id; `_` is literal)",
                    spec.intent.id, expected
                ),
            });
        }
    }

    // unique_id — every row id in the file is unique.
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for (table, row) in rows(spec) {
        if let Some(first_table) = seen.insert(row.id.as_str(), table) {
            report.issues.push(Issue {
                rule: "unique_id".into(),
                file: file.clone(),
                message: format!("duplicate id `{}` (in {first_table} and {table})", row.id),
            });
        }
    }

    // guard_required — every transition's guard is present.
    for t in &spec.transitions {
        if t.guard.is_none() {
            report.issues.push(Issue {
                rule: "guard_required".into(),
                file: file.clone(),
                message: format!(
                    "transition `{}` has an empty guard — every guard must be non-null",
                    t.id
                ),
            });
        }
    }

    // model_present — States list and Transitions table both exist.
    if spec.states.is_empty() || spec.transitions.is_empty() {
        report.issues.push(Issue {
            rule: "model_present".into(),
            file: file.clone(),
            message: format!(
                "Model section incomplete: {} states, {} transitions — both sections must exist (empty-but-present beats absent)",
                spec.states.len(),
                spec.transitions.len()
            ),
        });
    }

    // ears_statement — the intent statement matches one of the 5 EARS patterns.
    if ears::classify(&spec.intent.statement).is_none() {
        let hint = if !ears::has_shall(&spec.intent.statement) {
            "no imperative `SHALL` found (`has_shall`)"
        } else {
            "does not match any of: Ubiquitous / Event-Driven / State-Driven / Unwanted-Behavior / Optional-Feature"
        };
        report.issues.push(Issue {
            rule: "ears_statement".into(),
            file: file.clone(),
            message: format!("intent statement {hint}"),
        });
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
            report.issues.push(Issue {
                rule: "no_conjoined_id".into(),
                file: file.clone(),
                message: format!("{label} id `{id}` encodes two capabilities joined by and/or"),
            });
        }
        if tokens
            .iter()
            .any(|t| matches!(t.as_str(), "all" | "every" | "any" | "always" | "never"))
        {
            report.issues.push(Issue {
                rule: "no_universal_in_id".into(),
                file: file.clone(),
                message: format!(
                    "{label} id `{id}` contains a universal token (all/every/any/always/never)"
                ),
            });
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
    fn build(specs: &[Spec]) -> Index {
        let mut files = BTreeMap::new();
        for spec in specs {
            files.insert(
                spec.intent.id.clone(),
                spec.defined_ids().into_iter().collect(),
            );
        }
        Index { files }
    }

    /// Resolve a link target: a file id, `file_id.row_id`, a
    /// `model.state`/`model.transition` section anchor (bare or
    /// `file_id.model.…`), or a member of a row (`file_id.row.member`).
    fn resolves(&self, target: &str) -> bool {
        // Bare section anchors.
        if target == "model.state" || target == "model.transition" {
            return true;
        }
        // Exact file id.
        if self.files.contains_key(target) {
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
    let index = Index::build(specs);
    for spec in specs {
        let file = spec
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("<{}>", spec.intent.id));
        for link in &spec.links {
            if is_metasyntactic(&link.target, &index) {
                continue;
            }
            // Skip example links inside expr cells of non-resolvable shape
            // (e.g. `[[x.y]]` used as format documentation).
            if !index.resolves(&link.target) {
                report.issues.push(Issue {
                    rule: "total_refs".into(),
                    file: file.clone(),
                    message: format!(
                        "dangling reference `[[{}]]` from {}{} — target not defined in any spec file",
                        link.target,
                        link.field,
                        if link.column.is_empty() { String::new() } else { format!(".{}", link.column) }
                    ),
                });
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
                report.issues.push(Issue {
                    rule: "coverage".into(),
                    file: file_id.clone(),
                    message: format!(
                        "constraint `{cid}` has no deriving property — ∃ property.derives_from == `{full}` is required"
                    ),
                });
            }
        }
        // no_orphan_property — every property derives from something.
        for pid in &property_ids {
            let has_source = spec
                .links
                .iter()
                .any(|l| l.source == *pid && l.field == "properties" && l.column == "derives_from");
            if !has_source {
                report.issues.push(Issue {
                    rule: "no_orphan_property".into(),
                    file: file_id.clone(),
                    message: format!("property `{pid}` derives from nothing"),
                });
            }
        }
    }
}

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
        "dual_format_valid",
        "a file carrying `## ADDED Requirements` must be a dual-format file — declare `id: spec` and pair it with a sibling `## Requirements` section",
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
            if is_metasyntactic(&link.target, index) {
                continue;
            }
            // Skip example links inside expr cells of non-resolvable shape
            // (e.g. `[[x.y]]` used as format documentation).
            if !index.resolves(&link.target) {
                report.issues.push(Issue::new("total_refs", file.clone(), format!(
                        "dangling reference `[[{}]]` from {}{} — target not defined in any spec file",
                        link.target,
                        link.field,
                        if link.column.is_empty() { String::new() } else { format!(".{}", link.column) }
                    )));
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
                        "property `{pid}` derives from nothing — derives_from takes a file-qualified wiki-link like `[[{file_id}.<constraint-id>]]` (bare ids and bare text do not resolve)",
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
        ]
    }

    #[test]
    fn catalog_covers_every_rule_the_linter_can_emit() {
        let report = lint_corpus(&fixture_corpus());
        let emitted: BTreeSet<&str> = report
            .issues
            .iter()
            .map(|i| i.rule_id.strip_prefix("linter.").expect("rule_id prefix"))
            .collect();
        let catalog: BTreeSet<&str> = RULE_TABLE.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            emitted, catalog,
            "the rule table and the emittable rule ids must coincide — \
             a rule missing from the table panics at construction, one \
             missing from the corpus means the fixture stopped covering it"
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
}

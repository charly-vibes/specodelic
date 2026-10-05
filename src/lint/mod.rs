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

use crate::checklist::{self, Checklist};
use crate::spec::Spec;

mod graph;
mod observability;
mod rules;

// Moved to submodules (specodelic-g17 file_lines split) — re-imported so
// existing call sites (lint_one, lint_all, advisory_findings, tests) stay put.
pub(crate) use graph::{
    Index, is_metasyntactic, lint_graph_shape, lint_references, resolve_node, resolves_row, rows,
};
pub(crate) use observability::{lint_coverage, lint_observability};
pub(crate) use rules::{
    lint_ears_family, lint_failure_shape_family, lint_frontmatter_family,
    lint_model_family, lint_referential_family, lint_schema_shape_family,
};

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
        "every transition must carry a non-null guard (a guard may be prose, or cite an invariant Constraint or a State — target typing is `ref_kind_compatible`'s beat)",
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
        "law_cases",
        "every law-kind property must enumerate its required cases as **name:** labels in its own predicate — the identity and associativity floor is mandatory, extra named cases are checkable declarations",
    ),
    (
        "requirement_drift",
        "a dual-format file's ## Requirements mirror must hold every delta requirement (ADDED and MODIFIED sections alike) with identical requirement text, compared per requirement so mixed-delta files are satisfiable (blank lines and trailing space ignored)",
    ),
    (
        "dual_format_valid",
        "a file carrying `## ADDED Requirements` must be a dual-format file — declare `id: spec` and pair it with a sibling `## Requirements` section",
    ),
    (
        "terminal_states_emit",
        "every failure terminal state must emit exactly one file-owned effect Constraint — a mute failure terminal is a finding (specs/linter-failure_shape.md; timed_out/exploration_only are the stated v1 non-goal)",
    ),
    (
        "error_labels_unique",
        "within one file, no two error Constraints may share a variant head — the label is file-id-namespaced (errors.md error_expr_shape), so collisions are a per-file property",
    ),
    (
        "guard_negation_total",
        "every failure transition must cite exactly the union of its success siblings' citation sets, or be on the recorded carve-out list (orchestrate.md's stage-fail transitions) — a zero-citation failure guard off the list is a finding",
    ),
    (
        "every_state_used",
        "every declared state must appear as from or to in at least one transition — a state no transition reaches is machinery the model can never enter or leave",
    ),
    (
        "every_transition_valid",
        "every transition's from and to must name states declared in the same file's States section",
    ),
    (
        "no_self_ref",
        "a row must not reference itself via traces_to or derives_from — a self-tracing row has no owning purpose",
    ),
    (
        "acyclic",
        "the directed graph formed by constraint-traces_to ∪ property-derives_from ∪ guard-as-edge must contain no cycle (derives_from edges are property-sourced — the Reference Typing Appears-on column is normative, so a Constraint-row derives_from is typing's beat, never an edge)",
    ),
    (
        "single_root_reachable",
        "every constraint/property/state/transition row must reach its file's OWN intent row through own-file primary linkage (traces_to/derives_from chains resolved within the file, plus the model's own from/to/guard/emits edges) — cross-file typed edges (guard citations of foreign constraints, satisfies, observes) are outbound leaves, never reachability paths; tiered: cross-file-only rows warn (advisory, exit 0), rows with no path to ANY intent hard-fail",
    ),
    (
        "observability",
        "every effect Constraint must be the target of ≥1 `observes` reference from a different row — advisory: warned on the warnings channel (exit 0), never a failure",
    ),
    (
        "checklist_well_formed",
        "a declared checklist manifest (`*.checklist.md`) must be a flat item list with stable ids plus a mapping table with exactly item/status/mapped_ids/rationale columns — a manifest the linter cannot read is a checklist going silently unconsulted",
    ),
    (
        "every_item_accounted",
        "every checklist item must have exactly one mapping row with status `covered` or `waived` — an unconsulted item is the failure this checker exists to prevent",
    ),
    (
        "covered_maps_resolve",
        "a `covered` mapping row must name a non-empty mapped_ids list whose ids resolve to real constraint or property rows — a claim resting on nothing is not a claim",
    ),
    (
        "waiver_has_rationale",
        "a `waived` mapping row must carry non-empty rationale prose — an unexplained waiver is an unconsulted item with extra steps",
    ),
    (
        "no_duplicate_claim",
        "no two mapping rows may target the same checklist item — one claim per item, on the record",
    ),
    (
        "constraint_kind_closed",
        "every Constraint row's kind must be in {invariant, advisory, effect, extension_point} — an unreadable kind cell is outside the closed set (specs/linter-schema_shape.md)",
    ),
    (
        "property_kind_closed",
        "every Property row's kind must be in {unit, law} — an unreadable kind cell is outside the closed set (specs/linter-schema_shape.md)",
    ),
    (
        "pack_shape",
        "a kind: profile pack file's manifest must carry all six facet tables (Sections/Kinds/References/Checkers/Floors/Requires) with well-formed two-column rows, its Kinds rows must be pack-qualified (never a base closed-set name — the narrowing rejection), and manifest tables may not appear on non-profile files (specs/packs.md, Revision 14)",
    ),
    (
        "orphan_vocabulary",
        "orphan vocabulary is a labeled failure naming the candidate pack and both remediations (enable/declare the pack, or fix the vocabulary) — a declared uses edge targeting an id no discovered kind: profile pack carries, or a pack-qualified token used in a kind/field position with no discovered pack in its namespace (candidate prefix-derived when only the namespace is known) (specs/packs.md, Revision 14)",
    ),
    (
        "skew_advisory",
        "a declared pack's Requires base pin older than the workspace corpus revision is a warnings-channel advisory naming the pack's base pin and the corpus revision — never silent, never failing (specs/packs.md, Revision 14)",
    ),
    (
        "schema_matches_typing_table",
        "when the lint target carries the format doc, its Reference Typing table must equal the Schema value row for row — the document and the code cannot drift; a corpus without the format doc is out of the gate's scope (no-op, never fabricated expected rows) (add-acset-core, linter-schema_shape family)",
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
    /// Advisory findings (specs/linter-observability.md): rendered onto
    /// the success envelope's warnings channel, exit 0 — never counted
    /// by [`Report::failures`]. The `Issue` model carries no severity,
    /// so advisory findings must not ride the issues channel.
    pub warnings: Vec<Issue>,
    /// How many `*.checklist.md` manifests were consulted (external
    /// completeness). Zero means `not_applicable` — the honest count a
    /// consumer needs to tell an empty pass from a skipped one
    /// (Rule-of-5 EXCL-002, specodelic-b15).
    pub checklists_declared: usize,
    /// Discovered domain packs (specs/packs.md, Revision 14) — surfaced
    /// in the envelope data only when at least one `kind: profile` file
    /// exists; absent otherwise, so a pack-free workspace's lint output
    /// stays byte-identical to pre-mechanism.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<crate::packs::PackInfo>,
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
        warnings: vec![],
        checklists_declared: 0,
        packs: vec![],
    };
    for spec in specs {
        lint_one(spec, specs, &mut report);
    }
    lint_references(specs, &mut report);
    lint_coverage(specs, &mut report);
    lint_graph_shape(specs, &mut report);
    lint_observability(specs, &mut report);
    report.issues.extend(schema_drift_findings(specs));
    crate::packs::pack_pass(specs, &mut report);
    report
}

/// The full pass: internal-consistency checks over the specs plus the
/// external-completeness check over any declared `*.checklist.md`
/// manifests. The CLI lint command runs this; `lint_corpus` alone stays
/// checklist-free for callers whose gate must not see the optional
/// checker (compile's precondition — external_completeness never gates
/// a stage, specs/linter-external_completeness.md Notes).
pub fn lint_all(specs: &[Spec], checklists: &[Checklist]) -> Report {
    let mut report = lint_corpus(specs);
    lint_checklists(specs, checklists, &mut report);
    report
}

/// An empty [`Report`] scaffold for single-checker invocations
/// (specodelic-8kk: the orchestrator invokes checkers individually in
/// Checker Ownership dependency order, never as one flat pass).
fn empty_report(files_linted: usize) -> Report {
    Report {
        files_linted,
        issues: vec![],
        warnings: vec![],
        checklists_declared: 0,
        packs: vec![],
    }
}

/// Run ONE Checker Ownership checker over the corpus and return its
/// findings. Each function is the checker's engine-side invocation —
/// the orchestrator (src/orchestrate.rs) calls them in dependency
/// order, so a checker whose dependency failed is never run at all
/// (specs/orchestrate.md `dependency_respecting_skip`).
///
/// `linter-frontmatter.md` — the first gate: frontmatter validity,
/// id/filename law, and the dual-format file-structure rules that
/// postdate the ownership table (attribution documented on
/// [`lint_frontmatter_family`]).
pub fn frontmatter_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    for spec in specs {
        lint_frontmatter_family(spec, &mut report);
    }
    report.issues
}

/// `linter-referential_integrity.md` — unique ids per file plus the
/// corpus-wide reference rules (`total_refs`, `no_self_ref`).
pub fn referential_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    for spec in specs {
        lint_referential_family(spec, &mut report);
    }
    lint_references(specs, &mut report);
    report.issues
}

/// `linter-graph_shape.md` — acyclicity and root reachability.
pub fn graph_shape_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    lint_graph_shape(specs, &mut report);
    report.issues
}

/// `linter-model_shape.md` — guard/model/transition well-formedness.
pub fn model_shape_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    for spec in specs {
        lint_model_family(spec, &mut report);
    }
    report.issues
}

/// `linter-ears_syntax.md` — statement pattern + id-shape rules.
pub fn ears_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    for spec in specs {
        lint_ears_family(spec, &mut report);
    }
    report.issues
}

/// `linter-schema_shape.md` — the closed-kind-set table-walkers
/// (specodelic-7h8): `constraint_kind_closed` and
/// `property_kind_closed`. Row shape itself stays parser-enforced and
/// the closed sets live in guide.rs ([`crate::guide::CONSTRAINT_KINDS`]
/// / [`crate::guide::PROPERTY_KINDS`]) — these rules walk the parsed
/// rows and reject any kind outside the closed set, including an
/// unreadable (absent) kind cell. The remaining checker residue
/// (`id_set_grows_only` / `id_set_order_stable` across revisions,
/// `no_prose_field_parsed`) stays structural/cross-revision and is not
/// a per-file table walk.
pub fn schema_shape_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    for spec in specs {
        lint_schema_shape_family(spec, specs, &mut report);
    }
    report.issues
}

/// The `schema_matches_typing_table` drift gate (add-acset-core task 2.4,
/// design.md decision 3): when the corpus carries the format doc, its
/// Reference Typing table must equal the canonical Schema row for row — a
/// lint finding on divergence, in either direction. A corpus without the
/// format doc is out of the gate's scope: no-op, never fabricated expected
/// rows. Uninterpretable cells fire too — an unrecognized doc edit is a
/// drift signal by construction, never silence.
pub fn schema_drift_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let Some(doc) = specs
        .iter()
        .find(|s| s.intent.id == "specodelic" && !s.reference_typing_body.trim().is_empty())
    else {
        return issues;
    };
    let file = doc.intent.id.clone();
    let rows = match crate::acset::doc::parse_typing_table(&doc.reference_typing_body) {
        Ok(rows) => rows,
        Err(errors) => {
            for e in errors {
                issues.push(Issue::new("schema_matches_typing_table", file.clone(), e));
            }
            return issues;
        }
    };
    if let Err(drift) =
        crate::acset::doc::matches_typing_table(&crate::acset::schema::canonical(), &rows)
    {
        for d in drift {
            issues.push(Issue::new("schema_matches_typing_table", file.clone(), d));
        }
    }
    issues
}

/// Checker-family slice: `linter-schema_shape.md`'s two closed-set
/// table-walkers (specodelic-7h8). Per-file, appended last in
/// [`lint_one`]'s composition. The effective closed set is the base
/// set extended with the active packs' fiber kinds (specodelic-ung:
/// base ∪ active-pack-fiber, never narrower — specs/packs.md); the
/// finding message still names the base set, whose members are
/// unconditional.
pub fn coverage_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    lint_coverage(specs, &mut report);
    report.issues
}

/// `linter-failure_shape.md` — the three failure-shape rules
/// (specodelic-ct5), exposed as the checker's engine-side invocation for
/// the orchestrator's lint stage (specodelic-hhp decision a: the checker
/// has a Checker Ownership row and runs after `linter.model_shape`).
pub fn failure_shape_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    for spec in specs {
        lint_failure_shape_family(spec, &mut report);
    }
    report.issues
}

/// `linter-external_completeness.md` — runs only when a checklist is
/// declared; its outcome never gates a stage. Returns (findings, how
/// many manifests were consulted).
pub fn external_completeness_findings(
    specs: &[Spec],
    checklists: &[Checklist],
) -> (Vec<Issue>, usize) {
    let mut report = empty_report(specs.len());
    lint_checklists(specs, checklists, &mut report);
    (report.issues, report.checklists_declared)
}

/// Advisory findings (specs/linter-observability.md) — warnings, never
/// gate; the orchestrator surfaces them on the warnings channel.
pub fn advisory_findings(specs: &[Spec]) -> Vec<Issue> {
    let mut report = empty_report(specs.len());
    lint_observability(specs, &mut report);
    report.warnings
}

/// Can `target` resolve to a CONSTRAINT or PROPERTY row? Stricter than
/// [`Index::resolves`]: a bare file id or a `model.state`/`model.transition`
/// section anchor is machinery, not a claim to rest a checklist item on.
pub fn lint_checklists(specs: &[Spec], checklists: &[Checklist], report: &mut Report) {
    report.checklists_declared = checklists.len();
    if checklists.is_empty() {
        return; // not_applicable — nothing external to be incomplete against
    }
    let index = Index::build(specs);
    for cl in checklists {
        let file = cl.path.display().to_string();
        // checklist_well_formed — the parser's defect list, verbatim.
        for defect in &cl.defects {
            report
                .issues
                .push(Issue::new("checklist_well_formed", &file, defect.clone()));
        }
        // Group rows by claimed item for the exactly-one checks.
        let mut by_item: BTreeMap<&str, Vec<&checklist::MappingRow>> = BTreeMap::new();
        for row in &cl.mapping {
            by_item.entry(row.item.as_str()).or_default().push(row);
        }
        for item in &cl.items {
            match by_item.get(item.id.as_str()).map(Vec::as_slice) {
                // every_item_accounted: ∃ exactly one row with a legal
                // status. Zero rows → unconsulted item. One row with a
                // status outside {covered, waived} → same failure.
                None => report.issues.push(Issue::new(
                    "every_item_accounted",
                    &file,
                    format!(
                        "checklist item `{}` has no mapping row — nothing was consulted for it",
                        item.id
                    ),
                )),
                Some([row]) if row.status != "covered" && row.status != "waived" => {
                    report.issues.push(Issue::new(
                        "every_item_accounted",
                        &file,
                        format!(
                            "mapping row for item `{}` has status `{}` — must be `covered` or `waived`",
                            item.id, row.status
                        ),
                    ));
                }
                Some([row]) => {
                    if row.status == "covered" {
                        // covered_maps_resolve: non-empty, row-resolving ids.
                        if row.mapped_ids.is_empty() {
                            report.issues.push(Issue::new(
                                "covered_maps_resolve",
                                &file,
                                format!(
                                    "covered mapping row for item `{}` has an empty mapped_ids list",
                                    item.id
                                ),
                            ));
                        }
                        for id in &row.mapped_ids {
                            if !resolves_row(&index, id) {
                                // A bare single-segment token can never
                                // be a row reference — teach the dotted
                                // spelling (Rule-of-5 CLAR-003, same
                                // pattern as the orphan-property hint).
                                let hint = if id.contains('.') {
                                    String::new()
                                } else {
                                    " — mapped ids are row references: use the dotted file_id.row_id spelling".to_string()
                                };
                                report.issues.push(Issue::new(
                                    "covered_maps_resolve",
                                    &file,
                                    format!(
                                        "covered mapping row for item `{}` maps to `{id}`, which does not resolve to a constraint or property row{hint}",
                                        item.id
                                    ),
                                ));
                            }
                        }
                    } else {
                        // waiver_has_rationale: non-empty rationale prose.
                        if row.rationale.trim().is_empty() {
                            report.issues.push(Issue::new(
                                "waiver_has_rationale",
                                &file,
                                format!(
                                    "waived mapping row for item `{}` carries no rationale",
                                    item.id
                                ),
                            ));
                        }
                    }
                }
                // >1 rows: no_duplicate_claim's beat — do NOT also fire
                // every_item_accounted (the item IS claimed, just twice).
                Some(_) => {
                    report.issues.push(Issue::new(
                        "no_duplicate_claim",
                        &file,
                        format!(
                            "{} mapping rows target checklist item `{}` — one claim per item",
                            by_item[item.id.as_str()].len(),
                            item.id
                        ),
                    ));
                }
            }
        }
    }
}

/// Corpus-wide graph-shape pass (specs/linter-graph_shape.md): build the
/// resolved reference graph once, then check `no_self_ref`, `acyclic`
/// (over traces_to ∪ derives_from ∪ guard-as-edge — a self-loop is
/// `no_self_ref`'s beat, never double-reported as a cycle), and
/// `single_root_reachable` (tiered own-file reachability,
/// specodelic.md Revision 10: every row reaches its file's OWN intent
/// through own-file primary linkage; cross-file typed edges are
/// outbound leaves — advisory tier for cross-file-only rows, hard
/// failure for rows with no path to ANY intent). Resolution reuses [`Index`] with the same
/// metasyntactic skip as [`lint_references`]; dangling targets are
/// `total_refs`'s job, not ours — the two checks compose without
/// double-reporting the same row.

/// Per-file invariants.
/// The file label every lint finding carries: the path when on disk,
/// else `<id>`.
fn lint_one(spec: &Spec, corpus: &[Spec], report: &mut Report) {
    lint_frontmatter_family(spec, report);
    lint_referential_family(spec, report);
    lint_model_family(spec, report);
    lint_ears_family(spec, report);
    lint_failure_shape_family(spec, report);
    lint_schema_shape_family(spec, corpus, report);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_at(text: &str, path: &str) -> Spec {
        let mut spec = crate::spec::parse_str(text).expect("fixture parses");
        spec.path = Some(std::path::PathBuf::from(path));
        spec
    }

    /// Lint-clean-unless-law_cases fixture: a law-kind property row
    /// whose predicate is substituted per test. Fires `law_cases` and
    /// nothing else (task 3.x).
    const LAW_FIXTURE: &str = "---\nid: demo.law\nkind: intent\nstatement: \"THE demo SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| m | invariant | `x holds` | [[demo.law]] |\n\n## Model\n\n### States\n\n- s1\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| nat | law | [[demo.law.m]] | `g()` | PREDICATE |\n";

    /// Fixture corpus that triggers every rule in the catalog, so the
    /// test can assert the catalog covers exactly the rule ids the
    /// linter can emit (task 3.3).
    fn fixture_corpus() -> Vec<Spec> {
        vec![
            // fires law_cases (update-law-named-cases): a law-kind row
            // whose predicate mentions identity/associativity in prose
            // only — no **name:** labels, no enumeration.
            spec_at(
                &LAW_FIXTURE.replace("PREDICATE", "associative and has an identity element"),
                "b-law.md",
            ),
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
            // fires requirement_drift (mirrored ## Requirements drifted
            // from ## ADDED Requirements — gh#4)
            spec_at(
                "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds BUT THE MIRROR DRIFTED\n",
                "spec.md",
            ),
            // fires every_state_used (s2 unused) AND single_root_reachable
            // (s2 is an island — no transition, no link touches it)
            spec_at(
                "---\nid: b.spec\nkind: intent\nstatement: \"the system should maybe work\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | |\n| a | invariant | `y` | |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | |\n",
                "b-spec.md",
            ),
            // fires every_transition_valid (to a state that was never
            // declared) and no_self_ref (row tracing to itself)
            spec_at(
                "---\nid: b.spec\nkind: intent\nstatement: \"THE b SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| self_ref | invariant | `[[b.spec.self_ref]]` | [[b.spec.self_ref]] |\n| other | invariant | `y` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | ghost | [[b.spec.other]] |\n",
                "b-spec.md",
            ),
            // fires acyclic (two constraints mutually tracing via
            // traces_to — the checker spec's edge set)
            spec_at(
                "---\nid: b.cycle\nkind: intent\nstatement: \"THE b SHALL cycle\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| x | invariant | `x` | [[b.cycle.y]] |\n| y | invariant | `y` | [[b.cycle.x]] |\n",
                "b-cycle.md",
            ),
            // fires observability (warning, never an issue): an effect
            // Constraint with no observes edge — the advisory-severity
            // fixture. Adding it to the corpus keeps
            // catalog_covers_every_rule_the_linter_can_emit true after
            // the rule joined RULE_TABLE.
            spec_at(
                "---\nid: obs.unwatched\nkind: intent\nstatement: \"THE watcher SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `v finite` | [[obs.unwatched]] |\n| eff | effect | `output == {v}` | [[obs.unwatched]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.unwatched.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.unwatched.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.unwatched.eff]] | `g()` | `x` |\n",
                "obs-unwatched.md",
            ),
            // fires terminal_states_emit (mute failure terminal: `failed`
            // has inbound, no outbound, no emits) AND guard_negation_total
            // (zero-citation failure guard `boom`, off the orchestrate
            // carve-out) — the two legs of the failure-shape checker's
            // mute-terminal shape in one fixture.
            spec_at(
                "---\nid: fs.mute\nkind: intent\nstatement: \"THE tool SHALL label its failures\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `x` | [[fs.mute]] |\n\n## Model\n\n### States\n\n- s\n- failed\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| ok | s | s | [[fs.mute.inv]] |\n| boom | s | failed | `the world ends` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[fs.mute.inv]] | `g()` | `x` |\n",
                "fs-mute.md",
            ),
            // fires error_labels_unique: two emitted error Constraints
            // sharing variant head `x_failure` — a collision the
            // file-id-namespacing law makes structurally impossible in a
            // well-formed corpus, kept here so the rule stays covered.
            spec_at(
                "---\nid: fs.lbl\nkind: intent\nstatement: \"THE tool SHALL label its failures\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `x` | [[fs.lbl]] |\n| x_failure | effect | `fs.lbl.x_failure(detail)` | [[fs.lbl]] |\n| sub.x_failure | effect | `fs.lbl.sub.x_failure(detail)` | [[fs.lbl]] |\n\n## Model\n\n### States\n\n- s\n- f1 (emits: `[[fs.lbl.x_failure]]`)\n- f2 (emits: `[[fs.lbl.sub.x_failure]]`)\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| ok | s | s | [[fs.lbl.inv]] |\n| boom1 | s | f1 | [[fs.lbl.inv]] |\n| boom2 | s | f2 | [[fs.lbl.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[fs.lbl.inv]] | `g()` | `x` |\n| p_x | unit | [[fs.lbl.x_failure]] | `g()` | `x` |\n| p_sub | unit | [[fs.lbl.sub.x_failure]] | `g()` | `x` |\n",
                "fs-lbl.md",
            ),
            // fires constraint_kind_closed (kind `made_up` outside the
            // closed set) AND property_kind_closed (kind `audit`) — the
            // 7h8 table-walkers, kept covered by the catalog test.
            spec_at(
                "---\nid: k.made\nkind: intent\nstatement: \"THE kinds SHALL stay closed\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | made_up | `x` | [[k.made]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | audit | [[k.made.c]] | `g()` | `x` |\n",
                "k-made.md",
            ),
            // fires schema_matches_typing_table (add-acset-core task 2.4):
            // a file with the format doc's id whose Reference Typing table
            // is MISSING the traces_to row — the drift gate fires on the
            // code-side row the doc lost. (It also fires model_present and
            // coverage — both already covered by other fixtures.)
            spec_at(
                "---\nid: specodelic\nkind: intent\nstatement: \"THE specodelic SHALL define the format\"\n---\n\
                 \n### Reference Typing\n\
                 \n| Field | Appears on | Must resolve to |\n\
                 |-------|------------|-----------------|\n\
                 | `guard` | Transition | an invariant Constraint — or a State |\n",
                "specodelic.md",
            ),
        ]
    }

    /// Checklist fixtures that trigger every external_completeness rule
    /// (mp1 row 10's manifest format), so the catalog-coverage test
    /// sees them alongside the spec-side fixtures.
    fn checklist_fixtures() -> Vec<crate::checklist::Checklist> {
        use crate::checklist::parse_str;
        vec![
            // One manifest, five beats: the nested item fires
            // checklist_well_formed; `unmapped` fires every_item_accounted;
            // `dangling` fires covered_maps_resolve (id resolves to
            // nothing); `norationale` fires waiver_has_rationale; the two
            // rows targeting `dup.target` fire no_duplicate_claim.
            parse_str(
                "dup.checklist.md".into(),
                "## Items\n\
                 \n- **unmapped**: never claimed\n\
                 - **dangling**: claimed against nothing\n\
                 - **norationale**: waived in silence\n\
                 - **dup.target**: claimed twice\n  - **nested.child**: indented under another item\n\
                 \n## Mapping\n\
                 \n| item | status | mapped_ids | rationale |\n\
                 |------|--------|------------|-----------|\n\
                 | dangling | covered | [[absent.row]] | |\n\
                 | norationale | waived | | |\n\
                 | dup.target | covered | [[absent.row]] | |\n\
                 | dup.target | waived | | fine on its own |\n",
            ),
        ]
    }

    // ---- external_completeness (specs/linter-external_completeness.md) ----

    use crate::checklist::parse_str;

    /// One spec whose constraint row a checklist can legitimately map to.
    fn mapped_spec() -> Spec {
        spec_at(
            "---\nid: x.file\nkind: intent\nstatement: \"THE x SHALL exist\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[x.file]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[x.file.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p1 | unit | [[x.file.c1]] | `g()` | `x` |\n",
            "x-file.md",
        )
    }

    fn checklist(text: &str) -> crate::checklist::Checklist {
        parse_str("ship.checklist.md".into(), text)
    }

    /// A fully mapped checklist passes: `covered` rows resolve, `waived`
    /// rows carry rationale — zero issues, zero warnings.
    #[test]
    fn fully_mapped_checklist_passes() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n- **a.second**: out of scope here\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[x.file.c1]], [[x.file.p1]] | |\n| a.second | waived | | tracked in the other repo |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        assert!(report.issues.is_empty(), "{:?}", report.issues);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    }

    /// No declared checklist → not_applicable: the pass runs but emits
    /// nothing — a repo with no checklist is out of scope, not vacuous.
    #[test]
    fn no_checklist_is_not_applicable() {
        let report = lint_all(&[mapped_spec()], &[]);
        assert!(report.issues.is_empty(), "{:?}", report.issues);
    }

    /// every_item_accounted: an item with no mapping row at all.
    #[test]
    fn unmapped_item_rejected() {
        let cl = checklist(
            "## Items\n\n- **a.lost**: never claimed\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.every_item_accounted")
            .collect();
        assert_eq!(hits.len(), 1, "{:?}", report.issues);
        assert!(hits[0].message.contains("a.lost"), "{:?}", hits);
        assert!(hits[0].file.contains("ship.checklist.md"), "{:?}", hits);
    }

    /// every_item_accounted also owns the status set: a row whose
    /// status is neither `covered` nor `waived` leaves the item
    /// unaccounted-for.
    #[test]
    fn unknown_status_leaves_item_unaccounted() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | pending | [[x.file.c1]] | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.every_item_accounted"),
            "{:?}",
            report.issues
        );
    }

    /// covered_maps_resolve: an empty mapped_ids list and a dangling id
    /// both fail the covered claim.
    #[test]
    fn dangling_mapped_id_rejected() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[absent.row]] | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.covered_maps_resolve")
            .collect();
        assert_eq!(hits.len(), 1, "{:?}", report.issues);
        assert!(hits[0].message.contains("absent.row"), "{:?}", hits);
        // The empty-list variant.
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.covered_maps_resolve"),
            "empty mapped_ids on a covered row: {:?}",
            report.issues
        );
    }

    /// covered_maps_resolve must reject a target that resolves to a
    /// file id or section anchor but NOT to a constraint/property row —
    /// the claim rests on a row, not on a file existing.
    #[test]
    fn covered_target_must_be_a_row_not_a_file_or_anchor() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[x.file]], [[x.file.model.state]] | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.covered_maps_resolve")
            .count();
        assert_eq!(
            hits, 2,
            "file-id and anchor targets are not rows: {:?}",
            report.issues
        );
    }

    /// waiver_has_rationale: an empty rationale is an unexplained waiver.
    #[test]
    fn unrationalized_waiver_rejected() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | waived | |   |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.waiver_has_rationale")
            .collect();
        assert_eq!(hits.len(), 1, "{:?}", report.issues);
        assert!(hits[0].message.contains("a.first"), "{:?}", hits);
    }

    /// no_duplicate_claim: two rows targeting one item — and the checks
    /// compose without double-reporting it as every_item_accounted too.
    #[test]
    fn duplicate_claim_rejected() {
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| a.first | covered | [[x.file.c1]] | |\n| a.first | waived | | contradicts the first row |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        let dup: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.no_duplicate_claim")
            .collect();
        assert_eq!(dup.len(), 1, "{:?}", report.issues);
        assert!(dup[0].message.contains("a.first"), "{:?}", dup);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.every_item_accounted"),
            "the duplication is no_duplicate_claim's beat — no double-report: {:?}",
            report.issues
        );
    }

    /// checklist_well_formed: parser defects surface as findings — a
    /// manifest the linter cannot read must never silently vanish.
    #[test]
    fn malformed_checklist_rejected() {
        let cl = checklist("# just a title\n");
        let report = lint_all(&[mapped_spec()], &[cl]);
        let hits = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.checklist_well_formed")
            .count();
        assert_eq!(
            hits, 2,
            "missing Items + Mapping sections: {:?}",
            report.issues
        );
        // Orphan mapping row references an undeclared item.
        let cl = checklist(
            "## Items\n\n- **a.first**: sessions expire\n\n## Mapping\n\n| item | status | mapped_ids | rationale |\n|------|--------|------------|-----------|\n| ghost.item | covered | [[x.file.c1]] | |\n",
        );
        let report = lint_all(&[mapped_spec()], &[cl]);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.checklist_well_formed"
                    && i.message.contains("ghost.item")),
            "{:?}",
            report.issues
        );
    }

    #[test]
    fn catalog_covers_every_rule_the_linter_can_emit() {
        let report = lint_all(&fixture_corpus(), &checklist_fixtures());
        // Warnings count as emittable rule ids too — the observability
        // rule emits on the warnings channel, never as an Issue.
        let emitted: BTreeSet<String> = report
            .issues
            .iter()
            .chain(report.warnings.iter())
            .map(|i| {
                i.rule_id
                    .strip_prefix("linter.")
                    .expect("rule_id prefix")
                    .to_string()
            })
            .collect();
        let catalog: BTreeSet<&str> = RULE_TABLE
            .iter()
            .map(|(name, _)| *name)
            // the pack rules are exempt here — they cannot fire from this
            // text-only fixture corpus (the pack pass reads files from
            // disk); their coverage contract is tests/cli.rs's pack suite
            // (task 3.3: every manifest facet mutation → labeled finding)
            .filter(|name| !matches!(*name, "pack_shape" | "orphan_vocabulary" | "skew_advisory"))
            .collect();
        let catalog_set: BTreeSet<String> = catalog.into_iter().map(String::from).collect();
        assert_eq!(
            emitted, catalog_set,
            "the rule table and the emittable rule ids must coincide — \
             a rule missing from the table panics at construction, one \
             missing from the corpus means the fixture stopped covering it"
        );
    }

    /// Observability semantics (specs/linter-observability.md): the
    /// warning channel, not the issues channel; self-observation does
    /// not count; observed effects are silent; failures() is untouched.
    #[test]
    fn unobserved_effect_warns_but_never_fails() {
        let spec = spec_at(
            "---\nid: obs.solo\nkind: intent\nstatement: \"THE solo SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `v finite` | [[obs.solo]] |\n| eff | effect | `output == {v}` | [[obs.solo]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.solo.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.solo.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.solo.eff]] | `g()` | `x` |\n",
            "obs-solo.md",
        );
        let report = lint_corpus(&[spec]);
        assert!(
            report.issues.is_empty(),
            "advisory must never be an issue: {:?}",
            report.issues
        );
        assert_eq!(report.failures(), 0, "failures() counts issues only");
        let w = report
            .warnings
            .iter()
            .find(|w| w.rule_id == "linter.observability")
            .expect("the unobserved effect must be warned");
        assert!(
            w.message.contains("obs.solo.eff"),
            "warning names the row: {}",
            w.message
        );
        assert_eq!(w.rule_semantics, rule_semantics("observability").unwrap());
    }

    #[test]
    fn observed_effect_is_silent_and_self_observation_does_not_count() {
        let make = |observes: &str| {
            spec_at(
                &format!(
                    "---\nid: obs.pair\nkind: intent\nstatement: \"THE pair SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| inv | invariant | `v finite` | [[obs.pair]] | |\n| eff | effect | `output == {{v}}` | [[obs.pair]] | {observes} |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.pair.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.pair.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.pair.eff]] | `g()` | `x` |\n"
                ),
                "obs-pair.md",
            )
        };
        // A DIFFERENT row in the same file observing counts.
        let observing = spec_at(
            "---\nid: obs.pair\nkind: intent\nstatement: \"THE pair SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| inv | invariant | `v finite` | [[obs.pair]] | |\n| eff | effect | `output == {{v}}` | [[obs.pair]] | |\n| watch | invariant | `seen` | [[obs.pair]] | [[obs.pair.eff]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.pair.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.pair.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.pair.eff]] | `g()` | `x` |\n| p_w | unit | [[obs.pair.eff]] | `g()` | `x` |\n",
            "obs-pair.md",
        );
        let report = lint_corpus(&[observing]);
        assert!(
            report
                .warnings
                .iter()
                .all(|w| w.rule_id != "linter.observability"),
            "a different-row observation counts: {:?}",
            report.warnings
        );
        // The effect row observing ITSELF does not count.
        let self_observing = make("[[obs.pair.eff]]");
        let report = lint_corpus(&[self_observing]);
        assert!(
            report
                .warnings
                .iter()
                .any(|w| w.rule_id == "linter.observability"),
            "self-observation is vacuous — still warned: {:?}",
            report.warnings
        );
    }

    #[test]
    fn dangling_observes_is_total_refs_beat_not_observability() {
        // The effect is observed by a resolving edge; a second row's
        // observes dangles. The dangling target must produce exactly one
        // total_refs finding and zero observability warnings — the
        // checks compose without double-reporting the same row (the
        // effect itself is genuinely observed, so it is silent too).
        let spec = spec_at(
            "---\nid: obs.ghost\nkind: intent\nstatement: \"THE ghost SHALL emit\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to | observes |\n|----|------|------|-----------|----------|\n| inv | invariant | `v finite` | [[obs.ghost]] | |\n| eff | effect | `output == {{v}}` | [[obs.ghost]] | |\n| watch | invariant | `seen` | [[obs.ghost]] | [[obs.ghost.eff]] |\n| lost | invariant | `other` | [[obs.ghost]] | [[obs.absent.effect]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[obs.ghost.inv]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p_inv | unit | [[obs.ghost.inv]] | `g()` | `x` |\n| p_eff | unit | [[obs.ghost.eff]] | `g()` | `x` |\n| p_watch | unit | [[obs.ghost.eff]] | `g()` | `x` |\n| p_lost | unit | [[obs.ghost.inv]] | `g()` | `x` |\n",
            "obs-ghost.md",
        );
        let report = lint_corpus(&[spec]);
        let dangling_refs: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.total_refs")
            .collect();
        assert_eq!(
            dangling_refs.len(),
            1,
            "exactly one total_refs finding for the dangling target: {:?}",
            report.issues
        );
        assert!(
            report
                .warnings
                .iter()
                .all(|w| w.rule_id != "linter.observability"),
            "zero observability warnings — the checks compose without double-reporting: {:?}",
            report.warnings
        );
    }

    #[test]
    fn orphan_property_fires_when_derives_from_resolves_to_property() {
        // specodelic-rk3: linter-coverage.md's no_orphan_property reads
        // "p.derives_from resolves to a real CONSTRAINT" — presence of a
        // link alone is not enough. A unit property deriving from a
        // non-law Property row must fire the rule (the law→Property edge
        // is the one legal same-kind derivation, specodelic-cxq).
        let spec = spec_at(
            "---\nid: t\nkind: intent\nstatement: \"THE system SHALL derive\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| base | unit | [[t.c]] | `g()` | `x` |\n| extra | unit | [[t.base]] | `g()` | `x` |\n",
            "t.md",
        );
        let report = lint_corpus(&[spec]);
        let orphan = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.no_orphan_property")
            .expect("deriving from a non-law Property is not coverage");
        assert!(
            orphan.message.contains("extra"),
            "the finding names the offending property: {}",
            orphan.message
        );
        assert!(
            orphan.message.contains("constraint"),
            "the finding states the required target kind: {}",
            orphan.message
        );
    }

    #[test]
    fn law_property_deriving_from_property_is_not_an_orphan() {
        // The cxq law-restates-law edge is the sanctioned same-kind
        // derivation — no_orphan_property must stay silent for it.
        let spec = spec_at(
            "---\nid: t\nkind: intent\nstatement: \"THE system SHALL derive\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| base | unit | [[t.c]] | `g()` | `x` |\n| restated | law | [[t.base]] | `g()` | `x` |\n",
            "t.md",
        );
        let report = lint_corpus(&[spec]);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.no_orphan_property"),
            "law→Property is a legal derivation: {:?}",
            report
                .issues
                .iter()
                .filter(|i| i.rule_id == "linter.no_orphan_property")
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn unresolved_derives_from_target_stays_total_refs_beat() {
        // A dangling derives_from is total_refs' finding (the invariant
        // restates total_refs scoped to this edge) — no_orphan_property
        // must not double-fire on a target that resolves to nothing.
        let spec = spec_at(
            "---\nid: t\nkind: intent\nstatement: \"THE system SHALL derive\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[t.ghost]] | `g()` | `x` |\n",
            "t.md",
        );
        let report = lint_corpus(&[spec]);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.total_refs"),
            "the dangling target is total_refs' finding"
        );
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.no_orphan_property"),
            "unresolved targets are not no_orphan_property's beat"
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
    fn requirement_drift_fires_on_mirrored_section_drift() {
        // gh#4: explain dual-format promises a drift check; the mirror
        // must match the ADDED section (normalized like
        // scripts/check_section_sync.py: per-line trailing space and
        // blank lines ignored).
        let drifted = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds BUT THE MIRROR DRIFTED\n",
            "spec.md",
        );
        let report = lint_corpus(&[drifted]);
        let drift = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.requirement_drift")
            .expect("drift finding fires on mismatched mirrors");
        assert!(
            drift.message.contains("## Requirements"),
            "finding must name the mirror section: {}",
            drift.message
        );
        // Identical mirrors (differing only in blank lines / trailing
        // space) stay clean.
        let clean = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds  \n\n\n",
            "spec.md",
        );
        let report = lint_corpus(&[clean]);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.requirement_drift"),
            "blank-line/trailing-space-only differences must not fire: {:?}",
            report.issues
        );
    }

    #[test]
    fn requirement_drift_allows_mixed_added_and_modified_deltas() {
        // gh#8 (specodelic-eh0): a dual-format file carrying BOTH delta
        // sections was unsatisfiable — whole-section equality can never
        // match a mirror that holds both sections' requirements. The rule
        // is per-requirement: every delta requirement must appear in the
        // mirror with identical normalized text.
        let satisfiable = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: Two\ntwo holds\n\n## Requirements\n\n### Requirement: One\none holds\n\n### Requirement: Two\ntwo holds\n",
            "spec.md",
        );
        let report = lint_corpus(&[satisfiable]);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.requirement_drift"),
            "a mirror holding both delta sections' requirements must be clean: {:?}",
            report.issues
        );

        // Mirror missing one delta requirement → fires, naming it.
        let missing = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## MODIFIED Requirements\n\n### Requirement: Two\ntwo holds\n\n## Requirements\n\n### Requirement: One\none holds\n",
            "spec.md",
        );
        let report = lint_corpus(&[missing]);
        let drift = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.requirement_drift")
            .expect("mirror missing a delta requirement must fire");
        assert!(
            drift.message.contains("Two") && drift.message.contains("## MODIFIED Requirements"),
            "finding must name the missing requirement and its delta section: {}",
            drift.message
        );

        // One requirement's text drifted in the mirror → fires, naming it.
        let drifted = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## ADDED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds BUT THE MIRROR DRIFTED\n",
            "spec.md",
        );
        let report = lint_corpus(&[drifted]);
        let drift = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.requirement_drift")
            .expect("drifted requirement text must fire");
        assert!(
            drift.message.contains("One"),
            "finding must name the drifted requirement: {}",
            drift.message
        );
    }

    #[test]
    fn total_refs_hint_points_at_the_file_qualified_form() {
        // gh#5: a bare dotted row ref in a self-contained dual-format file
        // dangles with 'not defined in any spec file' — the finding must
        // hint that the row exists right here and show the fixed form.
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| thing.one | invariant | `x` | [[spec]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s1 | [[thing.one]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[spec.thing.one]] | `g()` | `x` |\n",
            "spec.md",
        );
        let report = lint_corpus(&[spec]);
        let dangling = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.total_refs")
            .expect("bare dotted row ref dangles");
        assert!(
            dangling.message.contains("[[spec.thing.one]]"),
            "hint must show the file-qualified fix: {}",
            dangling.message
        );
        assert!(
            dangling.message.contains("defined in this file"),
            "hint must say the row is local: {}",
            dangling.message
        );
    }

    #[test]
    fn total_refs_hint_absent_for_genuinely_unknown_targets() {
        // gh#5: the hint only fires when the row actually exists in the
        // same file — unknown targets keep the plain message.
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[spec]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s1 | [[spec.nope]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[spec.c]] | `g()` | `x` |\n",
            "spec.md",
        );
        let report = lint_corpus(&[spec]);
        let dangling = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.total_refs")
            .expect("unknown row dangles");
        assert!(
            !dangling.message.contains("defined in this file"),
            "no hint for a target that exists nowhere: {}",
            dangling.message
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
    fn law_row_with_prose_only_cases_is_rejected() {
        // update-law-named-cases 3.1: the distinguishing case — a prose
        // mention of a case name is not an enumeration (specodelic.md
        // Revision 13). This file lints clean before the rule ships.
        let spec = spec_at(
            &LAW_FIXTURE.replace("PREDICATE", "associative and has an identity element"),
            "b-law.md",
        );
        let report = lint_all(&[spec], &checklist_fixtures());
        let finding = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.law_cases")
            .expect("prose-only law predicate must fire linter.law_cases");
        assert!(
            finding
                .message
                .contains("[\"identity\", \"associativity\"]"),
            "the finding names both missing floor cases: {:?}",
            finding.message
        );
    }

    #[test]
    fn law_row_missing_one_floor_case_names_only_it() {
        // update-law-named-cases 3.2: identity labeled, associativity
        // not — the finding names the missing one.
        let spec = spec_at(
            &LAW_FIXTURE.replace("PREDICATE", "**identity:** `f(a) == a`"),
            "b-law.md",
        );
        let report = lint_all(&[spec], &checklist_fixtures());
        let finding = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.law_cases")
            .expect("missing associativity must fire");
        assert!(
            finding.message.contains("[\"associativity\"]"),
            "only the missing case is named: {:?}",
            finding.message
        );
    }

    #[test]
    fn law_row_with_floor_and_extra_case_is_clean() {
        // update-law-named-cases 3.3: the floor plus an extra named
        // case — a first-class checkable declaration, no finding.
        let spec = spec_at(
            &LAW_FIXTURE.replace(
                "PREDICATE",
                "**identity:** `f(a) == a` **associativity:** `f(f(a)) == f(a)` **commutativity:** `f(a, b) == f(b, a)`",
            ),
            "b-law.md",
        );
        let report = lint_all(&[spec], &checklist_fixtures());
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.law_cases"),
            "floor + extra case is clean: {:?}",
            report.issues
        );
    }

    #[test]
    fn misspelled_floor_label_is_not_the_floor() {
        // update-law-named-cases 3.3b: **identiy:** is not identity —
        // the floor must be present by name.
        let spec = spec_at(
            &LAW_FIXTURE.replace("PREDICATE", "**identiy:** `x`"),
            "b-law.md",
        );
        let report = lint_all(&[spec], &checklist_fixtures());
        let finding = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.law_cases")
            .expect("a misspelled floor label is not the floor");
        assert!(
            finding
                .message
                .contains("[\"identity\", \"associativity\"]"),
            "{:?}",
            finding.message
        );
    }

    #[test]
    fn unit_rows_never_trigger_law_cases() {
        // update-law-named-cases 3.4: the rule is law-kind only — a
        // unit row with case-like text in its predicate never fires.
        let spec = spec_at(
            &LAW_FIXTURE
                .replace("| nat | law |", "| nat | unit |")
                .replace("PREDICATE", "identity of the accumulator holds"),
            "b-law.md",
        );
        let report = lint_all(&[spec], &checklist_fixtures());
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.law_cases"),
            "{:?}",
            report.issues
        );
    }

    #[test]
    fn modified_delta_with_drifted_mirror_fires_requirement_drift() {
        // update-law-named-cases 4.1: the repo's first ## MODIFIED
        // delta must be mirror-checked exactly like an ADDED one.
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## MODIFIED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds BUT THE MIRROR DRIFTED\n",
            "spec.md",
        );
        let report = lint_all(&[spec], &checklist_fixtures());
        let finding = report
            .issues
            .iter()
            .find(|i| i.rule_id == "linter.requirement_drift")
            .expect("a drifted MODIFIED mirror must fire requirement_drift");
        assert!(
            finding.message.contains("## MODIFIED Requirements"),
            "the finding names the delta section: {:?}",
            finding.message
        );
    }

    #[test]
    fn modified_delta_with_non_spec_id_fires_dual_format_valid() {
        // update-law-named-cases 4.2: a MODIFIED-carrying file must
        // declare id: spec, exactly like an ADDED-carrying one.
        let spec = spec_at(
            "---\nid: demo.thing\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## MODIFIED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds\n",
            "spec.md",
        );
        let report = lint_all(&[spec], &checklist_fixtures());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.dual_format_valid"),
            "MODIFIED-carrying file with a non-spec id must fire dual_format_valid: {:?}",
            report.issues
        );
    }

    #[test]
    fn modified_delta_in_sync_lints_clean() {
        // update-law-named-cases 4.x: the widening is additive — an
        // in-sync MODIFIED delta with id: spec fires neither rule.
        let spec = spec_at(
            "---\nid: spec\nkind: intent\nstatement: \"THE change SHALL be dual-format\"\n---\n\n## MODIFIED Requirements\n\n### Requirement: One\none holds\n\n## Requirements\n\n### Requirement: One\none holds\n",
            "spec.md",
        );
        let report = lint_all(&[spec], &checklist_fixtures());
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.dual_format_valid"
                    || i.rule_id == "linter.requirement_drift"),
            "{:?}",
            report.issues
        );
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

    // --- specodelic-b15: model_shape remainder + graph_shape ---

    /// model_shape.every_state_used: a declared state that no transition
    /// references is a mode the machine can never enter or leave.
    #[test]
    fn unused_state_fails_every_state_used() {
        let spec = spec_at(
            "---\nid: d.shape\nkind: intent\nstatement: \"THE system SHALL shape\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[d.shape]] |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | `g` |\n",
            "d-shape.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.every_state_used")
            .collect();
        assert_eq!(hits.len(), 1, "s2 is declared but never used: {:?}", hits);
        assert!(
            hits[0].message.contains("s2"),
            "finding must name the unused state: {}",
            hits[0].message
        );
    }

    #[test]
    fn every_state_in_a_transition_passes() {
        let spec = spec_at(
            "---\nid: d.shape\nkind: intent\nstatement: \"THE system SHALL shape\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[d.shape]] |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | `g` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.shape.c]] | `g()` | `x` |\n",
            "d-shape.md",
        );
        let report = lint_corpus(&[spec]);
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "linter.every_state_used"),
            "all states used — no finding: {:?}",
            report.issues
        );
    }

    /// model_shape.every_transition_valid: from/to must name declared
    /// states — a dangling `to` would make the model-checker simulate a
    /// different graph than the author wrote.
    #[test]
    fn transition_to_undeclared_state_fails_every_transition_valid() {
        let spec = spec_at(
            "---\nid: d.shape\nkind: intent\nstatement: \"THE system SHALL shape\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[d.shape]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | ghost | `g` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.shape.c]] | `g()` | `x` |\n",
            "d-shape.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.every_transition_valid")
            .collect();
        assert_eq!(hits.len(), 1, "`ghost` is not a declared state: {:?}", hits);
        assert!(
            hits[0].message.contains("ghost"),
            "finding must name the undeclared state: {}",
            hits[0].message
        );
    }

    /// graph_shape.no_self_ref: a row referencing itself traces to
    /// nothing that owns it.
    #[test]
    fn self_referencing_row_fails_no_self_ref() {
        let spec = spec_at(
            "---\nid: d.self\nkind: intent\nstatement: \"THE system SHALL not self-reference\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[d.self.a]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[d.self.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.self.a]] | `g()` | `x` |\n",
            "d-self.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.no_self_ref")
            .collect();
        assert_eq!(hits.len(), 1, "a traces_to itself: {:?}", hits);
        assert!(
            hits[0].message.contains("traces_to"),
            "finding names the offending column: {}",
            hits[0].message
        );
    }

    /// graph_shape.acyclic: a traces_to cycle is rejected. The edge set
    /// is traces_to ∪ derives_from ∪ guard-as-edge (spec text) — a
    /// two-row mutual trace is the minimal cycle.
    #[test]
    fn mutual_traces_cycle_fails_acyclic() {
        let spec = spec_at(
            "---\nid: d.cycle\nkind: intent\nstatement: \"THE system SHALL stay acyclic\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| a | invariant | `x` | [[d.cycle.b]] |\n| b | invariant | `y` | [[d.cycle.a]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[d.cycle.a]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.cycle.a]] | `g()` | `x` |\n",
            "d-cycle.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.acyclic")
            .collect();
        assert_eq!(hits.len(), 1, "one 2-cycle, one finding: {:?}", hits);
        assert!(
            hits[0].message.contains("a") && hits[0].message.contains("b"),
            "finding names the cycle members: {}",
            hits[0].message
        );
    }

    /// graph_shape.single_root_reachable: every row must reach its
    /// file's OWN intent through own-file primary linkage (specodelic.md
    /// Revision 10) — a row with no path to ANY intent (an island of
    /// rows tracing only to each other) has no owning purpose and
    /// hard-fails.
    #[test]
    fn orphan_cluster_fails_single_root_reachable() {
        let spec = spec_at(
            "---\nid: d.island\nkind: intent\nstatement: \"THE system SHALL anchor every row\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| anchored | invariant | `x` | [[d.island]] |\n| lost | invariant | `y` | |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[d.island.anchored]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[d.island.lost]] | `g()` | `y` |\n",
            "d-island.md",
        );
        let report = lint_corpus(&[spec]);
        let hits: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "linter.single_root_reachable")
            .collect();
        assert_eq!(
            hits.len(),
            1,
            "`lost` has no path to any intent: {:?}",
            hits
        );
        assert!(
            hits[0].message.contains("lost"),
            "finding names the disconnected row: {}",
            hits[0].message
        );
    }

    /// Pinning test (specodelic-erb item 4): reachability semantics are
    /// OWN-FILE (Revision 10), not corpus-wide — a component whose only
    /// tie to the graph is a cross-file typed edge must produce the
    /// advisory warning (never a silent pass), while the same shape with
    /// an own-file guard chain passes. Regressing to the old
    /// some-intent connectivity must fail this test.
    #[test]
    fn cross_file_only_rows_warn_advisory_own_file_chain_passes() {
        let base = spec_at(
            "---\nid: base\nkind: intent\nstatement: \"THE base SHALL anchor\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| base_inv | invariant | `x` | [[base]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[base.base_inv]] |\n",
            "base.md",
        );
        // Consumer: the transition+state component's only tie is the
        // cross-file guard citation of base's constraint.
        let consumer = spec_at(
            "---\nid: consumer\nkind: intent\nstatement: \"THE consumer SHALL consume\"\n---\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[base.base_inv]] |\n",
            "consumer.md",
        );
        let report = lint_corpus(&[base.clone(), consumer.clone()]);
        assert!(
            report
                .issues
                .iter()
                .all(|i| i.rule_id != "linter.single_root_reachable"),
            "cross-file-only rows must not hard-fail: {:?}",
            report.issues
        );
        assert!(
            report
                .warnings
                .iter()
                .any(|w| w.rule_id == "linter.single_root_reachable"
                    && w.file == "consumer"
                    && w.message.contains("t")),
            "cross-file-only component must ride the advisory tier: {:?}",
            report.warnings
        );
        // Same shape with an OWN-FILE guard chain: no warning, no issue.
        let own = spec_at(
            "---\nid: own.file\nkind: intent\nstatement: \"THE own SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| inv | invariant | `x` | [[own.file]] |\n\n## Model\n\n### States\n\n- `s1`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s1 | [[own.file.inv]] |\n",
            "own-file.md",
        );
        let report = lint_corpus(&[own]);
        assert!(
            report
                .issues
                .iter()
                .all(|i| i.rule_id != "linter.single_root_reachable")
                && report
                    .warnings
                    .iter()
                    .all(|w| w.rule_id != "linter.single_root_reachable"),
            "own-file primary linkage must pass clean: {:?} / {:?}",
            report.issues,
            report.warnings
        );
    }

    #[test]
    fn well_formed_file_passes_graph_and_model_shape() {
        // A well-formed file (every state used, valid transitions, no
        // self-refs, everything anchored) emits none of the new rules.
        let spec = spec_at(
            "---\nid: ok.shape\nkind: intent\nstatement: \"THE system SHALL shape\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c | invariant | `x` | [[ok.shape]] |\n\n## Model\n\n### States\n\n- `s1`\n- `s2`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t1 | s1 | s2 | [[ok.shape.c]] |\n| t2 | s2 | s1 | `x` |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[ok.shape.c]] | `g()` | `x` |\n",
            "ok-shape.md",
        );
        let report = lint_corpus(&[spec]);
        for rule in [
            "every_state_used",
            "every_transition_valid",
            "no_self_ref",
            "acyclic",
            "single_root_reachable",
        ] {
            assert!(
                !report
                    .issues
                    .iter()
                    .any(|i| i.rule_id == format!("linter.{rule}")),
                "well-formed file must not fire {rule}: {:?}",
                report.issues
            );
        }
    }

    // --- specodelic-njh: reference-resolution algorithm pinned (gh#1) ---

    /// Index fixture from (file id, row ids) pairs.
    fn index_from(pairs: &[(&str, &[&str])]) -> Index {
        Index {
            files: pairs
                .iter()
                .map(|(f, rows)| {
                    (
                        f.to_string(),
                        rows.iter().map(|r| r.to_string()).collect::<BTreeSet<_>>(),
                    )
                })
                .collect(),
        }
    }

    /// RED (specodelic-njh): a multi-segment MEMBER path in the
    /// first-dot fallback arm — `a.b.c.d` = file `a`, row `b`, member
    /// path `c.d` — must resolve the same way the last-dot arm does.
    /// `resolves_row` (checklists) already split members in both arms;
    /// `Index::resolves` and `resolve_node` did not, so a checklist item
    /// could rest on a row that lint itself reported as dangling.
    #[test]
    fn member_path_resolves_in_first_dot_arm() {
        let index = index_from(&[("a", &["b"])]);
        assert!(
            index.resolves("a", "a.b.c.d"),
            "file a + row b + member path c.d must resolve"
        );
        assert_eq!(
            resolve_node(&index, "a", "a.b.c.d"),
            Some("a.b".to_string()),
            "graph node is the row, not the member"
        );
        assert!(
            resolves_row(&index, "a.b.c.d"),
            "checklist side already resolved"
        );
    }

    /// Pinned (specodelic-njh): ROW ids are single-segment — a target
    /// whose tail names a dotted "row" means row.member, never a dotted
    /// row. `a.b.c.d` with file `a` holding only row `b.c` does NOT
    /// resolve: it means row `b` (absent) with member path `c.d`.
    #[test]
    fn dotted_row_ids_are_unaddressable() {
        let index = index_from(&[("a", &["b.c"])]);
        assert!(!index.resolves("a", "a.b.c.d"));
        assert_eq!(resolve_node(&index, "a", "a.b.c.d"), None);
    }

    /// Pinned (specodelic-njh): the same 4-segment shape when the FILE id
    /// carries the dot — last-dot arm, the path gh#1 most likely hit.
    #[test]
    fn dotted_file_id_row_member_resolves_in_last_dot_arm() {
        let index = index_from(&[("a.b", &["c"])]);
        assert!(index.resolves("a.b", "a.b.c.d"));
        assert_eq!(
            resolve_node(&index, "a.b", "a.b.c.d"),
            Some("a.b.c".to_string())
        );
    }

    /// Pinned (specodelic-njh): dotted FILE ids resolve via last-dot split —
    /// the case the gh#1 reporter named (intent id contains a dot).
    #[test]
    fn dotted_file_id_row_resolves() {
        let index = index_from(&[("extraction.claims", &["span"])]);
        assert!(index.resolves("other", "extraction.claims.span"));
        assert_eq!(
            resolve_node(&index, "other", "extraction.claims.span"),
            Some("extraction.claims.span".to_string())
        );
    }

    /// Pinned (specodelic-njh): precedence — an exact file id beats any
    /// split interpretation (`a.b` is file `a.b`, not file `a`'s row `b`).
    #[test]
    fn exact_file_id_beats_split_arms() {
        let index = index_from(&[("a", &["b"]), ("a.b", &[])]);
        assert!(index.resolves("a", "a.b"));
        assert_eq!(resolve_node(&index, "a", "a.b"), Some("a.b".to_string()));
    }

    /// Pinned (specodelic-njh): section anchors resolve for lint (they are
    /// machinery, not dangling) but name no graph row — no edge.
    #[test]
    fn anchors_resolve_but_name_no_row() {
        let index = index_from(&[("a", &[])]);
        assert!(index.resolves("a", "model.state"));
        assert!(index.resolves("a", "a.model.state"));
        assert_eq!(resolve_node(&index, "a", "model.state"), None);
        assert_eq!(resolve_node(&index, "a", "a.model.state"), None);
        assert!(
            !resolves_row(&index, "a.model.state"),
            "anchors are not rows"
        );
    }

    /// Pinned (specodelic-njh): bare-local rows resolve only against the
    /// SOURCE file's own rows; a bare target that is some other file's id
    /// resolves as that file.
    #[test]
    fn bare_local_row_and_bare_file_id() {
        let index = index_from(&[("a", &["r"]), ("f", &[])]);
        assert!(index.resolves("a", "r"), "own row in bare spelling");
        assert_eq!(resolve_node(&index, "a", "r"), Some("a.r".to_string()));
        assert!(index.resolves("a", "f"), "bare file id wins over nothing");
        assert!(
            !index.resolves("f", "r"),
            "another file's row is not bare-local"
        );
    }

    /// Pinned (specodelic-njh): metasyntactic targets — single-segment
    /// non-file ids and ellipsis — never dangle (format prose, not refs).
    #[test]
    fn metasyntactic_targets_are_skipped() {
        let index = index_from(&[("a", &[])]);
        assert!(is_metasyntactic("x", &index));
        assert!(is_metasyntactic("...", &index));
        assert!(is_metasyntactic("…", &index));
        assert!(
            !is_metasyntactic("a", &index),
            "a real file id is not metasyntactic"
        );
        assert!(
            !is_metasyntactic("a.b", &index),
            "dotted targets are never metasyntactic"
        );
    }
}

#[cfg(test)]
mod checker_tests {
    //! specodelic-8kk: the orchestrator invokes checkers individually —
    //! pin each checker family's rule attribution so a rule can never
    //! silently migrate between checkers.

    use super::*;

    fn spec_from(text: &str, path: &str) -> Spec {
        let mut s = crate::spec::parse_str(text).unwrap();
        s.path = Some(std::path::PathBuf::from(path));
        s
    }

    const CLEAN: &str = "---\nid: orchestrate.fix\nkind: intent\nstatement: \"THE system SHALL hold\"\n---\n\n## Constraints\n\n| id | kind | expr | traces_to |\n|----|------|------|-----------|\n| c1 | invariant | `holds` | [[orchestrate.fix]] |\n\n## Model\n\n### States\n\n- s1\n- s2\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| t | s1 | s2 | [[orchestrate.fix.c1]] |\n\n## Properties\n\n| id | kind | derives_from | generator | predicate |\n|----|------|--------------|-----------|------------|\n| p | unit | [[orchestrate.fix.c1]] | `g()` | `x` |\n";

    fn rules_of(issues: &[Issue]) -> Vec<String> {
        let mut v: Vec<String> = issues.iter().map(|i| i.rule_id.clone()).collect();
        v.sort();
        v.dedup();
        v
    }

    #[test]
    fn frontmatter_family_owns_file_structure_rules() {
        // Wrong kind fires frontmatter_valid; also verify a dual-format
        // drift file attributes to this family, not another checker.
        let bad = spec_from(
            "---\nid: orchestrate.fix\nkind: feature\nstatement: \"THE system SHALL hold\"\n---\n",
            "orchestrate-fix.md",
        );
        let rules = rules_of(&frontmatter_findings(&[bad]));
        assert_eq!(rules, vec!["linter.frontmatter_valid"]);
    }

    #[test]
    fn referential_family_owns_unique_id_and_total_refs() {
        let dup = spec_from(
            &CLEAN.replace(
                "| p | unit | [[orchestrate.fix.c1]] |",
                "| c1 | unit | [[orchestrate.fix.c1]] |",
            ),
            "orchestrate-fix.md",
        );
        let rules = rules_of(&referential_findings(&[dup]));
        assert!(
            rules.contains(&"linter.unique_id".to_string()),
            "duplicate row id fires in the referential family: {rules:?}"
        );
        assert!(
            !rules.contains(&"linter.acyclic".to_string()),
            "graph rules never leak into the referential family"
        );
    }

    #[test]
    fn graph_family_owns_acyclic_and_reachability() {
        // A self-tracing row: no_self_ref is referential's; acyclic is
        // graph's. Use a traces_to cycle across two rows in one file.
        let cyc = spec_from(
            &CLEAN.replace("| c1 | invariant | `holds` | [[orchestrate.fix]] |",
                           "| c1 | invariant | `holds` | [[orchestrate.fix.p]] |\n| c2 | invariant | `holds2` | [[orchestrate.fix.c1]] |")
                   .replace("| p | unit | [[orchestrate.fix.c1]] |", "| p | unit | [[orchestrate.fix.c2]] |"),
            "orchestrate-fix.md",
        );
        let rules = rules_of(&graph_shape_findings(&[cyc]));
        assert!(
            rules.contains(&"linter.acyclic".to_string())
                || rules.contains(&"linter.single_root_reachable".to_string()),
            "cycle fires in the graph family: {rules:?}"
        );
        assert!(
            !rules.contains(&"linter.unique_id".to_string()),
            "referential rules never leak into the graph family"
        );
    }

    #[test]
    fn model_family_owns_guard_and_transition_rules() {
        let bad = spec_from(
            &CLEAN.replace(
                "| t | s1 | s2 | [[orchestrate.fix.c1]] |",
                "| t | s1 | s2 |  |",
            ),
            "orchestrate-fix.md",
        );
        let rules = rules_of(&model_shape_findings(&[bad]));
        assert_eq!(rules, vec!["linter.guard_required"]);
    }

    #[test]
    fn ears_family_owns_statement_and_id_shape_rules() {
        let bad = spec_from(
            "---\nid: orchestrate.always_fix\nkind: intent\nstatement: \"the system should maybe work\"\n---\n",
            "orchestrate-always-fix.md",
        );
        let rules = rules_of(&ears_findings(&[bad]));
        assert_eq!(
            rules,
            vec!["linter.ears_syntax", "linter.no_universal_in_id"]
        );
    }

    #[test]
    fn families_composed_equal_the_flat_pass() {
        // The family split must be behavior-preserving: composing the
        // per-file families reproduces lint_corpus's per-file issues
        // exactly (order included).
        let corpus = r#"
---
id: orchestrate.bad
kind: intent
statement: "maybe works"
---

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | `holds` | [[orchestrate.bad]] |
| c1 | effect | `x` | [[orchestrate.bad]] |

## Model

### States

- s1

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t | s1 | s9 |  |
"#;
        let spec = spec_from(corpus, "orchestrate-bad.md");
        let flat = {
            let mut r = empty_report(1);
            lint_one(&spec, std::slice::from_ref(&spec), &mut r);
            r.issues
        };
        let composed = {
            let mut r = empty_report(1);
            lint_frontmatter_family(&spec, &mut r);
            lint_referential_family(&spec, &mut r);
            lint_model_family(&spec, &mut r);
            lint_ears_family(&spec, &mut r);
            lint_failure_shape_family(&spec, &mut r);
            lint_schema_shape_family(&spec, std::slice::from_ref(&spec), &mut r);
            r.issues
        };
        assert_eq!(flat.len(), composed.len(), "same finding count");
        for (f, c) in flat.iter().zip(composed.iter()) {
            assert_eq!(f.rule_id, c.rule_id, "same rule in same order");
            assert_eq!(f.file, c.file);
            assert_eq!(f.message, c.message);
        }
    }

    #[test]
    fn schema_shape_checker_reports_no_table_rules() {
        // Honest emptiness: a CLEAN file fires nothing — the checker must
        // not fabricate findings for well-formed rows.
        let s = spec_from(CLEAN, "orchestrate-fix.md");
        assert!(schema_shape_findings(&[s]).is_empty());
    }

    #[test]
    fn made_up_constraint_kind_fires_constraint_kind_closed() {
        // specodelic-7h8: linter-schema_shape.md's constraint_kind_closed
        // becomes a real table-walking rule — the METER row: kind
        // `made_up` lints failed, message naming the closed set.
        let s = spec_from(
            &CLEAN.replace("| c1 | invariant |", "| c1 | made_up |"),
            "orchestrate-fix.md",
        );
        let issues = schema_shape_findings(&[s]);
        let issue = issues
            .iter()
            .find(|i| i.rule_id == "linter.constraint_kind_closed")
            .expect("kind made_up is outside the closed set");
        assert!(
            issue.message.contains("made_up"),
            "the finding names the offending kind: {}",
            issue.message
        );
        for k in crate::guide::CONSTRAINT_KINDS {
            assert!(
                issue.message.contains(k),
                "the finding lists the closed set member {k}: {}",
                issue.message
            );
        }
    }

    #[test]
    fn made_up_property_kind_fires_property_kind_closed() {
        let s = spec_from(
            &CLEAN.replace("| p | unit |", "| p | audit |"),
            "orchestrate-fix.md",
        );
        let issues = schema_shape_findings(&[s]);
        let issue = issues
            .iter()
            .find(|i| i.rule_id == "linter.property_kind_closed")
            .expect("kind audit is outside the closed set");
        assert!(
            issue.message.contains("audit") && issue.message.contains("law"),
            "the finding names the kind and the closed set: {}",
            issue.message
        );
    }

    #[test]
    fn missing_kind_cell_fires_the_kind_closed_rule() {
        // A kind cell the parser could not read is None — `None ∈ closed
        // set` is false; the row is outside the set by the invariant's
        // own reading.
        let s = spec_from(
            &CLEAN.replace("| c1 | invariant |", "| c1 |  |"),
            "orchestrate-fix.md",
        );
        let issues = schema_shape_findings(&[s]);
        assert!(
            issues
                .iter()
                .any(|i| i.rule_id == "linter.constraint_kind_closed"),
            "an unreadable kind cell is outside the closed set: {:?}",
            issues
        );
    }
}

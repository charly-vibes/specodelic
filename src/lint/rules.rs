//! Per-file checker families (split from mod.rs — specodelic-g17
//! file_lines ratchet): frontmatter, referential, model, failure_shape,
//! ears, and schema-shape rules.

use std::collections::{BTreeMap, BTreeSet};

use super::super::spec::Spec;
use super::graph::rows;
use super::{Issue, Report};
use crate::ears;
use crate::guide;

pub(crate) fn lint_schema_shape_family(spec: &Spec, corpus: &[Spec], report: &mut Report) {
    let fiber = crate::packs::active_fiber_kinds(spec, corpus);
    let fiber_kinds: Vec<&str> = fiber.iter().map(String::as_str).collect();
    let constraint_closed =
        |k: &str| crate::guide::CONSTRAINT_KINDS.contains(&k) || fiber_kinds.contains(&k);
    let property_closed =
        |k: &str| crate::guide::PROPERTY_KINDS.contains(&k) || fiber_kinds.contains(&k);
    let file_id = &spec.intent.id;
    for c in &spec.constraints {
        if !constraint_closed(c.kind.as_deref().unwrap_or("")) {
            report.issues.push(Issue::new(
                "constraint_kind_closed",
                file_id.clone(),
                format!(
                    "constraint `{}` has kind `{}` — outside the closed set {{{}}} (kinds.md constraint_row_shape)",
                    c.id,
                    c.kind.as_deref().unwrap_or(""),
                    crate::guide::CONSTRAINT_KINDS.join(", ")
                ),
            ));
        }
    }
    for p in &spec.properties {
        if !property_closed(p.kind.as_deref().unwrap_or("")) {
            report.issues.push(Issue::new(
                "property_kind_closed",
                file_id.clone(),
                format!(
                    "property `{}` has kind `{}` — outside the closed set {{{}}} (kinds.md property_row_shape)",
                    p.id,
                    p.kind.as_deref().unwrap_or(""),
                    crate::guide::PROPERTY_KINDS.join(", ")
                ),
            ));
        }
    }
}

/// `linter-coverage.md` — the compile gate ([[specodelic.coverage]]):
/// every constraint needs a deriving property. NOT one of the six
/// Checker Ownership checkers — the orchestrator runs it as compile's
/// gate, never as part of the lint gate.
pub(crate) fn file_label(spec: &Spec) -> String {
    spec.path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| format!("<{}>", spec.intent.id))
}

/// The expected frontmatter id for a file path (Revision 18,
/// specodelic-mcy): a file named `spec.md` derives its id from the
/// PARENT DIRECTORY name (`-` maps to `.`, `_` literal) — single-tree
/// spec authoring needs no `id: spec`; any other file derives from its
/// own stem. A bare `spec.md` with no parent directory falls back to
/// the stem (`spec`).
pub(crate) fn expected_id_from_path(path: &std::path::Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if stem == "spec"
        && let Some(dir) = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
    {
        return dir.replace('-', ".");
    }
    stem.replace('-', ".")
}

/// Checker-family slice of [`lint_one`]: the file-structure gate
/// (specs/specodelic.md Checker Ownership, `linter-frontmatter.md`).
/// Attribution note: `dual_format_valid`/`requirement_drift` postdate
/// the ownership table and ride this family — they are file-structure
/// rules (frontmatter id + section shape) enforced with the first gate.
pub(crate) fn lint_frontmatter_family(spec: &Spec, report: &mut Report) {
    let file = file_label(spec);

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

    // id_matches_file — frontmatter.id == expected_id_from_path(path):
    // stem-derived for ordinary files, parent-dir-derived for spec.md
    // (Revision 18, specodelic-mcy).
    if let Some(path) = &spec.path {
        let expected = expected_id_from_path(path);
        if spec.intent.id != expected {
            report.issues.push(Issue::new("id_matches_file", file.clone(), format!(
                    "frontmatter id `{}` does not match filename (expected `{}` — `-` maps to `.` in id; `_` is literal; a `spec.md` file derives its id from its parent directory)",
                    spec.intent.id, expected
                )));
        }
    }

    // dual_format_valid — a file carrying `## ADDED Requirements` or
    // `## MODIFIED Requirements` (the openspec delta halves;
    // update-law-named-cases widened the mirror rules to MODIFIED
    // additively — the repo's first MODIFIED delta must be gated
    // exactly like an ADDED one) must be a dual-format file: pair
    // each delta section with a sibling `## Requirements` section (the
    // capability half that survives archiving). Plain corpus specs (no
    // delta section) are exempt. Revision 18 (specodelic-mcy): the id
    // requirement retired — dual-format files carry REAL ids derived
    // by the naming law (spec.md from parent dir), not `id: spec`;
    // id_matches_file is the only id gate a dual-format file answers to.
    let delta_sections: Vec<(&str, &str)> = [
        (
            spec.has_added_requirements,
            "## ADDED Requirements",
            spec.added_requirements_body.as_str(),
        ),
        (
            spec.has_modified_requirements,
            "## MODIFIED Requirements",
            spec.modified_requirements_body.as_str(),
        ),
    ]
    .into_iter()
    .filter(|(present, _, _)| *present)
    .map(|(_, name, body)| (name, body))
    .collect();
    if !delta_sections.is_empty() {
        if !spec.has_requirements_section {
            report.issues.push(Issue::new(
                "dual_format_valid",
                file.clone(),
                format!(
                    "file carries {} without a sibling `## Requirements` section — not a dual-format file (the capability half is missing; migrate per the recipe in openspec/project.md: mirror the requirement content into ## Requirements, keep the specodelic tables alongside, then gates: spk lint + openspec validate + scripts/check_section_sync.py for drift)",
                    delta_sections.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(" and ")
                ),
            ));
        }

        // requirement_drift — when both halves of a dual-format file are
        // present, the mirror must match (gh#4: the migration recipe has
        // agents hand-create the mirror, so drift is easy and was previously
        // only caught by this repo's local section-sync script, never by
        // `spk lint`), for every delta section the file carries.
        // Comparison is PER-REQUIREMENT (gh#8 / specodelic-eh0): whole-section
        // equality is unsatisfiable for a file carrying both `## ADDED` and
        // `## MODIFIED Requirements` — no mirror matches both sections at
        // once. Every requirement in each delta section must appear in the
        // mirror with identical normalized text instead. Normalization
        // mirrors scripts/check_section_sync.py: per-line trailing space
        // and blank lines are ignored.
        if spec.has_requirements_section {
            // Split a requirements-section body into (heading, normalized
            // text) pairs at `### Requirement:` headings.
            let parse = |body: &str| -> Vec<(String, String)> {
                let mut reqs: Vec<(String, Vec<&str>)> = vec![];
                for line in body.lines() {
                    let t = line.trim();
                    if let Some(h) = t.strip_prefix("### Requirement:") {
                        reqs.push((h.trim().to_string(), vec![]));
                    } else if let Some((_, lines)) = reqs.last_mut() {
                        lines.push(t);
                    }
                }
                reqs.into_iter()
                    .map(|(h, lines)| {
                        (
                            h,
                            lines
                                .iter()
                                .filter(|l| !l.trim().is_empty())
                                .copied()
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    })
                    .collect()
            };
            let mirror = parse(&spec.requirements_body);
            for (section, body) in &delta_sections {
                for (heading, text) in parse(body) {
                    match mirror.iter().find(|(h, _)| *h == heading) {
                        None => report.issues.push(Issue::new(
                            "requirement_drift",
                            file.clone(),
                            format!(
                                "{heading} from {section} is missing from ## Requirements — the mirror must hold identical requirement text for every delta requirement (blank lines and trailing space ignored); regenerate it from the delta section"
                            ),
                        )),
                        Some((_, mirror_text)) if *mirror_text != text => {
                            report.issues.push(Issue::new(
                                "requirement_drift",
                                file.clone(),
                                format!(
                                    "{heading} in ## Requirements does not match {section} — the mirror must hold identical requirement text (blank lines and trailing space ignored); regenerate it from the delta section"
                                ),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Checker-family slice: `linter-referential_integrity.md`'s per-file
/// rule (`unique_id`; the corpus rules `total_refs`/`no_self_ref` live
/// in [`lint_references`], invoked by the same checker).
pub(crate) fn lint_referential_family(spec: &Spec, report: &mut Report) {
    let file = file_label(spec);

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
}

/// Checker-family slice: `linter-model_shape.md`'s per-file rules.
pub(crate) fn lint_model_family(spec: &Spec, report: &mut Report) {
    let file = file_label(spec);

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

    // every_state_used — a declared state no transition reaches is
    // machinery the model can never enter or leave (specs/
    // linter-model_shape.md).
    let state_ids: BTreeSet<&str> = spec.states.iter().map(|s| s.id.as_str()).collect();
    let used: BTreeSet<&str> = spec
        .transitions
        .iter()
        .flat_map(|t| [t.from.as_str(), t.to.as_str()])
        .collect();
    for sid in &spec.states {
        if !spec.transitions.is_empty() && !used.contains(sid.id.as_str()) {
            report.issues.push(Issue::new(
                "every_state_used",
                file.clone(),
                format!(
                    "state `{}` is declared but no transition enters or leaves it — every state must appear as from or to in at least one transition",
                    sid.id
                ),
            ));
        }
    }

    // every_transition_valid — from/to must name declared states; a
    // dangling `to` makes the model-checker simulate a different graph
    // than the author wrote.
    for t in &spec.transitions {
        for (label, endpoint) in [("from", &t.from), ("to", &t.to)] {
            if !state_ids.contains(endpoint.as_str()) {
                report.issues.push(Issue::new(
                    "every_transition_valid",
                    file.clone(),
                    format!(
                        "transition `{}` has {label} `{}` — not a declared state (states: {:?})",
                        t.id,
                        endpoint,
                        state_ids.iter().copied().collect::<Vec<_>>()
                    ),
                ));
            }
        }
    }
}

/// The recorded carve-out list (specs/linter-failure_shape.md Notes):
/// exactly orchestrate.md's stage-fail transitions. That file's Notes
/// decline to restate upstream files' logic as guard-citable rows, so
/// these guards stay prose — membership here is CHECKED data, never an
/// assumption: a zero-citation failure guard anywhere else is a finding.
const GUARD_CARVEOUT_FILE: &str = "orchestrate";
const GUARD_CARVEOUT_TRANSITIONS: &[&str] = &[
    "lint_fail",
    "compile_fail",
    "model_check_fail",
    "verify_fail",
];

/// Checker-family slice: `linter-failure_shape.md`'s three rules
/// (specodelic-ct5) — the tier-2 half of the error contract
/// (`errors.md`'s enforcement_routed row). Everything is graph-decidable
/// per file from the same derivation the
/// `failure_terminals_emit_labeled_errors` fixture pins at corpus
/// altitude (tests/cli.rs); nothing here reads prose.
///
/// v1 scope is failure terminals only — `model_check.md`'s `timed_out`
/// and `exploration_only` are the stated non-goal (the exit-code
/// mapping question is deferred at `errors.md`'s `exit_code_mapping`).
pub(crate) fn lint_failure_shape_family(spec: &Spec, report: &mut Report) {
    let file = file_label(spec);
    let file_id = spec.intent.id.as_str();
    let declared: BTreeSet<&str> = spec.states.iter().map(|s| s.id.as_str()).collect();
    // The failure-state test stays on the STATE segment: a node id is
    // `<file-id>.<state-id>` and a file id may itself contain
    // "failure" (linter.failure_shape) — within a Spec the state ids
    // are already local.
    let is_fail_state = |s: &str| s.contains("fail");

    // A transition's citation set: the sorted, deduped [[link]] targets
    // in its guard cell (the graph's transitions.guard edges).
    let citations = |tid: &str| -> Vec<String> {
        let mut v: Vec<String> = spec
            .links
            .iter()
            .filter(|l| l.field == "transitions" && l.column == "guard" && l.source == tid)
            .map(|l| l.target.clone())
            .collect();
        v.sort();
        v.dedup();
        v
    };
    // A state's emitted labels: the targets of its states.emits edges.
    let emits = |sid: &str| -> Vec<String> {
        spec.links
            .iter()
            .filter(|l| l.field == "states" && l.column == "emits" && l.source == sid)
            .map(|l| l.target.clone())
            .collect()
    };
    // A label is file-owned when it names a Constraint row of THIS file:
    // either the full file-namespaced spelling (`<file-id>.<row-id>`) or
    // the bare row id. Returns the row so kind and id can be checked.
    let owned_error_row = |target: &str| -> Option<&crate::spec::Row> {
        let rest = target
            .strip_prefix(&format!("{file_id}."))
            .unwrap_or(target);
        spec.constraints.iter().find(|c| c.id == rest)
    };

    let inbound: BTreeSet<&str> = spec.transitions.iter().map(|t| t.to.as_str()).collect();
    let outbound: BTreeSet<&str> = spec.transitions.iter().map(|t| t.from.as_str()).collect();

    // terminal_states_emit — every failure terminal (inbound transitions,
    // no outbound transitions, fail-named state) emits exactly one
    // file-owned effect Constraint.
    for st in &spec.states {
        if !inbound.contains(st.id.as_str())
            || outbound.contains(st.id.as_str())
            || !is_fail_state(&st.id)
        {
            continue;
        }
        let targets = emits(&st.id);
        if targets.is_empty() {
            report.issues.push(Issue::new(
                "terminal_states_emit",
                file.clone(),
                format!(
                    "failure terminal `{}` emits nothing — a mute failure terminal is a finding; add an emits edge to a file-owned effect Constraint: `- {} (emits: [[{file_id}.<label>]])`",
                    st.id, st.id
                ),
            ));
            continue;
        }
        if targets.len() > 1 {
            report.issues.push(Issue::new(
                "terminal_states_emit",
                file.clone(),
                format!(
                    "failure terminal `{}` emits {} labels ({:?}) — exactly one labeled error per failure state; split distinct failure classes into their own states",
                    st.id,
                    targets.len(),
                    targets
                ),
            ));
            continue;
        }
        let target = &targets[0];
        let Some(row) = owned_error_row(target) else {
            report.issues.push(Issue::new(
                "terminal_states_emit",
                file.clone(),
                format!(
                    "failure terminal `{}` emits `{target}` — not a file-owned Constraint; error labels name their owning file: `[[{file_id}.<label>]]`",
                    st.id
                ),
            ));
            continue;
        };
        if row.kind.as_deref() != Some("effect") {
            report.issues.push(Issue::new(
                "terminal_states_emit",
                file.clone(),
                format!(
                    "failure terminal `{}` emits `{target}` (kind {:?}) — a failure terminal must emit an effect Constraint (errors.md failure_state_emits)",
                    st.id,
                    row.kind.as_deref().unwrap_or("none")
                ),
            ));
        }
    }

    // error_labels_unique — no two emitted error Constraints share a
    // variant head (the label's last `.` segment). Cross-file collisions
    // are structurally impossible by errors.md's error_expr_shape; this
    // per-file check exists so a future format change cannot silently
    // drop the namespacing law.
    {
        let mut by_head: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for st in &spec.states {
            for target in emits(&st.id) {
                if owned_error_row(&target).is_none() {
                    continue; // terminal_states_emit already owns this finding
                }
                let head = target.rsplit('.').next().unwrap_or(&target).to_string();
                by_head.entry(head).or_default().push(target);
            }
        }
        for (head, labels) in &by_head {
            if labels.len() > 1 {
                report.issues.push(Issue::new(
                    "error_labels_unique",
                    file.clone(),
                    format!(
                        "error labels {labels:?} share variant head `{head}` — labels are file-id-namespaced (errors.md error_expr_shape); rename one row so each head is unique within the file",
                    ),
                ));
            }
        }
    }

    // guard_negation_total — every failure transition (its `to` is a
    // declared fail-named state) either cites exactly the union of its
    // success siblings' citation sets (the negated disjunction), or is on
    // the recorded carve-out list. A zero-citation failure guard off the
    // list is a finding, never a silent pass.
    let to_state: BTreeMap<&str, &str> = spec
        .transitions
        .iter()
        .map(|t| (t.id.as_str(), t.to.as_str()))
        .collect();
    let from_state: BTreeMap<&str, &str> = spec
        .transitions
        .iter()
        .map(|t| (t.id.as_str(), t.from.as_str()))
        .collect();
    for t in &spec.transitions {
        if !declared.contains(t.to.as_str()) || !is_fail_state(&t.to) {
            continue;
        }
        let c = citations(&t.id);
        if c.is_empty() {
            let carved_out = file_id == GUARD_CARVEOUT_FILE
                && GUARD_CARVEOUT_TRANSITIONS.contains(&t.id.as_str());
            if !carved_out {
                report.issues.push(Issue::new(
                    "guard_negation_total",
                    file.clone(),
                    format!(
                        "failure transition `{}` cites nothing and is not on the carve-out list ({}'s stage-fails) — cite the union of its success siblings' citation sets, or record a carve-out in linter-failure_shape.md",
                        t.id, GUARD_CARVEOUT_FILE
                    ),
                ));
            }
            continue;
        }
        let src = from_state.get(t.id.as_str()).copied().unwrap_or(&t.from);
        let siblings: Vec<&&str> = to_state
            .iter()
            .filter(|(id, to)| from_state.get(*id).copied() == Some(src) && !is_fail_state(to))
            .map(|(id, _)| id)
            .collect();
        if siblings.is_empty() {
            report.issues.push(Issue::new(
                "guard_negation_total",
                file.clone(),
                format!(
                    "failure transition `{}` has no success sibling from `{src}` — nothing to negate, so its citations cannot be verified; add the success transition it negates",
                    t.id
                ),
            ));
            continue;
        }
        let mut want: Vec<String> = siblings.iter().flat_map(|s| citations(s)).collect();
        want.sort();
        want.dedup();
        if c != want {
            report.issues.push(Issue::new(
                "guard_negation_total",
                file.clone(),
                format!(
                    "failure transition `{}` cites {:?} but its success siblings cite {:?} — the negated disjunction cites exactly the union of its branches' citation sets (errors.md guard_negation_typed)",
                    t.id, c, want
                ),
            ));
        }
    }
}

/// Checker-family slice: `linter-ears_syntax.md`'s rules (statement
/// pattern plus the id-shape rules `no_conjoined_id`/
/// `no_universal_in_id`).
pub(crate) fn lint_ears_family(spec: &Spec, report: &mut Report) {
    let file = file_label(spec);

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

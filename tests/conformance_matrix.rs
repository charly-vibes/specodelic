//! Purpose: conformance matrix over the spk checkers — the independent
//! acceptance gate for the lint/graph surfaces (beads specodelic-vv8).
//!
//! Responsibilities:
//! - Hold the data-driven case table: every case is a minimal synthetic
//!   corpus (fixture files, optional `*.checklist.md` manifests) plus the
//!   EXPECTED findings, derived from the checker specs (`specs/linter-*.md`,
//!   `specs/errors.md`, the Reference Typing table in `specs/specodelic.md`)
//!   — never from current tool behavior.
//! - Run each case through the same library entry points the CLI uses
//!   (`lint::lint_all` for the lint surface, `graph::build` for the graph
//!   surface) and assert the fired findings equal the expectation exactly.
//!
//! Rationale: expectation surfaces are encoded so each checker surface is
//! pinned independently:
//! - `"find:linter.<rule>"` — a failure-severity lint issue must fire;
//! - `"warn:linter.<rule>"` — an advisory warning must ride the warnings
//!   channel AND no failure may carry that rule;
//! - `"graph.dangling"` / `"graph.typing"` / `"graph.supersedes_cycle"` —
//!   the corresponding `GraphReport` vec must be non-empty;
//! - `"clean"` — the corpus must produce zero issues, zero warnings, and
//!   an empty graph report.
//!
//! Fixture format: each case's `files` are `(path, text)` pairs; the path
//! stem participates in `id_matches_file` (stem `-` ⇔ `.` id). Cases that
//! deliberately violate one rule often legitimately trip dependent rules
//! (e.g. a dangling link is both `total_refs` and a graph dangling node) —
//! such cascades are part of the expectation, chosen from the specs, not
//! minimized away. Anti-goal: never weaken a case to match current
//! behavior — a mismatch is either a broken fixture or a checker gap;
//! gaps get a bug ticket and an `#[ignore]` with a documented flip, not
//! a relaxed expectation.

use specodelic::checklist::{self, Checklist};
use specodelic::graph;
use specodelic::lint;
use specodelic::spec::parse_str;
use std::collections::BTreeSet;
use std::path::PathBuf;

struct Case {
    id: &'static str,
    /// `(path, raw text)` corpus files.
    files: Vec<(String, String)>,
    /// `(path, raw text)` checklist manifests (optional).
    checklists: Vec<(String, String)>,
    /// Expected findings, exactly (see the header for the encoding).
    expect: &'static [&'static str],
}

// ---------------------------------------------------------------------------
// Fixture building blocks
// ---------------------------------------------------------------------------

/// The clean single-file baseline most rule cases mutate.
const CLEAN: &str = r#"---
id: clean
kind: intent
statement: "WHEN a user submits a paid order, THE system SHALL record the order."
---

# Clean

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| recorded | invariant | `recorded == submitted` | [[clean]] |
| notify | effect | `notify(recorded)` | [[clean]] |

## Model

### States

- `open`
- `logged`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| record | open | logged | [[clean.recorded]] |

## Properties

| id | kind | derives_from | generator | predicate | observes |
|----|------|--------------|-----------|-----------|----------|
| recorded_once | unit | [[clean.recorded]] | `arbitrary_order()` | `record(o) == o.recorded` | |
| notify_sent | unit | [[clean.notify]] | `arbitrary_order()` | `sent(o) == o.recorded` | [[clean.notify]] |
"#;

/// [`CLEAN`] renamed to file id `b` — a second clean corpus file.
const CLEAN_B: &str = r#"---
id: b
kind: intent
statement: "WHEN a user submits a paid order, THE system SHALL record the order."
---

# B

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| recorded | invariant | `recorded == submitted` | [[b]] |
| notify | effect | `notify(recorded)` | [[b]] |

## Model

### States

- `open`
- `logged`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| record | open | logged | [[b.recorded]] |

## Properties

| id | kind | derives_from | generator | predicate | observes |
|----|------|--------------|-----------|-----------|----------|
| recorded_once | unit | [[b.recorded]] | `arbitrary_order()` | `record(o) == o.recorded` | |
| notify_sent | unit | [[b.notify]] | `arbitrary_order()` | `sent(o) == o.recorded` | [[b.notify]] |
"#;

/// A clean dual-format file: `id: spec`, self-contained refs, mirrored
/// requirement sections (naming law: the file must be `spec.md`).
const DUAL_CLEAN: &str = r#"---
id: spec
kind: intent
statement: "THE openspec workflow SHALL carry every change's requirements as one dual-format markdown file that validates under both parsers."
---

# spec-integration Specification

## Purpose

Minimal dual-format fixture.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| self_contained | invariant | `every wiki-ref resolves within this file` | [[spec]] |

## Model

### States

- `proposed`
- `archived`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | archived | [[spec.self_contained]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| self_contained_holds | unit | [[spec.self_contained]] | `arbitrary_delta()` | `refs(delta) within delta` |

## ADDED Requirements

### Requirement: Dual-format deltas
The workflow SHALL accept a delta file that is simultaneously a valid openspec delta and a lint-clean specodelic spec.

## Requirements

### Requirement: Dual-format deltas
The workflow SHALL accept a delta file that is simultaneously a valid openspec delta and a lint-clean specodelic spec.
"#;

/// A file carrying `## ADDED Requirements` WITHOUT `id: spec` and without
/// the sibling `## Requirements` — the half-format shape
/// `dual_format_valid` exists to reject.
const DUAL_INVALID: &str = r#"---
id: plain
kind: intent
statement: "WHEN a delta is authored, THE system SHALL validate it under both parsers."
---

# Plain

## ADDED Requirements

### Requirement: Half-format delta
The system SHALL reject a delta that is not dual-format.
"#;

/// A dual-format file whose two requirement sections drifted apart.
const DUAL_DRIFT: &str = r#"---
id: spec
kind: intent
statement: "THE openspec workflow SHALL carry every change's requirements as one dual-format markdown file that validates under both parsers."
---

# spec-integration Specification

## Purpose

Minimal dual-format fixture.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| self_contained | invariant | `every wiki-ref resolves within this file` | [[spec]] |

## Model

### States

- `proposed`
- `archived`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| approve | proposed | archived | [[spec.self_contained]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| self_contained_holds | unit | [[spec.self_contained]] | `arbitrary_delta()` | `refs(delta) within delta` |

## ADDED Requirements

### Requirement: Dual-format deltas
The workflow SHALL accept a delta file that is simultaneously a valid openspec delta and a lint-clean specodelic spec.

## Requirements

### Requirement: Dual-format deltas
The workflow SHALL accept every delta file under both parsers without exception.
"#;

/// A same-file two-constraint trace cycle (fixture `two_node_cycle`),
/// shaped like lint.rs's own `mutual_traces_cycle_fails_acyclic` unit
/// fixture plus a Model so only the spec-true findings fire.
const CYCLE: &str = r#"---
id: d.cycle
kind: intent
statement: "WHEN the derivation graph is walked, THE system SHALL order it."
---

# D Cycle

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| a | invariant | `x` | [[d.cycle.b]] |
| b | invariant | `y` | [[d.cycle.a]] |

## Model

### States

- `s1`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t | s1 | s1 | [[d.cycle.a]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[d.cycle.a]] | `g()` | `x == y` |
| q | unit | [[d.cycle.b]] | `g()` | `y == x` |
"#;

/// supersedes-cycle halves (fixture `graph_supersedes_cycle`): constraints
/// superseding each other across two files — same-kind edges, so typing
/// allows them and only the DAG rule may fire.
const SUPERSEDES_A: &str = r#"---
id: sup.a
kind: intent
statement: "WHEN a is superseded, THE system SHALL keep the derivation order."
---

# Sup A

## Constraints

| id | kind | expr | supersedes | traces_to |
|----|------|------|------------|-----------|
| a_old | invariant | `a_old_ok` | [[sup.b.b_old]] | [[sup.a]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| a_old_holds | unit | [[sup.a.a_old]] | `arbitrary_a()` | `a(x) == x` |
"#;

const SUPERSEDES_B: &str = r#"---
id: sup.b
kind: intent
statement: "WHEN b is superseded, THE system SHALL keep the derivation order."
---

# Sup B

## Constraints

| id | kind | expr | supersedes | traces_to |
|----|------|------|------------|-----------|
| b_old | invariant | `b_old_ok` | [[sup.a.a_old]] | [[sup.b]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| b_old_holds | unit | [[sup.b.b_old]] | `arbitrary_b()` | `b(x) == x` |
"#;

/// Failure-shape fixture: `open → done` success and `open → failed`
/// failure transitions, with the failure guard CITING the invariant (the
/// negation of the success sibling's citation set), so
/// `guard_negation_total` stays quiet unless a case targets it. `failed`
/// emits NOTHING — `terminal_states_emit` fires for the mute terminal.
const FAILURE_MUTE: &str = r#"---
id: clean
kind: intent
statement: "WHEN a user submits a paid order, THE system SHALL record the order."
---

# Clean

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| recorded | invariant | `recorded == submitted` | [[clean]] |

## Model

### States

- `open`
- `done`
- `failed`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| record | open | done | [[clean.recorded]] |
| reject | open | failed | [[clean.recorded]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| recorded_once | unit | [[clean.recorded]] | `arbitrary_order()` | `record(o) == o.recorded` |
"#;

/// Failure-shape fixture with two failure terminals emitting constraints
/// whose ids share the variant head `reject` (`a.reject`, `b.reject`) —
/// `error_labels_unique` fires.
const FAILURE_DUP_HEAD: &str = r#"---
id: clean
kind: intent
statement: "WHEN a user submits a paid order, THE system SHALL record the order."
---

# Clean

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| recorded | invariant | `recorded == submitted` | [[clean]] |
| a.reject | effect | `err_a()` | [[clean]] |
| b.reject | effect | `err_b()` | [[clean]] |

## Model

### States

- `open`
- `done`
- `failed_a` (emits: [[clean.a.reject]])
- `failed_b` (emits: [[clean.b.reject]])

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| record | open | done | [[clean.recorded]] |
| reject_a | open | failed_a | [[clean.recorded]] |
| reject_b | open | failed_b | [[clean.recorded]] |

## Properties

| id | kind | derives_from | generator | predicate | observes |
|----|------|--------------|-----------|-----------|----------|
| recorded_once | unit | [[clean.recorded]] | `arbitrary_order()` | `record(o) == o.recorded` | |
| a_reject_cover | unit | [[clean.a.reject]] | `arbitrary_o()` | `a(o) == o` | [[clean.a.reject]] |
| b_reject_cover | unit | [[clean.b.reject]] | `arbitrary_o()` | `b(o) == o` | [[clean.b.reject]] |
"#;

/// Failure-shape fixture whose failure transition carries NO guard:
/// `guard_required` and `guard_negation_total` fire (zero citations off
/// the carve-out list), and the mute terminal adds
/// `terminal_states_emit`.
const FAILURE_ZERO_GUARD: &str = r#"---
id: clean
kind: intent
statement: "WHEN a user submits a paid order, THE system SHALL record the order."
---

# Clean

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| recorded | invariant | `recorded == submitted` | [[clean]] |

## Model

### States

- `open`
- `done`
- `failed`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| record | open | done | [[clean.recorded]] |
| reject | open | failed | |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| recorded_once | unit | [[clean.recorded]] | `arbitrary_order()` | `record(o) == o.recorded` |
"#;

/// A well-formed, fully-accounted checklist over [`CLEAN`].
const CHECKLIST_OK: &str = r#"# clean.checklist

## Items

- **record_dedup**: recording twice is idempotent

## Mapping

| item | status | mapped_ids | rationale |
|------|--------|------------|-----------|
| record_dedup | covered | [[clean.recorded]], [[clean.recorded_once]] | |
"#;

/// Malformed manifest: no `## Items` section.
const CHECKLIST_NO_ITEMS: &str = r#"# clean.checklist

## Mapping

| item | status | mapped_ids | rationale |
|------|--------|------------|-----------|
| ghost | covered | [[clean.recorded]] | |
"#;

/// Manifest with one declared item and no mapping row for it.
const CHECKLIST_UNACCOUNTED: &str = r#"# clean.checklist

## Items

- **record_dedup**: recording twice is idempotent

## Mapping
"#;

/// Manifest whose `covered` row names a nonexistent row.
const CHECKLIST_BAD_TARGET: &str = r#"# clean.checklist

## Items

- **record_dedup**: recording twice is idempotent

## Mapping

| item | status | mapped_ids | rationale |
|------|--------|------------|-----------|
| record_dedup | covered | [[clean.no_such_row]] | |
"#;

/// Manifest with a rationale-less waiver.
const CHECKLIST_WAIVER_NO_RATIONALE: &str = r#"# clean.checklist

## Items

- **record_dedup**: recording twice is idempotent

## Mapping

| item | status | mapped_ids | rationale |
|------|--------|------------|-----------|
| record_dedup | waived | | |
"#;

/// Manifest where two rows claim the same item.
const CHECKLIST_DUP_CLAIM: &str = r#"# clean.checklist

## Items

- **record_dedup**: recording twice is idempotent

## Mapping

| item | status | mapped_ids | rationale |
|------|--------|------------|-----------|
| record_dedup | covered | [[clean.recorded]] | |
| record_dedup | covered | [[clean.recorded_once]] | second claim on purpose |
"#;

// ---------------------------------------------------------------------------
// Case table
// ---------------------------------------------------------------------------

/// Swap file id `clean` → `id` in a copy of [`CLEAN`].
fn clean_as(id: &str) -> String {
    CLEAN
        .replace("clean.", &format!("{id}."))
        .replace("id: clean", &format!("id: {id}"))
        .replace("[[clean]]", &format!("[[{id}]]"))
}

/// The matrix: one case per checker-spec expectation. Each lint rule in
/// `lint::RULE_TABLE` plus the three graph-only surfaces gets at least
/// one violation case; the clean baselines pin the vacuous pass.
fn cases() -> Vec<Case> {
    let f = |id: &'static str, files: Vec<(&str, String)>, expect: &'static [&'static str]| Case {
        id,
        files: files.into_iter().map(|(p, t)| (p.to_string(), t)).collect(),
        checklists: vec![],
        expect,
    };
    let with_checklist = |id: &'static str, cl: &str, expect: &'static [&'static str]| Case {
        id,
        files: vec![("clean.md".to_string(), CLEAN.to_string())],
        checklists: vec![("clean.checklist.md".to_string(), cl.to_string())],
        expect,
    };

    vec![
        // -- clean baselines ------------------------------------------------
        f("clean_single_file", vec![("clean.md", CLEAN.to_string())], &["clean"]),
        f("clean_dual_format", vec![("spec.md", DUAL_CLEAN.to_string())], &["clean"]),
        with_checklist("clean_checklist_consistent", CHECKLIST_OK, &["clean"]),
        // -- linter-frontmatter family ---------------------------------------
        f(
            "fm_kind_not_intent",
            vec![("clean.md", CLEAN.replace("kind: intent", "kind: capability"))],
            &["find:linter.frontmatter_valid"],
        ),
        f(
            "id_mismatch_file",
            vec![("wrong_stem.md", CLEAN.to_string())],
            &["find:linter.id_matches_file"],
        ),
        // -- id-shape rules (capability ids) -----------------------------------
        f(
            "id_conjoined",
            vec![(
                "order.cancel_and_refund.md",
                clean_as("order.cancel_and_refund"),
            )],
            &["find:linter.no_conjoined_id"],
        ),
        f(
            "id_universal",
            vec![("order.always_validate.md", clean_as("order.always_validate"))],
            &["find:linter.no_universal_in_id"],
        ),
        // -- linter-schema_shape (unique ids) ------------------------------------
        f(
            "duplicate_row_id",
            vec![(
                "clean.md",
                CLEAN.replace(ORPHAN_FREE_PROPERTY, &ORPHAN_FREE_PROPERTY.repeat(2)),
            )],
            &["find:linter.unique_id"],
        ),
        // -- linter-model_shape family ---------------------------------------------
        f(
            "transition_no_guard",
            vec![(
                "clean.md",
                CLEAN.replace(
                    "| record | open | logged | [[clean.recorded]] |",
                    "| record | open | logged | |",
                ),
            )],
            // With the guard null, the model component (states +
            // transition) has no edge into the intent's component —
            // single_root_reachable cascades per its spec text.
            &[
                "find:linter.guard_required",
                "find:linter.single_root_reachable",
            ],
        ),
        f(
            "model_missing",
            vec![("clean.md", CLEAN.replace(MODEL_SECTION, ""))],
            &["find:linter.model_present"],
        ),
        f(
            "state_unused",
            vec![(
                "clean.md",
                CLEAN.replace("- `logged`\n", "- `logged`\n- `orphaned`\n"),
            )],
            // `orphaned` has no transitions, so its island is also
            // unreachable from any intent (single_root_reachable).
            &[
                "find:linter.every_state_used",
                "find:linter.single_root_reachable",
            ],
        ),
        f(
            "transition_undeclared_state",
            vec![(
                "clean.md",
                CLEAN.replace(
                    "| record | open | logged | [[clean.recorded]] |",
                    "| record | open | void | [[clean.recorded]] |",
                ),
            )],
            &[
                "find:linter.every_transition_valid",
                "find:linter.every_state_used",
                "find:linter.single_root_reachable",
                "graph.dangling",
            ],
        ),
        // -- linter-ears_syntax ---------------------------------------------------------
        f(
            "ears_no_shall",
            vec![(
                "clean.md",
                CLEAN.replace(
                    "statement: \"WHEN a user submits a paid order, THE system SHALL record the order.\"",
                    "statement: \"The system records paid orders when submitted.\"",
                ),
            )],
            &["find:linter.ears_syntax"],
        ),
        // -- linter-referential_integrity family --------------------------------------------
        // graph.md `total_extraction`: the edge set is the CLOSED union of
        // typed reference columns — a dangling link inside an UNTYPED
        // column (here `predicate`) is `total_refs`' beat in the linter,
        // never a graph edge, so it must NOT dangle in the graph report
        // (beads specodelic-mlg; flipped from conformance_matrix_known_gaps).
        f(
            "total_refs_dangling",
            vec![(
                "clean.md",
                CLEAN.replace("`sent(o) == o.recorded`", "`[[missing.file]] says so`"),
            )],
            &["find:linter.total_refs"],
        ),
        f(
            "self_ref",
            vec![(
                "clean.md",
                CLEAN.replace(
                    "| recorded | invariant | `recorded == submitted` | [[clean]] |",
                    "| recorded | invariant | `recorded == submitted` | [[clean.recorded]] |",
                ),
            )],
            &[
                "find:linter.no_self_ref",
                "find:linter.single_root_reachable",
                "graph.typing",
            ],
        ),
        f(
            "two_node_cycle",
            // Mutual constraint↔constraint TRACES_TO cycle — the
            // traces_to edge set is acyclic's; typing fires alongside
            // (traces_to must resolve to an Intent). The cycle component
            // never reaches the intent row — an orphaned island is part
            // of this fixture's spec-true expectation.
            vec![("d.cycle.md", CYCLE.to_string())],
            &[
                "find:linter.acyclic",
                "find:linter.single_root_reachable",
                "graph.typing",
            ],
        ),
        f(
            "constraint_derives_cycle",
            // Mutual constraint↔constraint DERIVES_FROM pseudo-cycle —
            // out-of-format since specodelic-huf: the Appears-on column
            // is normative (derives_from appears on Property rows only),
            // so the graph layer reports each malformed edge as a labeled
            // typing violation and records no edge — no acyclic edge
            // exists to cycle. The constraints remain an island
            // unreachable from the intent (single_root_reachable).
            vec![("d.cycle.md", CYCLE_DERIVES.to_string())],
            &["find:linter.single_root_reachable", "graph.typing"],
        ),
        f(
            "single_root_island",
            vec![(
                "clean.md",
                CLEAN.replace(
                    "| notify | effect | `notify(recorded)` | [[clean]] |",
                    "| floating | effect | `float(x)` | |",
                )
                .replace(
                    "| notify_sent | unit | [[clean.notify]] | `arbitrary_order()` | `sent(o) == o.recorded` | [[clean.notify]] |",
                    "| floater | unit | [[clean.floating]] | `arbitrary_x()` | `f(x) == x` | [[clean.floating]] |",
                ),
            )],
            &["find:linter.single_root_reachable"],
        ),
        // -- linter-coverage --------------------------------------------------------------------
        f(
            "coverage_missing",
            vec![("clean.md", CLEAN.replace(ORPHAN_FREE_PROPERTY, ""))],
            &["find:linter.coverage"],
        ),
        // no_orphan_property resolves the derives_from target, not just
        // its presence (specodelic-rk3): a unit property deriving from a
        // non-law PROPERTY row is not coverage — the graph layer's
        // typing check fires alongside (derives_from must resolve to a
        // Constraint unless the source is a law).
        f(
            "orphan_property",
            vec![(
                "clean.md",
                CLEAN.replace(
                    ORPHAN_FREE_PROPERTY,
                    &format!(
                        "{ORPHAN_FREE_PROPERTY}| extra_rule | unit | [[clean.recorded_once]] | `arbitrary_x()` | `x(o) == o` | |\n"
                    ),
                ),
            )],
            &["find:linter.no_orphan_property", "graph.typing"],
        ),
        // -- dual-format file rules -----------------------------------------------------------------
        f(
            "dual_format_invalid",
            vec![("plain.md", DUAL_INVALID.to_string())],
            &["find:linter.dual_format_valid", "find:linter.model_present"],
        ),
        f(
            "requirement_drift",
            vec![("spec.md", DUAL_DRIFT.to_string())],
            &["find:linter.requirement_drift"],
        ),
        // -- linter-failure_shape family ----------------------------------------------------------------
        f(
            "failure_terminal_mute",
            vec![("clean.md", FAILURE_MUTE.to_string())],
            &["find:linter.terminal_states_emit"],
        ),
        f(
            "error_labels_duplicate_head",
            vec![("clean.md", FAILURE_DUP_HEAD.to_string())],
            &["find:linter.error_labels_unique"],
        ),
        f(
            "failure_guard_zero_citations",
            vec![("clean.md", FAILURE_ZERO_GUARD.to_string())],
            &[
                "find:linter.guard_required",
                "find:linter.guard_negation_total",
                "find:linter.terminal_states_emit",
            ],
        ),
        // -- linter-observability (advisory) -----------------------------------------------------------------
        f(
            "effect_unobserved_is_advisory",
            vec![(
                "clean.md",
                CLEAN.replace(" | [[clean.notify]] |\n", " |  |\n"),
            )],
            &["warn:linter.observability"],
        ),
        // -- linter-external_completeness (checklists) ----------------------------------------------------------
        with_checklist(
            "checklist_malformed_missing_items",
            CHECKLIST_NO_ITEMS,
            &["find:linter.checklist_well_formed"],
        ),
        with_checklist(
            "checklist_item_unaccounted",
            CHECKLIST_UNACCOUNTED,
            &["find:linter.every_item_accounted"],
        ),
        with_checklist(
            "covered_maps_unresolved",
            CHECKLIST_BAD_TARGET,
            &["find:linter.covered_maps_resolve"],
        ),
        with_checklist(
            "waiver_without_rationale",
            CHECKLIST_WAIVER_NO_RATIONALE,
            &["find:linter.waiver_has_rationale"],
        ),
        with_checklist(
            "duplicate_claim",
            CHECKLIST_DUP_CLAIM,
            &["find:linter.no_duplicate_claim"],
        ),
        // -- graph-only surfaces (Reference Typing, supersedes DAG) ------------------------------------------------
        f(
            "graph_typing_traces_to_constraint",
            vec![
                // `clean.recorded` traces to `b.recorded` (a CONSTRAINT, not
                // an Intent) — the Reference Typing table forbids it.
                (
                    "clean.md",
                    CLEAN.replace(
                        "| recorded | invariant | `recorded == submitted` | [[clean]] |",
                        "| recorded | invariant | `recorded == submitted` | [[b.recorded]] |",
                    ),
                ),
                ("b.md", CLEAN_B.to_string()),
            ],
            &["graph.typing"],
        ),
        f(
            "graph_supersedes_cycle",
            vec![
                ("sup.a.md", SUPERSEDES_A.to_string()),
                ("sup.b.md", SUPERSEDES_B.to_string()),
            ],
            &["find:linter.model_present", "graph.supersedes_cycle"],
        ),
    ]
}

/// Out-of-format-since-specodelic-huf constraint↔constraint derives_from
/// pseudo-cycle (case `constraint_derives_cycle`): the Appears-on column
/// is normative, so each malformed edge is a typing violation and no
/// acyclic edge exists.
const CYCLE_DERIVES: &str = r#"---
id: d.cycle
kind: intent
statement: "WHEN the derivation graph is walked, THE system SHALL order it."
---

# D Cycle

## Constraints

| id | kind | expr | derives_from |
|----|------|------|--------------|
| a | invariant | `x` | [[d.cycle.b]] |
| b | invariant | `y` | [[d.cycle.a]] |

## Model

### States

- `s1`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t | s1 | s1 | [[d.cycle.a]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|------------|
| p | unit | [[d.cycle.a]] | `g()` | `x == y` |
| q | unit | [[d.cycle.b]] | `g()` | `y == x` |
"#;

/// The `recorded_once` property line of [`CLEAN`] — the deriving property
/// that covers the `recorded` invariant.
const ORPHAN_FREE_PROPERTY: &str = "| recorded_once | unit | [[clean.recorded]] | `arbitrary_order()` | `record(o) == o.recorded` | |\n";

/// The Model section of [`CLEAN`] (removed whole for `model_missing`).
const MODEL_SECTION: &str = "## Model\n\n### States\n\n- `open`\n- `logged`\n\n### Transitions\n\n| id | from | to | guard |\n|----|------|----|-------|\n| record | open | logged | [[clean.recorded]] |\n\n";

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

fn run_case(c: &Case) -> Result<(), String> {
    let mut specs = Vec::new();
    for (path, text) in &c.files {
        let mut s =
            parse_str(text).map_err(|e| format!("case {}: parsing {path} failed: {e}", c.id))?;
        s.path = Some(PathBuf::from(path));
        specs.push(s);
    }
    let checklists: Vec<Checklist> = c
        .checklists
        .iter()
        .map(|(p, t)| checklist::parse_str(PathBuf::from(p), t))
        .collect();

    let report = lint::lint_all(&specs, &checklists);
    let graph = graph::build(&specs);

    let mut got: BTreeSet<String> = BTreeSet::new();
    for i in &report.issues {
        got.insert(format!("find:{}", i.rule_id));
    }
    for w in &report.warnings {
        got.insert(format!("warn:{}", w.rule_id));
    }
    if !graph.dangling.is_empty() {
        got.insert("graph.dangling".into());
    }
    if !graph.violations.is_empty() {
        got.insert("graph.typing".into());
    }
    if !graph.supersedes_cycles.is_empty() {
        got.insert("graph.supersedes_cycle".into());
    }
    if got.is_empty() {
        got.insert("clean".into());
    }

    let want: BTreeSet<String> = c.expect.iter().map(|s| s.to_string()).collect();
    if want == got {
        return Ok(());
    }

    let mut msg = format!("case {}: expectation mismatch\n", c.id);
    for m in want.difference(&got) {
        msg.push_str(&format!("  MISSING    {m}\n"));
    }
    for u in got.difference(&want) {
        msg.push_str(&format!("  UNEXPECTED {u}\n"));
    }
    msg.push_str("  detail:");
    for i in &report.issues {
        msg.push_str(&format!("\n    [{}] {}", i.rule_id, i.message));
    }
    for w in &report.warnings {
        msg.push_str(&format!("\n    (warn) [{}] {}", w.rule_id, w.message));
    }
    if !graph.dangling.is_empty() {
        msg.push_str(&format!("\n    graph.dangling: {:?}", graph.dangling));
    }
    if !graph.violations.is_empty() {
        msg.push_str(&format!("\n    graph.typing: {:?}", graph.violations));
    }
    if !graph.supersedes_cycles.is_empty() {
        msg.push_str(&format!(
            "\n    graph.supersedes_cycle: {:?}",
            graph.supersedes_cycles
        ));
    }
    Err(msg)
}

#[test]
fn conformance_matrix() {
    let matrix = cases();
    let mut failures = Vec::new();
    for case in &matrix {
        if let Err(msg) = run_case(case) {
            failures.push(msg);
        }
    }
    if !failures.is_empty() {
        panic!(
            "conformance matrix: {} of {} cases failed\n\n{}",
            failures.len(),
            matrix.len(),
            failures.join("\n\n")
        );
    }
}

// (No known gaps remain — both vv8 gap cases have been flipped into
// `cases()` as their tracking tickets closed: `orphan_property` via
// specodelic-rk3, `constraint_derives_cycle` via specodelic-huf. The
// known-gaps harness pattern stays documented here for the next gap:
// a spec-true expectation that current behavior fails gets an
// `#[ignore]`d case here with a documented flip, never a weakened
// expectation in `cases()`.)

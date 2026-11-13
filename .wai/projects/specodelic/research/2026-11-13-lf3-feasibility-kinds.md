---
tags: [pipeline-step:research, pipeline-run:epic-orchestrator-2026-11-13-specodelic-lf3-scoped-verify-migration]
---

# LF3 feasibility — `kinds` property rows

**Ticket:** specodelic-lf3 (scoped-first) · **run:** 2026-11-13 ·
**Method:** row-by-row classification of every Property row in
`openspec/specs/kinds/spec.md` against the runner as it exists at HEAD.

## Verdict

| metric | value |
|--------|-------|
| property rows | 17 |
| expressible now | **0** |
| needs runner harness | **17** |

**No row of this file is honestly expressible today.** Every row is a
claim about the tool's own behavior (the linter, the compiler, the pack
scanner, the verify gates) or about corpus content — never a closed-form
predicate over generated values. See
[the summary](./2026-11-13-lf3-feasibility-summary.md) for the runner
measurements behind this verdict and the corpus-wide fallback sweep.

## Blockers

| code | meaning |
|------|---------|
| **T** | tool dependency — the row asserts the tool's own checker behavior; an honest executable form must CALL the tool |
| **F** | corpus/fixture access — the row asserts something about corpus content; the fragment would have to read the corpus (`std::fs` is banned by `fragment_hygiene`) or use a fixture API |
| **G** | generator vocabulary — the row needs varied generated values; today a named generator compiles to `Just(name)` — one constant input per property |
| **L** | law-case fragments — a law-kind row cannot carry a `**rust:**` fragment at all (`fragment_law_rejected`), so each named case needs its own body (a format Revision) |

## Row-by-row

| # | row id | kind | generator (fixture named) | predicate (gist) | verdict | blockers | why |
|---|--------|------|---------------------------|------------------|---------|----------|-----|
| 1 | `unknown_kind_rejected` | unit | `row_belonging_to_a_sixth_section("Actions")` | `check(row) == failed` | needs-harness | **T** | the kind-shape checker must fail a sixth section row |
| 2 | `intent_missing_statement_rejected` | unit | `intent_row_without("statement")` | `check(row) == failed` | needs-harness | **T** | the kind-shape checker must fail an intent row without `statement` |
| 3 | `constraint_row_extra_field_rejected` | unit | `constraint_row_with(extra_field: "guard")` | `check(row) == failed` | needs-harness | **T** | the kind-shape checker must fail an extra field |
| 4 | `state_row_with_extra_field_rejected` | unit | `state_row_with(extra_field: "kind")` | `check(row) == failed` | needs-harness | **T** | the kind-shape checker must fail an extra field |
| 5 | `transition_missing_guard_rejected` | unit | `transition_row_without("guard")` | `check(row) == failed` | needs-harness | **T** | the kind-shape checker must fail a guardless transition |
| 6 | `property_row_bad_subkind_rejected` | unit | `property_row_with(kind: "audit")` | `check(row) == failed` | needs-harness | **T** | the kind-shape checker must fail a bad property subkind |
| 7 | `subkind_addition_without_revision_rejected` | unit | `diff_adding_subkind_value_with_no_new_Revision_heading( | `check(diff) == failed` | needs-harness | **T** **F** | the diff/revision checker must fail a subkind added without a Revision heading |
| 8 | `constraint_row_advisory_accepted` | unit | `constraint_row_with(kind: "advisory")` | `check(row) == passed` | needs-harness | **T** | the kind-shape checker must PASS an advisory constraint row |
| 9 | `constraint_row_effect_accepted` | unit | `constraint_row_with(kind: "effect")` | `check(row) == passed` | needs-harness | **T** | the kind-shape checker must PASS an effect constraint row |
| 10 | `state_row_emits_accepted` | unit | `state_row_with(emits: an_effect_kind_constraint_id)` | `check(row) == passed` | needs-harness | **T** | the kind-shape checker must PASS an `emits` to an effect |
| 11 | `state_row_without_emits_still_passes` | unit | `state_row_with(fields: {id})` — no `emits` | `check(row) == passed` — `emits` is optional, not a second requi | needs-harness | **T** | the kind-shape checker must PASS a state with no `emits` |
| 12 | `state_row_emits_wrong_kind_rejected` | unit | `state_row_with(emits: an_invariant_kind_constraint_id)` | `check(row) == failed` | needs-harness | **T** | the kind-shape checker must FAIL an `emits` to an invariant |
| 13 | `constraint_row_extension_point_accepted` | unit | `constraint_row_with(kind: "extension_point")` | `check(row) == passed` | needs-harness | **T** | the kind-shape checker must PASS an extension_point constraint row |
| 14 | `well_formed_row_passes` | unit | `arbitrary_well_formed_row_of_one_kind()` | `check(row) == passed` | needs-harness | **T** | the kind-shape checker must PASS a well-formed row |
| 15 | `kind_shape_naturality` | law | `arbitrary_row_of_one_kind(), arbitrary_id_rename()` | **identity:** `assigned_kind(rename(row,a,a)) == assigned_kind(r | needs-harness | **L** **T** | law row: two named cases over row renaming — law rows carry NO fragment today, and the transform is the tool's |
| 16 | `kind_assignment_failure_label_asserted` | unit | `kind_assignment_failure_raised()` | `error_label == "kinds.kind_assignment_failure"` — renaming the  | needs-harness | **F** | the label `"kinds.kind_assignment_failure"` must hold in the error Constraint, this property and the note — a three-site corpus consistency claim |
| 17 | `shape_check_failure_label_asserted` | unit | `shape_check_failure_raised()` | `error_label == "kinds.shape_check_failure"` — renaming the labe | needs-harness | **F** | the label `"kinds.shape_check_failure"` must hold in three corpus sites — a corpus consistency claim |

## What would unblock this file

1. **Tool-dependency seam** — the scratch crate must be able to depend on
   the `specodelic` lib (`src/verify.rs` `SCRATCH_MANIFEST` is
   `proptest`-only today), so a fragment can call `specodelic::lint::lint_corpus`
   / `specodelic::compile::*` on a fixture instead of re-implementing them.
2. **Fixture API** — the generators in this file name *fixtures*
   (`spec_file_with(transition.to_not_in_states: true)`,
   `row_belonging_to_a_sixth_section("Actions")`, …). Nothing in the tree
   turns a generator name into a concrete spec/row today.
3. **Generator vocabulary** — a named generator yields `Just(name)`
   (`src/compile.rs:1323`), i.e. exactly one input per property. Owned by
   specodelic-jwjj.
4. **Law-case fragments** — this file carries a law-kind row; law rows cannot carry a `**rust:**` fragment at all today (`fragment_law_rejected`, verified empirically this run), so the file stays unmigratable even after seams 1–3 land. A per-case fragment body is a format Revision.

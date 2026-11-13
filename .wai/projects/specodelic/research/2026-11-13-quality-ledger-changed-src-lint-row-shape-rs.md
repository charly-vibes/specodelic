---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-3632-lint-revision-seams-for-the-four-blocked-rows, pipeline-step:quality-ledger]
---

QUALITY LEDGER:
Changed:
- src/lint/row_shape.rs (new): the linter.row_shape_closed rule — a row's field set must be exactly its kind's base fields plus the typed reference columns the Schema-as-data declares.
- src/lint/revision.rs (new): the revision-pair seam — specodelic::lint::revision::check_pair(prior_text, current_text) diffs the governed id-sets (State ids, Constraint/Property kind values, Reference Typing fields) and reports Removed / GrownSilently.
- src/lint/mod.rs, src/lint/rules.rs: the rule enters the schema-shape family; the RULE_TABLE carries its rule_semantics.
- src/spec.rs: Spec::revision_headings records the '## Revision N' headings; a State bullet's named tail field is surfaced as an extra cell instead of being swallowed as an emits value.
- src/packs.rs: namespace_of becomes public so the row-shape rule can recognize pack-qualified columns.
- src/fixtures/specs.rs: the corpus fixture module (moved verbatim from src/lib.rs) plus the six new fixtures (constraint_row_with_extra_field, state_row_with_extra_field, revision_before_governed_id_set_change, revision_after_id_set_member_removed, diff_adding_subkind_value_with_no_new_Revision_heading and the declared-growth twin).
- openspec/specs/kinds/spec.md, openspec/specs/specodelic/spec.md: the four blocked property rows now carry executable **rust:** predicates; the kinds Notes block and the specodelic Checker Ownership table name the deciding checker.
- justfile: verify-specs now covers [compile, packs, specodelic, kinds] and fails unless each file's properties state is exactly 'pass'.
- tests/specodelic_fixtures.rs, tests/conformance_matrix.rs: the red tests for the four rows, the no-placeholder test, and the conformance fixtures corrected to the corpus's Reference Typing.
- specodelic/*: the regenerated compiled artifacts and check reports.
Verified:
- cargo fmt --check clean; cargo clippy -- -D warnings clean; cargo test: 50 test binaries green (cargo test --lib 428 passed; specodelic_fixtures 28; conformance_matrix 1).
- just ci: exit 0 (fmt-check, clippy, tests, release build, openspec validate, lint-deltas, model-check, verify-specs, sync-sections, doc gates, pretender, sibling guards, ah check).
- just lint-specs: ok true, 48 files linted, no issues.
- ah check --run-tests --all-tests: 641 passed, 0 findings.
- Non-vacuity: an inverted row-shape predicate failed the two row-shape property blocks; a malformed **rust:** fragment made the compile fail and just verify-specs exit 1 ('properties not passed'); both mutations were reverted and the gate returned to green.
Review:
- RO5U (rule-of-5) over the whole diff: no Critical, no High; one Medium (row_shape was public) and two Lows (a duplicated revision-heading predicate, a dangling doc line) — all three fixed and re-verified.
Risks:
- The conformance matrix's fixtures changed shape (observes moved to the Constraints table, a 'notified' constraint row added): the corpus's Reference Typing says observes is a Constraint field, so the old fixtures were non-conformant and the new closed row shape caught them.
- The four rows' generator cells now name concrete fixtures (constraint_row_with_extra_field, state_row_with_extra_field) instead of the parameterized families; the scenarios mirror the new names.
Next:
- Ship: commit the implementation, then hand the ticket back to the orchestrator for its gate verification.

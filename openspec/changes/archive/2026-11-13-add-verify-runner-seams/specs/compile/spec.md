## MODIFIED Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: precondition_violation_rejected

- **WHEN** `file_that_has_not_passed_lint()`
- **THEN** `check(file) == failed`
- **VERIFIES** [[compile.precondition_violation_rejected]]

#### Scenario: toml_roundtrips

- **WHEN** `arbitrary_constraint_row()`
- **THEN** `parse_toml(compile(row)) == row`
- **VERIFIES** [[compile.toml_roundtrips]]

#### Scenario: tla_disjunct_count_matches

- **WHEN** `arbitrary_model_section(n_transitions)`
- **THEN** `count(disjuncts(compile(model).Next)) == n_transitions + 1` — the closing stuttering disjunct
- **VERIFIES** [[compile.tla_disjunct_count_matches]]

#### Scenario: output_function_covers_emitting_states_only

- **WHEN** `arbitrary_model_section(n_states_with_emits, n_states_without)`
- **THEN** `domain(compile(model).Output) == the_states_with_emits` — no entry for the rest
- **VERIFIES** [[compile.output_function_covers_emitting_states_only]]

#### Scenario: proptest_block_per_property

- **WHEN** `property_row_with_one_property()` — a materialized fixture, not
  the generator's own name
- **THEN** `count(compile(row).blocks) >= 1` — the compiled `**rust:**`
  fragment counts the blocks the real compiler emitted for the fixture
- **VERIFIES** [[compile.proptest_block_per_property]]

#### Scenario: law_property_compiles_required_cases

- **WHEN** `law_property_row_with(cases: ["identity", "associativity"])`
- **THEN** `count(compile(row).blocks) == 2`
- **VERIFIES** [[compile.law_property_compiles_required_cases]]

#### Scenario: compile_completes_or_fails_cleanly

- **WHEN** `arbitrary_linted_and_covered_file()`
- **THEN** `compile(file) ∈ {all_three_artifacts, single_labeled_failure}` — never partial
- **VERIFIES** [[compile.compile_completes_or_fails_cleanly]]

#### Scenario: id_preservation_holds

- **WHEN** `arbitrary_linted_and_covered_file()`
- **THEN** `ids_in(file) ⊆ ids_in(compile(file))`
- **VERIFIES** [[compile.id_preservation_holds]]

#### Scenario: roundtrip_stable

- **WHEN** `arbitrary_linted_and_covered_file()`
- **THEN** `compile(file) == compile(parse(compile(file)))`
- **VERIFIES** [[compile.roundtrip_stable]]

#### Scenario: extraction_failure_label_asserted

- **WHEN** `extraction_stage_fails_on_row()`
- **THEN** `error_label == "compile.extraction_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002)
- **VERIFIES** [[compile.extraction_failure_label_asserted]]

#### Scenario: emission_failure_label_asserted

- **WHEN** `emission_stage_fails()`
- **THEN** `error_label == "compile.emission_failure"` — same three-site rename rule (EDGE-002)
- **VERIFIES** [[compile.emission_failure_label_asserted]]

#### Scenario: fragment_body_emitted_verbatim

- **WHEN** `property_row_with_rust_fragment()`
- **THEN** `the emitted block body contains the fragment verbatim`
- **VERIFIES** [[compile.fragment_body_emitted_verbatim]]

#### Scenario: prose_predicate_stays_placeholder

- **WHEN** `property_row_without_fragment()`
- **THEN** `compile(row).body contains todo_predicate!` — no widening of pre-Revision behavior
- **VERIFIES** [[compile.prose_predicate_stays_placeholder]]

#### Scenario: fragment_values_bind_generators

- **WHEN** `property_row_with_two_generators()`
- **THEN** `the emitted block's args are v0, v1 — one per named generator, each a String`
- **VERIFIES** [[compile.fragment_values_bind_generators]]

#### Scenario: rust_fragment_semantics_unchanged

- **WHEN** `revision_15_rust_fragment_cell()`
- **THEN** `extract(cell) == revision_15_extract(cell)` — byte-identical, pure widening
- **VERIFIES** [[compile.rust_fragment_semantics_unchanged]]

#### Scenario: unknown_tag_labeled

- **WHEN** `property_row_with_marker_tag("**go:")`
- **THEN** `error_stage == fragment_extraction ∧ message names the tag and the closed set`
- **VERIFIES** [[compile.unknown_tag_labeled]]

#### Scenario: non_fragment_tag_is_mention

- **WHEN** `property_row_with_mid_span_marker_tag("**py:")`
- **THEN** `compile(row) == compile(row_without_marker)` — mid-span occurrence never extracts
- **VERIFIES** [[compile.non_fragment_tag_is_mention]]

#### Scenario: one_tag_extracts_per_cell

- **WHEN** `property_row_with_marker_tag("**py:")`
- **THEN** `extract(cell) == Some(fragment_under("py"))` — a fragment-position cell extracts under exactly one tag from the closed set {rust, py, ts}; acceptance is grammar-level and emitter-independent
- **VERIFIES** [[compile.one_tag_extracts_per_cell]]

#### Scenario: missing_emitter_labeled

- **WHEN** `property_row_with_marker_tag("**py:")`
- **THEN** `error_label == "compile.emission_failure" ∧ the remediation hint names the per-language emitter follow-up` — never a silent fall-through to Rust emission
- **VERIFIES** [[compile.missing_emitter_labeled]]

#### Scenario: law_fragment_labeled

- **WHEN** `law_row_with_fragment()`
- **THEN** `error_stage == fragment_extraction`
- **VERIFIES** [[compile.law_fragment_labeled]]

#### Scenario: hygiene_violation_labeled

- **WHEN** `fragment_with(banned_token: "std::fs")`
- **THEN** `error_stage == fragment_extraction ∧ message names the token`
- **VERIFIES** [[compile.hygiene_violation_labeled]]

#### Scenario: guard_fragment_labeled

- **WHEN** `transition_row_with_fragment_guard()`
- **THEN** `error_stage == fragment_extraction`
- **VERIFIES** [[compile.guard_fragment_labeled]]

#### Scenario: non_invariant_fragment_labeled

- **WHEN** `effect_row_with_fragment()`
- **THEN** `error_stage == fragment_extraction`
- **VERIFIES** [[compile.non_invariant_fragment_labeled]]

#### Scenario: kernel_expr_extracts

- **WHEN** `invariant_cell_with_kernel_marker()`
- **THEN** `the expression extracts under the closed grammar and evaluates to exactly one of verified, counterexample, unknown`
- **VERIFIES** [[compile.kernel_expr_extracts]]

#### Scenario: kernel_nonmember_labeled

- **WHEN** `invariant_cell_with_nonmember_atomic()`
- **THEN** `error_stage == kernel_grammar ∧ the message names the offending atomic and the closed set`
- **VERIFIES** [[compile.kernel_nonmember_labeled]]

#### Scenario: binding_text_surfaces_verbatim

- **WHEN** `constraint_row_with_arbitrary_binding_text()`
- **THEN** `the compiled TOML carries the binding cell text verbatim — no interpretation, no validation of its internals`
- **VERIFIES** [[compile.binding_text_surfaces_verbatim]]

#### Scenario: claim_path_is_flags

- **WHEN** `external_checker_claiming_a_constraint()`
- **THEN** `the claim is carried by contract-TOML flags binding; the toolchain builds no registry`
- **VERIFIES** [[compile.claim_path_is_flags]]

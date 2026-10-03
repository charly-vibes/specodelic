---
id: compile
kind: intent
checked_against_core: clear
statement: "THE Compile transition SHALL translate a linted-and-covered spec file's Constraints table, Model section, and Properties table into a TOML document, a TLA+ module, and a set of proptest! blocks respectively, without dropping or altering any id."
---

# Compile

`specodelic.md` names `compile` as the transition from `linted` to
`compiled`, gated on `coverage` and `law_requires_cases`, but never
specifies what compiling actually *produces* (`STATUS.md` §4, P0). This
file is that specification: `Compile` is the functor `Set^𝒦 → TOML`
(constraints), `Set^𝒦 → TLA+` (the model), and
`Set^𝒦 → proptest!` (properties) that `STATUS.md` §1 describes — one
functor with three target categories, not three unrelated tools that
happen to run at the same pipeline stage.

## Constraints

| id                        | kind      | expr                                                                                                                             | traces_to | satisfies |
|-----------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------|-----------|-----------|
| precondition_satisfied       | invariant | `Compile only runs on a file that has passed both lint and coverage` — restates [[specodelic.compile]]'s own guard (transition, not row) | [[compile]] |           |
| constraint_table_to_toml     | invariant | `every Constraint row compiles to one TOML table entry {id, kind, expr, traces_to}, field-for-field, with no lossy transformation`      | [[compile]] |           |
| model_to_tla                 | invariant | `the Model section compiles to one TLA+ module — always emitted, regardless of which model_check backend (native stateright or TLC) later runs against the model: the module opens with a single-line TLA+ module header (dashes + MODULE name + dashes — the only form an engine parses); each State becomes a value in the module's state variable's range, each Transition becomes one disjunct of the Next action guarded by its guard field, and one closing stuttering disjunct (`UNCHANGED vpc`) ends Next — guards travel as prose comments, so a terminal state must not read as an engine-side deadlock; a State with an `emits` field additionally becomes one entry in an `Output` function from that state value to the effect-kind Constraint's `expr` — absent for a state with no `emits`, never a default/null entry` | [[compile]] |           |
| properties_to_proptest       | invariant | `each Property row compiles to one proptest! block: generator becomes the block's input strategy, predicate becomes its assertion body — verbatim as a **rust:** fragment when the row opts in ([[compile.predicate_fragment_opt_in]]), else the un-translated todo_predicate! placeholder that compiles here and fails only at execution; a law-kind property compiles to one block per case enumerated in its predicate's **name:** case labels (the identity and associativity floor is lint-enforced ahead of compile, specodelic.md Revision 13)` | [[compile]] |           |
| predicate_fragment_opt_in    | invariant | `a Property's predicate cell may opt into executable translation with exactly one **rust:** marker and a non-empty fragment after it — the fragment is a Rust boolean expression emitted verbatim as the block's assertion body, binding the block's generated values (v0…, one per generator named in the generator cell, each a String); a cell with no marker compiles exactly as before (the placeholder) — pure widening, nothing valid before this Revision is invalidated` | [[compile]] |           |
| invariant_fragment_opt_in    | invariant | `a Constraint's expr cell may carry the same **rust:** fragment when the row's kind is invariant — the fragment becomes an executable invariant model_check's native backend executes (model_check.md Revision 15); a fragment on a Constraint of any other kind (effect, advisory, a pack fiber kind) is a labeled extraction failure, never a silently ignored marker` | [[compile]] |           |
| fragment_law_rejected        | invariant | `a law-kind Property row's predicate cell must not carry a **rust:** fragment — each required case needs its own assertion body and one fragment cannot honestly serve several named cases; violation is a labeled extraction failure` | [[compile]] |           |
| fragment_guard_rejected      | invariant | `a Transition's guard cell must not carry a **rust:** fragment in this Revision — the program-counter model has no data binding a guard could constrain, so executable guards have no defined semantics; violation is a labeled extraction failure, never a silently ignored marker (decision of record: deferred to a future Revision alongside a data-carrying state space)` | [[compile]] |           |
| fragment_hygiene             | invariant | `a fragment's text must not contain unsafe, extern, include!, include_str!, include_bytes!, std::fs, std::process, std::net, std::env, asm!, or Command — defense-in-depth against a fragment escaping its artifact's role (fragments run on the invoking user's machine with the invoking user's privileges, exactly like every other compiled artifact; the Rust compiler is not a sandbox); violation is a labeled extraction failure naming the token` | [[compile]] |           |
| compile_is_total             | invariant | `∀ file that has passed lint and coverage: Compile either produces all three artifacts or reports, for exactly one of them, which stage failed and why — it never returns a silent partial result` | [[compile]] |           |
| compile_preserves_ids        | invariant | `every id present in the source file appears, unchanged, in at least one compiled artifact — no id is silently dropped in translation` | [[compile]] |           |
| no_semantic_drift            | invariant | `re-parsing a compiled artifact and re-emitting it yields output identical to the original compile — Compile is idempotent under its own round trip` | [[compile]] |           |
| extraction_failure           | effect    | `compile.extraction_failure(row_id, reason) — the extraction stage failed on row row_id because reason` | [[compile]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |
| emission_failure             | effect    | `compile.emission_failure(detail) — the emission stage failed because detail` | [[compile]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `not_started`
- `extracting`
- `emitting`
- `compiled`
- `extract_failed` (emits: `[[compile.extraction_failure]]`)
- `emit_failed` (emits: `[[compile.emission_failure]]`)

### Transitions

| id           | from        | to          | guard                                                                                             |
|--------------|-------------|-------------|--------------------------------------------------------------------------------------------------------|
| begin        | not_started | extracting  | [[compile.precondition_satisfied]]                                                                     |
| extract_ok   | extracting  | emitting    | [[compile.constraint_table_to_toml]] ∧ [[compile.model_to_tla]] ∧ [[compile.properties_to_proptest]]     |
| extract_fail | extracting  | extract_failed | `¬([[compile.constraint_table_to_toml]] ∧ [[compile.model_to_tla]] ∧ [[compile.properties_to_proptest]])` |
| accept       | emitting    | compiled    | [[compile.compile_preserves_ids]] ∧ [[compile.no_semantic_drift]]                                       |
| reject       | emitting    | emit_failed | `¬([[compile.compile_preserves_ids]] ∧ [[compile.no_semantic_drift]])`                                    |

## Properties

| id                                | kind | derives_from                             | generator                                              | predicate                                                                     |
|-------------------------------------|------|-----------------------------------------------|-------------------------------------------------------------|-----------------------------------------------------------------------------------|
| precondition_violation_rejected      | unit | [[compile.precondition_satisfied]]           | `file_that_has_not_passed_lint()`                            | `check(file) == failed`                                                          |
| toml_roundtrips                      | unit | [[compile.constraint_table_to_toml]]         | `arbitrary_constraint_row()`                                 | `parse_toml(compile(row)) == row`                                                |
| tla_disjunct_count_matches           | unit | [[compile.model_to_tla]]                     | `arbitrary_model_section(n_transitions)`                     | `count(disjuncts(compile(model).Next)) == n_transitions + 1` — the closing stuttering disjunct |
| output_function_covers_emitting_states_only | unit | [[compile.model_to_tla]]              | `arbitrary_model_section(n_states_with_emits, n_states_without)` | `domain(compile(model).Output) == the_states_with_emits` — no entry for the rest |
| proptest_block_per_property          | unit | [[compile.properties_to_proptest]]           | `arbitrary_property_row()`                                   | `count(compile(row).blocks) ≥ 1`                                                |
| law_property_compiles_required_cases | unit | [[compile.properties_to_proptest]]           | `law_property_row_with(cases: ["identity", "associativity"])` | `count(compile(row).blocks) == 2`                                               |
| compile_completes_or_fails_cleanly   | unit | [[compile.compile_is_total]]                 | `arbitrary_linted_and_covered_file()`                         | `compile(file) ∈ {all_three_artifacts, single_labeled_failure}` — never partial |
| id_preservation_holds                | unit | [[compile.compile_preserves_ids]]            | `arbitrary_linted_and_covered_file()`                         | `ids_in(file) ⊆ ids_in(compile(file))`                                          |
| roundtrip_stable                     | unit | [[compile.no_semantic_drift]]                | `arbitrary_linted_and_covered_file()`                         | `compile(file) == compile(parse(compile(file)))`                                |
| extraction_failure_label_asserted    | unit | [[compile.extraction_failure]]               | `extraction_stage_fails_on_row()`                             | `error_label == "compile.extraction_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
| emission_failure_label_asserted      | unit | [[compile.emission_failure]]                 | `emission_stage_fails()`                                      | `error_label == "compile.emission_failure"` — same three-site rename rule (EDGE-002) |
| fragment_body_emitted_verbatim      | unit | [[compile.predicate_fragment_opt_in]]    | `property_row_with_rust_fragment()`                              | `the emitted block body contains the fragment verbatim`  |
| prose_predicate_stays_placeholder   | unit | [[compile.predicate_fragment_opt_in]]    | `property_row_without_fragment()`                                | `compile(row).body contains todo_predicate!` — no widening of pre-Revision behavior |
| fragment_values_bind_generators     | unit | [[compile.predicate_fragment_opt_in]]    | `property_row_with_two_generators()`                             | `the emitted block's args are v0, v1 — one per named generator, each a String` |
| law_fragment_labeled                | unit | [[compile.fragment_law_rejected]]        | `law_row_with_fragment()`                                        | `error_stage == fragment_extraction` |
| hygiene_violation_labeled           | unit | [[compile.fragment_hygiene]]             | `fragment_with(banned_token: "std::fs")`                         | `error_stage == fragment_extraction ∧ message names the token` |
| guard_fragment_labeled              | unit | [[compile.fragment_guard_rejected]]      | `transition_row_with_fragment_guard()`                           | `error_stage == fragment_extraction` |
| non_invariant_fragment_labeled      | unit | [[compile.invariant_fragment_opt_in]]    | `effect_row_with_fragment()`                                     | `error_stage == fragment_extraction` |

## Notes

**This file is what makes `specodelic.md`'s `rename_naturality` law fully
stated.** That property's **naturality** case reads
`compile(rename(I)) == rename(compile(I))` — until now that referenced a
`compile` function with no specification of its own, so the claim was
unfalsifiable in practice even though it was written down. With
`constraint_table_to_toml`, `model_to_tla`, and `properties_to_proptest`
now pinned down field-for-field, `compile(·)` in that law means something
concrete: three specific, checkable translations, not an unnamed black
box.

`compile_is_total`'s "never a silent partial result" is the reason
`extracting`/`emitting` are split into two states rather than one: a
failure during extraction (a malformed Constraint/Model/Property row) and
a failure during emission (the three extracted representations don't
agree well enough to produce one coherent output — e.g. a Transition's
`guard` references a Constraint id that extraction resolved but the TOML
and TLA+ passes disagree about its type) are different failure classes,
and collapsing them into one `failed` transition would have been the same
God-transition pattern `specodelic.md` Revision 2 already removed from
`lint`.

**Updated for `specodelic.md` Revision 6:** `model_to_tla` now also
compiles a State's optional `emits` field into a small `Output` function
alongside `Next` — the piece that turns a compiled automaton into a
compiled *Moore machine*. This is additive to the existing translation,
not a restructuring of it: a Model with no `emits` fields anywhere
compiles exactly as before, `Output` is simply empty.

`checked_against_core: clear` (see `AGENTS.md`'s convention) — every
constraint here traces to this file's own intent rather than to a
top-level invariant, the same shape `linter-frontmatter.md` through
`linter-coverage.md` use for constraints scoped entirely to one feature.

**The `.tla` module is a product of `compile` alone, not of any checker
choice.** `model_check` may run a native interpreter backend that never
reads the module (see `model_check.md`'s Notes on backends), but the
module is still emitted: it is the human-reviewable, engine-portable form
of the compiled model, and TLC — the opt-in reference engine — consumes it
directly. Correspondingly the corpus language no longer offers "or
Alloy": with a native default backend and a reference TLA+ engine it had
no remaining role, and dropping it keeps `model_to_tla` a single, fully
specified translation.

`model_check` and `verify` (`STATUS.md` §4, both now done — see
`model_check.md` and `verify.md`) both consume this
file's output directly: `model_check` runs a model-check backend against
the TLA+ module `model_to_tla` produces, and `verify` runs the
proptest! blocks `properties_to_proptest` produces. Neither has its own
spec yet.

**Executable predicate fragments (specodelic.md Revision 15, specodelic-rjb).**
Every predicate and expr cell may now opt into executable translation with a
`**rust:**` marker — the cell text after the marker is a Rust boolean
expression, emitted verbatim, no mini-language in between. This closes the
gap the compile functor shipped with: a predicate was compiled to a
`todo_predicate!` placeholder that compiles and panics at execution, so
`verify`'s properties gate failed honestly but could never pass. The four
rejection rows keep the marker honest: a fragment on a law row, a
non-invariant Constraint, or a Transition guard is a labeled extraction
failure, not a silently ignored marker (guards have no binding over the
program-counter model — deferred of record). `fragment_hygiene` is the
security/totality guard, deliberately defense-in-depth rather than a
sandbox: fragments run on the invoking user's machine with the invoking
user's privileges, the same trust every compiled artifact already had —
the banned-token list catches the escape hatches a table cell should never
legitimately need. The scaffolding's element type is now `String` (the
`GenVal` tuple-struct wrapper retired in the same Revision) so fragments
bind plain generator values — the wrapper existed only to carry the
placeholder story, and fragments make that story optional.

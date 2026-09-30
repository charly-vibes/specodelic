---
id: graph
kind: intent
checked_against_core: clear
statement: "THE graph tool SHALL derive, from every typed reference field
  in the repo, a single authoritative adjacency structure over 𝒦's kinds,
  kept as a generated artifact that is never hand-edited, and SHALL answer
  transitive-closure (blast-radius) queries against it without re-walking
  markdown per query."
---

# Graph

Every checker in this repo (`linter-referential_integrity.md`,
`linter-graph_shape.md`, `rename.md`) already walks the reference edges
`specodelic.md`'s Reference Typing table defines — but each does it
independently, on demand, by re-reading markdown. Nothing keeps a single
standing structure of "what points at what" that a tool could query
directly. This file specifies that structure: a derived artifact, not a
hand-authored one, so `merge.md`'s conflict check and `refactor.md`'s
fan-in signal can both query it instead of each re-deriving reachability
their own way — the same "one stated fact, not several hand-maintained
copies" discipline `specodelic.md` Revision 6 already applied to
`append_only_variants`, here applied to the graph itself rather than a
constraint.

## Constraints

| id                              | kind      | expr                                                                                                                                                              | traces_to  satisfies |
|-----------------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------|
| graph_is_derived_not_authored     | invariant | `every edge in the graph artifact is produced by extraction from repo files; the artifact contains no information a hand edit to it could add that isn't already present in some file's reference field` | [[graph]] |          |
| total_extraction                  | invariant | `∀ reference field instance (traces_to/derives_from/guard/from/to/supersedes/emits/observes) in any parsed file: exactly one corresponding edge exists in the graph` — a guard's conjunction of several `[[id]]` citations counts as one edge per citation, not one edge per transition | [[graph]] |          |
| edge_kind_matches_typing          | invariant | `∀ edge (source, field, target): (source.kind, field, target.kind) ∈ [[specodelic]]'s Reference Typing table — the graph never records an edge the typing table wouldn't allow, whether or not the underlying file is otherwise well-formed` | [[graph]] |          |
| deterministic_derivation          | invariant | `re-deriving the graph from an unchanged repo produces a byte-identical artifact` | [[graph]] |          |
| blast_radius_is_transitive_closure | invariant | `blast_radius(id) == the forward and backward transitive closure of edges reachable from id, computed entirely within the derived artifact — never by re-walking source files` | [[graph]] |          |
| stale_graph_detected               | invariant | `∃ file whose on-disk content hash differs from the hash recorded at the artifact's last extraction ⟹ the artifact reports itself stale rather than silently serving an outdated query result` | [[graph]] |          |
| external_boundary_derived          | invariant | `a file is classified as an external boundary iff it hosts ≥1 extension_point Constraint — the classification is a pure derivation over extracted structure, never an authored tag: removing the rows removes the classification, so a stale boundary tag cannot exist` | [[graph]] |          |
| extraction_failure | effect | `graph.extraction_failure(detail) — the label names its owning file per error_expr_shape` | [[graph]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `unindexed`
- `extracting`
- `indexed`
- `queryable`
- `failed` (emits: `[[graph.extraction_failure]]`)

### Transitions

| id           | from        | to          | guard                                                                                          |
|--------------|-------------|-------------|--------------------------------------------------------------------------------------------------|
| extract      | unindexed   | extracting  | `every file in the repo has independently reached parsed` — see [[specodelic]]'s own lifecycle    |
| extract_ok   | extracting  | indexed     | [[graph.total_extraction]] ∧ [[graph.edge_kind_matches_typing]]                                  |
| extract_fail | extracting | failed | `¬([[graph.total_extraction]] ∧ [[graph.edge_kind_matches_typing]])` |
| publish      | indexed     | queryable   | [[graph.deterministic_derivation]] ∧ [[graph.graph_is_derived_not_authored]]                     |
| invalidate   | queryable   | unindexed   | [[graph.stale_graph_detected]]                                                                    |


## Properties

| id                          | kind | derives_from                            | generator                                                        | predicate                                                                                          |
|-------------------------------|------|--------------------------------------------|-----------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
| every_edge_extracted          | unit | [[graph.total_extraction]]               | `arbitrary_spec_repo()`                                                | `count(edges(graph)) == count(reference_field_instances(repo))`                                        |
| wrongly_typed_edge_rejected   | unit | [[graph.edge_kind_matches_typing]]       | `hand_inserted_edge(source_kind: State, field: "guard", target_kind: State)` | `check(artifact) == rejected` — `guard` never targets a State per the Reference Typing table         |
| rerun_deterministic           | unit | [[graph.deterministic_derivation]]       | `run_extraction_twice_against_unchanged_repo()`                        | `artifact(run_1) == artifact(run_2)`                                                                    |
| blast_radius_transitive       | unit | [[graph.blast_radius_is_transitive_closure]] | `chain(a traces_to b, b derives_from c)`                          | `c ∈ blast_radius(a)` — reachable through two hops, no direct edge required                            |
| blast_radius_bidirectional    | unit | [[graph.blast_radius_is_transitive_closure]] | `edge(a → b)`                                                     | `a ∈ blast_radius(b) ∧ b ∈ blast_radius(a)` — renaming `a` must be able to find dependent `b`, and vice versa |
| stale_detected_on_edit        | unit | [[graph.stale_graph_detected]]           | `edit_one_reference_field_without_re_extracting()`                     | `check(artifact) == stale`                                                                              |
| hand_authored_edge_rejected   | unit | [[graph.graph_is_derived_not_authored]]  | `graph_artifact_with_a_manually_inserted_edge_absent_from_any_file()`  | `check(artifact) == rejected`                                                                            |
| boundary_tracks_extension_points | unit | [[graph.external_boundary_derived]] | `file_hosting_extension_point_row_then_all_removed()` | `classified(file) == true before, == false after removal — no authored residue` |
| observes_edge_extracted       | unit | [[graph.total_extraction]]               | `constraint_row_carrying_observes_pointing_at_effect()` | `exactly one edge, kind constraints.observes — and zero edges when the column is absent`               |
| extraction_failure_label_asserted | unit | [[graph.extraction_failure]] | `extraction_failure_raised()` | `error_label == "graph.extraction_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

`checked_against_core: clear` (see `AGENTS.md`'s convention). One resolved
ambiguity worth recording anyway: `total_extraction` counts each `[[id]]`
citation inside a conjunctive guard (`A ∧ B ∧ C`) as its own edge, not the
whole guard as one edge — otherwise `blast_radius` would under-count what a
transition actually depends on.

**`observes` (specodelic.md Revision 9) joins `total_extraction` but no
cycle set.** An observation claim is not a dependency: the acyclic
invariant's edge set (`linter-graph_shape.md`) is a closed union that
`observes` deliberately does not join — `satisfies` and `emits` are
excluded by the same closure — so mutual cross-file observation is
well-formed. The external-boundary classification is the same
derived-not-authored discipline applied to visibility: a file is an
external boundary because it *published a contract*, and the fact
disappears when the publication does.

**Scope boundary, `Needs Human Review`:** this file does not replace
`linter-referential_integrity.md`'s `total_refs`/`unique_id` or
`linter-graph_shape.md`'s `acyclic_traces`/`single_root_reachable` — those
checkers still own those constraints. What's genuinely open is whether
those two files should be refactored to query this artifact internally
rather than independently re-deriving reachability, now that a shared
artifact exists. That's an efficiency and single-source-of-truth question,
not a correctness gap, so it's flagged rather than forced — changing
either linter file's internal mechanism without changing what it checks
is exactly the kind of edit `AGENTS.md`'s workflow should route through a
deliberate decision, not a side effect of adding this file.

**Why extraction only requires `parsed`, not `linted`.** Building the
adjacency structure needs a reference field's syntactic presence, not its
resolution — a dangling `[[typo_id]]` still produces an edge (to a target
that doesn't yet exist), which is precisely the shape `total_refs` needs
to check against. Requiring `linted` first would make the graph unable to
represent the very files whose brokenness the existing checkers need to
detect.

`merge.md` and `refactor.md` both depend on this file for blast-radius and
fan-in queries respectively, rather than each re-deriving reachability —
see each file's own Constraints for how.

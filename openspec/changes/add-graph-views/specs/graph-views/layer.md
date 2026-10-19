---
id: graph.views.layer
kind: intent
statement: "THE graph-views capability SHALL emit a deterministic edge-list projection with canonical ids and annotated typing violations from any specodelic-compliant corpus, and SHALL derive per-file state-machine, file-level traceability, and revision-labeled schema views from that artifact plus the lint-gated acset Schema value and the format revision marker alone — never from prose."
---

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                                                  | traces_to |
|--------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[graph.views.layer]] |
| canonical_ids            | invariant | `every edge-list endpoint is a canonical node ID (intent ID or qualified row ID) value; display labels (e.g. `refactor (intent)`-style label-qualified nodes) never appear in the projection — normalization happens at or before projection`                                                                 | [[graph.views.layer]]  |
| deterministic_projection | invariant | `re-running the projection over an unchanged corpus produces a byte-identical edge list — rows sorted, no timestamps, no iteration-order leakage`                                                                                                                        | [[graph.views.layer]]  |
| violations_annotated     | invariant | `every violation in the graph artifact appears as an annotation row in the edge list; a view rendered from a corpus with violations is never silently clean — dashed/annotated rendering is mandatory, omission forbidden`                                                | [[graph.views.layer]]  |
| derived_views_only       | invariant | `corpus views consume only graph artifacts; the schema view consumes only a versioned export of the lint-gated acset Schema value and format revision; no view parses prose, re-walks markdown, or embeds hand-authored structure`                                                                                            | [[graph.views.layer]]  |
| build_time_generation    | invariant | `rendered views are generated into the docs build at build time and never committed; no hand-edit path exists, so graph_is_derived_not_authored holds by construction and no staleness check is needed`                                                                    | [[graph.views.layer]]  |
| empty_corpus_valid       | invariant | `spk graph --format edges over any parseable corpus — including a directory with zero spec files, or a single intent with no cross-file edges — exits 0 and emits a well-formed (possibly empty) row set: graph extraction requires parsed, not linted (openspec/specs/graph/spec.md's scope note); rendered views are transform-level and additionally gated by corpus_scope_operational — an intentless corpus is refused there with a remediation hint, never silently rendered`                                                        | [[graph.views.layer]]  |
| corpus_scope_operational | invariant | `a corpus is in scope iff spk lint over it reports no invariant-rule findings and ≥1 intent file parses; corpora failing this (e.g. openspec-layout repositories) are out of scope until a parse-boundary adapter change lands — no view special-cases them`                                | [[graph.views.layer]]  |

## Model

### States
- `proposed`
- `approved`
- `implemented`
- `archived`

### Transitions

| id          | from        | to          | guard                                                                                     |
|-------------|-------------|-------------|-------------------------------------------------------------------------------------------|
| approve     | proposed    | approved    | [[graph.views.layer.process_lifecycle]] ∧ `proposal reviewed and approved by the maintainer`  |
| implement   | approved    | implemented | [[graph.views.layer.process_lifecycle]] ∧ `all tasks.md items complete; spk lint, transform tests, and openspec validate --strict pass`  |
| archive     | implemented | archived    | [[graph.views.layer.process_lifecycle]] ∧ `just archive-change id=add-graph-views ran with the dual-format recipe`  |

## Properties

| id                            | kind | derives_from                     | generator                                             | predicate                                                                 |
|-------------------------------|------|-----------------------------------|-----------------------------------------------------------------------------------------------------|
| process_lifecycle_checked | unit | [[graph.views.layer.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| ids_never_labels              | unit | [[graph.views.layer.canonical_ids]]           | `corpus_parsed_then_projected()`                      | `no TSV field matches a label-qualified node pattern`                     |
| rerun_byte_identical          | unit | [[graph.views.layer.deterministic_projection]] | `same_corpus_projected_twice()`                       | `both runs byte-identical; also holds across invocation order of files`   |
| violations_survive_projection | unit | [[graph.views.layer.violations_annotated]]    | `corpus_with_known_typing_violations()`               | `edge list contains one annotation row per violation, none missing`       |
| clean_corpus_no_annotations   | unit | [[graph.views.layer.violations_annotated]]    | `lint_clean_corpus()`                                 | `edge list contains zero annotation rows`                                 |
| views_from_artifact_only      | unit | [[graph.views.layer.derived_views_only]]      | `view_rendered_with_prose_perturbation()`             | `rendered view byte-identical after perturbing prose blocks`              |
| schema_view_revision_labeled  | unit | [[graph.views.layer.derived_views_only]]      | `schema_value_rendered()`                         | `schema view names the format revision it was derived from`               |
| build_output_not_committed    | unit | [[graph.views.layer.build_time_generation]]   | `docs_build_completed()`                              | `git status clean — no rendered artifact tracked or staged`               |
| empty_corpus_projection       | unit | [[graph.views.layer.empty_corpus_valid]]      | `directory_with_zero_spec_files()`                    | `projection exits 0 with empty output — the view half is governed by corpus_scope_operational (an intentless corpus is refused: out_of_scope_refused)` |
| single_intent_sane            | unit | [[graph.views.layer.empty_corpus_valid]]      | `corpus_with_one_intent_no_cross_file_edges()`        | `file-level view renders one node; no crash`                              |
| out_of_scope_refused          | unit | [[graph.views.layer.corpus_scope_operational]] | `corpus_with_invariant_findings_or_no_intents()`      | `transform exits non-zero with a remediation hint naming the failed gate` |


---
id: refactor
kind: intent
checked_against_core: clear
statement: "WHEN a proposed changeset would edit rows owned by a node with
  high, unrelated fan-in in [[graph]], THE refactor advisor SHALL surface a
  non-gating suggestion to split that node first, and SHALL never block or
  delay any transition in [[specodelic]]'s or [[orchestrate]]'s lifecycle."
---

# Refactor Advisor

`STATUS.md` §2 already names the pathology this mechanizes — "God object /
cyclic dependency" — and `specodelic.md` Revision 2 already fixed one
real instance of it by hand (splitting the flat 7-clause `lint` guard into
the Checker Ownership table) before anyone had a tool that would have
flagged it proactively. This file is that tool's spec: Kent Beck's
tidy-first rule ("separate the structural change from the behavioral one")
read as a graph query — a node with fan-in from unrelated namespaces, or a
changeset that only needs a fraction of what a node owns, is a candidate
for splitting *before* the behavioral edit lands, not after.

## Constraints

| id                              | kind      | expr                                                                                                                                                                     | traces_to  |
|-------------------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------|
| fan_in_read_from_graph              | invariant | `the advisor's dependent-count for node N is [[graph]]'s blast_radius(N) restricted to incoming edges — never independently recomputed by walking markdown`                | [[refactor]] |
| unrelated_fan_in_defined            | invariant | `a dependent is "unrelated" to N when N and the dependent share no namespace segment below the repo root — i.e. their nearest common ancestor in the id namespace is the root itself` | [[refactor]] |
| narrow_diff_heuristic               | invariant | `a changeset naming a strict subset of node N's owned Constraint/Property rows as its actual target, while depending on none of N's other owned rows, is flagged regardless of N's fan-in count` | [[refactor]] |
| threshold_is_per_repo_setting       | invariant | `the fan-in count that counts as "high" is a configured value, not a number fixed in this file — "high" is relative to a given repo's typical fan-in`                       | [[refactor]] |
| advisory_finding_emitted            | effect    | `output == {node_id, dependent_count, unrelated_namespace_count, suggested_split: bool}` | [[refactor]] |

## Model

### States
- `idle`
- `analyzing`
- `clean`
- `found` `emits: [[refactor.advisory_finding_emitted]]`

### Transitions

| id        | from       | to         | guard                                                                                          |
|-----------|------------|------------|---------------------------------------------------------------------------------------------------|
| analyze   | idle       | analyzing  | `[[graph]] has reached queryable` — see `graph.md`'s own lifecycle                                  |
| flag      | analyzing  | found      | [[refactor.unrelated_fan_in_defined]] ∨ [[refactor.narrow_diff_heuristic]]                       |
| clear     | analyzing  | clean      | `¬flag.guard`                                                                                       |

## Properties

| id                              | kind | derives_from                              | generator                                                                    | predicate                                                                                     |
|-----------------------------------|------|----------------------------------------------|-------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------|
| fan_in_never_rewalked             | unit | [[refactor.fan_in_read_from_graph]]         | `node_with_precomputed_blast_radius_in_graph()`                                     | `advisor_source_calls(markdown_walker) == 0` — every count comes from `graph`'s artifact           |
| unrelated_fan_in_flagged          | unit | [[refactor.unrelated_fan_in_defined]]       | `node_referenced_by_dependents_in_three_disjoint_top_level_namespaces()`            | `check(node) == found`                                                                              |
| related_fan_in_not_flagged        | unit | [[refactor.unrelated_fan_in_defined]]       | `node_referenced_only_by_dependents_within_its_own_namespace_subtree()`             | `check(node) == clean`                                                                              |
| narrow_diff_flagged_low_fan_in    | unit | [[refactor.narrow_diff_heuristic]]          | `changeset_touching_one_of_five_constraints_owned_by_a_node_with_fan_in(1)`         | `check(node) == found` — flagged even though fan-in alone wouldn't trigger it                       |
| emitted_finding_shape             | unit | [[refactor.advisory_finding_emitted]]       | `found state reached for node N with dependent_count 4, 2 unrelated`               | `emits(found) == {node_id: N, dependent_count: 4, unrelated_namespace_count: 2, suggested_split: true}` |
| threshold_read_from_config        | unit | [[refactor.threshold_is_per_repo_setting]]  | `same_node_same_changeset_under_two_repo_configs_with_different_high_fan_in_values()` | `flagged(config_A) != flagged(config_B)` — the decision follows the repo's configured threshold, never a constant embedded in this file |

## Notes

`checked_against_core: clear` (see `AGENTS.md`'s convention). Nothing new
was needed to make the finding non-gating: `advisory_finding_emitted` is
`kind == effect`, and `[[specodelic.ref_kind_compatible]]`'s own typing
already restricts every transition's `guard` field to `kind == invariant`
Constraints only, tested generically by `[[specodelic.advisory_cannot_gate]]`.
An `effect`-kind Constraint is excluded from guard-eligibility by that same
rule, for the same reason an `advisory`-kind one is — so this file needed
no invariant of its own asserting "the finding can't gate anything"; that
was already true, globally, before this file existed. (An earlier draft of
this file carried two such invariants plus a redundant unit test — removed
as an instance of the exact "one rule, restated three ways" pattern
`specodelic.md` Revision 6 already caught once, for `append_only_variants`.)
`advisory_finding_emitted`'s `emits` reuses the Moore-machine mechanism
Revision 6 built for `State`; this file adds no sixth `𝒦` object and no
new kind of Constraint — a new *consumer* of an existing mechanism, not a
new one.

**This file never decides how to split a node.** It only detects the
fan-in/narrow-diff shape and emits a finding; the actual split — a new
file, an updated `traces_to`, an updated Checker Ownership-style table if
the node is itself a dependency hub — is an ordinary edit, checked the
normal way by the six existing linters. Deciding *how* to split is a
design judgment this file deliberately doesn't automate.

**Depends on `graph.md`**, not on re-deriving reachability: `analyze`'s
guard cites `graph`'s own `queryable` state in prose (the same "cite
another file's state fact in prose when it isn't itself a Constraint row"
pattern `orchestrate.md` already uses for `model_check.md`/`verify.md`),
and every fan-in count is a `graph.blast_radius` query, never an
independent markdown walk.

**Decision of record (2026-09-30, user-approved; advised by a typed
Jev evaluation, `jev-1.13.0`, conf 0.54):** "unrelated fan-in" keys off
the **id namespace** — the "shares no namespace segment below root"
test stands as written. Physical directory placement is rejected as the
keying: ids are the format's primary identity (`id_matches_file` ties
them to filenames, the Reference Typing table types them as foreign
keys), so relatedness judged by namespace is consistent with how every
other checker reads the corpus, while directory placement would make
the advisory's meaning depend on filesystem layout the format doesn't
own. The flat-namespace failure mode is real but contained: the finding
is `kind == advisory`-shaped (non-gating by typing), and the threshold
is already per-repo configurable — a flat-namespace repo raises the
threshold or lives with noise. A configurable keying mode (namespace or
directory) stays a possible future widening under
`kind_field_extensible`-style governance if a real flat-namespace corpus
ever adopts the format; nothing in this file's constraint text changes.

**Open question, `Needs Human Review`:** `threshold_is_per_repo_setting`
deliberately leaves the actual number unset here — this file specifies
*that* the threshold is configurable, not what a reasonable default is.

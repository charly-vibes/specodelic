---
id: linter.graph_shape
kind: intent
statement: "WHEN a spec repo has passed referential integrity, THE linter SHALL reject it if the traces_to/derives_from reference graph contains a cycle, or if the supersedes reference graph contains a cycle."
---

# Linter: Graph Shape Check

Runs after `linter.referential_integrity` — it needs every reference already
proven to resolve before it can walk the graph those references form.
Corresponds to the modularity-diagnostician's "dependency graph must form a
DAG" check, applied to specs instead of code modules.

## Constraints

| id                   | kind      | expr                                                                                  | traces_to | satisfies |
|-----------------------|-----------|------------------------------------------------------------------------------------------|--------------------------------------------|
| acyclic               | invariant | `the directed graph formed by traces_to ∪ derives_from ∪ guard-as-edge has no cycle`      | [[linter.graph_shape]] |          |
| single_root_reachable | invariant | `∀ constraint/property/state/transition row: reachable(row, the file's own intent row through own-file primary linkage)` — same claim as [[specodelic.single_root_reachable]] at checker altitude | [[linter.graph_shape]] |          |
| no_self_ref           | invariant | `∀ row: row.traces_to != row.id and row.derives_from != row.id`                           | [[linter.graph_shape]] |          |
| supersedes_dag        | invariant | `the directed graph formed by supersedes edges alone (Constraint→Constraint, Property→Property) has no cycle` | [[linter.graph_shape]] |          |
| check_failure | effect | `linter.graph_shape.check_failure(detail)` | [[linter.graph_shape]] | [[errors.envelope_error_kind]] ∧ [[errors.exit_code_mapping]] ∧ [[errors.remediation_hint_present]] |

## Model

### States
- `unbuilt`
- `built`
- `traversing`
- `passed`
- `failed` (emits: `[[linter.graph_shape.check_failure]]`)

### Transitions

| id            | from        | to          | guard                                                                                 |
|---------------|-------------|-------------|------------------------------------------------------------------------------------------|
| build_graph   | unbuilt     | built       | `repo passed linter.referential_integrity`                                               |
| traverse      | built       | traversing  | `graph built successfully`                                                               |
| accept        | traversing  | passed      | [[linter.graph_shape.acyclic]] ∧ [[linter.graph_shape.single_root_reachable]] ∧ [[linter.graph_shape.no_self_ref]] ∧ [[linter.graph_shape.supersedes_dag]] |
| reject | traversing | failed | `¬([[linter.graph_shape.acyclic]] ∧ [[linter.graph_shape.single_root_reachable]] ∧ [[linter.graph_shape.no_self_ref]] ∧ [[linter.graph_shape.supersedes_dag]])` |


## Properties

| id                     | kind | derives_from                                    | generator                                         | predicate                                                                 |
|-------------------------|------|-----------------------------------------------------|--------------------------------------------------------|-----------------------------------------------------------------------------|
| cycle_rejected           | unit | [[linter.graph_shape.acyclic]]                    | `spec_repo_with(traces_to_cycle: length ≥ 2)`           | `check(repo) == failed`                                                    |
| self_ref_rejected        | unit | [[linter.graph_shape.no_self_ref]]                | `spec_row_with(traces_to == self.id)`                   | `check(repo) == failed`                                                    |
| orphan_subgraph_rejected | unit | [[linter.graph_shape.single_root_reachable]]       | `spec_repo_with(disconnected_constraint_cluster: true)` | `check(repo) == failed`                                                    |
| dag_passes               | unit | [[linter.graph_shape.acyclic]]                    | `arbitrary_dag_shaped_repo()`                           | `check(repo) == passed`                                                    |
| supersedes_cycle_rejected | unit | [[linter.graph_shape.supersedes_dag]]            | `spec_repo_with(supersedes_cycle: length ≥ 2)`          | `check(repo) == failed`                                                    |
| supersedes_dag_ignores_traces_cycle | unit | [[linter.graph_shape.supersedes_dag]]  | `spec_repo_with(traces_to_cycle: true, supersedes: acyclic)` | `check(repo).supersedes_dag == passed` — the two graphs are checked independently; a cycle in one must not be attributed to the other |
| topo_sort_naturality     | law  | [[linter.graph_shape.acyclic]]                    | `arbitrary_dag_repo(), arbitrary_id_rename()`           | **naturality:** `topo_sort(rename(I)) == rename(topo_sort(I))` — renaming a node doesn't change relative order of unrelated nodes |
| check_failure_label_asserted | unit | [[linter.graph_shape.check_failure]] | `check_failure_raised()` | `error_label == "linter.graph_shape.check_failure"` — renaming the label touches the error Constraint, this property, and its note together (EDGE-002) |
## Notes

**Reference-typing reconciliation (2026-09-30, `specodelic-cxq`):** the
Constraints table's `traces_to` cells previously pointed at the
`specodelic.md` rows these checks re-own — a Constraint→Constraint target,
which the Reference Typing table forbids (`traces_to` resolves to Intent
only). Each now traces to this file's own intent: `specodelic.md` keeps
the corpus-wide statement of record, this file owns the checkable one
(the "same claim, two altitudes" pattern the Notes below already use).

`single_root_reachable` is new relative to `specodelic.md`'s original
`acyclic_traces` invariant — acyclicity alone permits an orphaned island of
constraints/properties that trace only to each other and never up to a real
`intent`. That's not a cycle, but it is the spec-level equivalent of
`modularity-diagnostician`'s "missing boundary": a cluster with no owning
purpose. The gap was folded back into `specodelic.md`'s constraint list,
and the open question this file raised (any intent vs the file's own) was
**decided of record 2026-09-30 (HITL `specodelic-mp1` row 8)**: reachability
lands on the file's OWN intent through own-file primary linkage, with
cross-file typed edges (`guard` citations of foreign constraints,
`satisfies`, `observes`) counted as outbound leaves, never reachability
paths — see `specodelic.md` Revision 10. This file's constraint expr
reflects the decided reading.

`topo_sort_naturality` is included because the model-checking and
PBT-generation pipeline both need a stable topological order to compile
against — if renaming perturbed that order, generated artifacts (TLA+
modules, `proptest!` blocks) would churn on every unrelated rename, which
defeats the "refactor-safe end to end" property `specodelic.md` promises.

`supersedes_dag` (added per `specodelic.md`'s Revision 5) is kept as its
own constraint and its own conjunct in `accept`'s guard, rather than
unioned into `acyclic`'s edge set. Lineage (`supersedes`) and provenance
(`traces_to`/`derives_from`/`guard`) answer different questions about a
row, and a cycle in one says nothing about the other —
`supersedes_dag_ignores_traces_cycle` exists specifically to pin that
independence down, so a future edit can't accidentally merge the two
graphs back into one God-check.

**`observes` (specodelic.md Revision 9) is deliberately absent from
`acyclic`'s edge set.** The union above is closed — `satisfies` and
`emits` never joined it either — and an observation claim is not a
dependency: two files mutually observing each other's effects are
well-formed. The edge set grows only under a new Revision of this file
(same discipline as `supersedes_dag`'s independence, pinned by
`supersedes_dag_ignores_traces_cycle`).

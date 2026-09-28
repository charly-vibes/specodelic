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

| id                   | kind      | expr                                                                                  | traces_to                    |
|-----------------------|-----------|------------------------------------------------------------------------------------------|---------------------------------|
| acyclic               | invariant | `the directed graph formed by traces_to ∪ derives_from ∪ guard-as-edge has no cycle`      | [[specodelic.acyclic_traces]] |
| single_root_reachable | invariant | `∀ constraint/property/state/transition row: reachable(row, some intent row)`             | [[specodelic.acyclic_traces]] |
| no_self_ref           | invariant | `∀ row: row.traces_to != row.id and row.derives_from != row.id`                           | [[specodelic.acyclic_traces]] |
| supersedes_dag        | invariant | `the directed graph formed by supersedes edges alone (Constraint→Constraint, Property→Property) has no cycle` | [[specodelic.supersedes_acyclic]] |

## Model

### States
- `unbuilt`
- `built`
- `traversing`
- `passed`
- `failed`

### Transitions

| id            | from        | to          | guard                                                                                 |
|---------------|-------------|-------------|------------------------------------------------------------------------------------------|
| build_graph   | unbuilt     | built       | `repo passed linter.referential_integrity`                                               |
| traverse      | built       | traversing  | `graph built successfully`                                                               |
| accept        | traversing  | passed      | [[linter.graph_shape.acyclic]] ∧ [[linter.graph_shape.single_root_reachable]] ∧ [[linter.graph_shape.no_self_ref]] ∧ [[linter.graph_shape.supersedes_dag]] |
| reject        | traversing  | failed      | `¬accept.guard`                                                                            |

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

## Notes

`single_root_reachable` is new relative to `specodelic.md`'s original
`acyclic_traces` invariant — acyclicity alone permits an orphaned island of
constraints/properties that trace only to each other and never up to a real
`intent`. That's not a cycle, but it is the spec-level equivalent of
`modularity-diagnostician`'s "missing boundary": a cluster with no owning
purpose. Flagging as a gap to fold back into `specodelic.md`'s constraint
list (`Needs Human Review`: should reachability be required from *any*
intent in the repo, or only the file's own declared intent? The latter is
stricter and probably correct, since it also catches misfiled cross-feature
references — but should be confirmed rather than assumed here).

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

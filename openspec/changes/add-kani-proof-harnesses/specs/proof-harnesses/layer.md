---
id: proof.harnesses.layer
kind: intent
statement: "WHEN specodelic's verdict-deciding internals execute, THE harness suite SHALL machine-check their claimed laws at the declared run points, labeling every result's scope as exhaustive or bounded and never presenting a bounded result as a general proof."
---

# proof-harnesses — dual-format layer (sidecar)

## Purpose
Make the tool's own decision-carrying internals — the verdict algebra, the
guard-expr parser, the acyclicity engine, the reachability classifier — carry
machine-checked laws the way `spk model-check` makes user corpora carry
checked invariants, with the same honesty discipline the ladder applies
everywhere else.

## Constraints

| id                     | kind      | expr                                                                                                                                                                        | traces_to                 |
|------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------|
| algebra_exhaustive     | invariant | `the verdict-algebra family checks commutativity, associativity, counterexample-dominance, verified-identity, and double-negation involution over the entire three-valued domain — a complete proof, never a sample` | [[proof.harnesses.layer]] |
| admit_wellformed_only  | invariant | `parsing yields no expression for any bracketed-id cell, round-trips every accepted cell to an equivalent expression, and panics on no input cell whatsoever`                 | [[proof.harnesses.layer]] |
| oracle_equivalence     | invariant | `within the stated unwinding bound the cycle engine reports exactly the reference checker's cycle set — no invented cycle, no missed cycle`                                   | [[proof.harnesses.layer]] |
| total_exclusive_tiers  | invariant | `within the stated bound the reachability classifier assigns every row exactly one tier — none unclassified, none classified twice`                                           | [[proof.harnesses.layer]] |
| gate_separation        | invariant | `the suite runs only at its declared recipe and nightly points — never inside the per-commit quality gate, and no harness code reaches release builds`                        | [[proof.harnesses.layer]] |
| honest_scope           | invariant | `every family result is labeled exhaustive or bounded at its stated limit — a bounded result never claims generality`                                                         | [[proof.harnesses.layer]] |

## Model

### States
- `gated`
- `checked`
- `reported`

### Transitions

| id      | from    | to       | guard                                        |
|---------|---------|----------|----------------------------------------------|
| check   | gated   | checked  | [[proof.harnesses.layer.gate_separation]]    |
| label   | checked | reported | [[proof.harnesses.layer.honest_scope]]       |

## Properties

| id        | kind | derives_from                                      | generator                        | predicate                                                          |
|-----------|------|---------------------------------------------------|----------------------------------|--------------------------------------------------------------------|
| p_algebra | unit | [[proof.harnesses.layer.algebra_exhaustive]]    | `arbitrary_verdict_tuple()`      | `all five laws hold over the whole three-valued domain`             |
| p_parser  | unit | [[proof.harnesses.layer.admit_wellformed_only]] | `arbitrary_cell_within_bound()`  | `zero panics ∧ zero admitted bracketed ids`                         |
| p_oracle  | unit | [[proof.harnesses.layer.oracle_equivalence]]    | `arbitrary_bounded_edge_set()`   | `engine cycle set == reference cycle set`                           |
| p_tiers   | unit | [[proof.harnesses.layer.total_exclusive_tiers]] | `arbitrary_bounded_spec_graph()` | `every row assigned exactly one tier`                               |
| p_gate    | unit | [[proof.harnesses.layer.gate_separation]]       | `release_build_symbol_scan()`    | `no harness symbols ∧ no suite execution in the commit-gate record` |
| p_scope   | unit | [[proof.harnesses.layer.honest_scope]]          | `arbitrary_family_result()`      | `each result labeled exhaustive ∨ bounded-at-N`                     |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `proof-harnesses` capability
and SHALL be deployed verbatim by the archive companion; the plain deltas in
`spec.md` beside it are its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** `just archive-change id=add-kani-proof-harnesses` runs after the
  change is implemented
- **THEN** the capability deploys with this layer attached fail-closed

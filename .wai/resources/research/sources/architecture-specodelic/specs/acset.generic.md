---
id: acset.generic
kind: intent
statement: "THE schema-parametric incidence core capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Schema-parametric incidence core

A domain-neutral, optional ACSet-inspired instance model without changing Specodelic canonical schema or existing graph behavior.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| isolated | invariant | `The generic instance engine SHALL be a separate module; the existing five-object Specodelic canonical schema, validation and public behavior remain unchanged.` | [[acset.generic]] |
| schema | invariant | `Schema declarations SHALL type object sets and morphisms with one source and one target, unique per source/name; non-functional relations SHALL use explicit incidence objects.` | [[acset.generic]] |
| instances | invariant | `Instances SHALL support stable external IDs, deterministic serialization, typed attribute columns and total morphisms; optional values SHALL use explicitly modeled option objects or partial-reference diagnostics.` | [[acset.generic]] |
| typing | invariant | `Invalid endpoints, wrong target types, duplicate IDs and unresolved references SHALL be reported without silent data loss.` | [[acset.generic]] |
| laws | invariant | `For accepted schemas and instances, morphism composition SHALL preserve identity and composition; schemas with equations SHALL check those equations on instances.` | [[acset.generic]] |
| edits | invariant | `Insertion, deletion and replacement SHALL be atomic and leave no dangling confirmed references; rejection SHALL preserve prior state.` | [[acset.generic]] |
| queries | invariant | `Forward/backward reachability, incidence lookup and serialization SHALL have deterministic output independent of insertion order.` | [[acset.generic]] |
| scope | invariant | `General limits, colimits, DPO rewriting and categorical data migrations SHALL NOT be claimed implemented until separately specified and tested.` | [[acset.generic]] |
| comparison | invariant | `A conventional typed adjacency-graph baseline SHALL be compared on correctness, code size, performance and implementation effort before reusing the generic core.` | [[acset.generic]] |

## Model

### States

- `draft`
- `schema_validated`
- `instance_built`
- `rejected`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_schema_validated | draft | schema_validated | `preconditions for schema_validated hold` ∧ [[acset.generic.isolated]] |
| advance_instance_built | schema_validated | instance_built | `preconditions for instance_built hold` ∧ [[acset.generic.isolated]] |
| advance_rejected | instance_built | rejected | `preconditions for rejected hold` ∧ [[acset.generic.isolated]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| isolated_holds | unit | [[acset.generic.isolated]] | `fixture_isolated()` | `The generic instance engine SHALL be a separate module; the existing five-object Specodelic canonical schema, validation and public behavior remain unchanged.` |
| schema_holds | unit | [[acset.generic.schema]] | `fixture_schema()` | `Schema declarations SHALL type object sets and morphisms with one source and one target, unique per source/name; non-functional relations SHALL use explicit incidence objects.` |
| instances_holds | unit | [[acset.generic.instances]] | `fixture_instances()` | `Instances SHALL support stable external IDs, deterministic serialization, typed attribute columns and total morphisms; optional values SHALL use explicitly modeled option objects or partial-reference diagnostics.` |
| typing_holds | unit | [[acset.generic.typing]] | `fixture_typing()` | `Invalid endpoints, wrong target types, duplicate IDs and unresolved references SHALL be reported without silent data loss.` |
| laws_holds | unit | [[acset.generic.laws]] | `fixture_laws()` | `For accepted schemas and instances, morphism composition SHALL preserve identity and composition; schemas with equations SHALL check those equations on instances.` |
| edits_holds | unit | [[acset.generic.edits]] | `fixture_edits()` | `Insertion, deletion and replacement SHALL be atomic and leave no dangling confirmed references; rejection SHALL preserve prior state.` |
| queries_holds | unit | [[acset.generic.queries]] | `fixture_queries()` | `Forward/backward reachability, incidence lookup and serialization SHALL have deterministic output independent of insertion order.` |
| scope_holds | unit | [[acset.generic.scope]] | `fixture_scope()` | `General limits, colimits, DPO rewriting and categorical data migrations SHALL NOT be claimed implemented until separately specified and tested.` |
| comparison_holds | unit | [[acset.generic.comparison]] | `fixture_comparison()` | `A conventional typed adjacency-graph baseline SHALL be compared on correctness, code size, performance and implementation effort before reusing the generic core.` |

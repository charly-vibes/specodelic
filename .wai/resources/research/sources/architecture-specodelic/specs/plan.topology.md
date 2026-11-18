---
id: plan.topology
kind: intent
statement: "THE architectural incidence model capability SHALL satisfy typed structural consistency, provenance, reversible human correction and independently testable behavior as defined below."
---

# Architectural incidence model

Maintain an editable graph of wall faces, vertices, rooms, wall thicknesses and openings with validated incidence.

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| shared | invariant | `Incident wall segments SHALL reference shared stable vertex IDs rather than duplicate endpoint coordinate values.` | [[plan.topology]] |
| faces | invariant | `Interior faces SHALL be distinguished from wall centerlines, thickness and exterior outlines; unknown thickness SHALL remain unknown.` | [[plan.topology]] |
| boundary | invariant | `A room boundary SHALL be an ordered oriented cycle or an explicit unresolved boundary; open and self-intersecting cycles SHALL NOT be valid closed rooms.` | [[plan.topology]] |
| openings | invariant | `Every confirmed opening SHALL reference an existing host wall, bounded offset/width and elevations; invalid attachments SHALL be rejected.` | [[plan.topology]] |
| nonmanhattan | invariant | `No universal orthogonality, rectangularity or fixed number of room edges SHALL be imposed.` | [[plan.topology]] |
| rewrite | invariant | `Split/merge/delete SHALL atomically update incidence, rooms, hosted openings and referenced dimensions or report a conflict without mutation.` | [[plan.topology]] |
| identities | invariant | `Semantic entity IDs SHALL remain stable across geometric edits; replacement operations SHALL record old-to-new mappings.` | [[plan.topology]] |

## Model

### States

- `empty`
- `partial`
- `valid`
- `conflicted`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| advance_partial | empty | partial | `preconditions for partial hold` ∧ [[plan.topology.shared]] |
| advance_valid | partial | valid | `preconditions for valid hold` ∧ [[plan.topology.shared]] |
| advance_conflicted | valid | conflicted | `preconditions for conflicted hold` ∧ [[plan.topology.shared]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| shared_holds | unit | [[plan.topology.shared]] | `fixture_shared()` | `Incident wall segments SHALL reference shared stable vertex IDs rather than duplicate endpoint coordinate values.` |
| faces_holds | unit | [[plan.topology.faces]] | `fixture_faces()` | `Interior faces SHALL be distinguished from wall centerlines, thickness and exterior outlines; unknown thickness SHALL remain unknown.` |
| boundary_holds | unit | [[plan.topology.boundary]] | `fixture_boundary()` | `A room boundary SHALL be an ordered oriented cycle or an explicit unresolved boundary; open and self-intersecting cycles SHALL NOT be valid closed rooms.` |
| openings_holds | unit | [[plan.topology.openings]] | `fixture_openings()` | `Every confirmed opening SHALL reference an existing host wall, bounded offset/width and elevations; invalid attachments SHALL be rejected.` |
| nonmanhattan_holds | unit | [[plan.topology.nonmanhattan]] | `fixture_nonmanhattan()` | `No universal orthogonality, rectangularity or fixed number of room edges SHALL be imposed.` |
| rewrite_holds | unit | [[plan.topology.rewrite]] | `fixture_rewrite()` | `Split/merge/delete SHALL atomically update incidence, rooms, hosted openings and referenced dimensions or report a conflict without mutation.` |
| identities_holds | unit | [[plan.topology.identities]] | `fixture_identities()` | `Semantic entity IDs SHALL remain stable across geometric edits; replacement operations SHALL record old-to-new mappings.` |

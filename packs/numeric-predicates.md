---
id: numeric.predicates
kind: profile
checked_against_core: clear
statement: "WHEN a workspace enables the numeric-predicates standard pack, THE format SHALL provide a namespaced ## Quantities section whose rows carry name, kind, unit, and domain; numeric.quantity, numeric.bound, and numeric.tolerance kinds; a measured_by reference field resolving to declared ## Quantities rows of the same file; and a tolerance case-label floor — while the unit and domain columns stay opaque and files using none of this vocabulary lint byte-identically."
---

# numeric.predicates

The numeric-predicates standard pack: the second standard pack on the
domain-pack mechanism (`packs.md`). A workspace opts in by declaring a
typed `## Quantities` table and `measured_by` edges; unit systems (UCUM,
ISO4217, microscopy units, ...) stay authoritative — the pack bridges
them through opaque `unit`/`domain` columns, it never absorbs them into
any base closed set. Checkers ship as declarations (honest-empty): this
file declares vocabulary, the mechanism (`packs.md`) interprets it.
Vocabulary hygiene is pre-paid: no bare-English tokens are declared
(the `within`/`bound`/`against`/`unit`/`domain` audit — see the pack's
change design D2).

## Constraints

| id             | kind      | expr                                                                                                           | traces_to               |
|----------------|-----------|----------------------------------------------------------------------------------------------------------------|-------------------------|
| vocab_declared | invariant | `the six manifest tables below declare exactly the pack's vocabulary — no undeclared facet, no base-set growth` | [[numeric.predicates]]  |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from      | to         | guard                              |
|-----------|-----------|------------|------------------------------------|
| publish   | draft     | published  | `[[numeric.predicates.vocab_declared]] — pack_shape reports zero findings over the declared vocabulary` |
| deprecate | published | deprecated | `[[numeric.predicates.vocab_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id           | kind | derives_from                          | generator                          | predicate                         |
|--------------|------|---------------------------------------|------------------------------------|-----------------------------------|
| p_pack_shape | unit | [[numeric.predicates.vocab_declared]] | `pack_file_linted_in_each_state()` | `pack_shape reports zero findings` |

## Sections

| section    | row_shape |
|------------|-----------|
| Quantities | `\| name \| kind \| unit \| domain \|` |

## Kinds

| kind              | vocabulary                                    |
|-------------------|-----------------------------------------------|
| numeric.quantity  | `a measured or derived real-valued quantity`  |
| numeric.bound     | `a stated real-valued limit on a quantity`    |
| numeric.tolerance | `an empirically held tolerance on a quantity` |

## References

| field       | resolves_to                                 |
|-------------|---------------------------------------------|
| measured_by | `a ## Quantities row of the declaring file` |

## Checkers

| rule                    | semantics                                                                                                              |
|-------------------------|------------------------------------------------------------------------------------------------------------------------|
| numeric.quantity_closed | `every measured_by name resolves to a ## Quantities row declared in the same file; with no ## Quantities rows the checked-set is empty` |
| numeric.tolerance_labels | `numeric.tolerance laws enumerate their bound and against case labels per the pack's floor`                            |

## Floors

| kind              | required_cases   |
|-------------------|------------------|
| numeric.tolerance | `bound, against` |

## Requires

| dep  | revision                   |
|------|----------------------------|
| base | specodelic.md Revision 14 |
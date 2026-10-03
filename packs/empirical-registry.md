---
id: empirical.registry
kind: profile
checked_against_core: clear
statement: "WHEN a workspace enables the empirical-registry standard pack, THE format SHALL provide a namespaced ## StatTests section whose rows carry name, metric, alpha, and window; an empirical.statistic property kind; a tested_by reference field resolving to declared ## StatTests rows of the same file; and a per-kind case-label floor enumerating alpha and window on statistic properties — while the base law floor stays untouched and files using none of this vocabulary lint byte-identically."
---

# empirical.registry

The empirical-registry standard pack: the third standard pack on the
domain-pack mechanism (`packs.md`) and the Floors facet's first real
consumer — a floor driven by statistical practice (significance +
observation window), not algebra. A workspace opts in by declaring a
typed `## StatTests` registry and `empirical.statistic` properties;
statistical tests stay authoritative outside the format — the pack
declares the registry, the mechanism (`packs.md`) interprets it.
Checkers ship as declarations (honest-empty): this file declares
vocabulary, never prose rules.

## Constraints

| id             | kind      | expr                                                                                                                     | traces_to          |
|----------------|-----------|--------------------------------------------------------------------------------------------------------------------------|--------------------|
| vocab_declared | invariant | `the six manifest tables below declare exactly the pack's vocabulary — no undeclared facet, no base-set growth, floors purely additive` | [[empirical.registry]] |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from      | to         | guard                              |
|-----------|-----------|------------|------------------------------------|
| publish   | draft     | published  | `[[empirical.registry.vocab_declared]] — pack_shape reports zero findings over the declared vocabulary` |
| deprecate | published | deprecated | `[[empirical.registry.vocab_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id           | kind | derives_from                    | generator                          | predicate                         |
|--------------|------|---------------------------------|------------------------------------|-----------------------------------|
| p_pack_shape | unit | [[empirical.registry.vocab_declared]] | `pack_file_linted_in_each_state()` | `pack_shape reports zero findings` |

## Sections

| section   | row_shape                     |
|-----------|-------------------------------|
| StatTests | `\| name \| metric \| alpha \| window \|` |

## Kinds

| kind                 | vocabulary                                        |
|----------------------|---------------------------------------------------|
| empirical.statistic  | `an empirically held statistical claim, stated with a declared stat test` |

## References

| field       | resolves_to                              |
|-------------|------------------------------------------|
| tested_by   | `a ## StatTests row of the declaring file` |

## Checkers

| rule                        | semantics                                                                                              |
|-----------------------------|--------------------------------------------------------------------------------------------------------|
| empirical.statistic_labels  | `statistic properties enumerate their **alpha:** and **window:** case labels per the floor declaration` |
| empirical.stat_test_closed  | `every tested_by name resolves to a ## StatTests row declared in the same file; with no ## StatTests rows the checked-set is empty` |

## Floors

| kind                 | required_cases     |
|----------------------|--------------------|
| empirical.statistic  | `alpha, window`    |

## Requires

| dep  | revision                   |
|------|----------------------------|
| base | specodelic.md Revision 14 |

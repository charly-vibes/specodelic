---
id: data.lineage
kind: profile
checked_against_core: clear
statement: "WHEN a workspace enables the data/lineage standard pack, THE format SHALL provide a namespaced ## Data section whose rows carry name, kind, dtype, units, and binding; data.dataset, data.artifact, and data.environment kinds; produced_by, consumed_by, and produces reference fields resolving to declared ## Data rows of the same file; and a data.lineage.closure checker — while the binding column stays opaque and files using none of this vocabulary lint byte-identically."
---

# data.lineage

The data/lineage standard pack: the first standard pack on the
domain-pack mechanism (`packs.md`). A workspace opts in by declaring a
typed `## Data` table and lineage edges; external data standards
(OME/NGFF, Parquet, BioImage.IO, ...) stay authoritative — the pack
bridges them through an opaque `binding` pointer, it never absorbs
them into any base closed set. Checkers ship as declarations
(honest-empty): this file declares vocabulary, the mechanism
(`packs.md`) interprets it.

## Constraints

| id             | kind      | expr                                                                                                           | traces_to        |
|----------------|-----------|----------------------------------------------------------------------------------------------------------------|------------------|
| vocab_declared | invariant | `the six manifest tables below declare exactly the pack's vocabulary — no undeclared facet, no base-set growth` | [[data.lineage]] |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from      | to         | guard                              |
|-----------|-----------|------------|------------------------------------|
| publish   | draft     | published  | `[[data.lineage.vocab_declared]] — pack_shape reports zero findings over the declared vocabulary` |
| deprecate | published | deprecated | `[[data.lineage.vocab_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id           | kind | derives_from                    | generator                          | predicate                         |
|--------------|------|---------------------------------|------------------------------------|-----------------------------------|
| p_pack_shape | unit | [[data.lineage.vocab_declared]] | `pack_file_linted_in_each_state()` | `pack_shape reports zero findings` |

## Sections

| section | row_shape |
|---------|-----------|
| Data | `\| name \| kind \| dtype \| units \| binding \|` |

## Kinds

| kind             | vocabulary                                        |
|------------------|---------------------------------------------------|
| data.dataset     | `a bounded collection of measurements`            |
| data.artifact    | `a derived object a pipeline produces or consumes` |
| data.environment | `the execution or measurement context of a dataset or artifact` |

## References

| field       | resolves_to                              |
|-------------|------------------------------------------|
| produced_by | `a ## Data row of the declaring file`    |
| consumed_by | `a ## Data row of the declaring file`    |

## Checkers

| rule                 | semantics                                                                                              |
|----------------------|--------------------------------------------------------------------------------------------------------|
| data.lineage.closure | `every lineage reference name resolves to a ## Data row declared in the same file; with no ## Data rows the checked-set is empty` |

## Floors

| kind          | required_cases |
|---------------|----------------|
| data.artifact | `provenance`   |

## Requires

| dep  | revision                   |
|------|----------------------------|
| base | specodelic.md Revision 14 |

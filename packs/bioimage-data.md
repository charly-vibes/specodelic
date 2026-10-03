---
id: bioimage.data
kind: profile
checked_against_core: clear
statement: "WHEN a workspace enables the bioimage-data domain pack, THE format SHALL provide a namespaced ## Axes section whose rows carry name, axis, scale, and unit; a bioimage.transform property kind typeable in base-table kind columns when the pack is active; a same_shape_as reference field resolving to declared ## Data rows of the same file; the data-shaped predicate grammar declared as named pack-qualified checkers (bioimage.dtype_is, bioimage.shape_eq, bioimage.units_convertible) resolving against declared ## Data rows; and a kind-dependent case-label floor enumerating preserves and dtype on transform properties — while OME/NGFF and BioImage.IO stay authoritative external standards, the base law floor stays untouched, and files using none of this vocabulary lint byte-identically."
---

# bioimage.data

The bioimage-data domain pack: the D6 pilot on the domain-pack mechanism
(`packs.md`) — the thin instance that consumes the three standard packs'
vocabulary rather than redeclaring any of it, and the mechanism's first
cross-pack `## Requires` consumer. A workspace opts in by declaring a
typed `## Axes` table, `## Data` rows (via `data.lineage`), and
`bioimage.transform` laws; OME/NGFF and BioImage.IO stay authoritative —
the pack bridges them through opaque `scale`/`unit` columns, it never
absorbs them into any closed set. Checkers ship as declarations
(honest-empty): this file declares vocabulary, the mechanism (`packs.md`)
interprets it. Vocabulary hygiene is pre-paid: no bare-English tokens are
declared (the `bioimage`/`pipeline`/`dtype` audit — see the pack's change
design D2); the predicate names ride pack-qualified checker rule names.

## Constraints

| id             | kind      | expr                                                                                                           | traces_to        |
|----------------|-----------|----------------------------------------------------------------------------------------------------------------|------------------|
| vocab_declared | invariant | `the six manifest tables below declare exactly the pack's vocabulary — no undeclared facet, no base-set growth, floors purely additive` | [[bioimage.data]] |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from      | to         | guard                              |
|-----------|-----------|------------|------------------------------------|
| publish   | draft     | published  | `[[bioimage.data.vocab_declared]] — pack_shape reports zero findings over the declared vocabulary` |
| deprecate | published | deprecated | `[[bioimage.data.vocab_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id           | kind | derives_from                    | generator                          | predicate                         |
|--------------|------|---------------------------------|------------------------------------|-----------------------------------|
| p_pack_shape | unit | [[bioimage.data.vocab_declared]] | `pack_file_linted_in_each_state()` | `pack_shape reports zero findings` |

## Sections

| section | row_shape                     |
|---------|-------------------------------|
| Axes    | `\| name \| axis \| scale \| unit \|` |

## Kinds

| kind               | vocabulary                                        |
|--------------------|---------------------------------------------------|
| bioimage.transform | `a scientific image-transform law: a stated property about how a pipeline step transforms image data` |

## References

| field         | resolves_to                              |
|---------------|------------------------------------------|
| same_shape_as | `a ## Data row of the declaring file`    |

## Checkers

| rule                       | semantics                                                                                              |
|----------------------------|--------------------------------------------------------------------------------------------------------|
| bioimage.dtype_is          | `every dtype_is predicate occurrence resolves against a declared ## Data row's dtype column of the same file; with no ## Data rows the checked-set is empty — the mechanism names this predicate, never executes it` |
| bioimage.shape_eq          | `every shape_eq predicate occurrence resolves against a declared ## Data row of the same file; with no ## Data rows the checked-set is empty — the mechanism names this predicate, never executes it` |
| bioimage.units_convertible | `every units_convertible predicate occurrence bridges the numeric.predicates pack's opaque unit domain; with no ## Quantities rows the checked-set is empty — the mechanism names this predicate, never executes it` |
| bioimage.same_shape_closed | `every same_shape_as name resolves to a ## Data row declared in the same file; with no ## Data rows the checked-set is empty` |

## Floors

| kind               | required_cases     |
|--------------------|--------------------|
| bioimage.transform | `preserves, dtype` |

## Requires

| dep                 | revision                   |
|---------------------|----------------------------|
| base                | specodelic.md Revision 14 |
| data.lineage        | specodelic.md Revision 14 |
| numeric.predicates  | specodelic.md Revision 14 |
| empirical.registry  | specodelic.md Revision 14 |

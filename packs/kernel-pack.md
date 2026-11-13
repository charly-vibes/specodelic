---
id: kernel.pack
kind: profile
checked_against_core: clear
statement: "WHEN a workspace enables the kernel vocabulary pack, THE format SHALL provide the kernel.binding reference field as pack-qualified vocabulary, a kernel.predicate fiber kind typeable in base-table kind columns while the pack is active, a KernelPredicates section for pack-side predicates, the kernel.binding_opaque and kernel.predicate_grounded checkers, and a kernel.predicate identity case-label floor — while the binding contents stay opaque to the toolchain and files using none of this vocabulary lint byte-identically."
---

# kernel.pack

The kernel vocabulary pack: the declared extension surface for the
`kernel.binding` claim carrier (`openspec/specs/kernel/spec.md`) and for
the pack-side predicates the kernel's widening law keeps out of its
closed atomic set. A workspace opts in by using the pack's vocabulary —
the `kernel.binding` column on its Constraints table, a `kernel.predicate`
row, a `## KernelPredicates` section — and the pack is discovered by
corpus scan alone: no config file, no registry, no carve-out. The
mechanism (`openspec/specs/packs/spec.md`) interprets this file; this
file only declares.

Two laws ride the declaration. The binding stays **opaque**: the pack
declares the column, the toolchain never parses, validates, or executes
its contents, and no registry is built. The predicates stay
**pack-side**: a `kernel.predicate` row is checked by the external runner
it names, never by the kernel's closed atomic set — the widening law's
decidability gate stays in force, and an undecidable predicate simply
does not register. Checkers ship as declarations (honest-empty): with no
`kernel.binding` cells and no `kernel.predicate` rows, the checked-set is
empty and nothing is fabricated.

Vocabulary hygiene is pre-paid: the declared tokens are dotted
(`kernel.binding`, `kernel.predicate`) or distinctive
(`KernelPredicates`) — no bare English word that appears in corpus prose
becomes vocabulary, so files using none of this vocabulary lint
byte-identically with the pack discovered.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                          | traces_to      |
|--------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------|
| process_lifecycle        | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds`                                                       | [[kernel.pack]] |
| vocab_declared           | invariant | `the six manifest tables below declare exactly the pack's vocabulary — the KernelPredicates section, the kernel.predicate kind, the kernel.binding reference field, the kernel.binding_opaque and kernel.predicate_grounded checkers, the kernel.predicate identity floor, and the specodelic.md Revision 20 base pin — no undeclared facet, no base-set growth, floors purely additive` | [[kernel.pack]] |
| binding_stays_opaque     | invariant | `the kernel.binding column stays an opaque claim carrier under this pack — the pack declares the vocabulary, the toolchain surfaces the cell's contents verbatim and never interprets them, and no registry is built`     | [[kernel.pack]] |
| predicates_stay_pack_side | invariant | `a kernel.predicate row is checked by the external runner it names and never by the kernel's closed atomic set — the pack declares the vocabulary, and the widening law's decidability gate stays in force`                | [[kernel.pack]] |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from      | to         | guard                                                                                                                        |
|-----------|-----------|------------|------------------------------------------------------------------------------------------------------------------------------|
| publish   | draft     | published  | `[[kernel.pack.vocab_declared]] — pack_shape reports zero findings over the declared vocabulary`                              |
| deprecate | published | deprecated | `[[kernel.pack.vocab_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id                          | kind | derives_from                              | generator                                      | predicate                                                                                             |
|-----------------------------|------|-------------------------------------------|------------------------------------------------|---------------------------------------------------------------------------------------------------------|
| process_lifecycle_checked   | unit | [[kernel.pack.process_lifecycle]]         | `lifecycle_model_present()`                    | `check(file) == passed`                                                                                |
| pack_shape_clean_all_states | unit | [[kernel.pack.vocab_declared]]             | `pack_file_linted_in_each_lifecycle_state()`   | `pack_shape reports zero findings over the declared vocabulary in draft, published, and deprecated`   |
| binding_never_interpreted   | unit | [[kernel.pack.binding_stays_opaque]]       | `constraint_with_arbitrary_binding_text()`    | `the binding cell contents surface verbatim; no parse, no interpretation, no registry`                 |
| predicates_stay_external    | unit | [[kernel.pack.predicates_stay_pack_side]]  | `kernel_predicate_row_with_external_runner()` | `the row names its external runner and never enters the kernel's closed atomic set`                   |

## Sections

| section          | row_shape                          |
|------------------|------------------------------------|
| KernelPredicates | `\| name \| predicate \| runner \|` |

## Kinds

| kind             | vocabulary                                                                                                     |
|------------------|---------------------------------------------------------------------------------------------------------------|
| kernel.predicate | `a pack-side kernel predicate: a claim checked by an external contract-TOML runner, never by the kernel's closed atomic set` |

## References

| field          | resolves_to                                                                                     |
|----------------|--------------------------------------------------------------------------------------------------|
| kernel.binding | `the opaque per-constraint binding claim carrier — the cell's contents surface verbatim and are never interpreted` |

## Checkers

| rule                      | semantics                                                                                                                                   |
|---------------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| kernel.binding_opaque     | `every kernel.binding cell is surfaced verbatim and never parsed, validated, or executed; with no kernel.binding cells the checked-set is empty` |
| kernel.predicate_grounded | `every kernel.predicate row names the external runner that checks it; with no kernel.predicate rows the checked-set is empty`                   |

## Floors

| kind             | required_cases |
|------------------|----------------|
| kernel.predicate | `identity`     |

## Requires

| dep  | revision                   |
|------|----------------------------|
| base | specodelic.md Revision 20 |

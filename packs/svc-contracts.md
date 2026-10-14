---
id: svc.contracts
kind: profile
checked_against_core: clear
statement: "WHEN a workspace enables the svc-contracts pack, THE format SHALL provide a namespaced ## Interfaces section whose rows carry name, kind, repo, binding, and evolution; a svc.compat kind; a published_as reference field resolving to a declared ## Interfaces row of the same file; the svc.interface_closed and svc.compat_cases pack-qualified checkers; and a kind-dependent compatibility floor — while the binding column stays opaque, no provider-to-consumer field is added, and files using none of this vocabulary lint byte-identically."
---

# svc.contracts

The svc-contracts standard pack (DRAFT, proposed by the
`add-svc-contracts-pack` change): vocabulary for services whose published
contracts live in different repositories. The base format already carries
the cross-file edges — `extension_point` + `satisfies` for a request-style
contract a consumer conforms to, `effect` + `observes` for an event a
consumer watches (`openspec/specs/specodelic/spec.md` Reference Typing; `USAGE.md`
§2.6) — so this pack adds none. It declares only what the base lacks: a
typed registry row that ties a published contract to its owning repo and
to the authoritative external artifact (an OpenAPI, AsyncAPI, protobuf, or
JSON Schema file) through an opaque `binding` column, a declared evolution
mode, and a compatibility law kind whose floor forces the two cases that
decide whether existing consumers keep working: what is added and what is
removed. The pack never lists a contract's consumers: `USAGE.md` §2.6
says a publisher must not enumerate its conformers, so provider-to-consumer
direction stays derived from the graph, never authored. The `binding` and
`repo` columns are opaque strings — the pack bridges to external standards
and to repo identity, it never parses or absorbs them. The `kind` column
of an `## Interfaces` row is a short label closed per-pack to `request`,
`event`, `schema`; the `evolution` column is closed per-pack to
`additive`, `versioned`, `frozen` (the mechanism never checks row-kind
cells; these closures are per-pack, so the base kind sets stay untouched).
Checkers ship as declarations (honest-empty): this file declares
vocabulary, the mechanism (`packs.md`) interprets it.

## Constraints

| id             | kind      | expr                                                                                                                                    | traces_to        |
|----------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------|------------------|
| vocab_declared | invariant | `the six manifest tables below declare exactly the pack's vocabulary — no undeclared facet, no base-set growth, floors purely additive` | [[svc.contracts]] |

## Model

### States
- `draft`

### Transitions

| id      | from  | to    | guard                                                                                 |
|---------|-------|-------|---------------------------------------------------------------------------------------|
| declare | draft | draft | `[[svc.contracts.vocab_declared]] — pack_shape reports zero findings over the declared vocabulary` |

## Properties

| id           | kind | derives_from                    | generator                          | predicate                          |
|--------------|------|---------------------------------|------------------------------------|------------------------------------|
| p_pack_shape | unit | [[svc.contracts.vocab_declared]] | `pack_file_linted_in_each_state()` | `pack_shape reports zero findings` |

## Sections

| section    | row_shape                                         |
|------------|---------------------------------------------------|
| Interfaces | `\| name \| kind \| repo \| binding \| evolution \|` |

## Kinds

| kind       | vocabulary                                                                                                                         |
|------------|------------------------------------------------------------------------------------------------------------------------------------|
| svc.compat | `a compatibility law over a published interface: for each change case it states whether existing consumers keep working`           |

## References

| field        | resolves_to                                   |
|--------------|-----------------------------------------------|
| published_as | `an ## Interfaces row of the declaring file`  |

## Checkers

| rule                 | semantics                                                                                                                                                                              |
|----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| svc.interface_closed | `every published_as name resolves to an ## Interfaces row declared in the same file; with no ## Interfaces rows the checked-set is empty — the mechanism names this rule, never executes it` |
| svc.compat_cases     | `svc.compat laws enumerate their added and removed case labels per the pack's floor — the mechanism names this rule, never executes it`                                                  |

## Floors

| kind       | required_cases   |
|------------|------------------|
| svc.compat | `added, removed` |

## Requires

| dep  | revision                  |
|------|---------------------------|
| base | specodelic.md Revision 18 |

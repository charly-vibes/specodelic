---
id: quant.finance
kind: profile
checked_against_core: clear
statement: "WHEN a workspace enables the quant-finance standard pack, THE format SHALL provide a namespaced ## Limits section whose rows carry name, kind, unit, and bound; quant.risk and quant.pricing kinds; a capped_by reference field resolving to declared ## Limits rows of the same file; the limit-closure and risk-label pack-qualified checkers (quant.limit_closed, quant.risk_labels); and a kind-dependent case-label floor enumerating horizon and confidence on risk laws — while ISO4217 and market-data conventions stay authoritative external standards, the base law floor stays untouched, and files using none of this vocabulary lint byte-identically."
---

# quant.finance

The quant-finance standard pack: the D6 second domain pack on the
domain-pack mechanism (`packs.md`) — the restraint instance. Most of what
the vendor designs asked for already ships in the standard packs: tolerance
semantics are `numeric.tolerance` rows (`numeric.predicates`), statistics
are `empirical.statistic` rows (`empirical.registry`), and backtest
fixtures and datasets are `## Data` rows (`data.lineage`). This pack
declares only the vocabulary no standard pack carries: a typed `## Limits`
section for bounded risk measures and pricing invariants, the
`quant.risk`/`quant.pricing` kinds, and the `capped_by` outbound-leaf edge.
Unit systems (ISO4217, bps, ...) stay authoritative — the pack bridges them
through the opaque `unit` column, it never absorbs them into any closed
set. `## Limits` rows carry a short kind label closed to `risk`, `pricing`
— the row-level form of the pack's `quant.risk`/`quant.pricing` kinds
(the mechanism never checks row-kind cells; the closure is per-pack, so
the base kind sets stay untouched). Checkers ship as declarations
(honest-empty): this file declares vocabulary, the mechanism (`packs.md`)
interprets it. Vocabulary hygiene is pre-paid: no bare-English tokens are
declared (the `limit`/`confidence`/`Fixtures` audit — see the pack's
change design D2); `horizon`/`confidence` survive only as floor case
labels, which the activation scanner never reads.

## Constraints

| id             | kind      | expr                                                                                                                                   | traces_to        |
|----------------|-----------|----------------------------------------------------------------------------------------------------------------------------------------|------------------|
| vocab_declared | invariant | `the six manifest tables below declare exactly the pack's vocabulary — no undeclared facet, no base-set growth, floors purely additive` | [[quant.finance]] |

## Model

### States
- `draft`
- `published`
- `deprecated`

### Transitions

| id        | from      | to         | guard                              |
|-----------|-----------|------------|------------------------------------|
| publish   | draft     | published  | `[[quant.finance.vocab_declared]] — pack_shape reports zero findings over the declared vocabulary` |
| deprecate | published | deprecated | `[[quant.finance.vocab_declared]] — maintainer marks the pack deprecated; findings name the deprecation, vocabulary still checks` |

## Properties

| id           | kind | derives_from                     | generator                          | predicate                          |
|--------------|------|----------------------------------|------------------------------------|------------------------------------|
| p_pack_shape | unit | [[quant.finance.vocab_declared]] | `pack_file_linted_in_each_state()` | `pack_shape reports zero findings` |

## Sections

| section | row_shape                     |
|---------|-------------------------------|
| Limits  | `\| name \| kind \| unit \| bound \|` |

## Kinds

| kind          | vocabulary                                                                                          |
|---------------|-----------------------------------------------------------------------------------------------------|
| quant.risk    | `a bounded risk measure: a VaR, expected shortfall, drawdown, or exposure cap stated with its bound` |
| quant.pricing | `a valuation or pricing invariant: a no-arbitrage, put-call parity, or NAV consistency claim`        |

## References

| field     | resolves_to                              |
|-----------|------------------------------------------|
| capped_by | `a ## Limits row of the declaring file`  |

## Checkers

| rule               | semantics                                                                                                                                                                      |
|--------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| quant.limit_closed | `every capped_by name resolves to a ## Limits row declared in the same file; with no ## Limits rows the checked-set is empty — the mechanism names this rule, never executes it` |
| quant.risk_labels  | `quant.risk laws enumerate their horizon and confidence case labels per the pack's floor — the mechanism names this rule, never executes it`                                    |

## Floors

| kind       | required_cases        |
|------------|-----------------------|
| quant.risk | `horizon, confidence` |

## Requires

| dep                | revision                   |
|--------------------|----------------------------|
| base               | specodelic.md Revision 18  |
| numeric.predicates | specodelic.md Revision 14  |
| data.lineage       | specodelic.md Revision 14  |

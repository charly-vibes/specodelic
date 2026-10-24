---
id: preservation.layer
kind: intent
statement: "WHEN a spec file is revised, THE preserve command SHALL classify each oracle scenario's conformance verdict across the before-and-after compiled models as preserved, intentionally_changed, undeclared_change, or unchecked, carrying corpus-level evidence-scope labeling and never claiming behavioral equality."
---

# preservation — dual-format layer (sidecar)

## Purpose
Give spec revisions the regression-detection half of the red→green→refactor
discipline: the same oracle corpus is walked against both compiled models and
verdict classes are diffed, with honest scope labeling derived from conform.

## Constraints

| id                  | kind      | expr                                                                                                                                        | traces_to |
|---------------------|-----------|----------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| reuse_walk          | invariant | `classification reuses conform's walk and verdict taxonomy unchanged — no second trace evaluator is introduced`                               | [[preservation.layer]] |
| total_classification | invariant | `∀ scenario in the corpus: the report assigns exactly one class of preserved, intentionally_changed, undeclared_change, unchecked — never none, never two` | [[preservation.layer]] |
| undeclared_is_finding | invariant | `a verdict-class difference with no oracle-record declaration is a finding naming both verdicts — a silent class change is impossible`      | [[preservation.layer]] |
| scope_labeled       | invariant | `every report and record carries the evidence-scope statement: agreement on the supplied corpus, never a proof of behavioral equality`        | [[preservation.layer]] |
| corpus_digest_bound | invariant | `both runs report the oracle corpus digest, and a digest mismatch fails labeled before any classification is emitted`                         | [[preservation.layer]] |
| gate_reused         | invariant | `the command reuses conform's artifact gate — refusing stale or uncompiled inputs labeled, compiling nothing, writing nothing, advancing no lifecycle transition` | [[preservation.layer]] |

## Model

### States
- `gated`
- `walked`
- `reported`

### Transitions

| id       | from     | to       | guard                                    |
|----------|----------|----------|------------------------------------------|
| walk     | gated    | walked   | [[preservation.layer.gate_reused]]       |
| classify | walked   | reported | [[preservation.layer.total_classification]] ∧ [[preservation.layer.corpus_digest_bound]] |

## Properties

| id            | kind | derives_from                          | generator            | predicate                                        |
|---------------|------|---------------------------------------|----------------------|--------------------------------------------------|
| p_total       | unit | [[preservation.layer.total_classification]] | `arbitrary_corpus_pair()` | `report covers each scenario id exactly once`   |
| p_scope       | unit | [[preservation.layer.scope_labeled]]  | `arbitrary_report()` | `scope statement present in records ∧ header`    |
| p_gate        | unit | [[preservation.layer.gate_reused]]    | `arbitrary_stale_input()` | `stale input ⇒ labeled refusal ∧ zero verdict records` |
| p_walk_reuse  | unit | [[preservation.layer.reuse_walk]]     | `arbitrary_classification_run()` | `classification outcomes match a conform-walk oracle reference run` |
| p_finding     | unit | [[preservation.layer.undeclared_is_finding]] | `arbitrary_verdict_pair()` | `differing verdicts ∧ no declaration ⇒ finding record naming both verdicts` |
| p_digest      | unit | [[preservation.layer.corpus_digest_bound]] | `arbitrary_corpus_pair()` | `digest match ⇒ classification emitted; digest mismatch ⇒ labeled failure ∧ zero records` |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `preservation` capability and
SHALL be deployed verbatim by the archive companion; the plain deltas in
`spec.md` beside it are its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** `just archive-change id=add-preservation-check` runs after the
  change is implemented
- **THEN** the capability deploys with this layer attached fail-closed

---
id: audit.layer
kind: intent
statement: "WHEN a lint-clean spec file with current compiled artifacts is audited against a declared evidence corpus and an optional prior report, THE audit capability SHALL classify every evidence row with exactly one class from the closed set {current, stale, unchecked, contradicted, binds_nothing, new} — reusing conform's walk machinery and artifact gate unchanged — and SHALL report the highest evidence-backed ladder rung, never treating unchecked evidence as verified and never claiming behavioral equality."
---

# audit — dual-format layer (sidecar)

## Purpose
Compute the graded-evidence discipline the toolchain already names (the
ladder, evidence_scope, exploration_only) uniformly over everything that
claims to constrain behavior: recorded example corpora, pack checker
results, fragment/claim bindings, and prior audit reports.

## Constraints

| id                | kind      | expr                                                                                                                                         | traces_to |
|-------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| reuse_walk        | invariant | `classification reuses conform's walk machinery and artifact gate unchanged — a second trace evaluator is never introduced`                      | [[audit.layer]] |
| closed_classes    | invariant | `∀ evidence row: exactly one audit class from {current, stale, unchecked, contradicted, binds_nothing, new}, each record also carrying the underlying conform verdict with its reason — none or two is impossible` | [[audit.layer]] |
| digest_bound_diff | invariant | `--before diffing refuses labeled on a scope-digest mismatch between reports, with zero classification emitted — cross-corpus-version diffs are impossible` | [[audit.layer]] |
| honesty_channels  | invariant | `unchecked and binds_nothing ride the warnings channel and never affect the exit code — unchecked evidence is countable, never a pass, and never a failure` | [[audit.layer]] |
| binds_respect_sparse | invariant | `binds_nothing requires that the fragment or claim cites no declared model element — legitimately sparse models with declared elements are never flagged` | [[audit.layer]] |
| rung_derived      | invariant | `the summary's ladder rung is derived at report time from evidence actually present and never exceeds it — a rung without its evidence is impossible` | [[audit.layer]] |
| read_only         | invariant | `audit compiles nothing, writes no artifacts, and advances no lifecycle transition — refusal on ungated inputs is labeled with a remediation hint` | [[audit.layer]] |

## Model

### States
- `gated`
- `classified`
- `diffed`
- `reported`

### Transitions

| id        | from       | to         | guard                                                        |
|-----------|------------|------------|--------------------------------------------------------------|
| classify  | gated      | classified | [[audit.layer.reuse_walk]] ∧ [[audit.layer.closed_classes]]   |
| diff      | classified | diffed     | [[audit.layer.digest_bound_diff]]                             |
| report    | diffed     | reported   | [[audit.layer.rung_derived]] ∧ [[audit.layer.honesty_channels]] ∧ [[audit.layer.read_only]] |

## Properties

| id            | kind | derives_from                       | generator                    | predicate                                                    |
|---------------|------|-------------------------------------|------------------------------|--------------------------------------------------------------|
| p_classes     | unit | [[audit.layer.closed_classes]]      | `arbitrary_evidence_corpus()` | `every row carries exactly one class ∧ an underlying verdict` |
| p_diff        | unit | [[audit.layer.digest_bound_diff]]   | `arbitrary_report_pair()`    | `digest match ⇒ diff; mismatch ⇒ labeled refusal ∧ zero rows` |
| p_honesty     | unit | [[audit.layer.honesty_channels]]    | `arbitrary_row_mix()`        | `unchecked/binds_nothing ⇒ exit 0 ∧ counts surfaced`          |
| p_sparse      | unit | [[audit.layer.binds_respect_sparse]] | `sparse_phase_model()`      | `declared-element model ⇒ no binds_nothing flag`              |
| p_rung        | unit | [[audit.layer.rung_derived]]        | `arbitrary_evidence_mix()`   | `rung == highest evidence-backed rung ∧ never exceeds it`     |
| p_read_only   | unit | [[audit.layer.read_only]]           | `ungated_input()`            | `labeled refusal ∧ corpus byte-identical ∧ zero records`      |
| p_walk_reuse  | unit | [[audit.layer.reuse_walk]]          | `arbitrary_classification_run()` | `audit classes match a conform-walk oracle reference run` |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `audit` capability and SHALL
be deployed verbatim by the archive companion; the plain deltas in
`spec.md` beside it are its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** `just archive-change id=add-evidence-audit` runs after the change
  is implemented
- **THEN** the capability deploys with this layer attached fail-closed

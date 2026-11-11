---
id: audit
kind: intent
statement: "WHEN a lint-clean spec file with current compiled artifacts is audited against a declared evidence corpus and an optional prior report, THE audit capability SHALL classify every evidence row with exactly one class from the closed set {current, stale, unchecked, contradicted, binds_nothing, new} — reusing conform's walk machinery and artifact gate unchanged — and SHALL report the highest evidence-backed ladder rung, never treating unchecked evidence as verified and never claiming behavioral equality."
---

# audit Specification

## Purpose
Compute the graded-evidence discipline the toolchain already names (the
ladder, evidence_scope, exploration_only) uniformly over everything that
claims to constrain behavior: recorded example corpora, pack checker
results, fragment/claim bindings, and prior audit reports.

## Constraints

| id                | kind      | expr                                                                                                                                         | traces_to |
|-------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| reuse_walk        | invariant | `classification reuses conform's walk machinery and artifact gate unchanged — a second trace evaluator is never introduced`                      | [[audit]] |
| closed_classes    | invariant | `∀ evidence row: exactly one audit class from {current, stale, unchecked, contradicted, binds_nothing, new}, each record also carrying the underlying conform verdict with its reason — none or two is impossible` | [[audit]] |
| digest_bound_diff | invariant | `--before diffing refuses labeled on a scope-digest mismatch between reports, with zero classification emitted — cross-corpus-version diffs are impossible` | [[audit]] |
| honesty_channels  | invariant | `unchecked and binds_nothing ride the warnings channel and never affect the exit code — unchecked evidence is countable, never a pass, and never a failure` | [[audit]] |
| binds_respect_sparse | invariant | `binds_nothing requires that the fragment or claim cites no declared model element — legitimately sparse models with declared elements are never flagged` | [[audit]] |
| rung_derived      | invariant | `the summary's ladder rung is derived at report time from evidence actually present and never exceeds it — a rung without its evidence is impossible` | [[audit]] |
| read_only         | invariant | `audit compiles nothing, writes no artifacts, and advances no lifecycle transition — refusal on ungated inputs is labeled with a remediation hint` | [[audit]] |

## Model

### States
- `gated`
- `classified`
- `diffed`
- `reported`

### Transitions

| id        | from       | to         | guard                                                        |
|-----------|------------|------------|--------------------------------------------------------------|
| classify  | gated      | classified | [[audit.reuse_walk]] ∧ [[audit.closed_classes]]   |
| diff      | classified | diffed     | [[audit.digest_bound_diff]]                             |
| report    | diffed     | reported   | [[audit.rung_derived]] ∧ [[audit.honesty_channels]] ∧ [[audit.read_only]] |

## Properties

| id            | kind | derives_from                       | generator                    | predicate                                                    |
|---------------|------|-------------------------------------|------------------------------|--------------------------------------------------------------|
| p_classes     | unit | [[audit.closed_classes]]      | `arbitrary_evidence_corpus()` | `every row carries exactly one class ∧ an underlying verdict` |
| p_diff        | unit | [[audit.digest_bound_diff]]   | `arbitrary_report_pair()`    | `digest match ⇒ diff; mismatch ⇒ labeled refusal ∧ zero rows` |
| p_honesty     | unit | [[audit.honesty_channels]]    | `arbitrary_row_mix()`        | `unchecked/binds_nothing ⇒ exit 0 ∧ counts surfaced`          |
| p_sparse      | unit | [[audit.binds_respect_sparse]] | `sparse_phase_model()`      | `declared-element model ⇒ no binds_nothing flag`              |
| p_rung        | unit | [[audit.rung_derived]]        | `arbitrary_evidence_mix()`   | `rung == highest evidence-backed rung ∧ never exceeds it`     |
| p_read_only   | unit | [[audit.read_only]]           | `ungated_input()`            | `labeled refusal ∧ corpus byte-identical ∧ zero records`      |
| p_walk_reuse  | unit | [[audit.reuse_walk]]          | `arbitrary_classification_run()` | `audit classes match a conform-walk oracle reference run` |

## Requirements

### Requirement: Evidence row classification

`spk audit <file> --oracle <corpus.jsonl>` SHALL classify every oracle
record and pack-check result against the current compiled model with exactly
one audit class from the closed set `{current, unchecked, contradicted,
binds_nothing, stale, new}`, each record carrying both the audit class and
the underlying conform verdict with its reason, reusing conform's walk
machinery and artifact gate unchanged.

#### Scenario: Current evidence agrees

- **WHEN** a record's verdict class matches its class in the prior report
  (or no prior report exists and the walk classifies it)
- **THEN** the row is `current` and names the underlying verdict

#### Scenario: Contradicted evidence fails the run

- **WHEN** the walk classifies a record `forbidden` or `unsupported`
- **THEN** the row is `contradicted` and the run exits 1 per the exit-code
  requirement — never a silent pass

### Requirement: Evidence diff across reports

`spk audit --before <report.json>` SHALL compare per-row verdict classes
against the prior report, classifying dropped or worsened rows as `stale`,
unseen rows as `new`, and refusing labeled when the two reports carry
different scope digests — never silently diffing across corpus versions.

#### Scenario: Dropped class is stale

- **WHEN** a record was `current` in the prior report and is `unchecked` or
  worse in the current run
- **THEN** the row is `stale`, naming both classes

#### Scenario: Corpus-version mismatch refused

- **WHEN** the prior and current reports carry different scope digests
- **THEN** the run fails labeled naming the digest mismatch, with no
  classification emitted

### Requirement: Audit honesty channels

Audit SHALL treat `unchecked` and `binds_nothing` as warnings-channel
findings that never affect the exit code, and `binds_nothing` SHALL require
that the fragment or claim cites no declared model element — legitimately
sparse models with declared elements are never flagged.

#### Scenario: Empty checked-set is unchecked

- **WHEN** a pack checker runs over vocabulary it activates but produces an
  empty checked-set
- **THEN** the row is `unchecked`, countable in the summary — never a pass

#### Scenario: Unbound evidence flagged, sparsity respected

- **WHEN** a fragment cites no declared model element
- **THEN** the row is `binds_nothing` on the warnings channel; a model whose
  declared elements are simply not exercised stays unflagged

#### Scenario: Unchecked never fails the run

- **WHEN** every row is `unchecked` or `binds_nothing`
- **THEN** the exit code is 0 with counts surfaced in both output views

### Requirement: Evidence rung aggregate

Audit summaries SHALL report, per file, the highest verification-ladder rung
for which `current` evidence exists, derived at report time from existing
outputs — never exceeding the evidence actually present.

#### Scenario: Rung matches evidence

- **WHEN** a file is lint-clean but only model-checked, with no verify pass
- **THEN** its reported rung is 3, and never 4 or 5

#### Scenario: Rung never claims without evidence

- **WHEN** no audit, conform, or verify evidence exists for a file
- **THEN** its reported rung is 1 (lint-clean at most), stated as such

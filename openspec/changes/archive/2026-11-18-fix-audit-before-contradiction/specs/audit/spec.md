# audit Specification

## Purpose

Compute the graded-evidence discipline the toolchain already names (the
ladder, evidence_scope, exploration_only) uniformly over everything that
claims to constrain behavior: recorded example corpora, pack checker
results, fragment/claim bindings, and prior audit reports.

## MODIFIED Requirements

### Requirement: Evidence diff across reports

`spk audit --before <report.json>` SHALL compare per-row verdict classes
against the prior report — dropped or worsened ⇒ `stale`, unseen ⇒ `new` —
refusing labeled when the two reports carry different scope digests, and
SHALL never retire a contradiction: a row `contradicted` in the current run
stays `contradicted` whatever the prior report said about it, contradiction
outranking every currency label.

#### Scenario: Dropped class is stale

- **WHEN** a record was `current` in the prior report and is `unchecked` in
  the current run
- **THEN** the row is `stale`, naming both classes

#### Scenario: Sustained contradiction is never demoted

- **WHEN** a record is `contradicted` in the prior report and `contradicted`
  in the current run against the same corpus
- **THEN** the diffed row stays `contradicted` — never `current` — the
  class counts include it, and the run exits 1

#### Scenario: Worsening into a contradiction stays contradicted

- **WHEN** a record was `current` in the prior report and is `contradicted`
  in the current run
- **THEN** the row is `contradicted`, never `stale`, and the run exits 1 —
  the currency label never displaces live contradictory evidence

#### Scenario: Unseen contradiction is never relabelled new

- **WHEN** a record is `contradicted` in the current run and the prior
  report carries no row for it
- **THEN** the diffed row is `contradicted`, not `new`, and the run exits 1

#### Scenario: Corpus-version mismatch refused

- **WHEN** the prior and current reports carry different scope digests
- **THEN** the run fails labeled naming the digest mismatch, with no
  classification emitted
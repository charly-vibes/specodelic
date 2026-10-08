# Update: id derivation — delta for verification-claims

## MODIFIED Requirements

### Requirement: Evidence is versioned and scope bound
A stored report SHALL declare claim schema version 1, canonical expected
claim IDs, actual statuses, and a deterministic digest of the structured
corpus and consumed artifacts. Verify SHALL recompute scope and expected
claims from current inputs. Unsupported or old reports, omitted or duplicate
records, changed contributing inputs, or artifact mismatch SHALL fail with a
rerun hint. Reordering CLI paths or changing prose alone SHALL preserve scope.
Reordering CLI paths or changing prose alone SHALL preserve scope.
The intent IDs of a command's parsed inputs SHALL be unique: a combined
invocation whose inputs claim the same intent id SHALL fail with
duplicate_corpus_identity and a rename hint before compilation,
evaluation or report writes (Revision 18: dual-format files carry real
ids and compose like any corpus — the former `id: spec` isolated-scope
rule retired with `id: spec` itself). The digest SHALL bind structured
content, not filesystem paths, so opposite claims with identical names
cannot reuse each other's evidence.

#### Scenario: Cross-file changes invalidate a clean report
- **WHEN** a referenced file changes structurally or is omitted from the supplied corpus after the report was produced
- **THEN** verify rejects the report as stale or scope-mismatched and names the rerun command


#### Scenario: Cross-file changes invalidate a clean report
- **WHEN** a referenced file changes structurally or is omitted from the supplied corpus after the report was produced
- **THEN** verify rejects the report as stale or scope-mismatched and names the rerun command

#### Scenario: A same-id pair is refused before any writes
- **WHEN** two parsed inputs claim the same intent id (the transitional pair: an active change's delta and its deployed capability spec)
- **THEN** model-check, verify and orchestrate refuse the combined scope as duplicate_corpus_identity before writing artifacts or reports, with rename guidance; independent runs in separate directories retain each claim's own status, produce different scope digests and reject swapped reports, while lint accepts files with distinct ids together

#### Scenario: Old reports cannot acquire new evidence implicitly
- **WHEN** a stored report has no claim schema version or required records
- **THEN** verify rejects it with a model-check rerun hint without modifying the report

#### Scenario: Old reports cannot acquire new evidence implicitly
- **WHEN** a stored report has no claim schema version or required records
- **THEN** verify rejects it with a model-check rerun hint without modifying the report

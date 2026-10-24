# Change: Spec revision preservation checks via oracle diffing

## Why

The catitas kata studies (Claude bundle v3, O4; mistral mandate §3D) identified
behavior-preservation between spec revisions as the single biggest TDD
carry-over gap: `spk refactor` reported 0 split candidates even where 1,792
transition guards cite a single constraint, and nothing tells an author whether
a revised model still explains the examples the previous model explained.
Red→green→refactor presumes regression detection; spec revisions have none.

## What Changes

- New command `spk preserve --before <dir> --after <dir> --oracle <corpus.jsonl>`
  that runs the **existing** conform walk machinery over both models against the
  same oracle corpus and diffs the verdicts per scenario id.
- Classification per scenario: `preserved` (same verdict both sides),
  `intentionally_changed` (verdicts differ and the delta declares it),
  `unchecked` (either side cannot classify — prose guards, `underspecified`).
- Honest labeling throughout: the report carries conform's
  `evidence_scope` discipline verbatim — agreement on the supplied corpus,
  never a proof of behavioral equality (review finding CORR-001: v1 is
  example/oracle-level only; trace-level refinement claims are out of scope
  and are future work).
- No new checking machinery: reuse conform's artifact gate (lint-clean,
  current compiled artifacts) and its verdict taxonomy unchanged.

## Impact

- Affected specs: new capability `preservation`; no existing capability
  requirements are modified.
- Affected code: new `src/preserve.rs` (+ `src/commands/` wiring) reusing
  `src/conform.rs` walk machinery; CLI registration; docs.
- Provenance: catitas corpus findings item A4; conversations
  `claude/56d084d5…` (bundle v3), `mistral-work/923c311c…`.

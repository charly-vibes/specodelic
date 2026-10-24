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
  current compiled artifacts) and its verdict taxonomy unchanged. Change
  declarations live in a preserve-owned `--declarations` file
  (`{scenario_id, expect_change}`), NOT in conform's oracle record —
  conform's record schema is closed (`deny_unknown_fields`,
  src/conform.rs:245,257) and stays untouched.
- Exit-code contract mirroring conform's: 0 unless an `undeclared_change`
  exists (then 1); gate refusals 2 with zero verdict records.

## Impact

- Affected specs: new capability `preservation`; no existing capability
  requirements are modified.
- Dependencies: complements the Wave 2 A3 ticket (specodelic-fqte,
  stale-example revalidation) — same example-regression direction,
  different layer (embedded legality vs external oracle verdicts); no
  ordering constraint between them.
- Affected code: new `src/preserve.rs` (+ `src/commands/` wiring) reusing
  `src/conform.rs` walk machinery; CLI registration; docs.
- Provenance: catitas corpus findings item A4; conversations
  `claude/56d084d5…` (bundle v3), `mistral-work/923c311c…`.

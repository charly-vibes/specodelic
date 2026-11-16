# Change: Harden the archive companion — archiver bound, rollback honesty, head-prose derivation

## Why

Three failure modes of `spk archive-companion` are shipped in
`src/archive_companion.rs` (specodelic-54c6) but stand outside the
capability's contract: no requirement in
`openspec/specs/archive-companion/spec.md`, and no constraint in its layer,
describes them. Each one was a silent or unbounded failure before it was
fixed, and an unspecified fix is a fix a later refactor may silently undo:

- **Unbounded archiver.** The `openspec archive` child was waited on with no
  bound, so a hung archiver wedged `spk archive-companion` forever. The
  invocation is now bounded (`DEFAULT_ARCHIVE_TIMEOUT_SECS`,
  `SPK_ARCHIVE_TIMEOUT_SECS` override) and expiry is a labeled failure —
  `src/archive_companion.rs:1008` (default), `:1032` (bound resolution),
  `:1049` (runner), `:1072` (bounded wait), `:1138` (process-group kill),
  `:102-112` (the labeled error naming
  `openspec archive <id> --yes` and `timed out after <N>s`).
- **Discarded rollback failures.** `revert()` used to drop the result of every
  restoring write, so a revert that could not restore a saved file left the
  deployed corpus in neither the pre-run nor the assembled state with nothing
  reported. Each unrestorable entry is now a `rollback failed: <path>: <io
  error>` detail appended after the primary attach violation —
  `src/archive_companion.rs:765-776` (the collected failures) and
  `:742-758` (the append, primary violation first).
- **Placeholder head prose carried forward.** A delta archived without Purpose
  prose deploys with openspec's placeholder `TBD - created by archiving change
  <id>. Update Purpose after archive.`, and every companion rerun re-deployed
  that placeholder — an instruction to write a Purpose that could never be
  written by the tool. `assemble()` now derives the head prose from the
  archived delta's `## Purpose` when the deployed Purpose is empty or that
  placeholder, while real deployed prose stays the prose of record —
  `src/archive_companion.rs:451` (delta Purpose), `:466` (needs derivation),
  `:479` (derivation), `:519-523` (use in `assemble`).

Behavior and tests exist; the contract does not. This change supplies it.

## What Changes

- Delta `openspec/changes/harden-archive-companion/specs/archive-companion/spec.md`
  with three ADDED requirements and their scenarios:
  `Bounded archiver invocation` (default 600s bound, env override with
  non-positive/unparsable fallback, process-group kill, labeled timeout naming
  the command and the expired bound, no attach stage after expiry),
  `Rollback failure reporting` (`rollback failed: <path>: <io error>` appended
  after the primary violation, every entry listed, a clean revert appends
  nothing), and `Deployed head prose derivation` (placeholder/empty Purpose
  repaired from the archived delta's Purpose, authored prose preserved, a delta
  without Purpose a no-op).
- Layer sidecar `specs/archive-companion/layer.md`: the full post-change layer
  — the three constraints `archiver_bounded`, `rollback_reported`,
  `head_prose_derived`, the deriving properties
  `archiver_bound_enforced`, `bound_fallback_default`,
  `rollback_failure_surfaced`, `placeholder_repaired_on_rerun`,
  `authored_head_preserved`, the `archiver_timed_out` state with its `expire`
  transition, and `deploy`'s guard extended with `head_prose_derived`.
- Contract TOMLs under `.espectacular/archive-companion/` binding the new
  scenarios to the tests that already prove them.
- **No behavior change and no code change**: this is contract backfill of
  shipped behavior. The specification is deliberately written to the code's
  observed semantics, not to a redesigned one.

## Impact

- Affected specs: `archive-companion` only (three ADDED requirements; its layer
  gains three constraints, five properties, one state, one transition, and one
  extended guard). No other capability's requirements change.
- Affected code: none. The behavior is `src/archive_companion.rs` as shipped;
  the proving tests are `tests/cli/archive_companion.rs:159`
  (`archive_companion_bounds_hung_openspec_as_labeled_timeout`), `:236`
  (`archive_companion_reports_failed_rollback`), and
  `src/archive_companion/tests.rs:219` / `:249`
  (`restore_replaces_archiver_placeholder_with_delta_purpose`,
  `restore_keeps_real_deployed_purpose_over_delta_purpose`).
- Archived corpus: deployed through `just archive-change
  id=harden-archive-companion` (never hand-edited), after which
  `spk lint openspec/specs` must stay clean.
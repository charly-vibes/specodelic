# Tasks: harden-archive-companion

Contract backfill: the behavior ships (`src/archive_companion.rs`,
specodelic-54c6); the specification and layer do not describe it yet.

## 1. Shipped behavior (evidence, no work)

- [x] 1.1 Archiver wall-clock bound: `DEFAULT_ARCHIVE_TIMEOUT_SECS` = 600
      (`src/archive_companion.rs:1008`), `SPK_ARCHIVE_TIMEOUT_SECS` read at
      invocation time with a zero/unparsable fallback (`:1032`), bounded wait
      with both streams drained (`:1072`), process-group kill on expiry
      (`:1138`), labeled `OpenSpecTimedOut` naming `openspec archive <id> --yes`
      and `timed out after <N>s` (`:102-112`, mapped in `:941-958`).
      Proven by `tests/cli/archive_companion.rs:159`
      (`archive_companion_bounds_hung_openspec_as_labeled_timeout`).
- [x] 1.2 Rollback honesty: `revert()` collects every failed restore as
      `rollback failed: <path>: <io error>` (`:765-776`) and `restore()` appends
      them after the primary violation text (`:742-758`). Proven by
      `tests/cli/archive_companion.rs:236`
      (`archive_companion_reports_failed_rollback`).
- [x] 1.3 Head-prose derivation: `purpose_of_delta` (`:451`),
      `head_needs_delta_purpose` (`:466`), `derive_head` (`:479`), used by
      `assemble` (`:519-523`), with the placeholder prefix
      `TBD - created by archiving` (`:446`). Proven by
      `src/archive_companion/tests.rs:219`
      (`restore_replaces_archiver_placeholder_with_delta_purpose`), `:249`
      (`restore_keeps_real_deployed_purpose_over_delta_purpose`), and `:193`
      (`restore_assembles_dual_format_from_sidecar_and_applied_spec` — a delta
      without Purpose leaves the head untouched).

## 2. The contract (this change)

- [ ] 2.1 `specs/archive-companion/spec.md` — the plain delta: three ADDED
      requirements (`Bounded archiver invocation`, `Rollback failure
      reporting`, `Deployed head prose derivation`) with their scenarios, one
      scenario per observable behavior the tests already pin.
- [ ] 2.2 `specs/archive-companion/layer.md` — the full post-change layer
      sidecar: frontmatter `archive.companion.layer`, the ten existing
      constraints plus `archiver_bounded`, `rollback_reported`,
      `head_prose_derived`; the eleven existing properties plus
      `archiver_bound_enforced`, `bound_fallback_default`,
      `rollback_failure_surfaced`, `placeholder_repaired_on_rerun`,
      `authored_head_preserved`; the `archiver_timed_out` state with the
      `expire` transition; `deploy`'s guard extended with
      `head_prose_derived`. Every new constraint has a deriving property, every
      state appears in a transition, every guard is non-empty — the shape the
      companion's pre-archive `check_sidecars` gate and the post-attach corpus
      lint both require.
- [ ] 2.3 Contract TOMLs in `.espectacular/archive-companion/`, one per new
      scenario, bound to its proving test:
      `hung-archiver-killed-at-the-bound` → `archive_companion_bounds_hung_openspec_as_labeled_timeout`;
      `rollback-failure-is-reported` → `archive_companion_reports_failed_rollback`;
      `placeholder-repaired-on-rerun` → `restore_replaces_archiver_placeholder_with_delta_purpose`;
      `real-deployed-prose-preserved` → `restore_keeps_real_deployed_purpose_over_delta_purpose`;
      `delta-without-purpose-changes-nothing` →
      `restore_leaves_the_deployed_text_unchanged_when_the_delta_has_no_purpose`.
- [ ] 2.4 Deploy through the companion — `just archive-change
      id=harden-archive-companion` — so the deployed
      `openspec/specs/archive-companion/spec.md` is openspec's applied
      requirements plus this sidecar's frontmatter and tables (the corpus is
      never hand-edited).

## 3. Gates

- [ ] 3.1 `openspec validate harden-archive-companion --strict` clean.
- [ ] 3.2 `spk lint openspec/specs` clean after the archive (the deployed layer
      covers every new constraint, and the new properties reach the file's
      intent).
- [ ] 3.3 `ah check` clean — every new scenario resolves to a contract TOML
      bound to the test that proves it.
- [ ] 3.4 `just ci` green.
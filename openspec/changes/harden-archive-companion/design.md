# Design: harden-archive-companion

## Context

`spk archive-companion` gained three hardenings in
`src/archive_companion.rs` under specodelic-54c6 — an archiver wall-clock
bound, honest rollback reporting, and head-prose derivation — and pinned each
with a test (`tests/cli/archive_companion.rs:159`, `:236`,
`src/archive_companion/tests.rs:219`, `:249`). The capability's spec and layer
still describe the pre-hardening behavior: an unbounded invocation, a revert
whose failures vanish, and a deployed head that may be openspec's `TBD -
created by archiving …` placeholder forever. This change writes the contract
for what the code does.

## Goals / Non-Goals

- Goals: put the three behaviors under requirements and layer constraints;
  keep the description tied to the observed code and its tests; keep the
  deployed capability's layer internally consistent (every new constraint has a
  deriving property, the timeout has a lifecycle position, the head-prose
  obligation gates the deploy transition).
- Non-Goals: any code, CLI, or envelope change; a redesign of the bound
  (adaptive budgets, retries); re-litigating the Revision 19 decision that
  openspec performs the merge; specifying behavior for openspec revisions that
  do not exist yet.

## Decisions

- **Decision: specify the shipped semantics, verbatim where the code is
  explicit.** The requirement texts quote the rendered strings the code emits
  (`openspec archive <id> --yes`, `timed out after <N>s`, `rollback failed:
  <path>: <io error>`, `TBD - created by archiving`) because those strings are
  what tests and users observe. A spec that paraphrased them would let a
  message change pass unreviewed.
  - Alternatives considered: restating the pre-hardening contract and leaving
    the hardening to code comments — rejected; the capability's spec is the
    truth consumers gate on, and a silent third state (a half-reverted
    deployed corpus) is exactly what the requirements exist to forbid.

- **Decision: the bound is a wedge-breaker with a documented ops seam, not a
  latency budget.** `DEFAULT_ARCHIVE_TIMEOUT_SECS = 600` covers a bulk archive
  that really does apply delta merges across the corpus; `SPK_ARCHIVE_TIMEOUT_SECS`
  is read at invocation time (the documented `SPK_TLC_JAVA`-style seam) and a
  zero, negative, or unparsable value falls back to the default so no
  configuration can make the wait unbounded.
  - Alternatives considered: a per-change timeout flag — rejected for v1, the
    behavior of record is one bound with one override; a `spawn`-and-`recv_timeout`
    worker — the seam is a plain `FnMut(&Path, &str) -> Result<(), String>`
    (`ArchiveRunner`, `src/archive_companion.rs:176`), so the bound belongs
    inside the production runner (polling `try_wait` against
    `Instant::now()`, both streams drained on their own threads).

- **Decision: expiry kills the process group, and the seam reports a string
  the caller recognizes.** The child is spawned with `process_group(0)` and
  killed with a group-wide `SIGKILL`, so a wrapper `sh` cannot orphan its
  children past the deadline; a failed group signal falls back to the direct
  child kill. The production runner reports the expiry as `timed out after
  <N>s`, which `run` maps to the distinct `OpenSpecTimedOut` label instead of a
  generic `OpenSpecFailed`.
  - Alternatives considered: a distinct runner error type — rejected, the seam
    is a `Result<(), String>` and its other consumers (tests) inject plain
    strings; keeping the discrimination in a prefix protocol leaves the seam
    unchanged.

- **Decision: rollback failures are appended after the primary violation, never
  substituted for it.** The primary violation text stays first, then every
  `rollback failed: <path>: <io error>` detail, so an existing assertion on the
  violation wording keeps matching while the corrupted state becomes visible. A
  revert that restores everything appends nothing — no invented text.
  - Alternatives considered: a warnings channel on the envelope — rejected: the
    run fails, so warnings are not guaranteed to be read, and the detail is the
    one string an operator sees.

- **Decision: head prose is derived only when the deployed Purpose is
  unwritten.** The repair triggers on an empty `## Purpose` body, a body that
  opens with openspec's stable placeholder prefix (`TBD - created by
  archiving` — the change id varies, so the full sentence cannot be matched),
  or a head with no Purpose section at all (an entirely empty head adopts the
  delta's section as the whole head). Authored deployed prose is the prose of
  record and is never displaced; a delta without Purpose is a no-op, so the
  behavior cannot regress against today's output.
  - Alternatives considered: always prefer the archived delta's Purpose —
    rejected: a rerun would let a stale delta overwrite prose a human wrote in
    the deployed spec.

- **Decision: layer-side placement of the new obligations.** `archiver_bounded`
  gets a lifecycle position (`resolved → archiver_timed_out` via `expire`,
  guarded by `¬archiver_bounded`) because expiry is a terminal outcome of the
  invocation stage; `head_prose_derived` joins `deploy`'s guard because the
  assembly is not correct without it; `rollback_reported` stays out of guards
  because it constrains how a failure path reports, exactly like the existing
  `dry_run_never_mutates` / `newest_archive_wins` constraints that also never
  appear as guards.

## Risks / Trade-offs

- Risk: the bound fires on a legitimately slow archive, killing the archiver
  mid-merge → mitigation: the failure is labeled and names the remedy (raise
  `SPK_ARCHIVE_TIMEOUT_SECS`, re-run the companion); the companion writes
  nothing on that path, so the remedy is a plain rerun.
- Risk: openspec rewords its placeholder, so the stable-prefix match stops
  matching → worst case is today's behavior (placeholder carried forward), not
  a displaced real Purpose; the requirements name the prefix so a reword is a
  visible spec change.
- Risk: a long rollback detail makes the failure string noisy → one line per
  path, primary text first, and only on a failure that already corrupted the
  deployed corpus.

## Migration Plan

1. Land the delta and the layer sidecar in the change directory.
2. Archive through the companion (`just archive-change
   id=harden-archive-companion`): openspec merges the three ADDED requirements
   into `openspec/specs/archive-companion/spec.md`, then the companion
   re-attaches this directory's layer.
3. `spk lint openspec/specs` clean afterwards; the deployed corpus is the only
   truth once the change is archived.
4. Rollback is the archive's own: a failed attach reverts its writes and, for
   an openspec-invoking run, moves the change dir back out of the archive.

## Open Questions

- Should the effective bound be surfaced in the success envelope, so an
  operator can see the budget a run used? Deferred — that is an envelope
  change, not a contract backfill.
- Should a timed-out run attempt to move a half-archived change directory back?
  Deferred: on expiry openspec may be mid-move, and the companion only reverses
  its own writes (plus the archive move it can verify), so it attempts no
  rename it cannot check.
## ADDED Requirements

### Requirement: Bounded archiver invocation

The companion SHALL run `openspec archive <CHANGE_ID> --yes` under a
wall-clock bound — 600 seconds by default (`DEFAULT_ARCHIVE_TIMEOUT_SECS`),
overridable through `SPK_ARCHIVE_TIMEOUT_SECS`, whose zero, negative, or
unparsable value falls back to the default — and on expiry SHALL kill the
archiver's whole process group, SHALL fail with a labeled timeout naming the
command and the expired bound, and SHALL write no deployed spec, draining
both child streams concurrently.

#### Scenario: Hung archiver is killed at the bound

- **WHEN** the invoked archiver does not exit within the effective bound
- **THEN** the companion kills it, the run fails labeled with
  `timed out after <N>s` while naming `openspec archive <id> --yes`, and no
  deployed spec is written

### Requirement: Rollback failure reporting

When the post-attach assertion or the deployed-corpus lint fails and the
companion reverts its writes, every revert that could not restore its saved
bytes SHALL be reported in the attach-violation detail as `rollback failed:
<path>: <io error>`, appended after the primary violation text — never
silently discarded and never displacing the primary violation. A revert that
restores every saved entry appends nothing, and the run still fails as the
same attach violation.

#### Scenario: An unrestorable target is reported

- **WHEN** a revert cannot restore a saved file's prior bytes, or cannot
  remove a file the run itself created
- **THEN** the failure detail carries `rollback failed: <path>: <io error>` for
  that path after the primary violation text, and the run exits as the attach
  violation

### Requirement: Deployed head prose derivation

On attach, the deployed spec's head prose SHALL come from the deployed body,
except when its `## Purpose` body is empty or opens with openspec's archiver
placeholder `TBD - created by archiving` — then the archived delta's
`## Purpose` section SHALL supply it, and a head with no `## Purpose` section
adopts the delta's section as its whole head. Authored deployed prose SHALL
never be displaced, and a delta carrying no Purpose prose SHALL leave the
deployed head byte-for-byte unchanged.

#### Scenario: Placeholder replaced on rerun

- **WHEN** the deployed head's Purpose is empty or starts with
  `TBD - created by archiving`, and the archived delta carries Purpose prose
- **THEN** the deployed head carries the delta's `## Purpose` section and the
  archiver placeholder appears nowhere in the deployed spec

#### Scenario: Real deployed prose is preserved

- **WHEN** the deployed head's Purpose is authored prose and the archived delta
  also carries Purpose prose
- **THEN** the deployed Purpose is byte-identical to the pre-rerun text and the
  delta's prose is not written

#### Scenario: A delta without Purpose changes nothing

- **WHEN** the archived delta carries no `## Purpose` prose, or an empty one
- **THEN** the deployed head is unchanged even when its Purpose is the
  archiver placeholder
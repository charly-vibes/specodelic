## MODIFIED Requirements

### Requirement: Archive with dual-format preservation
The system SHALL provide `spk archive-companion <CHANGE_ID>` that archives
the named openspec change via the openspec CLI with spec application ON
(plain deltas are openspec's native merge — Revision 19), then re-attaches
the dual-format layer to each regenerated deployed spec from the change's
archived `layer.md` sidecar, so every deployed
`openspec/specs/<cap>/spec.md` ends up frontmatter-plus-tables dual-format
again: the archived delta stays plain, the sidecar stays archived beside
it, and the deployed spec is the layer-preserving re-assembly of the two.

#### Scenario: Dual-format delta survives archiving
- **WHEN** an active change carries a plain delta plus a `layer.md`
  sidecar for capability `c`
- **THEN** after `spk archive-companion <id>`, the archived delta remains
  a plain openspec file, the sidecar is archived beside it, and
  `openspec/specs/c/spec.md` is the dual-format re-assembly of the two

#### Scenario: Plain delta deploys a dual-format spec
- **WHEN** an active change carries a plain delta plus a `layer.md`
  sidecar for capability `c`
- **THEN** after `spk archive-companion <id>`,
  `openspec/specs/c/spec.md` carries the applied `## Requirements`
  section plus the sidecar's frontmatter and Constraints/Model/Properties
  tables, and the archived delta remains a plain openspec file

#### Scenario: Already-archived change re-runs cleanly
- **WHEN** the change directory no longer exists but its archive does
- **THEN** the openspec invocation is skipped and the layer re-attachment
  re-runs to a byte-identical deployed result

### Requirement: Observable resolution and preview
The system SHALL emit an envelope naming the archive directory used,
whether the openspec invocation ran, and the restored paths — and SHALL
support `--dry-run` producing that same resolution without invoking
openspec or writing any file.

#### Scenario: Dry-run reports the plan
- **WHEN** `spk archive-companion <id> --dry-run` runs over a
  resolvable change
- **THEN** the envelope lists the planned restores and no file is
  written and no openspec invocation occurs

## REMOVED Requirements

### Requirement: Fail closed on stripped deltas
**Reason**: The stripped-delta refusal polices the Stage-1 deploy (delta
minus its delta sections); Revision 19 replaces the subject — the
companion now refuses sidecar-less deltas before the destructive archive
instead of stripped deltas before a copy.
**Migration**: Replaced by "Fail closed on sidecar-less deltas" (ADDED
below), which carries the same fail-closed-before-destruction posture and
re-homes this requirement's scenario names.

## ADDED Requirements

### Requirement: Fail closed on sidecar-less deltas
The system SHALL refuse to archive any change whose delta lacks a sidecar
capable of restoring the deployed dual-format layer — no `layer.md`
beside the delta, or a sidecar that fails parse or `spk lint` — failing
the whole command before the destructive openspec archive runs, with a
listing of every refused delta and a remediation hint naming the `spk
migrate` wrap path, and writing no deployed spec.

#### Scenario: Sidecar-less delta is refused
- **WHEN** an active change carries a delta with no sibling `layer.md`
- **THEN** the command fails before `openspec archive` is invoked, the
  delta is listed, and no `openspec/specs/<cap>/spec.md` is modified

#### Scenario: Frontmatterless delta is refused
- **WHEN** an active change carries a delta with no sibling `layer.md`
- **THEN** the command fails, the delta is listed, and no
  `openspec/specs/<cap>/spec.md` is modified

#### Scenario: Unlintable sidecar is refused
- **WHEN** an active change carries a `layer.md` sidecar that fails
  `spk lint`
- **THEN** the command fails before `openspec archive` is invoked, the
  delta is listed with the lint findings, and no deployed spec is written

#### Scenario: Post-attach shape assertion
- **WHEN** the layer re-attachment would write a deployed spec carrying a
  delta header, or lacking frontmatter or any of the three tables
- **THEN** the write is refused and reverted and the command fails naming
  the capability — the deployed tree never gains a shape 1.x strict
  rejects

#### Scenario: New capability deploys from scratch
- **WHEN** a change introduces capability `c` with no existing deployed
  `openspec/specs/c/spec.md`
- **THEN** the companion assembles the first deployed spec from the
  delta's applied requirements plus the sidecar's frontmatter and tables
  — same shape assertion, same fail-closed refusal

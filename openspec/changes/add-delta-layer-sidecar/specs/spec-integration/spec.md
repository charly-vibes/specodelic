## MODIFIED Requirements

### Requirement: Dual-format delta
Every deployed capability spec under `openspec/specs/<cap>/spec.md` SHALL
be simultaneously a valid openspec capability spec and a lint-clean
specodelic spec file (frontmatter plus Constraints, Model, and Properties
sections), carrying the REAL id derived by the naming law — a `spec.md`
file takes its id from its parent directory name (`-` ⇔ `.`; Revision 18)
— and SHALL carry no delta sections and no `## Requirements` mirror: with
change deltas plain (Revision 19), the mirror pairing rules retire by
construction.

#### Scenario: Both parsers accept
- **WHEN** `openspec validate --specs --strict` and `spk lint <file>` run
  on any deployed capability spec
- **THEN** both exit 0 with zero issues

#### Scenario: No delta sections reach the deployed spec
- **WHEN** the archive-companion deploys a change for capability `c`
- **THEN** `openspec/specs/c/spec.md` carries no
  `## ADDED/MODIFIED/REMOVED Requirements` headers and no
  `## Requirements` mirror — only the single `## Requirements` section
  openspec applied

#### Scenario: Modified delta is dual-format
- **WHEN** a deployed capability file carries a delta section or a
  `## Requirements` mirror (the retired dual shape)
- **THEN** openspec 1.x strict validation fails and the companion's
  post-attach shape assertion refuses the deploy — the shape is
  unwritable, so no per-file lint pairing check remains

### Requirement: Change deltas are plain openspec deltas
A change's delta file under `openspec/changes/` SHALL be a plain openspec
delta — openspec validate is its gate; it SHALL carry no frontmatter and
no specodelic tables, and no `## Requirements` mirror — with the
specodelic layer carried by a sibling `layer.md` sidecar (Revision 19: a
linted specodelic file whose id derives from the parent directory plus a
`.layer` suffix), so a repo needs exactly one spec vocabulary.

#### Scenario: Coexisting identical ids
- **WHEN** two specodelic files claiming the same intent id (the pair: an
  active change's sidecar and its deployed capability spec — distinct via
  the `.layer` suffix) are linted together
- **THEN** the lint reports no unique-id violation (uniqueness is per-file);
  command paths (model-check, verify, orchestrate) refuse a true same-id
  pair with `duplicate_corpus_identity`

#### Scenario: Sidecar rides with the delta
- **WHEN** a change delta under `openspec/changes/<id>/specs/<cap>/` is
  authored
- **THEN** the delta carries `## ADDED/MODIFIED/REMOVED Requirements` only
  and the sibling `layer.md` carries the frontmatter and tables; `spk lint
  openspec` lints the sidecar and skips the frontmatterless delta

## REMOVED Requirements

### Requirement: Section sync
**Reason**: The requirement policed the pairing of a delta section with a
`## Requirements` mirror in the same file — a shape plain deltas (Revision
19) can no longer produce without failing openspec validation. Retired by
construction with the mirror itself.
**Migration**: None — the deployed capability spec's single `## Requirements`
section is written by openspec's own apply; no sync check remains to
migrate. The CI gate is `openspec validate --all --strict` (1.x), which
flags any residual mirror as an error rather than a drift.

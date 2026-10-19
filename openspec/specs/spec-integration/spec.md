---
id: spec.integration
kind: intent
statement: "WHEN a specodelic corpus lives under openspec, THE spec-integration capability SHALL keep change deltas plain openspec files with their specodelic layer in a sibling layer.md sidecar, SHALL keep deployed capability specs simultaneously openspec-valid and lint-clean with no delta sections and no mirror, and SHALL refuse a true same-id pair on command paths — so a repo needs exactly one spec vocabulary."
---

# spec-integration Specification

## Purpose
Widen the dual-format mirror rules to `## MODIFIED Requirements` so the
repo's first MODIFIED-carrying delta is gated exactly like an
ADDED-carrying one — the widening is additive; no ADDED-side behavior
changes.

## Constraints

| id                        | kind      | expr                                                                                                                                             | traces_to |
|---------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[spec.integration]] |
| plain_deltas_carried | invariant | `a change's delta file under openspec/changes/ is a plain openspec delta — openspec validate is its gate; it carries no frontmatter, no specodelic tables, and no ## Requirements mirror — with the specodelic layer in a sibling layer.md sidecar (Revision 19: a linted specodelic file whose id derives from the parent directory plus a .layer suffix)`      | [[spec.integration]]  |
| deployed_shape_closed | invariant | `a deployed capability spec carries no delta sections and no ## Requirements mirror — the retired dual shape is unwritable: openspec 1.x strict validation fails it and the companion's post-attach shape assertion refuses the deploy, so no per-file lint pairing check remains`      | [[spec.integration]]  |
| both_parsers_accept | invariant | `every deployed capability spec is simultaneously a valid openspec capability spec and a lint-clean specodelic spec file (frontmatter plus Constraints, Model, and Properties sections) carrying its real naming-law id — a spec.md file takes its id from its parent directory name (`-` ⇔ `.`)`                                   | [[spec.integration]]  |
| duplicate_corpus_identity | invariant | `the intent ids of a command's parsed inputs are unique — command paths (model-check, verify, orchestrate) refuse a true same-id pair with duplicate_corpus_identity before any write; lint accepts files with distinct ids together (uniqueness is per-file)`                                   | [[spec.integration]]  |

## Model

### States

- `plain_delta`
- `deployed`
- `gated`

### Transitions

| id      | from        | to        | guard                                                                                     |
|-------------|-------------|-----------|-------------------------------------------------------------------------------------------|
| deploy_pair | plain_delta | deployed  | [[spec.integration.plain_deltas_carried]] ∧ [[spec.integration.both_parsers_accept]]  |
| gate        | deployed    | gated     | [[spec.integration.deployed_shape_closed]] ∧ [[spec.integration.duplicate_corpus_identity]]  |

## Properties

| id                     | kind | derives_from                     | generator                                        | predicate                                                     |
|------------------------|------|-----------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| process_lifecycle_checked | unit | [[spec.integration.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| plain_delta_gate_green | unit | [[spec.integration.plain_deltas_carried]]   | `plain_delta_with_sidecar_pair()`           | `openspec validate passes the delta; spk lint passes the sidecar; the delta carries no frontmatter or tables` |
| deployed_shape_enforced | unit | [[spec.integration.deployed_shape_closed]]| `assembly_with_delta_header()`               | `openspec 1.x strict fails the retired shape; the companion refuses it pre-write`               |
| both_parsers_green | unit | [[spec.integration.both_parsers_accept]]    | `deployed_capability_spec()`           | `openspec validate --specs --strict and spk lint both exit 0 with zero issues`              |
| same_id_pair_refused | unit | [[spec.integration.duplicate_corpus_identity]]    | `sidecar_and_deployed_pair()`              | `command paths refuse a true same-id pair with duplicate_corpus_identity; lint accepts the distinct pair`               |

## Requirements

### Requirement: Dual-format delta
Every deployed capability spec under `openspec/specs/<cap>/spec.md`
SHALL be simultaneously a valid openspec capability spec and a lint-
clean specodelic spec file (frontmatter plus Constraints, Model, and
Properties sections) carrying its real naming-law id — a `spec.md` file
takes its id from its parent directory name — and SHALL carry no delta
sections and no `## Requirements` mirror: with change deltas plain
(Revision 19), the mirror pairing rules retire by construction.

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

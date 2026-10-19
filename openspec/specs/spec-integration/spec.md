---
id: spec.integration
kind: intent
statement: "WHEN a deployed capability spec carries a delta section, THE mirror rules SHALL treat that section exactly as the others — carrying the real parent-dir-derived id, pairing the sibling mirror, and failing CI on drift — while change deltas stay plain openspec files (Revision 18: no `id: spec`, no self-containment)."
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
| modified_declares_id_spec | invariant | `a file carrying ## MODIFIED Requirements pairs it with a sibling ## Requirements mirror, exactly as the ADDED rule requires; the file id is the naming law's business (spec.md derives from its parent directory — Revision 18) and command paths refuse a same-id delta/deployed pair (duplicate_corpus_identity)` | [[spec.integration]]  |
| modified_mirror_synced    | invariant | `the ## Requirements mirror holds the same requirement text as ## MODIFIED Requirements; divergence fails CI naming the divergent requirement`      | [[spec.integration]]  |
| added_rules_unchanged     | invariant | `every ADDED-section rule keeps firing with unchanged semantics — the widening is additive, never a replacement`                                   | [[spec.integration]]  |

## Model

### States

- `added_only`
- `widened`
- `gated`

### Transitions

| id     | from       | to       | guard                                                            |
|--------|------------|----------|------------------------------------------------------------------|
| widen  | added_only | widened  | [[spec.integration.modified_declares_id_spec]]  |
| gate   | widened    | gated    | [[spec.integration.modified_mirror_synced]] ∧ [[spec.integration.added_rules_unchanged]]  |

## Properties

| id                     | kind | derives_from                      | generator                                        | predicate                                                     |
|------------------------|------|-----------------------------------|--------------------------------------------------|----------------------------------------------------------------|
| process_lifecycle_checked | unit | [[spec.integration.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| modified_drift_fails   | unit | [[spec.integration.modified_mirror_synced]]   | `modified_delta_with_drifted_mirror()`           | `ci_sync_check(f) == failed` (names the divergent requirement) |
| modified_id_enforced   | unit | [[spec.integration.modified_declares_id_spec]]| `modified_delta_with_non_spec_id()`              | `spk_lint(f) == failed` (dual_format_valid)                    |
| added_tests_unchanged  | unit | [[spec.integration.added_rules_unchanged]]    | `the pre-widening ADDED fixture suite`           | `every existing ADDED-rule test passes unchanged`              |

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

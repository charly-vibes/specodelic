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

## MODIFIED Requirements

### Requirement: Dual-format delta
Every deployed capability spec under `openspec/specs/<cap>/spec.md` SHALL
be simultaneously a valid openspec capability spec and a lint-clean
specodelic spec file (frontmatter plus Constraints, Model, and Properties
sections), carrying the REAL id derived by the naming law — a `spec.md`
file takes its id from its parent directory name (`-` ⇔ `.`; Revision 18)
— and pairing its carried delta sections with a sibling `## Requirements`
mirror. A change's delta file under `openspec/changes/` is a plain
openspec delta (openspec validate is its gate; it carries no frontmatter
and no specodelic tables), so a repo needs exactly one spec vocabulary.

#### Scenario: Both parsers accept
- **WHEN** `openspec validate <change> --strict` and `spk lint <file>` run on any deployed capability spec
- **THEN** both exit 0 with zero issues

#### Scenario: Coexisting identical ids
- **WHEN** two dual-format files claiming the same intent id (the
  transitional pair: an active change's delta and its deployed capability
  spec) are linted together
- **THEN** the lint reports no unique-id violation (uniqueness is per-file);
  command paths (model-check, verify, orchestrate) refuse the same pair
  with `duplicate_corpus_identity`

#### Scenario: Modified delta is dual-format
- **WHEN** a deployed capability file carries `## MODIFIED Requirements`
  without the sibling mirror
- **THEN** `spk lint` reports the `dual_format_valid` finding, exactly as
  it does for an ADDED-carrying file
### Requirement: Section sync
The `## Requirements` mirror SHALL contain identical requirement and
scenario text to its delta section — `## ADDED Requirements` and, for
a MODIFIED-carrying file, `## MODIFIED Requirements` alike — and
divergence SHALL fail CI naming the divergent requirement.

#### Scenario: Drifted sections fail CI
- **WHEN** a requirement's text differs between the ADDED and Requirements sections
- **THEN** `just ci` fails and the failure names the divergent requirement

#### Scenario: Drifted MODIFIED mirror fails CI
- **WHEN** a requirement's text differs between the MODIFIED and
  Requirements sections of a delta file
- **THEN** `just ci` fails and the failure names the divergent
  requirement

## Requirements

### Requirement: Dual-format delta
Every deployed capability spec under `openspec/specs/<cap>/spec.md` SHALL
be simultaneously a valid openspec capability spec and a lint-clean
specodelic spec file (frontmatter plus Constraints, Model, and Properties
sections), carrying the REAL id derived by the naming law — a `spec.md`
file takes its id from its parent directory name (`-` ⇔ `.`; Revision 18)
— and pairing its carried delta sections with a sibling `## Requirements`
mirror. A change's delta file under `openspec/changes/` is a plain
openspec delta (openspec validate is its gate; it carries no frontmatter
and no specodelic tables), so a repo needs exactly one spec vocabulary.

#### Scenario: Both parsers accept
- **WHEN** `openspec validate <change> --strict` and `spk lint <file>` run on any deployed capability spec
- **THEN** both exit 0 with zero issues

#### Scenario: Coexisting identical ids
- **WHEN** two dual-format files claiming the same intent id (the
  transitional pair: an active change's delta and its deployed capability
  spec) are linted together
- **THEN** the lint reports no unique-id violation (uniqueness is per-file);
  command paths (model-check, verify, orchestrate) refuse the same pair
  with `duplicate_corpus_identity`

#### Scenario: Modified delta is dual-format
- **WHEN** a deployed capability file carries `## MODIFIED Requirements`
  without the sibling mirror
- **THEN** `spk lint` reports the `dual_format_valid` finding, exactly as
  it does for an ADDED-carrying file
### Requirement: Section sync
The `## Requirements` mirror SHALL contain identical requirement and
scenario text to its delta section — `## ADDED Requirements` and, for
a MODIFIED-carrying file, `## MODIFIED Requirements` alike — and
divergence SHALL fail CI naming the divergent requirement.

#### Scenario: Drifted sections fail CI
- **WHEN** a requirement's text differs between the ADDED and Requirements sections
- **THEN** `just ci` fails and the failure names the divergent requirement

#### Scenario: Drifted MODIFIED mirror fails CI
- **WHEN** a requirement's text differs between the MODIFIED and
  Requirements sections of a delta file
- **THEN** `just ci` fails and the failure names the divergent
  requirement
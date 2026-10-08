# spec-integration Specification

## Purpose
Widen the dual-format mirror rules to `## MODIFIED Requirements` so the
repo's first MODIFIED-carrying delta is gated exactly like an
ADDED-carrying one — the widening is additive; no ADDED-side behavior
changes.

## Constraints

| id                        | kind      | expr                                                                                                                                             | traces_to |
|---------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| modified_declares_id_spec | invariant | `a file carrying ## MODIFIED Requirements declares id: spec and pairs it with a sibling ## Requirements mirror, exactly as the ADDED rule requires` | [[spec.integration]]  |
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
| widen  | added_only | widened  | [[spec.integration.modified_declares_id_spec]]                               |
| gate   | widened    | gated    | [[spec.integration.modified_mirror_synced]] ∧ [[spec.integration.added_rules_unchanged]] |

## Properties

| id                     | kind | derives_from                      | generator                                        | predicate                                                     |
|------------------------|------|-----------------------------------|--------------------------------------------------|----------------------------------------------------------------|
| modified_drift_fails   | unit | [[spec.integration.modified_mirror_synced]]   | `modified_delta_with_drifted_mirror()`           | `ci_sync_check(f) == failed` (names the divergent requirement) |
| modified_id_enforced   | unit | [[spec.integration.modified_declares_id_spec]]| `modified_delta_with_non_spec_id()`              | `spk_lint(f) == failed` (dual_format_valid)                    |
| added_tests_unchanged  | unit | [[spec.integration.added_rules_unchanged]]    | `the pre-widening ADDED fixture suite`           | `every existing ADDED-rule test passes unchanged`              |

## MODIFIED Requirements

### Requirement: Dual-format delta
Every openspec change delta file SHALL be simultaneously a valid
openspec delta and a lint-clean specodelic spec file (frontmatter plus
Constraints, Model, and Properties sections), declaring `id: spec` per
the naming law applied to the required `spec.md` filename. A file
carrying `## ADDED Requirements` or `## MODIFIED Requirements` is a
dual-format file and declares `id: spec`, pairing its delta section
with a sibling `## Requirements` mirror.

#### Scenario: Both parsers accept
- **WHEN** `openspec validate <change> --strict` and `spk lint <delta>` run on any active change's delta file
- **THEN** both exit 0 with zero issues

#### Scenario: Coexisting identical ids
- **WHEN** two dual-format delta files, each declaring `id: spec`, are linted together
- **THEN** the lint reports no unique-id violation (uniqueness is per-file)

#### Scenario: Modified delta is dual-format
- **WHEN** a delta file carries `## MODIFIED Requirements` without
  declaring `id: spec` or without the sibling mirror
- **THEN** `spk lint` reports the `dual_format_valid` finding, exactly
  as it does for an ADDED-carrying file

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
Every openspec change delta file SHALL be simultaneously a valid
openspec delta and a lint-clean specodelic spec file (frontmatter plus
Constraints, Model, and Properties sections), declaring `id: spec` per
the naming law applied to the required `spec.md` filename. A file
carrying `## ADDED Requirements` or `## MODIFIED Requirements` is a
dual-format file and declares `id: spec`, pairing its delta section
with a sibling `## Requirements` mirror.

#### Scenario: Both parsers accept
- **WHEN** `openspec validate <change> --strict` and `spk lint <delta>` run on any active change's delta file
- **THEN** both exit 0 with zero issues

#### Scenario: Coexisting identical ids
- **WHEN** two dual-format delta files, each declaring `id: spec`, are linted together
- **THEN** the lint reports no unique-id violation (uniqueness is per-file)

#### Scenario: Modified delta is dual-format
- **WHEN** a delta file carries `## MODIFIED Requirements` without
  declaring `id: spec` or without the sibling mirror
- **THEN** `spk lint` reports the `dual_format_valid` finding, exactly
  as it does for an ADDED-carrying file

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
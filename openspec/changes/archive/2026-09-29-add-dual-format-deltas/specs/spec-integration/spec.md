---
id: spec
kind: intent
statement: "THE openspec workflow SHALL carry every change's requirements as one dual-format markdown file that validates under both the openspec and the specodelic parsers."
---

# spec-integration Specification

## Purpose
Define how openspec change artifacts carry specodelic-formatted requirement content, so requirements are authored once, validated by both parsers, and remain lintable after archive.

## Constraints

| id                 | kind      | expr                                                                                              | traces_to |
|--------------------|-----------|---------------------------------------------------------------------------------------------------|-----------|
| dual_format_valid  | invariant | `every change delta file passes openspec validate --strict and spk lint with zero issues`           | [[spec]]  |
| id_spec_accepted   | invariant | `dual-format files declare id: spec; uniqueness holds per-file and deltas stay self-contained`      | [[spec]]  |
| self_contained     | invariant | `every wiki-ref in a dual-format file resolves within that same file; domain specs referenced by prose path only` | [[spec]]  |
| archive_verbatim   | invariant | `archiving runs openspec archive --skip-specs then copies the delta file unchanged to openspec/specs/<cap>/spec.md` | [[spec]]  |
| section_sync       | invariant | `the ADDED Requirements section text equals the Requirements section text in every dual-format file` | [[spec]]  |
| lifecycle_modeled  | invariant | `every dual-format delta's Model section encodes proposed → approved → implemented → archived`       | [[spec]]  |
| ci_gated           | invariant | `just ci runs openspec validate --strict over all changes and spk lint over all dual-format deltas`  | [[spec]]  |

## Model

### States
- `proposed`
- `approved`
- `implemented`
- `archived`

### Transitions

| id         | from      | to          | guard                            |
|------------|-----------|-------------|----------------------------------|
| approve    | proposed  | approved    | [[spec.dual_format_valid]]       |
| implement  | approved  | implemented | [[spec.section_sync]]            |
| archive    | implemented | archived  | [[spec.archive_verbatim]]        |

## Properties

| id                | kind | derives_from                    | generator                              | predicate                                                       |
|-------------------|------|---------------------------------|----------------------------------------|-----------------------------------------------------------------|
| parsers_agree     | unit | [[spec.dual_format_valid]]      | `arbitrary_dual_format_delta()`        | `openspec_validate(f) == passed ∧ spk_lint(f) == passed`         |
| id_collision_safe | unit | [[spec.id_spec_accepted]]       | `two_dual_format_files_id_spec()`      | `spk_lint(f1, f2) == passed`                                     |
| no_ambiguous_refs | unit | [[spec.self_contained]]         | `delta_with_cross_file_ref()`          | `lint(f) == failed` (dangling ref reported)                      |
| verbatim_archive  | unit | [[spec.archive_verbatim]]       | `change_at_archive_gate()`             | `diff(openspec/specs/cap/spec.md, delta) == empty`               |
| sections_identical| unit | [[spec.section_sync]]           | `dual_format_file_with_drifted_sections()` | `ci_sync_check(f) == failed` (names divergent requirement)   |
| lifecycle_parseable | unit | [[spec.lifecycle_modeled]]    | `arbitrary_dual_format_delta()`        | `graph(f) resolves all lifecycle refs`                           |
| gates_run         | unit | [[spec.ci_gated]]               | `pr_skipping_validation()`             | `just ci == failed`                                              |

## ADDED Requirements

### Requirement: Dual-format delta
Every openspec change delta file SHALL be simultaneously a valid openspec delta and a lint-clean specodelic spec file (frontmatter plus Constraints, Model, and Properties sections), declaring `id: spec` per the naming law applied to the required `spec.md` filename.

#### Scenario: Both parsers accept
- **WHEN** `openspec validate <change> --strict` and `spk lint <delta>` run on any active change's delta file
- **THEN** both exit 0 with zero issues

#### Scenario: Coexisting identical ids
- **WHEN** two dual-format delta files, each declaring `id: spec`, are linted together
- **THEN** the lint reports no unique-id violation (uniqueness is per-file)

### Requirement: Self-contained deltas
A dual-format file SHALL resolve every wiki-reference within its own rows, and SHALL reference domain specs by prose path only.

#### Scenario: Cross-file ref rejected
- **WHEN** a delta contains a wiki-link pointing at a row defined in another file
- **THEN** `spk lint` reports a dangling reference for that link

### Requirement: Verbatim archiving
Archiving SHALL run `openspec archive <id> --skip-specs` and then copy the dual-format delta file unchanged from the archive directory to `openspec/specs/<capability>/spec.md`, so the specodelic layer survives into engineering truth.

#### Scenario: Archive preserves the specodelic layer
- **WHEN** a change is archived via the verbatim recipe
- **THEN** `openspec/specs/<cap>/spec.md` still passes `spk lint` with zero issues and `openspec validate --specs --strict`

#### Scenario: Default regeneration is bypassed
- **WHEN** the archive recipe runs
- **THEN** no file in `openspec/specs/` is regenerated from parsed deltas (frontmatter and tables intact)

### Requirement: Section sync
The `## ADDED Requirements` section and the `## Requirements` section of a dual-format file SHALL contain identical requirement and scenario text, and divergence SHALL fail CI naming the divergent requirement.

#### Scenario: Drifted sections fail CI
- **WHEN** a requirement's text differs between the ADDED and Requirements sections
- **THEN** `just ci` fails and the failure names the divergent requirement

### Requirement: Modeled change lifecycle
Every dual-format delta's Model section SHALL encode the change lifecycle states `proposed`, `approved`, `implemented`, `archived` with transitions guarded by that file's own invariant constraints.

#### Scenario: Lifecycle references resolve
- **WHEN** `spk graph` runs over a dual-format delta
- **THEN** every lifecycle transition guard resolves within the file and no state is orphaned

### Requirement: CI gates
`just ci` SHALL run `openspec validate --strict` over every active change and `spk lint` over every dual-format delta file, failing the build on any finding.

#### Scenario: Invalid change blocks the build
- **WHEN** an active change fails `openspec validate --strict` or its delta fails `spk lint`
- **THEN** `just ci` exits non-zero and names the offending change or file

## Requirements

### Requirement: Dual-format delta
Every openspec change delta file SHALL be simultaneously a valid openspec delta and a lint-clean specodelic spec file (frontmatter plus Constraints, Model, and Properties sections), declaring `id: spec` per the naming law applied to the required `spec.md` filename.

#### Scenario: Both parsers accept
- **WHEN** `openspec validate <change> --strict` and `spk lint <delta>` run on any active change's delta file
- **THEN** both exit 0 with zero issues

#### Scenario: Coexisting identical ids
- **WHEN** two dual-format delta files, each declaring `id: spec`, are linted together
- **THEN** the lint reports no unique-id violation (uniqueness is per-file)

### Requirement: Self-contained deltas
A dual-format file SHALL resolve every wiki-reference within its own rows, and SHALL reference domain specs by prose path only.

#### Scenario: Cross-file ref rejected
- **WHEN** a delta contains a wiki-link pointing at a row defined in another file
- **THEN** `spk lint` reports a dangling reference for that link

### Requirement: Verbatim archiving
Archiving SHALL run `openspec archive <id> --skip-specs` and then copy the dual-format delta file unchanged from the archive directory to `openspec/specs/<capability>/spec.md`, so the specodelic layer survives into engineering truth.

#### Scenario: Archive preserves the specodelic layer
- **WHEN** a change is archived via the verbatim recipe
- **THEN** `openspec/specs/<cap>/spec.md` still passes `spk lint` with zero issues and `openspec validate --specs --strict`

#### Scenario: Default regeneration is bypassed
- **WHEN** the archive recipe runs
- **THEN** no file in `openspec/specs/` is regenerated from parsed deltas (frontmatter and tables intact)

### Requirement: Section sync
The `## ADDED Requirements` section and the `## Requirements` section of a dual-format file SHALL contain identical requirement and scenario text, and divergence SHALL fail CI naming the divergent requirement.

#### Scenario: Drifted sections fail CI
- **WHEN** a requirement's text differs between the ADDED and Requirements sections
- **THEN** `just ci` fails and the failure names the divergent requirement

### Requirement: Modeled change lifecycle
Every dual-format delta's Model section SHALL encode the change lifecycle states `proposed`, `approved`, `implemented`, `archived` with transitions guarded by that file's own invariant constraints.

#### Scenario: Lifecycle references resolve
- **WHEN** `spk graph` runs over a dual-format delta
- **THEN** every lifecycle transition guard resolves within the file and no state is orphaned

### Requirement: CI gates
`just ci` SHALL run `openspec validate --strict` over every active change and `spk lint` over every dual-format delta file, failing the build on any finding.

#### Scenario: Invalid change blocks the build
- **WHEN** an active change fails `openspec validate --strict` or its delta fails `spk lint`
- **THEN** `just ci` exits non-zero and names the offending change or file
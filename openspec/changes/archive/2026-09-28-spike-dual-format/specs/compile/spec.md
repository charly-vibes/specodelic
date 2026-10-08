# Spike: dual-format delta (openspec grammar + specodelic schema)

## Constraints

| id                 | kind      | expr                                                                    | traces_to |
|--------------------|-----------|--------------------------------------------------------------------------|-----------|
| delta_grammar_valid | invariant | `file parses under openspec validate --strict`                           | [[compile]]  |
| no_domain_drift    | invariant | `every delta scenario maps to exactly one domain row in specs/compile.md` | [[compile]]  |
| domain_ids_preserved | invariant | `domain row ids appear unchanged in the delta text`                      | [[compile]]  |

## Model

### States
- `unsynced`
- `synced`

### Transitions

| id    | from     | to      | guard                    |
|-------|----------|---------|--------------------------|
| sync  | unsynced | synced  | [[compile.no_domain_drift]] |

## Properties

| id                | kind | derives_from                    | generator                  | predicate                                        |
|-------------------|------|---------------------------------|----------------------------|--------------------------------------------------|
| grammar_validates | unit | [[compile.delta_grammar_valid]]    | `arbitrary_dual_format_delta()` | `openspec_validate(delta) == passed`          |
| drift_absent      | unit | [[compile.no_domain_drift]]        | `delta_vs_domain_diff()`   | `∀ scenario: maps_to_domain_row(scenario)`       |
| ids_stable        | unit | [[compile.domain_ids_preserved]]   | `arbitrary_domain_id()`    | `id ∈ delta_scenario_text`                       |

## ADDED Requirements

### Requirement: Compile precondition gate
The system SHALL run `compile` only on a spec file that passes lint with zero issues.

#### Scenario: Lint-dirty file refused
- **WHEN** `spk compile <file>` is invoked on a file with a coverage-rule finding
- **THEN** the command exits non-zero

#### Scenario: Lint-clean corpus compiles
- **WHEN** `spk compile specs --json` is invoked on the repo's own corpus
- **THEN** the command exits 0

## Requirements

### Requirement: Compile precondition gate
The system SHALL run `compile` only on a spec file that passes lint with zero issues.

#### Scenario: Lint-dirty file refused
- **WHEN** `spk compile <file>` is invoked on a file with a coverage-rule finding
- **THEN** the command exits non-zero

#### Scenario: Lint-clean corpus compiles
- **WHEN** `spk compile specs --json` is invoked on the repo's own corpus
- **THEN** the command exits 0

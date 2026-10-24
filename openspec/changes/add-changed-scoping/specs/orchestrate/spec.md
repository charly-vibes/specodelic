## ADDED Requirements

### Requirement: Changed-path scoping for artifact-local stages

`spk orchestrate --changed <paths...>` SHALL restrict the compile,
model-check, and verify stages to the declared paths while lint continues to
run corpus-wide, and the summary SHALL state for each stage whether it ran
scoped or corpus-wide.

#### Scenario: Artifact-local stages scope to changed paths

- **WHEN** orchestrate runs with `--changed` naming one file of a
  three-file corpus
- **THEN** compile, model-check, and verify execute only for that file, and
  the summary marks each stage's scope

#### Scenario: Lint stays corpus-wide

- **WHEN** orchestrate runs with `--changed`
- **THEN** lint's findings span the corpus — cross-file reference
  resolution is unchanged and no scoped lint result is reported

#### Scenario: Same-stem shadowing stays labeled

- **WHEN** two same-stem files fall inside the changed scope
- **THEN** the run fails labeled per the compile scope guard — a later file
  never silently wins

#### Scenario: Unresolvable path refused

- **WHEN** a `--changed` path does not exist inside the corpus or names a
  non-spec file
- **THEN** the run fails labeled naming the unresolvable path — a
  zero-work silent success is impossible

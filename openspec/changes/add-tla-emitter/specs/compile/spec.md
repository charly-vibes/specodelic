# Delta spec: compile capability (mirrors specs/compile.md's model_to_tla)

## ADDED Requirements

### Requirement: Model section compiles to a TLA+ module
The system SHALL emit one TLA+ module per compiled spec file — always,
regardless of which model_check backend later runs against the model:
each State becomes a value in the module's state variable's range, each
Transition becomes one disjunct of the `Next` action, and a State with an
`emits` field additionally becomes one entry in an `Output` function from
that state value to the effect-Constraint reference — absent for a state
with no `emits`, never a default or null entry. Spec-side guard and emits
text is not TLA+; it SHALL be carried verbatim in comments while the
emitted disjuncts are valid TLA+ over the state variable.

#### Scenario: Disjunct count matches transitions
- **WHEN** a Model section with n transitions is compiled
- **THEN** the emitted module's `Next` action contains exactly n disjuncts

#### Scenario: Output covers emitting states only
- **WHEN** a Model has e states with `emits` and s states without
- **THEN** the emitted `Output` function's domain is exactly the e emitting states
- **AND** no entry exists for the rest — never a default or null

#### Scenario: Module always emitted
- **WHEN** a lint-clean spec file is compiled
- **THEN** the command writes `<stem>.tla` alongside the TOML and proptest artifacts
- **AND** recompiling yields byte-identical module text

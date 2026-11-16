## ADDED Requirements

### Requirement: Fragment citation advisory

A `**rust:**` fragment on an invariant Constraint's `expr` or a non-law
Property's `predicate` SHALL cite at least one declared model element — a
Model state id, a transition id, or a Constraint/Property row id, never the
intent id — matched whole-token with `.` as a token boundary, and a fragment
citing none SHALL be reported as `linter.fragment_binds_nothing` on the
warnings channel at exit 0, naming the row and the file; one declared citation
SHALL be sufficient.

#### Scenario: vacuous_fragment_warns

- **WHEN** an invariant Constraint's expr is `**rust:** flag != 0` and the
  file declares a state, a transition, that constraint, and a unit property
- **THEN** lint reports `linter.fragment_binds_nothing` for that row on the
  warnings channel, and the issues channel carries no finding under that rule
  (exit 0)
- **VERIFIES** [[linter.content_advisories.vacuous_fragment_warns]]

#### Scenario: whole_token_citation

- **WHEN** a fragment reads `**rust:** unlocked == false` and the file
  declares a `lock` state
- **THEN** the fragment does not cite `lock` and warns, while a fragment
  naming `demo.spec.held` cites a declared `held` and stays silent
- **VERIFIES** [[linter.content_advisories.whole_token_citation]]

#### Scenario: declared_element_sufficiency

- **WHEN** a fragment cites a declared state that no transition ever enters
- **THEN** no `linter.fragment_binds_nothing` finding is emitted for that row —
  citing one declared element is sufficient, and reachability is never judged
- **VERIFIES** [[linter.content_advisories.declared_element_sufficiency]]

#### Scenario: law_predicate_not_judged

- **WHEN** a law-kind Property's predicate carries `**identity:**` and
  `**associativity:**` case labels and no `**rust:**` fragment
- **THEN** no `linter.fragment_binds_nothing` finding is emitted for that row
- **VERIFIES** [[linter.content_advisories.law_predicate_not_judged]]

### Requirement: Forbidden-state declaration advisory

An invariant Constraint's `**rust:**` fragment comparing the bare `state`
binder for string inequality SHALL name a state declared under `Model/States`,
and each undeclared literal SHALL be reported as `linter.vacuous_invariant`
on the warnings channel, naming the constrained row and the literal, with the
run still exiting 0; a declared state SHALL be silent regardless of
reachability, and other binders, `==`, arithmetic, calls, and escaped or raw
literals SHALL be outside the surface.

#### Scenario: undeclared_forbidden_state_warns

- **WHEN** an invariant expr is `**rust:** state != "aborted"` and
  `Model/States` declares only `running` and `done`
- **THEN** lint reports `linter.vacuous_invariant` naming the row and
  `aborted`, on the warnings channel (exit 0)
- **VERIFIES** [[linter.content_advisories.undeclared_forbidden_state_warns]]

#### Scenario: phase_state_silent

- **WHEN** an invariant expr is `**rust:** state != "halted"` and
  `Model/States` declares `halted` even though the correct runs never enter it
- **THEN** no `linter.vacuous_invariant` finding is emitted — the check is
  citation versus declaration, never reachability
- **VERIFIES** [[linter.content_advisories.phase_state_silent]]

#### Scenario: narrow_state_surface

- **WHEN** an invariant expr uses `state == "halted"`, a different binder
  (`my_state` or `model.state`), or a raw/escaped literal
- **THEN** no `linter.vacuous_invariant` finding is emitted — only the bare
  `state` binder's `!=` string comparison is judged
- **VERIFIES** [[linter.content_advisories.narrow_state_surface]]

### Requirement: Vacuity advisory severity and family partition

Both vacuity rules SHALL ride the lint success envelope's warnings channel
with exit 0, SHALL never push a finding to the issues channel, SHALL emit at
most one finding per subject under their own rule id and remedy, and SHALL
NOT restate a `linter.placeholder_content` or `linter.tautological_predicate`
finding; the whole-cell rules SHALL NOT restate a vacuity finding.

#### Scenario: vacuity_never_gates

- **WHEN** a corpus contains a fragment citing no declared element and an
  invariant forbidding an undeclared state
- **THEN** the run exits 0, both findings appear on the warnings channel, and
  the issues channel carries neither rule id
- **VERIFIES** [[linter.content_advisories.vacuity_never_gates]]

#### Scenario: family_partition

- **WHEN** one corpus carries a whole-cell placeholder, a tautological
  predicate, and a vacuous fragment
- **THEN** each finding carries its owning rule id, no rule emits twice for
  one subject, and no finding is re-issued under another rule's id
- **VERIFIES** [[linter.content_advisories.family_partition]]
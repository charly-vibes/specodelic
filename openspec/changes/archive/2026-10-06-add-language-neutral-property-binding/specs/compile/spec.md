# compile Specification

## Purpose
Make the Properties leg's fragment grammar language-neutral so consumer
corpora can declare a property's implementation language instead of
being Rust-bound by the marker: the tag set is closed and lint-shaped
(mirroring `property_kind_closed`), unknown tags fail labeled rather
than silently, and tags without an emitter gate compile honestly while
the emitters land in per-language follow-ups. Binding those declarations
to real tests in each language is espectacular's existing seam and is
out of scope here.

## Constraints

| id                     | kind      | expr                                                                                                                                                                                                                                                                                     | traces_to |
|------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| fragment_language_closed | invariant | `a Property's predicate cell (or an invariant-kind Constraint's expr cell) opts into executable translation with exactly one tag from the closed set {rust, py, ts} — the fragment-position rule (specodelic-sd1) and the non-empty-fragment requirement carry over verbatim per tag`      | [[compile]]  |
| unknown_tag_rejected   | invariant | `a marker whose tag is not in the closed set (e.g. **go:**) is a labeled extraction failure naming the tag and the closed set — never treated as prose, never extracted, never silently ignored (sd1 discipline, design D7)`                                                              | [[compile]]  |
| rust_back_compat       | invariant | `every predicate or expr cell that extracted under the pre-change **rust:** grammar extracts to the identical fragment post-change — the widening is pure: nothing valid before this Revision is invalidated`                                                                             | [[compile]]  |
| no_emitter_labeled_failure | invariant | `a fragment tagged **py:** or **ts:** compiles to a labeled extraction failure whose remediation names the missing emitter and its follow-up change — never a silent fall-through to Rust emission, never prose, never a compilable-but-vacuous artifact`                                  | [[compile]]  |
| mention_not_extraction | invariant | `a tag occurrence in any non-fragment position — mid-span, as in the defining rows of this very table, or in prose between spans — is a mention of the mechanism and never extracts, per tag, unchanged from the sd1 rule`                                                                | [[compile]]  |

## Model

### States
- `rust_only`
- `tagged`
- `archived`

### Transitions

| id     | from      | to        | guard                                                                                  |
|--------|-----------|-----------|----------------------------------------------------------------------------------------|
| widen  | rust_only | tagged    | `Revision lands: tag set closed over {rust, py, ts}, rust extraction byte-identical`    |
| approve | tagged   | archived  | `just archive-change id=add-language-neutral-property-binding (dual-format recipe)`     |

## Properties

| id                  | kind | derives_from                     | generator                                  | predicate                                                              |
|---------------------|------|-----------------------------------|---------------------------------------------|-------------------------------------------------------------------------|
| py_tag_extracts     | unit | [[compile.fragment_language_closed]] | `predicate_cell_with_py_fragment()`         | `fragment extracted under the py tag with cell text after the marker`   |
| go_tag_rejected     | unit | [[compile.unknown_tag_rejected]]     | `predicate_cell_with_go_marker()`           | `extraction fails labeled, naming the tag and the closed set`           |
| rust_unchanged      | unit | [[compile.rust_back_compat]]         | `pre_change_rust_fixtures()`                | `extracted fragment identical to the pre-change extraction, byte-for-byte` |
| py_no_emitter_fails | unit | [[compile.no_emitter_labeled_failure]] | `py_fragment_compiled()`                  | `compile fails labeled; remediation names the py emitter follow-up`     |
| midspan_mention_ignored | unit | [[compile.mention_not_extraction]] | `cell_with_midspan_py_marker()`           | `no fragment extracted; cell compiles exactly as before`                |
| expr_tag_same_grammar | unit | [[compile.fragment_language_closed]] | `invariant_expr_cell_with_py_fragment()`  | `expr cell accepts the same tag grammar as predicate cells`             |

## ADDED Requirements

### Requirement: Closed language-tag fragment opt-in
The fragment opt-in grammar SHALL accept exactly one tag from the closed
set `{rust, py, ts}` in fragment position, SHALL reject an unknown tag
with a labeled extraction failure naming the tag and the closed set, and
SHALL fail labeled — with a remediation hint naming the missing emitter
and its follow-up change — when a `py` or `ts` fragment reaches compile.

#### Scenario: Rust opt-in unchanged
- **WHEN** a predicate cell carries a `**rust:**` fragment under the pre-change grammar
- **THEN** the extracted fragment is byte-identical to the pre-change extraction

#### Scenario: Unknown tag rejected labeled
- **WHEN** a predicate cell carries `**go:**` (or any tag outside the closed set)
- **THEN** extraction fails labeled, naming the tag and the closed set — the cell is never prose and never silently ignored

#### Scenario: Tag without emitter gates compile honestly
- **WHEN** a fragment tagged `**py:**` reaches compile before the py emitter lands
- **THEN** compile fails labeled and the remediation hint names the py-emitter follow-up

#### Scenario: Mid-span occurrence is a mention
- **WHEN** a cell mentions `**py:**` mid-span (as the defining rows of this very table do)
- **THEN** nothing extracts and the cell compiles exactly as before

### Requirement: Pure widening
The grammar change SHALL invalidate nothing that was valid before it:
every cell that extracted under the pre-change `**rust:**` grammar
extracts identically after, and cells without tags compile exactly as
before.

#### Scenario: No-tag cell untouched
- **WHEN** a Property row's predicate carries no tag (the `todo_predicate!` placeholder path)
- **THEN** its compiled artifact is byte-identical to the pre-change artifact

#### Scenario: Invariant expr cells share the grammar
- **WHEN** an invariant-kind Constraint's expr cell carries a `**py:**` fragment
- **THEN** the same tag grammar and failure modes apply as for predicate cells

### Requirement: Properties table compiles to proptest blocks
The system SHALL compile each Property row to at least one `proptest!`
block — generator as input strategy, predicate as assertion body — and
a law-kind property to exactly one block per case enumerated in the
row's predicate in machine-findable `**name:**` label form. The
identity and associativity floor (`law_requires_cases`) is mandatory
and lint-enforced before compile (the compile precondition gate);
extra named cases compile as first-class checkable blocks, never prose.

#### Scenario: Law expands to enumerated cases
- **WHEN** a law-kind property row's predicate carries `**identity:**`,
  `**associativity:**`, and `**commutativity:**` labels
- **THEN** exactly three blocks are emitted for that row, one per
  enumerated case

#### Scenario: Unlabeled floor fails the precondition
- **WHEN** a law-kind property row's predicate states its cases in
  prose only (no `**name:**` labels)
- **THEN** the compile precondition gate reports the `linter.law_cases`
  finding naming the missing floor cases
- **AND** no artifact set is reported as compiled

## Requirements

### Requirement: Closed language-tag fragment opt-in
The fragment opt-in grammar SHALL accept exactly one tag from the closed
set `{rust, py, ts}` in fragment position, SHALL reject an unknown tag
with a labeled extraction failure naming the tag and the closed set, and
SHALL fail labeled — with a remediation hint naming the missing emitter
and its follow-up change — when a `py` or `ts` fragment reaches compile.

#### Scenario: Rust opt-in unchanged
- **WHEN** a predicate cell carries a `**rust:**` fragment under the pre-change grammar
- **THEN** the extracted fragment is byte-identical to the pre-change extraction

#### Scenario: Unknown tag rejected labeled
- **WHEN** a predicate cell carries `**go:**` (or any tag outside the closed set)
- **THEN** extraction fails labeled, naming the tag and the closed set — the cell is never prose and never silently ignored

#### Scenario: Tag without emitter gates compile honestly
- **WHEN** a fragment tagged `**py:**` reaches compile before the py emitter lands
- **THEN** compile fails labeled and the remediation hint names the py-emitter follow-up

#### Scenario: Mid-span occurrence is a mention
- **WHEN** a cell mentions `**py:**` mid-span (as the defining rows of this very table do)
- **THEN** nothing extracts and the cell compiles exactly as before

### Requirement: Pure widening
The grammar change SHALL invalidate nothing that was valid before it:
every cell that extracted under the pre-change `**rust:**` grammar
extracts identically after, and cells without tags compile exactly as
before.

#### Scenario: No-tag cell untouched
- **WHEN** a Property row's predicate carries no tag (the `todo_predicate!` placeholder path)
- **THEN** its compiled artifact is byte-identical to the pre-change artifact

#### Scenario: Invariant expr cells share the grammar
- **WHEN** an invariant-kind Constraint's expr cell carries a `**py:**` fragment
- **THEN** the same tag grammar and failure modes apply as for predicate cells

### Requirement: Properties table compiles to proptest blocks
The system SHALL compile each Property row to at least one `proptest!`
block — generator as input strategy, predicate as assertion body — and
a law-kind property to exactly one block per case enumerated in the
row's predicate in machine-findable `**name:**` label form. The
identity and associativity floor (`law_requires_cases`) is mandatory
and lint-enforced before compile (the compile precondition gate);
extra named cases compile as first-class checkable blocks, never prose.

#### Scenario: Law expands to enumerated cases
- **WHEN** a law-kind property row's predicate carries `**identity:**`,
  `**associativity:**`, and `**commutativity:**` labels
- **THEN** exactly three blocks are emitted for that row, one per
  enumerated case

#### Scenario: Unlabeled floor fails the precondition
- **WHEN** a law-kind property row's predicate states its cases in
  prose only (no `**name:**` labels)
- **THEN** the compile precondition gate reports the `linter.law_cases`
  finding naming the missing floor cases
- **AND** no artifact set is reported as compiled

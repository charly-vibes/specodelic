# Add: migrate --to-revision — delta for migrate

## ADDED Requirements

### Requirement: Upgrade a corpus across the known format revisions
The system SHALL support `spk migrate --to-revision <N> [--from-revision
<R>] [--write] <paths...>`: the upgrade SHALL resolve the known format
revisions from the core-owned registry, SHALL apply the registered
migration steps for the revisions between the corpus's source revision
and the target revision in ascending revision order, and SHALL report
the outcome as envelope data. The upgrade SHALL NOT rewrite a file that
no registered step changes.

#### Scenario: Old-revision corpus upgrades to the target revision
- **WHEN** `spk migrate --to-revision 20 --write <corpus>` runs on a corpus whose capability file still declares the pre-Revision-18 `id: spec` shape
- **THEN** that file declares its parent-directory-derived id, its `[[spec…]]` refs read `[[<derived-id>…]]`, the corpus lints with zero findings, and the files that needed no step are byte-identical

#### Scenario: The source revision is declared or detected
- **WHEN** the upgrade runs over a corpus holding one file that still declares the pre-Revision-18 `id: spec` shape
- **THEN** the corpus's source revision resolves to the oldest revision the supplied files still satisfy, or to the revision declared by `--from-revision`, and the envelope data names the resolved source and target revisions

#### Scenario: Dry-run upgrade writes nothing
- **WHEN** `spk migrate --to-revision 20 <corpus>` runs without `--write`
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content of every file the upgrade would change

#### Scenario: The upgrade never touches files outside the supplied paths
- **WHEN** the upgrade runs over one directory of a corpus
- **THEN** no file outside that directory changes and no file is created beside the upgraded ones

#### Scenario: Conflicting modes are refused
- **WHEN** the upgrade is asked for together with `--rekey` (a different mode of the verb), or with both `--write` and `--dry-run`
- **THEN** the command fails (exit 2) with a labeled refusal, neither flag is silently ignored, and no file is changed

### Requirement: The revision registry is core-owned and thin
The system SHALL resolve the known format revisions from a core-owned
registry that records, for each known revision, its number, its date
(where the format corpus records one), a one-line change note, and an
OPTIONAL migration step — a revision whose change required no corpus
rewrite SHALL carry no step and SHALL upgrade as a byte-stable no-op.

#### Scenario: Known revisions resolve from the registry
- **WHEN** the upgrade resolves a target revision
- **THEN** the registry names the known revision range and carries a date and a change note for every known revision it records

### Requirement: Refusals label the unknown or impossible revision
An unknown source or target revision SHALL be refused (exit 2) with a
labeled message naming the known revision range and a remediation hint;
a target revision below the corpus's current revision SHALL be refused
(exit 2) with a labeled message naming the current revision and the
file that still carries the older shape. No file SHALL be written by a
refused upgrade.

#### Scenario: Unknown target revision is refused
- **WHEN** the upgrade is asked for a target revision outside the known range
- **THEN** the command fails (exit 2) with a labeled refusal naming the known revision range and a remediation hint, and no file is changed

#### Scenario: Unknown source revision is refused
- **WHEN** the upgrade is given a `--from-revision` outside the known range
- **THEN** the command fails (exit 2) with a labeled refusal naming the known revision range and a remediation hint, and no file is changed

#### Scenario: Target below the current revision is refused
- **WHEN** the upgrade is asked for a target revision below the corpus's current revision
- **THEN** the command fails (exit 2) with a labeled refusal naming the current revision and the file that still carries the older shape, and no file is changed

### Requirement: The upgrade is atomic, byte-stable and scope-confined
The upgrade SHALL write only the files whose content a registered step
changed, SHALL leave every other file byte-identical (its line endings
and final terminator preserved), SHALL create no file, and SHALL write
nothing at all when any file in the supplied set cannot be upgraded.

#### Scenario: Unchanged files stay byte-identical
- **WHEN** the upgrade runs over a corpus whose files need no step
- **THEN** every file's bytes are unchanged — CRLF line endings and the absence of a final terminator are preserved exactly

#### Scenario: A failed upgrade writes nothing
- **WHEN** the upgrade runs over a set in which one file cannot be upgraded
- **THEN** the command fails (exit 2) and every file in the set is unchanged

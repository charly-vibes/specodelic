## ADDED Requirements

### Requirement: Artifact locations share one routing decision

`spk verify` SHALL resolve every artifact location it consumes — the
`<stem>_props.rs` properties source, the `<stem>.check.json` model-check
report, every `<stem>.tla` module the scope digest hashes, and the digest's
per-file hashes — through `citation_corpus::artifact_dir`, the one shared
per-file routing decision, so a duplicate-stem corpus routes per file under
`out_dir/<stem>/<encoded source path>` injectively while a corpus-unique stem
keeps the flat layout byte-identically.

#### Scenario: Duplicate stems read their own report

- **WHEN** a duplicate-stem corpus is verified with no reports on disk
- **THEN** each file's model gate names its own routed
  `<stem>.check.json` and never the flat path a same-stem sibling would
  collapse onto

#### Scenario: Scope digest hashes routed modules

- **WHEN** the duplicate-stem corpus has reports present and modules
  absent from disk
- **THEN** the scope digest names the first routed `<stem>.tla` module
  and never the flat `<stem>.tla` — no cross-read of a sibling's module

#### Scenario: A sibling's artifacts never satisfy a gate

- **WHEN** only one same-stem file of a duplicate-stem corpus has
  artifacts on disk
- **THEN** the other file's gates stay missing or stale — the sibling's
  evidence is never read as this file's

#### Scenario: Unique stems keep the flat layout

- **WHEN** every file in the invocation has a distinct artifact stem
- **THEN** the consumed paths are `<out_dir>/<stem>_props.rs`,
  `<out_dir>/<stem>.check.json`, and `<out_dir>/<stem>.tla`, unchanged,
  and a single-spec entry point treats its spec as its own corpus

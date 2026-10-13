# Add: skill-pack — delta for the `spk skill install` verb and managed pack

## ADDED Requirements

### Requirement: Pack content is embedded from single tracked sources

The `specodelic-design` skill pack SHALL be generated from content
embedded in the binary via `include_str!` over tracked files in this
repository (`skill-packs/specodelic-design/SKILL.md` for the protocol
prose, `docs/src/examples/reservation-order.md` for the worked-example
reference), and the format reference SHALL be rendered by the same
in-binary machinery that serves the `spk explain` topics it summarizes.
The pack SHALL NOT contain a hand-copied copy of any explain table, lint
rule list, or worked example. Generation SHALL be deterministic: two
generations without an intervening source change produce byte-identical
files.

#### Scenario: Deterministic regeneration

- **WHEN** the pack generator runs twice without an intervening source
  or binary change
- **THEN** the two generated file sets are byte-identical, including the
  embedded example and the rendered topic and rule-name lists

#### Scenario: No drift class in the pack

- **WHEN** the served `spk explain` topic list or lint rule catalog
  changes in the binary
- **THEN** the next generated pack reflects the new list without any edit
  to the skill source, because the list is rendered from the served
  tables rather than restated in them

### Requirement: Mentioned identifiers are membership-tested

Every identifier mentioned in a generated pack file SHALL resolve against the in-binary tables that serve it. Specifically: every `spk explain` topic name, lint rule id, and EARS pattern name
mentioned in a generated pack file SHALL resolve against the in-binary
tables the `spk explain` command serves (topic list, lint rule catalog,
EARS grammar). This SHALL be enforced by an in-binary test that fails the
build when a generated mention does not resolve — a hand-maintained list
may be stale, a generated one cannot.

#### Scenario: Unresolvable rule mention fails the build

- **WHEN** the skill source names a lint rule id that is not present in
  the linter rule catalog table
- **THEN** the in-binary membership test fails, naming the unresolvable
  identifier, and the binary cannot ship with the dangling mention

### Requirement: The embedded example lints clean

The `references/example.md` file of a generated pack SHALL embed the
worked example from `docs/src/examples/reservation-order.md` verbatim and
SHALL pass `spk lint` over that example with zero findings and zero
warnings. This SHALL be enforced by an in-binary test that lints the
generated example file with the shipped linter.

#### Scenario: Generated example passes the shipped linter

- **WHEN** the in-binary lint test runs over the generated
  `references/example.md`
- **THEN** the linter reports zero issues and zero warnings, and any
  corpus change that would make the embedded example lint-dirty fails
  the test before release

### Requirement: Installed files carry the managed marker

Every file written by `spk skill install` SHALL begin with a managed
marker block that names the owning tool and the exact regenerate command
(`<!-- MANAGED BY specodelic — DO NOT EDIT MANUALLY. Regenerate: spk skill install -->`),
and the marker SHALL be present on every generated file even when that
file is a verbatim embedding of a repo source that lacks it.

#### Scenario: Marker survives verbatim embedding

- **WHEN** the generator writes the example reference that is a verbatim
  copy of a repository source file
- **THEN** the installed file starts with the managed marker block while
  the repository source remains marker-free

### Requirement: Install verb writes the pack and reports its hash

`spk skill install [--dir <dir>]` SHALL write the generated pack files
under the given directory (default `~/.claude/skills`, creating parent
directories), SHALL refuse to overwrite pre-existing non-marker-bearing
files at a target path instead of silently replacing them, and SHALL emit
a JSON envelope (human-readable on TTYs) listing the written paths and a
deterministic `pack_content_hash` computed as SHA-256 over the pack
files sorted by relative path, each hashed as `path` + NUL + `content`.
Exit codes follow the error contract: 0 installed, 1 write failure, 2
invalid invocation or overwrite refusal.

#### Scenario: Install into an empty directory

- **WHEN** `spk skill install --dir <dir>` runs against a directory that
  does not exist or contains no conflicting files
- **THEN** the pack files are written, the envelope lists exactly those
  paths with the managed marker files' hashes, and the exit code is 0

#### Scenario: Overwrite refusal

- **WHEN** a target path already holds a file without the managed marker
- **THEN** the verb refuses with exit 2 and a remediation hint naming the
  conflicting path, leaving the file untouched

### Requirement: Doctor reports managed-pack staleness

`spk doctor` SHALL include a managed-skill-pack check that recomputes the
generated pack hash and compares it against the on-disk installation under
each configured target directory: `pass` when the hashes match, `stale`
when an installed pack's hash differs from the generated one, and
`missing` when no pack is installed. The check SHALL be optional: a
missing pack is reported `missing` with a regenerate hint and SHALL NOT
fail the doctor run, while a `stale` pack SHALL be reported with the
regenerate command and the differing hash pair.

#### Scenario: Stale installed pack is flagged

- **WHEN** a pack installed from an older binary is checked by the
  current binary's doctor
- **THEN** the report contains a `stale` entry naming the target
  directory, the installed hash, the expected hash, and the regenerate
  command `spk skill install`

#### Scenario: Absent pack is not a failure

- **WHEN** no managed pack exists at any target directory
- **THEN** the doctor report contains a `missing` entry with the install
  hint and the doctor exit code is unchanged by the check's absence

### Requirement: Docs wire the pack as a thin pointer

The repository docs SHALL reference the skill pack by pointer only: a
thin book page (`docs/src/design-conversations.md`) links the conversation
procedure to its tracked source and the install command, and the README
command surface, `docs/src/SUMMARY.md`, and `llms.txt` SHALL name the
`spk skill install` verb — none of these pages SHALL duplicate the
protocol prose, the self-check list, or the example.

#### Scenario: No duplicate protocol in the book

- **WHEN** the docs-site capability checks run over the book
- **THEN** no book page contains a copy of the pack's protocol prose or
  worked example beyond the thin-pointer page's quoted one-line summary,
  keeping a single authoring surface for the procedure

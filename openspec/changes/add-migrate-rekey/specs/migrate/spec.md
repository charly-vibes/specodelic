# Add: migrate --rekey — delta for migrate

## ADDED Requirements

### Requirement: Rekey an id:spec dual-format file to its real id
The system SHALL support `--rekey` on the migrate command: a file
carrying frontmatter with `id: spec` SHALL be rewritten in place with
the real id derived by the naming law — the parent directory name for a
`spec.md` file (`-` ⇔ `.`, `_` literal) — every `[[spec]]` reference
re-keyed to `[[<derived-id>]]` and every `[[spec.` prefix re-keyed to
`[[<derived-id>.`, and every other byte of the file left untouched. A
file without frontmatter SHALL be refused (it is a plain delta — the
wrap path's job); a bare `spec.md` with no parent directory SHALL be
refused (no derivable id); a file whose id is already not `spec` SHALL
be a no-op with a warning, never rewritten, so a directory-wide rekey
sweep is idempotent.

#### Scenario: Legacy dual-format file rekeys
- **WHEN** `spk migrate openspec/specs/ge-cli/spec.md --rekey` runs on a file declaring `id: spec` with `[[spec.c1]]` refs
- **THEN** the file declares `id: ge.cli`, its refs read `[[ge.cli]]` / `[[ge.cli.c1]]`, and everything else is byte-identical

#### Scenario: Refusals never rewrite
- **WHEN** `--rekey` runs on a plain delta (no frontmatter) or a bare `spec.md` with no parent directory
- **THEN** the command fails (exit 2) with a labeled refusal naming the right path (the wrap path, or an unguessable id) and the file is unchanged

#### Scenario: Rekey sweep is idempotent
- **WHEN** `--rekey` runs twice, or on a file already carrying its real id
- **THEN** the second run is a success with a no-op warning and the file is unchanged

#### Scenario: Dry-run rekey writes nothing
- **WHEN** `spk migrate <file> --rekey --dry-run` runs
- **THEN** the on-disk bytes are unchanged and the envelope data carries the would-be content

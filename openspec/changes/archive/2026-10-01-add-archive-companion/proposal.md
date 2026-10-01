# Change: Add `spk archive-companion` — make the dual-format layer survive the openspec archive round-trip

## Why

`openspec archive` (v0.19.0) regenerates `openspec/specs/<cap>/spec.md`
from the parsed deltas and drops the specodelic half — frontmatter and
the Constraints/Model/Properties tables do not survive. Every archive
touching a capability silently reverts its deployed spec to plain
openspec grammar. This repo already holds two mitigations, both
repo-local: the hardened `just archive-change` recipe
(`openspec archive <id> --skip-specs` + verbatim cp of the archived
deltas — the dual-format archive recipe from the 2026-09-28 spike) and
the capability-format CI check that makes a silent strip non-silent.

GH#7 (filed by espectacular via `spk feedback`) asks for the mechanism
at the tool level. Guidance-only is already served by
`spk explain dual-format` (the migration recipe); the delta this ticket
adds is the tool itself. Decision of record: promote the recipe into a
`spk` subcommand — it is the verified, byte-exact path; a post-hoc
restore command that re-derives the layer from a stripped deployed spec
is the fragile alternative and is explicitly out of scope.

## What Changes

- **Tool — `spk archive-companion <CHANGE_ID>`:** new subcommand that
  archives an openspec change without destroying the dual-format layer:
  1. `openspec archive <id> --skip-specs --yes` (skipped when the change
     is already under `openspec/changes/archive/` — idempotent re-run);
  2. locate the newest archive directory matching `*-<id>`;
  3. for each `specs/<cap>/spec.md` inside it: verify the dual-format
     layer is present (frontmatter + `## Constraints`), then copy it
     verbatim to `openspec/specs/<cap>/spec.md`;
  4. emit the envelope: which steps ran, which files were restored.
- **Fail-closed dual-format guard:** a delta lacking the specodelic
  layer is refused with a remediation hint pointing at the
  `spk explain dual-format` migration recipe — the companion never
  deploys a stripped spec (the failure the capability exists to prevent).
- **`--dry-run`:** resolve and report the restore plan without invoking
  `openspec` or writing any file.
- **Just recipe delegation:** `just archive-change` keeps its interface
  but the semantics it encodes now live in the tool; the recipe may
  delegate to `spk archive-companion` (recipe kept as the CI-stable
  entry point).
- **Out of scope:** post-hoc repair of an already-stripped deployed spec
  (git-history row recovery is a human task with the migration recipe);
  upstream openspec changes (guidance lives in `spk explain dual-format`).

## Impact

- **Affected specs:** new capability `archive-companion` (ADDED
  requirements only; no existing capability modified).
- **Affected code:** `src/archive_companion.rs` (new module),
  `src/main.rs` (CLI arm + dispatch), `justfile` (recipe delegation),
  `docs/src/commands.md`, `specs/CHANGELOG.md`.

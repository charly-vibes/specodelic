# Change: Add `spk migrate` — wrap an openspec delta into the dual-format skeleton

## Why

Adopters migrating openspec deltas to dual format (gh#6 item 1 — 25 deltas
converted by hand) found the migration recipe fully manual: frontmatter,
empty layers, and above all the byte-identical `## Requirements` mirror,
which invites fragile sed one-liners. The recipe is documented
(`spk explain dual-format`) but there is no tool surface for it. `spk new`
scaffolds a plain spec; nothing wraps an EXISTING delta in place.

## What Changes

- New `spk migrate <spec.md>` command: wraps an existing openspec delta
  file in place into the dual-format four-layer skeleton.
- Generated scaffold lints clean as-is: frontmatter (`id: spec`, EARS
  statement stub — only when the file has none), a wired minimal layer set
  (scaffold Constraint, one-state Model, placeholder Property), and a
  byte-identical `## Requirements` mirror of `## ADDED Requirements`.
- Merge semantics: sections already present are kept verbatim; only
  missing pieces are added. A file already carrying the mirror is refused,
  never rewritten.
- `--dry-run` prints the resulting content without writing.
- Anti-goal (unchanged): no auto-derivation of constraints/properties
  from prose — the scaffold is a skeleton, the agent fills it.

## Impact

- **specs**: NEW capability `migrate` (dual-format delta).
- **code**: new `src/migrate.rs` (pure `migrate()` seam + scaffold
  constants), CLI arm in `src/main.rs`.
- **docs**: `docs/src/commands.md` gains a migrate section; CHANGELOG entry.
- Surface decided at issue-review 2026-09-29: `spk migrate <file>` — the
  `spk new --dual-format` alternative is dropped (one command, one job).

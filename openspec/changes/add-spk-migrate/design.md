# Design: spk migrate

## Context

The dual-format migration recipe (openspec/project.md, `spk explain
dual-format`) is manual. gh#6 item 1 asks for a tool surface. Surface
decided: `spk migrate <spec.md>` in place.

## Decisions

**D1 — pure seam.** `migrate::migrate(text: &str) -> Result<String,
MigrateError>` does all parsing/transformation on in-memory text; the CLI
arm only reads/writes the file and emits the envelope. Unit tests target
the seam; integration tests drive the binary.

**D2 — the mirror is the migration marker.** Refusal rule: a file that
already has BOTH frontmatter and `## Requirements` is already dual-format
→ refuse (exit 2, hint names the lint to verify it). A file with
`## Requirements` but no `## ADDED Requirements` is a plain spec, not a
delta → refuse. A file with neither is refused too (nothing to wrap).
This gives the idempotence the ticket demands: re-running on a migrated
file can never duplicate the mirror.

**D3 — merge adds only what is missing.** Missing frontmatter → insert
generated one (id `spec` per the naming law, kind intent, EARS scaffold
statement). Existing frontmatter is kept VERBATIM even when its id is not
`spec` (never destroy; warn that deltas conventionally carry `id: spec`).
Missing Constraints/Model/Properties layers → insert the wired scaffold
(D4). Existing layers are kept untouched — no row injection into
hand-authored tables. Missing mirror → append `## Requirements` with a
byte-identical copy of the `## ADDED Requirements` section body.

**D4 — the scaffold must lint clean.** An unwired skeleton does NOT lint
clean: an empty Model section fails `model_sections_paired` (0 states), a
stateless Model strands nothing but an empty Constraints layer fails
coverage, unwired rows form orphaned islands. The scaffold is therefore
wired end-to-end with placeholder rows whose ids carry the `scaffold_`
prefix: one invariant Constraint (`scaffold_constraint`, traces_to
`[[spec]]`), one draft state, one self-transition guarded by
`[[spec.scaffold_constraint]]`, one unit Property deriving from it. The
output envelope lists the hand-finish steps (replace scaffold ids, derive
real constraints/model/properties) and hints `spk lint <file>`. This is
scaffolding, not derivation — the anti-goal holds because every inserted
row is a marked placeholder, never inferred from prose.

**D5 — no backup files.** `--dry-run` prints the resulting content as
envelope data and writes nothing. Default writes in place; git is the
undo path (same stance as rename: all-or-nothing, no .bak litter).

**D6 — byte fidelity.** The mirror body is a byte-exact slice of the
`## ADDED Requirements` section body (from after the heading line to the
next `## ` heading or EOF). Inserted content uses the file's dominant
line ending (CRLF-safe, matching rename's discipline). A file lacking a
trailing newline gets one added; a file that already ends clean is not
double-spaced.

## Risks / Trade-offs

- The wired scaffold inserts four placeholder rows the author must
  replace; accepted because an UNwired scaffold would lint-fail on first
  run and teach the wrong baseline (worse onboarding, not better).
- Frontmatter-less deltas are currently skipped by lint (no frontmatter →
  not a spec file), so the "before" state is invisible to the linter;
  migrate's refusal messages carry the diagnosis instead.

## Migration Plan

Additive command; no existing surface changes. Deltas remain hand-migrable.

## Open Questions

None — surface and dependency (specodelic-hl8, closed) were settled at
issue-review 2026-09-29.

# Decision: openspec-validate gate is version-guarded on the 0.19.0 pin

- **Date**: 2026-10-19
- **Status**: Accepted
- **Tickets**: beads specodelic-uv3v · GH-20
- **Scope**: repo tooling (`justfile`, `scripts/spec_validate.py`) — no change to
  `openspec/specs/` or `openspec/changes/` (read-only rule untouched)

## Context

The org-wide openspec upgrade sweep installs `@fission-ai/openspec@1.x` on dev
machines. Under `1.14.2 --strict`, every dual-format file fails validation:
the byte-identical `## Requirements` mirror registers as orphaned
requirements, and the Specodelic Constraints/Model/Properties tables register
as uncarried content. Both are *required* by `spk lint`
(`linter.dual_format_valid`) and by the archive-companion contract — the two
tools' laws are directly opposed (GH-20, sunset-rtl sweep: 3/3 dual-format
changes fail under 1.14.2; all deployed plain specs pass both).

Locally the same holds: 46/46 deployed capability specs are dual-format and
25/51 validation items fail under 1.14.2, while 0.19.0 passes 51/51. 1.x has
no suppression config (checked 1.14.2 dist: `createReport` fails strict on
any WARNING; the project config schema has no validation-rule knobs).

CI was already safe (`ci.yml` installs `@fission-ai/openspec@0.19.0`
globally); the exposure was local/dev runs where bare `openspec` resolves to
whatever the sweep installed.

## Decision

`just openspec-validate` runs `scripts/spec_validate.py`:

1. Detect the installed `openspec --version`.
2. If it is in the ratified channel (0.19.x, caret semantics), run
   `openspec validate --all --strict --no-interactive` directly.
3. Otherwise run the same command through the pinned
   `npx -y @fission-ai/openspec@0.19.0`, preceded by a labeled note; an
   undetectable or missing binary also takes the pinned fallback (a missing
   detection must never silently run the wrong validator).
4. On non-zero exit, print a remediation hint naming the pin and the
   GH-20/uv3v reason.

The gate's verdict tracks corpus content, never the ambient npm state.

## Rejected alternatives

- **Format-revision accommodation now** (restructure the dual-format layer so
  1.x validates it): a corpus-wide format change plus archive-companion
  contract change — disproportionate before an upstream suppression
  mechanism exists or a format decision record rules on the 1.x layout.
- **Advisory 1.x validation run alongside the pin**: 46/46 deployed specs are
  dual-format, so a 1.x pass would report near-total failure as pure noise on
  every run; dropped until an accommodation exists.

## Consequences

- 1.x upgrade work stays blocked on the spk-side accommodation (tracked as
  the GH-20/uv3v follow-up decision: companion-contract change, scoped
  validation split, or upstream openspec suppression mechanism).
- CI keeps its explicit 0.19.0 install; the script adds no new CI
  prerequisite (node/npx already present).
- When an accommodation lands, the fallback note in
  `scripts/spec_validate.py` is the single place to retire the pin.

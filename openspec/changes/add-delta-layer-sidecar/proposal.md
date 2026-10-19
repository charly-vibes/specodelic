# Change: add-delta-layer-sidecar — plain deltas carry their specodelic layer beside them

## Why

The org-wide openspec 1.x sweep cannot complete: 1.x `--strict` rejects the
dual-format delta shape (a `## Requirements` mirror is "ignored", and the
Constraints/Model/Properties tables are "not carried"), which is why
`scripts/spec_validate.py` is version-guarded on the 0.19 pin (decision
2026-10-19-openspec-validate-pin.md). Stage 1 (specodelic-s8fg, delta-section
strip) already made every deployed capability spec 1.x-strict-valid; the last
3 failing items are the dual-format deltas of the active changes
(add-graph-views, add-py/ts-fragment-emission). GH-20 stays blocked until
change deltas are 1.x-valid too.

## What Changes

- **Change deltas become plain openspec deltas** — no frontmatter, no
  specodelic tables, no `## Requirements` mirror. The specodelic layer
  (frontmatter + Constraints/Model/Properties) moves to a sidecar file
  `layer.md` beside the delta under `openspec/changes/<id>/specs/<cap>/`.
- **Naming law gains the sidecar arm** — a `layer.md` file derives its
  expected id from its parent directory plus a `.layer` suffix
  (`openspec/changes/<id>/specs/cap-a/layer.md` → `id: cap.a.layer`);
  the sidecar is a real, corpus-linted specodelic file. **Format Revision 19.**
- **The mirror retires by construction** — `requirement_drift`,
  `dual_format_valid`'s delta-pairing arm, `migrate.mirror_byte_identical`,
  and spec-integration's "Section sync" police shapes that plain deltas can
  no longer produce. The dual-format *capability spec* class (deployed
  `openspec/specs/<cap>/spec.md`) stays; none of its lint rules change.
- **archive-companion re-assembly** — the companion invokes `openspec
  archive` with spec application ON (plain deltas are openspec's native
  merge; `openspec_not_forked` is preserved and the companion still re-
  implements no merging), then re-attaches the dual-format layer to the
  regenerated deployed spec from the archived sidecar. Fail-closed: a delta
  without a sidecar is refused before the destructive archive runs; the
  post-attach shape assertion (no delta headers, frontmatter + tables
  present) replaces the strip-time guards.
- **migrate scaffold teaches the new shape** — `spk migrate` wraps a plain
  delta into (plain delta unchanged) + generated `layer.md` sidecar instead
  of a dual-format file; `--rekey` behavior unchanged.
- **The 0.19 version-guard retires** — once every corpus item is 1.x-clean,
  `scripts/spec_validate.py` collapses to a plain `openspec validate --all
  --strict` (1.x becomes THE gate, not the pin).
- **The 3 remaining dual-format deltas migrate** to the plain delta +
  sidecar shape; the `/tmp/os1x` 1.14.2 fixture experiments are checked in
  as a regression fixture.

## Impact

- Affected specs: `spec-integration` (mirror rules retire), `archive-companion`
  (deploy semantics), `migrate` (wrap semantics). Format corpus
  `openspec/specs/specodelic/spec.md` gains Revision 19 directly (its
  openspec `## Requirements` section is unchanged).
- Affected code: `src/archive_companion.rs`, `src/migrate.rs`, the lint
  rules `requirement_drift` and `dual_format_valid` (delta arm),
  `scripts/spec_validate.py`, `justfile`, `.github/workflows/ci.yml`, root
  `AGENTS.md` (dual-format archive recipe + espectacular constraint), the
  embedded guide primer + `FORMAT_REVISION`.
- Sequencing: lands before `update-id-derivation-spec-md` and
  `add-migrate-rekey` archive — their plain deltas are already Stage-2
  shaped and become the first users of the re-assembling companion.
- Fold-in: specodelic-dlf (MODIFIED change for migrate mirror wording) is
  subsumed here; specodelic-3le9 (verbatim-deploy wording) is restated in
  the archive-companion delta below.

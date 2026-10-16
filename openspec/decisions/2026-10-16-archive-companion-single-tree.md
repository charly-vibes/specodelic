# Decision: archive-companion write path — redefine (post-deploy strip + re-run recognition)

- **Date**: 2026-10-16
- **Ticket**: specodelic-bhp (P2 rollout; parent context: specodelic-mcy
  rollout steps 4+6, post single-tree migration)
- **Status**: **ADOPTED — redefine, not retire.** The companion keeps the
  sanctioned write path (`openspec archive <id> --skip-specs` + fail-closed
  dual-format pre-check + verbatim deploy) and gains the post-deploy
  normalization step (D9): strip the frontmatter block from each archived
  delta copy. Retiring the companion (Option B) is rejected.

## Options considered

- **Option A — redefine** (adopted): the verbatim deploy remains the
  sanctioned write path (the sibling-tool constraints in AGENTS.md name it
  as the only sanctioned write path for capability specs); the companion
  additionally owns the manual post-archive strip that every archive since
  the quant-finance pack had to redo by hand (the "34-issue trap": a
  frontmattered archived `spec.md` collides with the deployed spec's id —
  a `spec.md` derives its expected id from its parent directory, so the
  archived copy carries the same id as the deployed capability spec).
- **Option B — retire the companion, manual recipe only**: rejected. The
  fail-closed pre-check exists precisely because a human-in-the-loop `cp`
  already deployed a stripped spec once (specodelic-fzo / GH#7); the
  single-tree world did not remove that failure mode, it added a new
  normalization step that is just as easy to forget by hand.

## The redefinition (design D9)

1. **Post-deploy strip**: after a successful `restore()`, the companion
   strips the frontmatter block from each archived delta copy under
   `openspec/changes/archive/<date-id>/specs/<cap>/spec.md`, returning it
   to the openspec-archival shape. Idempotent (already-stripped deltas are
   unreported); the deployed spec keeps its dual-format layer verbatim
   (D4 unchanged).
2. **Re-run recognition** (replaces blind re-copy in D5): a delta lacking
   the dual-format layer is skipped (not copied) when its deployed target
   already exists and carries the layer — the post-normalization state a
   prior run left behind. Any other unverifiable delta is still refused
   before any copy (D3 unchanged). Without this, a post-strip re-run
   would either fail-closed on its own normalization output or, worse,
   re-copy a stripped delta over the dual-format spec.
3. **Dry-run unchanged** (D7): resolves and verifies, never strips.

## Facts that decided it

1. All four archived deltas in this repo's history
   (2026-10-08-add-quant-finance-pack, 2026-10-13-add-skill-pack,
   2026-10-14-add-svc-contracts-pack, 2026-10-16-add-conform) are
   frontmatter-less in the archive while the active change deltas carry
   frontmatter — every archive performed the strip manually.
2. The trap reproduces mechanically: linting a frontmattered `spec.md`
   delta clone reports `linter.id_matches_file` (expected id derived from
   the parent directory, not the carried frontmatter id).
3. Tambor/espectacular contract re-key (mcy step 4) verified moot: tambor's
   `.espectacular` contracts carry no `[[spec.*]]` anchors (they key on
   `[[tests.shell]]`/`[[tests.vitest]]` flags), `ah check` on tambor is
   clean (0 findings), and espectacular's own contracts already key on
   real-id `[[cli.*]]`-style anchors post-migration. No re-key work exists.
4. `tools/deploy_specs.py`-style transforms survive in exactly one repo
   (gently, which has not yet done its own mcy step-3 migration); filed as
   gently beads work — specodelic's own transforms (the verbatim companion
   deploy is not a transform) stay.

## Consequences

- The archive flow loses its last manual post-step; `just archive-change`
  is now the complete recipe.
- `RestoreOutcome`/the `spk archive-companion` envelope gain a `stripped`
  list (relative paths); the human summary notes the count.
- Sibling-tool constraint updated in effect: the "verbatim cp" recipe in
  AGENTS.md's espectacular section is satisfied by `spk archive-companion`
  itself (it already was; the strip is a post-step on the archive copy,
  never on the deployed spec).

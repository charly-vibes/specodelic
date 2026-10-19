# Design: add-delta-layer-sidecar

## Context

s8fg asked for a decision between (a) spk-side format accommodation, (b)
scoped validation split, (c) upstream suppression. Decision: **(a)**, with
the shape fixed by the 1.14.2 fixture experiments of the 2026-10-19 session
(`/tmp/os1x`, recreated as a checked-in fixture in task 6):

- **Deployed-spec shape** (`openspec/specs/<cap>/spec.md`: frontmatter +
  `## Purpose` + `## Constraints` + `## Requirements`, no delta headers) —
  1.x-strict-valid (Stage 1 proved this corpus-wide).
- **Plain delta** (`## Purpose` + `## ADDED/MODIFIED/REMOVED Requirements`,
  no frontmatter, no layer tables) — 1.x-strict-valid.
- **Sidecar layer** (`layer.md` beside the delta: frontmatter + tables,
  `id: <cap>.layer`) — ignored by openspec entirely (not a delta file), so
  it needs no openspec accommodation at all.

Alternative (b) (two gate regimes) was rejected: drift between the 1.x gate
and the 0.19 pin is exactly the failure class the single-tree revision
discipline exists to prevent. Alternative (c) was rejected: 1.14.2 has no
suppression/ignore config, and waiting on upstream blocks the org-wide
sweep indefinitely.

## Decision 1 — the sidecar is a real specodelic file, not a comment block

`layer.md` carries frontmatter (`id: <cap>.layer`, `kind: intent`, EARS
statement) and the Constraints/Model/Properties tables, fully linted by
`spk lint openspec` (corpus-wide ref resolution). This is why the naming law
needs the `layer.md` arm (Revision 19): the stem law would expect
`id: layer`, colliding with every other sidecar. Deriving from the parent
directory keeps sidecars per-capability unique and co-archived with their
delta. Model sections stay per-file (`every_state_used`,
`single_root_reachable` resolve inside the sidecar); sidecar traces_to may
cite deployed capability rows corpus-wide (Revision 18 typing).

## Decision 2 — companion: openspec applies, companion re-attaches

Stage 1's companion skipped spec application and deployed a stripped copy
because the delta was dual-format (openspec's apply would have regenerat-
ed the deployed spec from the dual delta and destroyed the layer). With
plain deltas that reason is gone — openspec's apply is the native,
correct merge for ADDED/MODIFIED/REMOVED, and it is not re-implemented
(`openspec_not_forked` holds, now more strongly: the companion does even
less). The companion then re-attaches the dual-format layer to the
regenerated deployed spec:

1. Refuse (before invoking openspec — the archive is destructive) any
   affected capability that has no sidecar in the archived change, or whose
   sidecar fails `spk lint`/parse.
2. Invoke `openspec archive <id>` (default spec application; the change dir
   moves to `changes/archive/<date>-<id>/`, sidecar rides along).
3. For each affected cap: merge the archived sidecar's frontmatter +
   Constraints/Model/Properties tables + the deployed target's `## Purpose`
   with openspec's applied `## Requirements` section, write
   `openspec/specs/<cap>/spec.md`.
4. Post-attach shape assertion (fail-closed, EDGE-001's successor): the
   written deployed file must carry no delta headers, must carry
   frontmatter + all three tables, and must lint clean — or the write is
   reverted and the command fails.

The dual-era guards retire with their subject: `delta_self_contained`
(self-containedness) existed because the companion clobbered instead of
merging; existing-target coverage guarded the clobber; both are superseded
by openspec's native merge. Idempotent rerun and newest-archive-wins are
unchanged in spirit (rerun re-resolves from the archive, re-attaches, and
must converge to the identical deployed bytes).

## Decision 3 — mirror retirement is by-construction, not by-deletion

`requirement_drift`, `dual_format_valid`'s delta-pairing arm,
`migrate.mirror_byte_identical`, and spec-integration's "Section sync" each
police the pairing of a delta section with a `## Requirements` mirror in
the same file. Plain deltas cannot carry a mirror (it would be a 1.x
violation), so the class they police becomes unwritable; the rules are
removed together with the delta-section strip's residual wording rather
than kept as dead validators. The dual-format *capability spec* class keeps
every other rule (frontmatter, tables, refs, reachability) unchanged.

## Decision 4 — deltas restate requirements via openspec's native merge, not a convention

The dual-era convention `delta_self_contained` (every delta restates the
capability's full requirement set) is retired with the clobbering deploy.
Deltas may be partial (ADDED-only like add-migrate-rekey); openspec's apply
defines the merge. The fixture-proved risk (openspec 1.14.2 hard-errors a
zero-scenario requirement) is a validation-time concern 1.x now enforces on
every delta directly — the pin's job, minus the pin.

## Decision 5 — sidecar identity across archive history is an existing precedent, not a new hazard

Archived sidecars keep `id: <cap>.layer` across change history, so the
same id recurs in `changes/archive/*/specs/<cap>/layer.md` files. This
matches today's precedent — archived dual-format deltas carry real ids
and share them with the deployed spec, and `spk lint openspec` stays
green because uniqueness is per-file and command paths never take the
archive tree as a corpus. Note one residual wording obligation: this
tool's OWN capability layer (archive-companion's `openspec_not_forked`,
`layer_preserved`, `delta_self_contained`, `fail_closed_unverifiable`
constraint texts) still says "spec application skipped" and "byte-identical
to its archived delta" — that wording updates via this change's sidecar at
its own archive (task 5.1's shape covers every delta, including this one).

## Rollout order (tasks are sequenced to keep every gate green at each step)

1. Revision 19 format edit + lint-rule retirement (old shapes already
   unwritable by corpus convention — no dual delta is authored anymore).
2. migrate + archive-companion code (new shape, new assembly).
3. Migrate the 3 remaining dual deltas to plain + sidecar.
4. Retire the validate pin; point the gate at 1.x.
5. Archive the two pending plain-shaped changes through the new companion.

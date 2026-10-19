# Decision: archive-companion deploys a delta-section-stripped main spec

- **Date**: 2026-10-19
- **Status**: Accepted
- **Tickets**: beads specodelic-s8fg (Stage 1) · GH-20 · reviewed under Rule-of-5 (findings CORR-001, EDGE-001 folded in)
- **Scope**: tool behavior (`src/archive_companion.rs`) — no corpus or format-law change

## Context

openspec 1.x `--strict` rejects two shapes in main specs, and only two
(`spec-structure.js`): a delta header (`## ADDED/MODIFIED/REMOVED/RENAMED
Requirements`) and a requirement header outside `## Requirements`. Our
deployed dual-format specs carry both — inherited from the verbatim delta
copy — so all 46 deployed specs fail 1.x validation, while everything else
about the dual-format shape (frontmatter, `## Constraints`/`## Model`/
`## Properties` tables, `## Purpose`) is tolerated by 1.x.

The byte-identical `## Requirements` mirror (constraint
`mirror_byte_identical`, aggregated over ADDED+MODIFIED per specodelic-54v)
already carries every requirement block including scenarios — the deployed
copy's delta sections are fully redundant content.

## Decision

`archive-companion`'s deploy step now writes the delta **minus its delta
sections** (all four titles, fence-aware) to the deployed spec. The mirror
becomes the deployed `## Requirements` section; scenarios ride with it; ah
contract bindings are unaffected. The archived delta keeps the full
dual-format layer (D9's frontmatter strip unchanged), so the archive
round-trip still preserves the layer verbatim — D4's byte-identity now
applies to the archived copy; the deployed copy is the layer-preserving
re-assembly of the same content.

Three fail-closed guards run before any write:

1. **Self-containedness** (corpus constraint `delta_self_contained`, now
   tool-enforced): every ADDED/MODIFIED requirement name must appear in the
   delta's own mirror.
2. **Existing-target coverage** (CORR-001): when the deployed target
   already exists, every requirement name it carries must appear in the
   delta's full requirement set (mirror ∪ REMOVED ∪ RENAMED) — a deploy
   that would silently narrow the spec is refused, leaving the target
   untouched. (Today the companion clobbered blindly: `fs::copy` with no
   merge logic.)
3. **Post-strip shape assertion** (EDGE-001): the stripped deployed text
   must carry no delta headers and no requirement headers outside
   `## Requirements` — the exact two 1.x error classes — or the deploy is
   refused before the write.

## Consequences

- Deployed specs become 1.x-strict-valid at the next archive of each
  capability; the one-time migration of the existing 46 deployed specs is a
  separate step (same strip, applied in place).
- Change deltas keep the dual shape and remain 1.x-invalid — the
  version-guarded 0.19 pin (`scripts/spec_validate.py`, decision
  2026-10-19-openspec-validate-pin.md) keeps the gate green until Stage 2
  (plain deltas + sidecar layer) makes deltas 1.x-valid too.
- The archive-companion capability spec's "verbatim deployment" wording is
  now stale; a MODIFIED change will restate the contract (follow-up ticket,
  specodelic-54v/dlf precedent).

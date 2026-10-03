# Tasks: add-data-lineage-pack

No implementation code — pack checkers ship as declarations (honest-empty).
The deliverable is the pack artifact plus its capability contract. Each
phase is a verifiable step; gates run after every phase.

## 1. Pack artifact

- [ ] 1.1 Create `packs/data-lineage.md` (id `data.lineage`, `kind:
      profile`, state `draft`) — six manifest tables per design.md D1:
      `## Sections` (Data row shape), `## Kinds` (data.dataset /
      data.artifact / data.environment), `## References` (produces /
      produced_by / consumed_by), `## Checkers` (data.lineage.closure),
      `## Floors` (artifact provenance), `## Requires` (base =
      specodelic.md Revision 14). Four-layer spec shape so the file lints
      clean as a spec in its own right.
- [ ] 1.2 Verify `pack_shape` reports zero findings over the pack's own
      declared vocabulary and the pack self-exemption holds (its manifest
      rows trigger no checkers, no orphan findings).

## 2. Dogfood

- [ ] 2.1 `spk lint openspec` and `just lint-specs` with the pack
      discovered: zero findings on files using none of the vocabulary
      (byte-identical behavior); honest-empty closure (no `## Data` rows
      in this repo) reports an empty checked-set.
- [ ] 2.2 Orphan-vocabulary probe in a scratch workspace: `data.dataset`
      used with the pack absent names the candidate pack `data.lineage`
      and both remediations, exit non-zero.
- [ ] 2.3 Ro5 review verdict check: all fix-pass findings from design.md
      "Review outcome" are reflected in the artifact.

## 3. Publish + gates

- [ ] 3.1 Flip the pack state `draft` → `published` once the publish
      guard holds (pack_shape clean, all gates green) — the deprecation
      path stays declared but unused.
- [ ] 3.2 Gates: `just ci` + `just lint-specs` +
      `openspec validate --all --strict` + `spk lint openspec` green.
- [ ] 3.3 Docs: README packs section lists the standard pack; CHANGELOG
      entry; docs/src/SUMMARY.md gains the data-lineage-pack capability
      page after archive.

## 4. Close-out

- [ ] 4.1 Close beads specodelic-r56 with the review trail;
      `bd export -o .beads/issues.jsonl`; commit + push.
- [ ] 4.2 Unblocks: specodelic-0dn (bioimage D6 pilot) may now propose
      `uses: [[data.lineage]]`.
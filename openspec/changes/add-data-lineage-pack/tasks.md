# Tasks: add-data-lineage-pack

No implementation code — pack checkers ship as declarations (honest-empty).
The deliverable is the pack artifact plus its capability contract. Each
phase is a verifiable step; gates run after every phase.

## 1. Pack artifact

- [x] 1.1 Create `packs/data-lineage.md` (id `data.lineage`, `kind:
      profile`, state `draft`) — six manifest tables per design.md D1:
      `## Sections` (Data row shape), `## Kinds` (data.dataset /
      data.artifact / data.environment), `## References` (produces /
      produced_by / consumed_by), `## Checkers` (data.lineage.closure),
      `## Floors` (artifact provenance), `## Requires` (base =
      specodelic.md Revision 14). Four-layer spec shape so the file lints
      clean as a spec in its own right.
      (Deviation, recorded in design.md D6: bare `produces` is NOT
      declared — the word appears in corpus prose and would falsely
      activate the pack on every lint; `append_only_packs` lets a later
      release add it.)
- [x] 1.2 Verify `pack_shape` reports zero findings over the pack's own
      declared vocabulary and the pack self-exemption holds (its manifest
      rows trigger no checkers, no orphan findings). Verified in a
      scratch git workspace before landing: lint 0 issues, packs data
      surfaced (base_pin 14), graph 0 dangling/violations, compile 3
      artifacts, model-check exploration_only, no self-activation
      advisories; draft/published/deprecated single-state variants all
      pack_shape-clean.

## 2. Dogfood

- [x] 2.1 `spk lint openspec` and `just lint-specs` with the pack
      discovered: zero findings on files using none of the vocabulary
      (byte-identical behavior — issues+warnings diffed identical
      with/without the pack discovered); honest-empty closure (no `##`
      `Data` rows in this repo) reports an empty checked-set. One
      honest exception, recorded in design.md D7: the dual-format delta
      mirrors requirement text naming the vocabulary, so linting the
      openspec tree vocabulary-activates the pack for the `spec` delta
      file (warnings channel only, exit 0, findings empty).
- [x] 2.2 Orphan-vocabulary probe in a scratch workspace: a declared
      `uses: [[data.lineage]]` edge with the pack absent produces the
      labeled `linter.orphan_vocabulary` finding naming the candidate
      pack `data.lineage` and both remediations, exit 1 (alongside the
      expected dangling-reference finding from the base linter).
- [x] 2.3 Ro5 review verdict check: all fix-pass findings from design.md
      "Review outcome" are reflected in the artifact — D2 labeled-finding
      clause (orphan probe), D3 opacity consequence (binding-target swap
      byte-identical probe), D5 draft-first lifecycle (artifact landed
      at the full three-state machine; publish flip is 3.1).

## 3. Publish + gates

- [x] 3.1 Flip the pack state `draft` → `published` once the publish
      guard holds (pack_shape clean, all gates green) — the deprecation
      path stays declared but unused. (The full three-state Model IS the
      published state per `packs.rs` lifecycle_of; single-state `draft`
      and `deprecated` variants were proven clean in scratch before
      landing.)
- [x] 3.2 Gates: `just ci` + `just lint-specs` +
      `openspec validate --all --strict` + `spk lint openspec` green.
- [x] 3.3 Docs: README packs section lists the standard pack; CHANGELOG
      entry; docs/src/SUMMARY.md gains the data-lineage-pack capability
      page after archive.

## 4. Close-out

- [x] 4.1 Close beads specodelic-r56 with the review trail;
      `bd export -o .beads/issues.jsonl`; commit + push.
- [x] 4.2 Unblocks: specodelic-0dn (bioimage D6 pilot) may now propose
      `uses: [[data.lineage]]`.
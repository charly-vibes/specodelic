# Tasks: add-numeric-predicates-pack

No implementation code — pack checkers ship as declarations (honest-empty).
The deliverable is the pack artifact plus its capability contract. Each
phase is a verifiable step; gates run after every phase.

## 1. Pack artifact

- [x] 1.1 Create `packs/numeric-predicates.md` (id `numeric.predicates`,
      `kind: profile`, state `published` via the full three-state Model)
      — six manifest tables per design.md D1: `## Sections` (Quantities
      row shape), `## Kinds` (numeric.quantity / numeric.bound /
      numeric.tolerance), `## References` (measured_by), `## Checkers`
      (numeric.quantity_closed, numeric.tolerance_labels), `## Floors`
      (tolerance requires bound + against), `## Requires` (base =
      specodelic.md Revision 14). Four-layer spec shape so the file lints
      clean as a spec in its own right. No bare-English vocabulary
      tokens (design D2 audit).
- [x] 1.2 Verify `pack_shape` reports zero findings over the pack's own
      declared vocabulary and the pack self-exemption holds (its manifest
      rows trigger no checkers, no orphan findings); verified in a
      scratch git workspace before landing: lint 0 issues in draft,
      published, and deprecated variants; graph 0 dangling/violations;
      compile 3 artifacts; vocabulary surface exactly [numeric.quantity,
      numeric.bound, numeric.tolerance, Quantities, measured_by].

## 2. Dogfood

- [x] 2.1 `spk lint openspec` and `just lint-specs` with the pack
      discovered: zero findings on files using none of the vocabulary
      (issues+warnings diffed byte-identical with/without BOTH packs
      discovered — the D2 prose-safety claim verified empirically);
      honest-empty closure (no `## Quantities` rows in this repo)
      reports an empty checked-set. Known exception per design D7:
      the dual-format delta vocabulary-activates the pack for the
      `spec` delta file (warnings channel only, exit 0).
- [x] 2.2 Orphan probe in a scratch workspace: a declared
      `uses: [[numeric.predicates]]` edge with the pack absent produces
      the labeled `linter.orphan_vocabulary` finding naming the
      candidate pack `numeric.predicates` and both remediations, exit 1
      (alongside the expected dangling-reference finding).
- [x] 2.3 Unit-opacity probe: `## Quantities` rows naming UCUM vs
      ISO4217 vs a microscopy scale lint byte-identical (md5-verified
      over issues+warnings; design D4).
- [x] 2.4 Ro5 review verdict check: all fix-pass findings from design.md
      "Review outcome" are reflected in the artifact — D2 pre-paid
      hygiene audit (vocabulary surface verified token-by-token), D4
      opacity (unit probe), D6 lifecycle (ships at the full three-state
      machine; draft/deprecated variants proven clean), D7 honest noise.

## 3. Gates + docs

- [x] 3.1 Gates: `just ci` + `just lint-specs` +
      `openspec validate --all --strict` + `spk lint openspec` green
      (17/17 strict, 0 findings).
- [x] 3.2 Docs: README packs paragraph mentions the second standard
      pack; CHANGELOG #102; docs/src/SUMMARY.md gains the
      numeric-predicates-pack capability page after archive.

## 4. Close-out

- [x] 4.1 Archive via `just archive-change` (verbatim dual-format copy);
      close beads specodelic-0fb with the probe trail;
      `bd export -o .beads/issues.jsonl`; commit + push.
- [x] 4.2 Unblocks: specodelic-0dn (bioimage D6 pilot) — one of its
      three standard-pack deps closed; specodelic-aal (empirical
      registry) remains to scaffold.
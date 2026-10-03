# Tasks: add-bioimage-pack

The scaffold is this change itself (proposal + design + tasks +
dual-format delta), gated on user approval before implementation.
No implementation code — pack checkers ship as declarations
(honest-empty). Each phase is a verifiable step; gates run after
every phase.

## 1. Scaffold (this ticket — specodelic-0dn)

- [ ] 1.1 Scaffold `openspec/changes/add-bioimage-pack/`:
      proposal.md (Why/What/Capabilities/Out-of-scope/Alignment/
      Governance), design.md (D1–D8 + rejected alternatives + Ro5
      Review outcome), tasks.md, dual-format delta
      `specs/bioimage-data-pack/spec.md` (Constraints/Model/
      Properties + ADDED Requirements + mirrored `## Requirements`).
- [ ] 1.2 Vocabulary hygiene audit pre-paid (design D2): grep -rlw over
      `specs/` + `openspec/specs/` — `bioimage.transform`,
      `bioimage.dtype_is`, `bioimage.shape_eq`,
      `bioimage.units_convertible`, `same_shape_as`, `dtype_is`,
      `shape_eq`, `units_convertible`, `Axes`, `preserves`, `axis`,
      `scale`, `validated_against` all absent (0 files); bare
      `bioimage` (3+3) documented asymmetry, `pipeline` (9+1) rejected,
      `dtype` (1+1) consumed-pack column only; bare predicate tokens
      never declared (they ride pack-qualified checker rule names).
- [ ] 1.3 Gates on the scaffold: `openspec validate --all --strict`,
      `cargo run -- lint openspec` (0 issues — installed spk is
      stale, use cargo run), `just sync-sections`, `just lint-specs`.
- [ ] 1.4 Ro5 proposal-time review (design.md "Review outcome"):
      FIND-1 folded as D1's checker-row home for the predicate
      grammar; FIND-2 tightened D3 to intra-file resolution;
      FIND-3 recorded the `dtype` reuse rationale in D2; FIND-4
      scoped Requires pack-dep semantics as declaration-only (D8);
      FIND-5 stated the empirical registry's consumption role.

## 2. Pack artifact (after approval)

- [x] 2.1 Create `packs/bioimage-data.md` (id `bioimage.data`,
      `kind: profile`, state `published` via the full three-state Model)
      — six manifest tables per design.md D1: `## Sections` (Axes
      row shape), `## Kinds` (bioimage.transform), `## References`
      (same_shape_as), `## Checkers` (bioimage.dtype_is,
      bioimage.shape_eq, bioimage.units_convertible), `## Floors`
      (transform requires preserves + dtype), `## Requires` (base =
      specodelic.md Revision 14 + data.lineage, numeric.predicates,
      empirical.registry pack deps). Four-layer spec shape so the
      file lints clean as a spec in its own right. No bare-English
      vocabulary tokens (design D2 audit).
- [x] 2.2 Verify `pack_shape` reports zero findings over the pack's own
      declared vocabulary and the pack self-exemption holds (its manifest
      rows trigger no checkers, no orphan findings); verified in a
      scratch git workspace before landing: lint 0 issues in draft,
      published, and deprecated variants; graph 0 dangling/violations;
      compile 3 artifacts; vocabulary surface exactly [bioimage.transform,
      Axes, same_shape_as] (checker rule names never join vocabulary() —
      implementation fix-pass note in design.md).

## 3. Dogfood

- [x] 3.1 `cargo run -- lint openspec` and `just lint-specs` with all
      FOUR packs discovered: zero findings on files using none of the
      vocabulary (issues+warnings diffed byte-identical with/without
      all four packs discovered — the D2 prose-safety claim verified
      empirically); honest-empty closure (no `## Data`/`## Axes` rows
      in this repo) reports an empty checked-set. Known exception per
      design D7: the dual-format delta vocabulary-activates the pack
      for the `spec` delta file (warnings channel only, exit 0).
- [x] 3.2 Orphan probe in a scratch workspace: a declared
      `uses: [[bioimage.data]]` edge with the pack absent produces
      the labeled `linter.orphan_vocabulary` finding naming the
      candidate pack `bioimage.data` and both remediations, exit 1.
- [x] 3.3 Cross-pack probes (the pilot's stress tests, design D3/D8):
      (a) `same_shape_as` resolving against a declared `## Data` row
      with the data-lineage pack active — file-local, no join; a
      dangling one fires the labeled finding; (b) `kind =
      bioimage.transform` in a base Properties kind column accepted
      with the pack active (the ung mechanism), labeled finding
      without it; (c) a `## Requires` dep row naming a pack absent
      from the workspace yields the labeled advisory.
- [x] 3.4 Floor probes (design D4): base `law` property floor findings
      byte-identical with the pack discovered; a `bioimage.transform`
      property owing `**preserves:**`/`**dtype:**` exercises the
      transform floor; additive labels from the consumed packs ride
      along.
- [x] 3.5 Ro5 review verdict check: all fix-pass findings from
      design.md "Review outcome" are reflected in the artifact — D2
      pre-paid hygiene audit (vocabulary surface verified
      token-by-token), D3 intra-file cross-pack resolution, D4
      additivity (law-floor probe), D6 lifecycle (ships at the full
      three-state machine; draft/deprecated variants proven clean),
      D7 honest noise, D8 declaration-only Requires deps.

## 4. Gates + docs

- [ ] 4.1 Gates: `just ci` + `just lint-specs` +
      `openspec validate --all --strict` +
      `cargo run -- lint openspec` green.
- [ ] 4.2 Docs: README packs paragraph mentions the bioimage pilot;
      CHANGELOG entry; docs/src/SUMMARY.md gains the
      bioimage-data-pack capability page after archive.

## 5. Close-out

- [ ] 5.1 Archive via `just archive-change` (verbatim dual-format copy);
      close beads specodelic-0dn with the probe trail;
      `bd export -o .beads/issues.jsonl`; commit + push.
- [ ] 5.2 Unblocks: the quant pack (synthesis D6 sequencing step 4) —
      the mechanism now has a full thin-domain instance trail.

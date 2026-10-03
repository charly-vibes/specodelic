# Tasks: add-empirical-registry-pack

The scaffold is this change itself (proposal + design + tasks +
dual-format delta), gated on user approval before implementation.
No implementation code — pack checkers ship as declarations
(honest-empty). Each phase is a verifiable step; gates run after
every phase.

## 1. Scaffold (this ticket — specodelic-aal)

- [x] 1.1 Scaffold `openspec/changes/add-empirical-registry-pack/`:
      proposal.md (Why/What/Capabilities/Out-of-scope/Alignment/
      Governance), design.md (D1–D7 + rejected alternatives + Ro5
      Review outcome), tasks.md, dual-format delta
      `specs/empirical-registry-pack/spec.md` (Constraints/Model/
      Properties + ADDED Requirements + mirrored `## Requirements`).
- [x] 1.2 Vocabulary hygiene audit pre-paid (design D2): grep -rlw over
      `specs/` + `openspec/specs/` — `empirical.statistic`,
      `StatTests`, `tested_by`, `stat_test`, `alpha`, bare
      `statistic` all absent (0 files); `window` present in corpus
      prose → excluded from every vocabulary-carrying facet (floor
      case label + column only, neither in `vocabulary()`); bare
      `empirical` asymmetry documented (dotted tokens cannot match
      prose words).
- [x] 1.3 Gates on the scaffold: `openspec validate --all --strict`,
      `spk lint openspec` (0 issues), `just sync-sections`,
      `just lint-specs`.
- [x] 1.4 Ro5 proposal-time review (design.md "Review outcome"):
      FIND-1 folded as design D8 — base-table kind columns must accept
      active-pack fiber kinds for `empirical.statistic` to be typeable
      (`property_kind_closed` walks static `PROPERTY_KINDS = {unit,
      law}` today); delta gains the `fiber_kinds_typeable` constraint,
      `kind_column_accepts_fiber_kinds` property, and its requirement;
      the mechanism follow-up is filed as its own beads ticket (this
      change keeps its no-`src/`-edit anti-goal).

## 2. Pack artifact (after approval)

- [x] 2.1 Create `packs/empirical-registry.md` (id `empirical.registry`,
      `kind: profile`, state `published` via the full three-state Model)
      — six manifest tables per design.md D1: `## Sections` (StatTests
      row shape), `## Kinds` (empirical.statistic), `## References`
      (tested_by), `## Checkers` (empirical.statistic_labels,
      empirical.stat_test_closed), `## Floors` (statistic requires
      alpha + window), `## Requires` (base = specodelic.md Revision 14).
      Four-layer spec shape so the file lints clean as a spec in its
      own right. No bare-English vocabulary tokens (design D2 audit).
- [x] 2.2 Verify `pack_shape` reports zero findings over the pack's own
      declared vocabulary and the pack self-exemption holds (its manifest
      rows trigger no checkers, no orphan findings); verified in a
      scratch git workspace before landing: lint 0 issues in draft,
      published, and deprecated variants; graph 0 dangling/violations;
      compile 3 artifacts; vocabulary surface exactly [empirical.statistic,
      StatTests, tested_by].

## 3. Dogfood

- [x] 3.1 `spk lint openspec` and `just lint-specs` with all THREE packs
      discovered: zero findings on files using none of the vocabulary
      (issues+warnings diffed byte-identical with/without all three
      packs discovered — the D2 prose-safety claim verified
      empirically); honest-empty closure (no `## StatTests` rows in
      this repo) reports an empty checked-set. Known exception per
      design D7: the dual-format delta vocabulary-activates the pack
      for the `spec` delta file (warnings channel only, exit 0).
- [x] 3.2 Orphan probe in a scratch workspace: a declared
      `uses: [[empirical.registry]]` edge with the pack absent produces
      the labeled `linter.orphan_vocabulary` finding naming the
      candidate pack `empirical.registry` and both remediations, exit 1
      (alongside the expected dangling-reference finding).
- [x] 3.3 Floor-additivity probe (design D4): base `law` property floor
      findings byte-identical with the pack discovered; a
      `empirical.statistic` property owing `**alpha:**`/`**window:**`
      exercises the statistic floor declaration.
- [x] 3.4 Ro5 review verdict check: all fix-pass findings from
      design.md "Review outcome" are reflected in the artifact — D2
      pre-paid hygiene audit (vocabulary surface verified
      token-by-token), D4 additivity (law-floor probe), D6 lifecycle
      (ships at the full three-state machine; draft/deprecated
      variants proven clean), D7 honest noise.

## 4. Gates + docs

- [x] 4.1 Gates: `just ci` + `just lint-specs` +
      `openspec validate --all --strict` + `spk lint openspec` green.
- [x] 4.2 Docs: README packs paragraph mentions the third standard
      pack; CHANGELOG entry; docs/src/SUMMARY.md gains the
      empirical-registry-pack capability page after archive.

## 5. Close-out

- [x] 5.1 Archive via `just archive-change` (verbatim dual-format copy);
      close beads specodelic-aal with the probe trail;
      `bd export -o .beads/issues.jsonl`; commit + push.
- [x] 5.2 Unblocks: specodelic-0dn (bioimage D6 pilot) — all three of
      its standard-pack deps closed; the pilot proposal consumes the
      trio via `uses` edges.

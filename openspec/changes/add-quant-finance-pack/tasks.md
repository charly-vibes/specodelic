# Tasks: add-quant-finance-pack

The scaffold is this change itself (proposal + design + tasks +
dual-format delta), gated on user approval before implementation.
No implementation code — pack checkers ship as declarations
(honest-empty). Each phase is a verifiable step; gates run after
every phase.

## 1. Scaffold (this ticket — specodelic-up7)

- [x] 1.1 Scaffold `openspec/changes/add-quant-finance-pack/`:
      proposal.md (Why/What/Capabilities/Out-of-scope/Alignment/
      Governance), design.md (D1–D7 + rejection table + risks),
      tasks.md, dual-format delta
      `specs/quant-finance-pack/spec.md` (Constraints/Model/
      Properties + ADDED Requirements + mirrored `## Requirements`).
- [x] 1.2 Vocabulary hygiene audit pre-paid (design D2): word-boundary
      grep over `specs/` + `openspec/specs/` — `risk` (0+0), `Limits`
      (0+0), `horizon` (0+0), `capped_by` (0+0), `quant.risk`
      (0+0), `quant.pricing` (0+0), `quant.finance` (0+0), `quant`
      (0+0), `finance` (0+0), `backtest` (0+0), `pricing` (0+0) all
      absent; bare `limit` (4+1), `confidence` (5+0), and
      `Fixtures`/`fixture` (4+4) excluded from vocabulary-carrying
      facets — `limit` never a token, `confidence`/`horizon` only as
      floor case labels, `Fixtures` rejected as a section token.
- [x] 1.3 Gates on the scaffold: `openspec validate --all --strict`,
      `cargo run -- lint openspec` (0 issues), `just sync-sections`,
      `just lint-specs`.
- [x] 1.4 Ro5 proposal-time review folded into design.md (D1 pricing
      floor deferral; D3 labeled-remediation wording; D5 empirical
      registry declared not-required; D6 rejection table; D7 dual-format
      shape named; risk section states the restraint trade).

## 2. Pack artifact (after approval)

- [x] 2.1 Create `packs/quant-finance.md` (id `quant.finance`,
      `kind: profile`, state `published` via the full three-state
      Model) — six manifest tables per design.md D1: `## Sections`
      (Limits), `## Kinds` (quant.risk, quant.pricing), `## References`
      (capped_by), `## Checkers` (quant.limit_closed,
      quant.risk_labels), `## Floors` (quant.risk → horizon,
      confidence), `## Requires` (base Revision 18 +
      numeric.predicates + data.lineage).
- [x] 2.2 Gates: `openspec validate --all --strict`, `cargo run -- lint
      openspec` (0 issues), `just sync-sections`, `just lint-specs`,
      `just ci`.

## 3. Close-out

- [ ] 3.1 Update tasks.md statuses; `bd close specodelic-up7`; commit
      and push per the repo's commit-hygiene rule (stage only files
      this session authored).

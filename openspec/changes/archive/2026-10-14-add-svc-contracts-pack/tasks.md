# Tasks: add-svc-contracts-pack

Test-first per `AGENTS.md`: define the failing probe, then the artifact, then
tidy. `E…` probes refer to the bundle's `evidence/run-evidence.sh`.

## 1. Spec delta and gates (this proposal)

- [x] 1.1 Maintainer review of `proposal.md`, `design.md`, and the open questions OQ-P1…OQ-P6; decide ship-now vs hold (OQ-P1) — **decided 2026-10-14 (specodelic-kjzh): SHIP at `draft` lifecycle**; OQ-P2 (repo normalization) and OQ-P7 (column rename) stay open, no blocker
- [x] 1.2 `openspec validate add-svc-contracts-pack --strict --no-interactive` passes
- [x] 1.3 `spk lint openspec` (the repo's `lint-deltas` recipe) reports 0 findings for the delta
- [x] 1.4 `python3 scripts/check_section_sync.py openspec` passes (Requirements mirror byte-identical to the ADDED section)

## 2. Land the pack artifact (only after 1.1 approves)

- [x] 2.1 RED: reproduce E10c — a corpus spec using `svc.compat` with no pack reports `property_kind_closed` + `orphan_vocabulary`, exit 1
- [x] 2.2 Add `packs/svc-contracts.md` from the bundle draft (apply any change from OQ-P3/P4/P5)
- [x] 2.3 GREEN: E10b (pack discovered, advisory names draft status, exit 0) and E10a (`pack_shape` zero findings on the pack file)
- [x] 2.4 Re-run the vocabulary audit (E12) against the tree at landing; every token must have 0 exact-case hits
- [x] 2.5 Probe `no_pack_no_change` (E11): `spk lint specs` and `spk lint openspec` byte-identical before/after adding the pack
- [x] 2.6 Probe `honest_empty`: pack active with no `## Interfaces` rows reports an empty checked-set (not yet probed — **UNKNOWN** until run)
- [x] 2.7 If OQ-P6 says required: `spk compile packs/svc-contracts.md` and commit `specodelic/svc-contracts.*` — **answered by mechanism (2026-10-14): no per-pack artifact at the CHANGE phase — `specodelic/quant-finance.*` exists because its deployed capability spec (`openspec/specs/quant-finance-pack/`) compiles to keyed artifacts, not because `packs/quant-finance.md` does. At archive, the deployed `svc-contracts-pack` capability regenerates its `specodelic/svc-contracts-pack.*` artifacts via `just ci` (jf1 fact: capability spec deploys do not need a hand-run compile).**

## 3. Documentation and housekeeping

- [x] 3.1 `docs/src/SUMMARY.md`: add the capability entry (the repo's `summary-completeness` gate requires deployed capabilities to be listed)
- [x] 3.2 `openspec/CHANGELOG.md`: entry stating "no base change, no format revision"
- [x] 3.3 `just ci` passes
- [x] 3.4 Archive: `openspec archive add-svc-contracts-pack --yes` then the repo's `archive-companion` step to restore the dual-format layer (per `AGENTS.md`)

# Change: Implement the `compile` functor (Constraints→TOML, Properties→proptest!; model backend deferred)

## Why

`spk compile` is a stub that exits non-zero. The corpus just reached
covered (beads `specodelic-qc8` closed), so `compile`'s own
`precondition_satisfied` gate can now hold for the whole corpus — and
`specs/compile.md` fully specifies what compiling must produce. The P1
beads ticket `specodelic-lnq` tracks this work; it blocks `verify`
(`specodelic-1pv`) and `model_check` (`specodelic-nx7`).

**Deferred by decision:** the model-checker backend (TLA+ vs Alloy) is
not yet chosen — `specs/compile.md` says "TLA+ (or Alloy)" without
pinning one. That choice is now HITL row 6 of beads `specodelic-mp1`.
This change delivers the two decided translations plus the pipeline
contract, with model emission built behind a backend-neutral seam;
it lands in a follow-up once `mp1` row 6 is decided.

## What Changes

- Add `src/compile.rs` implementing the decided translations of
  `specs/compile.md` as one functor with three target categories:
  1. Constraints table → TOML document (`constraint_table_to_toml`)
  2. Model section → model-checker module (**deferred** — backend
     undecided, `mp1` row 6; this change ships a backend-neutral
     `ModelIR` extraction so either backend slots in later)
  3. Properties table → proptest! block sources, one block per required
     case for law-kind properties (`properties_to_proptest`)
- Gate compilation on the file being linted and covered
  (`precondition_satisfied`): compile runs `lint` internally and refuses
  a file with any coverage-rule finding
- Enforce `compile_preserves_ids` and `no_semantic_drift` (round-trip
  idempotence) before reporting `compiled`; failures are labeled and
  carry a remediation hint, never silent partials (`compile_is_total`)
- Wire the `Compile` subcommand in `main.rs` (currently a not-implemented
  stub) to emit the artifacts through the genesis envelope and to
  `--out-dir` on disk — TOML + proptest sources now, the model module
  after the backend decision
- Add the `proptest` dependency (emitted-code support; the emitted `.rs`
  sources reference it)
- Tests: unit tests per translation + integration tests extending
  `tests/cli.rs` (corpus-wide `spk compile specs --json` exits 0)

## Impact

- Affected specs: `compile` (new OpenSpec capability; domain spec of
  record remains `specs/compile.md` — the delta mirrors it, it does not
  reinterpret it; the model-backend requirement is stated backend-neutral
  pending `mp1` row 6)
- Affected code: `src/compile.rs` (new), `src/lib.rs` (module wiring),
  `src/main.rs` (one enum arm replacing the shared stub arm for
  `Compile` only — `ModelCheck`/`Verify` stay stubs), `Cargo.toml` (+proptest),
  `tests/cli.rs` (new corpus assertions)
- Beads: closes `specodelic-lnq` **only after** the model artifact ships
  (phases 1+3 + contract land first; the ticket stays open — see its
  notes); unblocks `specodelic-1pv` and `specodelic-nx7` once complete
- Decision record: TLA+ vs Alloy → `specodelic-mp1` row 6 (HITL)

# Change: Implement the `compile` functor (Constraints→TOML, Model→TLA+, Properties→proptest!)

## Why

`spk compile` is a stub that exits non-zero. The corpus just reached
covered (beads `specodelic-qc8` closed), so `compile`'s own
`precondition_satisfied` gate can now hold for the whole corpus — and
`specs/compile.md` fully specifies what compiling must produce. The P1
beads ticket `specodelic-lnq` tracks this work; it blocks `verify`
(`specodelic-1pv`) and `model_check` (`specodelic-nx7`).

## What Changes

- Add `src/compile.rs` implementing the three translations of
  `specs/compile.md` as one functor with three target categories:
  1. Constraints table → TOML document (`constraint_table_to_toml`)
  2. Model section → TLA+ module text, including the `Output` function
     for `emits`-carrying states (`model_to_tla`)
  3. Properties table → proptest! block sources, one block per required
     case for law-kind properties (`properties_to_proptest`)
- Gate compilation on the file being linted and covered
  (`precondition_satisfied`): compile runs `lint` internally and refuses
  a file with any coverage-rule finding
- Enforce `compile_preserves_ids` and `no_semantic_drift` (round-trip
  idempotence) before reporting `compiled`; failures are labeled and
  carry a remediation hint, never silent partials (`compile_is_total`)
- Wire the `Compile` subcommand in `main.rs` (currently a not-implemented
  stub) to emit the three artifacts through the genesis envelope and to
  `--out-dir` on disk
- Add the `proptest` dependency (emitted-code support; the emitted `.rs`
  sources reference it)
- Tests: unit tests per translation + integration tests extending
  `tests/cli.rs` (corpus-wide `spk compile specs --json` exits 0)

## Impact

- Affected specs: `compile` (new OpenSpec capability; domain spec of
  record remains `specs/compile.md` — the delta mirrors it, it does not
  reinterpret it)
- Affected code: `src/compile.rs` (new), `src/lib.rs` (module wiring),
  `src/main.rs` (one enum arm replacing the shared stub arm for
  `Compile` only — `ModelCheck`/`Verify` stay stubs), `Cargo.toml` (+proptest),
  `tests/cli.rs` (new corpus assertions)
- Beads: closes `specodelic-lnq`; unblocks `specodelic-1pv` and
  `specodelic-nx7`

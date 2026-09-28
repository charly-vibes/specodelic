# Tasks: add-tla-emitter

Beads ticket: `specodelic-lnq` (model-emission phase; claim before
starting). Regression bar: `just ci` green at every step.

## 1. Emitter

- [ ] 1.1 Implement `model_to_tla(spec) -> String` in `src/compile.rs`: one module per file — state variable with every State as a value in its range, `Init`, `Next` with one disjunct per Transition guarded on `from`/`to` (spec guard quoted verbatim in the disjunct comment), `Output` function with exactly the emitting states (empty function when none)
- [ ] 1.2 Module name is the file stem, sanitized to TLA+ identifier rules

## 2. Tests

- [ ] 2.1 Unit: disjunct count in `Next` == n transitions (`tla_disjunct_count_matches`)
- [ ] 2.2 Unit: `Output` domain == exactly the emitting states — no entry for the rest (`output_function_covers_emitting_states_only`); empty function when no emits
- [ ] 2.3 Unit: module name sanitized; guard text carried verbatim in comments

## 3. Wiring

- [ ] 3.1 `Compiled` gains `tla`; `compile_spec` produces it; envelope includes the artifact
- [ ] 3.2 `write_artifacts` writes `<stem>.tla`; integration tests assert three written artifacts
- [ ] 3.3 Regenerate the committed corpus artifacts under `specodelic/`

## 4. Close out

- [ ] 4.1 `just ci` green; `openspec validate add-tla-emitter --strict`
- [ ] 4.2 Update this checklist; commit + push; `bd close specodelic-lnq` (the model artifact now ships)

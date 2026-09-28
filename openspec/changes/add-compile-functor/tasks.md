# Tasks: add-compile-functor

Beads ticket: `specodelic-lnq` (claim before starting). Regression bar:
`tests/cli.rs` + `tests/spec_parse.rs` keep passing; `just ci` green at
every step.

## 1. Constraints → TOML

- [ ] 1.1 Create `src/compile.rs` with Purpose/Responsibilities/Rationale header; wire `pub mod compile;` in `src/lib.rs`
- [ ] 1.2 Implement `constraints_to_toml(rows) -> String`: one TOML table entry per Constraint row `{id, kind, expr, traces_to}`
- [ ] 1.3 Unit test: TOML round-trips (`parse_toml(compile(row)) == row`); ids preserved verbatim

## 2. Model → backend-neutral IR *(emission deferred: TLA+ vs Alloy → beads `specodelic-mp1` row 6)*

- [ ] 2.1 Implement `ModelIR` extraction: states, transitions with from/to/guard, `emits`→effect-Constraint mapping (emitting states only, no defaults)
- [ ] 2.2 Unit tests: IR transition count == n transitions; emits mapping covers exactly the emitting states
- [ ] 2.3 *(deferred — do not start)* Emit the model module from `ModelIR` once `mp1` row 6 decides the backend; follow-up change owns the emitter and `<stem>.tla`/`<stem>.alloy` artifact

## 3. Properties → proptest!

- [ ] 3.1 Add `proptest` to `Cargo.toml`
- [ ] 3.2 Implement `properties_to_proptest(rows) -> String`: one `proptest!` block per row (generator → strategy, predicate → assertion body), law-kind rows expanding to one block per required case
- [ ] 3.3 Unit tests: block count ≥ 1 per row; law with 2 named cases → exactly 2 blocks

## 4. Pipeline contract + CLI wiring

- [ ] 4.1 Implement `precondition_satisfied`: run the existing `lint` pass internally; refuse with labeled failure + remediation hint when any issue exists
- [ ] 4.2 Enforce `compile_is_total` (all artifacts due under the current backend decision, or one labeled failure), `compile_preserves_ids` (source ids ⊆ artifact ids + `ModelIR`), `no_semantic_drift` (TOML round-trip byte-identical)
- [ ] 4.3 Split the shared stub arm in `main.rs`: `Compile` emits the envelope (artifacts + `ModelIR` stats) and writes `<stem>.toml` / `<stem>_props.rs` to `--out-dir` (default `compiled/`, gitignored); `ModelCheck`/`Verify` keep the stub
- [ ] 4.4 Integration tests in `tests/cli.rs`: `spk compile specs --json` exits 0 corpus-wide; lint-dirty synthetic file exits non-zero naming `precondition_satisfied`; round-trip stability on one corpus file

## 5. Close out

- [ ] 5.1 `just ci` green; `bd close specodelic-lnq` — **only after** the model emitter lands (ticket stays open across the `mp1` row 6 decision); commit + push per phase
- [ ] 5.2 Update `openspec` checklist (all tasks `[x]`) for the archive stage

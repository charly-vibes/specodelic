# Tasks — fix-audit-before-contradiction

Each numbered step is one red→green cycle. The code edit and its pins are
one commit; `just ci` gates the whole change.

**Hard constraints:** no report-schema change, no new audit class, no
vocabulary change, no change to `check`'s behavior or to the exit contract
for `stale`/`unchecked`/`binds_nothing`/`new` rows. Stage only
session-authored paths.

## 1. Red — the diff retires a contradiction

- [x] 1.1 Unit pins in `src/audit.rs` (beside
  `binds_nothing_round_trips_and_rides_rank_1`): sustained contradiction
  (prior `contradicted`, current `contradicted`) ⇒ diffed `contradicted`,
  `class_counts["contradicted"] == 1`, `exit_code == 1`; unseen
  contradiction (no prior row) ⇒ `contradicted`, never `new`, exit 1;
  improvement (prior `contradicted` ⇒ current `current`) ⇒ `current`,
  exit 0.
- [x] 1.2 `tests/audit_before_contradiction.rs` (new CLI integration file,
  mirroring `tests/audit_oracle_rows.rs`'s fixture wiring): the same two
  holes end-to-end through `spk audit --before`, asserting exit 1 and the
  `contradicted` class in the envelope.

## 2. Green — the overlay keeps the evidence class

- [x] 2.1 `audit::diff_with_prior` preserves `contradicted` for both the
  held and the unseen case; `overlay_class` stays a pure currency decision.
- [x] 2.2 Doc comments in `src/audit.rs` and `src/check.rs` state the new
  fact (the overlay never retires a contradiction) and nothing that the
  code does not do.

## 3. Gates

- [x] 3.1 `cargo test --test audit_before_contradiction --test audit_oracle_rows --test audit_rung --test check_matrix --test check_rows` → green.
- [x] 3.2 `openspec validate fix-audit-before-contradiction --strict` → clean.
- [x] 3.3 `just lint-deltas` (layer sidecar included) → clean.
- [ ] 3.4 `just ci` → green.

## 4. Docs

- [x] 4.1 `docs/src/graded-evidence.md` §"Evidence diff across reports":
  state that a contradiction is never demoted by the overlay; correct the
  `check` paragraph that described the overlay relabelling the row's
  currency.

## 5. Archive

- [x] 5.1 `just archive-change fix-audit-before-contradiction` (the
  companion re-attaches the layer; never the stock archive).
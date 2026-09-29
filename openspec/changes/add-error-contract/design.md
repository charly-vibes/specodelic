# Design: add-error-contract

## Context

The Rule-of-5 review of the investigation (converged Stage 4) established
five corrections that shape every decision below. Evidence verified in the
corpus: `compile.md:46` and ≥5 sibling files carry untyped `¬x_ok.guard`
failure guards; `graph.md:32` counts only `[[id]]` citations as edges, so
those guards are outside `ref_kind_compatible`'s reach; `refactor.md:39`
already uses `emits` for an *output* (`found` emits
`[[refactor.advisory_finding_emitted]]`), proving the mechanism but also
that no *failure* state emits; `compile.md:60`'s predicate accepts
`single_labeled_failure` as an unexamined set member; and
`add-observability-contracts` has claimed `specodelic.md` Revision 9 /
`kinds.md` Revision 5 for its `observes` row.

## Goals / Non-Goals

- Goals: errors in the corpus become typed, namespaced, falsifiable
  facts — every failure state emits a labeled variant, every label has a
  falsifying property, the output contract is a published extension_point
  contract rather than CHANGELOG lore.
- Non-Goals: no core `specodelic.md`/`kinds.md` Revision (see D7); no
  checker implementation (spec-only, D6); no terminal-outcome typing for
  `timed_out`/`exploration_only` (phase 2, D5); no waiver machinery.

## Decisions

- **D1 — `errors.md` owns the error-expr shape** (Rule-of-5 CORR-001).
  The "tagged variant, not free-text `error` string" row in `STATUS.md`
  §2 had no owning constraint. `errors.md` states it: an error
  Constraint's expr is `<file-id>.<variant_head>(field, …)`. Without this,
  label uniqueness and the future checker have no syntax to key on.
- **D2 — failure guards become typed negated citations, scoped to
  intra-file constraints** (CORR-002). `¬extract_ok.guard` is replaced by
  `¬([[compile.constraint_table_to_toml]] ∧ [[compile.model_to_tla]] ∧
  [[compile.properties_to_proptest]])` — the negated citations *are* the
  failure-class definition, and they are all invariant-kind, satisfying
  Reference Typing. **Carve-out:** `orchestrate.md` keeps its prose stage
  guards — its Notes (`orchestrate.md:96-98`) deliberately decline to
  restate upstream files' logic as guard-citable rows; retyping them would
  reintroduce the duplication the file argues against. Stated in
  `errors.md`, not left implicit.
- **D3 — error labels are file-id-namespaced** (EDGE-003 → EXCL-001).
  `compile.extraction_failure`, not bare `extraction_failure`. Uniqueness
  becomes a per-file property the existing per-file linters decide;
  cross-file label collisions are structurally impossible. The promoted
  insight from the review — label = file-id-qualified variant head +
  payload fields + falsifying property naming the label — is the change's
  acceptance criterion.
- **D4 — split failure states where the file itself argues for classes;
  emits-only elsewhere.** `compile.md` splits `failed` into
  `extract_failed` / `emit_failed` (its `compile_is_total` Notes already
  argue the two-stage distinction); other tool files keep one `failed`
  state and add the `emits` edge. Payload fields live in the error
  Constraint's expr (`extraction_failure(row_id, reason)`) — `emits` ties
  one Constraint to a state, so the variant's fields carry the payload
  (EDGE-001).
- **D5 — `timed_out` / `exploration_only` are phase 2.**
  `model_check.md:85,96` treats them as real terminal outcomes; typing
  them via `terminal_states_emit` is the future checker's scope, listed in
  `linter-failure_shape.md` as a stated non-goal of v1 so the exit-code
  mapping question is visible rather than forgotten.
- **D6 — the future eighth checker is spec-only here.**
  `linter-failure_shape.md` follows the `linter-external_completeness.md`
  file shape; no Checker Ownership row lands until implementation (the
  table's invariant is that every listed checker exists). **Blocker
  status:** `specodelic-6pi` is CLOSED — AGENTS.md's sibling-tool note
  ("must be fixed before any CI gate chains lint") is stale and the
  implementation ticket is not gated. AGENTS.md itself is governance;
  flagging the staleness to the maintainer rather than editing it here.
- **D7 — no core Revision consumed.** Every new constraint lives in the
  new `errors.md` and per-tool files. This keeps `specodelic.md`
  Revision 9 free for `add-observability-contracts` and avoids a
  numbering collision between two pending changes. The Checker Ownership
  table is untouched.

## Risks / Trade-offs

- Corpus churn across ~5 tool files → mitigated by `just lint-specs`
  staying green as the gate after every phase; each file's edit is
  mechanically small (states list, one transition guard, one or two
  table rows, one property row).
- Splitting `compile.md`'s state set changes its compiled TLA+ module →
  additive under `model_to_tla`'s own wording (states map 1:1; the
  closing stuttering disjunct is unchanged); the round-trip property is
  re-run as part of the phase's verification.
- Untyped `¬x_ok.guard` idiom appears in ≥6 files → guard restructure is
  scoped by D2's carve-out; `linter-failure_shape.md` records which files
  are typed and which are carved out, so the gap is a stated fact, not a
  silent one.

## Migration Plan

Corpus-only change: each phase lands `just lint-specs`-green; no tool
behavior changes, so rollback is `git revert` of the phase commit. The
per-error unit properties are new coverage, not changed behavior.

## Open Questions

- Should `errors.md`'s contract rows be one `extension_point` per concern
  (envelope, exit codes, labels) or one merged row? Leaning one-per-concern
  so `satisfies` edges are granular — resolved during D1 implementation if
  the corpus reads better merged.
- Phase-2 exit-code derivation from terminal outcomes (`passed` → 0,
  `failed` → 1, invocation error → 2): confirm the 0/1/2 mapping covers
  `exploration_only` when terminal typing lands.

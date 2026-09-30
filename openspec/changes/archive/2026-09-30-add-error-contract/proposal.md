# Change: Add error contract — namespaced labeled errors, emitting failure states, typed negation guards

## Why

The tool's most cross-cutting error behavior is implemented but not specced:
the output contract (exit codes 0/1/2, `ok:false` envelopes, one labeled
failure per stage, remediation hints) lives only in CHANGELOG issue #48
(`specodelic-7rr`). Meanwhile the corpus's own models commit, at the
state-machine level, the exact sin the format forbids at the constraint
level (`STATUS.md` §2, "Flattened generic errors"): every tool file has a
single undifferentiated `failed` state that emits nothing, and failure-path
guards (`¬x_ok.guard`) are untyped prose idioms outside the typed-reference
system. Errors are therefore not falsifiable the way everything else in
the corpus is.

Design was reviewed under a Rule-of-5 pass (converged at Stage 4); five
corrections are folded in: `errors.md` owns the error-expr shape (the
"tagged variant" requirement previously had no owner), failure guards are
restructured into typed negated citations rather than prose-parsed, error
labels are file-id-namespaced so uniqueness is decidable per-file by the
existing checkers, the corpus-wide `emits` precedent (`refactor.md:39`) is
cited as prior art, and the `guard_negation_total` check is scoped to
intra-file citations with an explicit orchestrate carve-out.

## What Changes

- **New format corpus file `specs/errors.md`** owning the cross-cutting
  output contract as `extension_point` rows (envelope kind ↔ `ok:false`,
  exit codes 0/1/2, one labeled failure per stage, remediation hint
  required) plus the error-shape law: an error Constraint's expr is a
  **file-id-namespaced variant head** (`compile.extraction_failure(row_id,
  reason)`), and every error Constraint carries a unit property whose
  predicate asserts the exact label.
- **Per-tool corpus restructure (no core Revision needed):** `compile.md`
  splits its mute `failed` state into `extract_failed` / `emit_failed`
  (its own prose already argues extraction and emission are different
  failure classes); every tool file's failure terminal gains an `emits`
  edge to a file-owned labeled error Constraint; failure guards become
  typed negations `¬([[c1]] ∧ [[c2]] ∧ …)` where the cited constraints are
  intra-file; tool files point `satisfies` at the contract rows.
- **New checker file `specs/linter-failure_shape.md`, spec-only in this
  change** — the future eighth checker (`terminal_states_emit`,
  `error_labels_unique`, `guard_negation_total`). Implementation is a
  separate ticket; most of its enforcement already exists structurally via
  the corpus edits above (emits→effect typing, coverage, per-file label
  uniqueness by construction).
- **STATUS.md** file inventory gains the two new files.
- **No `specodelic.md` Revision is consumed by this change** — all new
  constraints live in the new file and per-tool files, deliberately
  leaving Revision 9 free for `add-observability-contracts` (design D7).

## Impact

- Affected specs: new `error-contract` capability; corpus files
  `compile.md`, `rename.md`, `orchestrate.md`, `linter-coverage.md`,
  `linter-referential_integrity.md` restructured; `STATUS.md` inventory.
- Affected code: no tool code in this change (spec corpus only); the
  per-error unit properties become the fixtures the eventual
  `linter-failure_shape` implementation tests against. `just lint-specs`
  must stay green throughout — the corpus is its own primary fixture.
- Blocked-by: none (`specodelic-6pi` is closed; AGENTS.md's blocker note
  is stale — flagged in design.md D6).

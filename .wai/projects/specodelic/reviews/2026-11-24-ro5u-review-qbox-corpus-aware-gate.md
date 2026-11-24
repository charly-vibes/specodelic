# RO5U review — qbox: corpus-aware artifact gate for `spk check`

Date: 2026-11-24 · Scope: `src/check.rs`, `src/conform.rs`, `src/audit.rs`,
`src/commands/corpus.rs`, `src/main.rs`, `tests/cli/check.rs`,
`tests/cli/main.rs`, `tests/archive_change_recipe.rs`

## Stage 1 — DRAFT (shape, scope)

One coherent seam: the CLI discovers the corpus (the machinery conform
already uses), the library gates with it. No second walker, no change to
conform/audit/verify semantics, no change to the lint rules, no change to
the gate's verdict rule (this file's own findings only).

## Stage 2 — CORRECTNESS

- The corpus-aware gate reuses `conform::gate_in_corpus` verbatim; the
  verdict still keys on the judged file's own findings inside the corpus
  lint.
- The refusal shape is unchanged: the message leads, the hint names the
  remediation, zero records. The label rides the warnings channel.
- The first wording of the isolation label contained the word "pass"
  ("pass the file inside its openspec tree") and broke the check
  capability's scenario contract "it never reads as a pass"
  (`tests/check_contract_exit.rs:374`). Caught by the existing suite and
  fixed before this review: the label now reads "put the file inside its
  openspec tree".

## Stage 3 — CLARITY

One wording per fact: `conform::ISOLATED_GATE_WARNING` and
`conform::partial_corpus_warning` live in the library; the binary never
words a gate fact, so conform and check cannot drift into two labels.

## Stage 4 — EDGE CASES

- A stray file inside an openspec tree but outside `specs/` —
  `gate_in_corpus` appends the target to the corpus slice.
- A partially parsed corpus — labeled warning, never a silent verdict.
- A corpus with no siblings — the file alone.
- No corpus root — the labeled isolation fallback.
- The corpus-less library API (`check::run`) — isolated, labeled.

## Findings

**CRITICAL:** none. **HIGH:** none.

**MEDIUM**

- **M1** `src/check.rs` (`cmd_check_in_corpus`) — the isolation label is
  suppressed by a string probe (`warnings.iter().any(|w| w.contains("ISOLATION"))`):
  fragile (any caller warning containing that word would silently drop the
  label) and redundant (`CorpusGate::isolated()` already carries it).
  *Fix:* one rule — `CorpusGate::isolated()` carries **no** label and
  `cmd_check_in_corpus` appends `conform::ISOLATED_GATE_WARNING` exactly
  when `gate.corpus.is_none()`; the string probe disappears.
  **→ fixed**
- **M2** The corpus-wide lint per invocation: `spk check` now lints the
  whole discovered corpus (measured ~24 s per file over the repo's
  50-file corpus), so a single-file judging verb costs corpus-sized work
  — the same trade conform's input gate already makes.
  *Fix in this change:* none (caching is out of scope); the cost is
  recorded in the verb's doc comment so callers know the price.
  **→ fixed (documented)**

**LOW**

- **L1** The isolation label rides refusals unrelated to the corpus
  (e.g. `verify_model`). It is factually true (the gate did lint in
  isolation) and the wording states the fact without claiming causation.
  *Fix:* keep.
- **L2** The partial-corpus label wording replaces conform's
  verb-specific wording ("conform's input gate linted a PARTIAL corpus")
  with the shared one. Deliberate — one fact, one wording — but it alters
  a conform envelope's text; *Fix:* keep the shared wording and record the
  wording change in the ledger/changelog.
- **L3** `check::run`'s public signature stays corpus-less, so
  `tests/fuzz.rs` and `tests/differential_reach.rs` keep the isolated
  behavior. Intentional API stability. *Fix:* none.

## Convergence

New CRITICAL: 0 · Total new: 5 · New vs previous stage: +25% ·
Estimated false positive rate: ~0% (each finding re-read at its cited
line) · Status: CONTINUE → apply M1/M2, re-run the narrow suite.

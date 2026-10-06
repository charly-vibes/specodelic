# RO5U Review — add-min-expr-kernel phase 3 (specodelic-7ga), Tier B kernel core

Artifact under review: the §3 implementation — `src/kernel.rs` (new),
`src/compile.rs` (guard_kernel extraction + labeled validation),
`tests/kernel_grammar.rs`, `tests/kernel_grounding.rs`,
`tests/kernel_status.rs`, `tests/kernel_registry.rs`,
`tests/kernel_compile_edge.rs`.

Self-reported validation: TYPESAFE_API_KEY not set — verified by
inspection plus probe execution; every claimed behavior is backed by a
test that was run (FP rate confirmed by execution, not assertion).

## Stage 1 — DRAFT (shape)

Sound. One new module (`src/kernel.rs`), tests out-of-module as new
integration files, `**kernel:**` marker opt-in in fragment position,
the D1 grounding table as data (`PredicateRegistry::v0()`), and
`src/model_check.rs` untouched (the shrink-only ratchet respected).

Findings: none.

## Stage 2 — CORRECTNESS

- **F-1 (HIGH, verified, FIXED)**: the first-draft opt-in rule
  (marker-free whole-cell parse) broke pure widening — the corpus
  already carries ∀-led **prose** cells (`specs/specodelic.md:20`
  `unique_id`, `specs/merge.md:33` `no_new_id_collision`);
  `just test` failed 12 corpus-compile files. Fix of record: opt-in is
  the explicit `**kernel:**` marker in fragment position (the sd1 rule
  carried over); pinned by `corpus_quantifier_prose_stays_prose`.
- **F-2 (MEDIUM, verified, FIXED)**: unqualified idents consumed dots,
  so `c.supersedes == x` parsed as one id literal and projection
  comparisons never fired. Fix: dots split; a dot after an unbound
  identifier is a file-qualified id literal, after a bound variable it
  is a row-typed projection — the binding decides (rule of record,
  pinned by test).

## Stage 3 — CLARITY

Module header carries the grammar, the opt-in boundary, and the
rationale; failure labels name the offending token and, for non-member
atomics, the atomic AND the closed set. No findings.

## Stage 4 — EDGE CASES (all verified by execution, pinned new)

- `acyclic` on a non-endo morphism → labeled (endo requirement).
- unknown morphism name in a reference atomic → labeled, naming the
  Reference Typing rows.
- unknown quantifier object → labeled, naming the closed five.
- ill-typed projection (morphism not sourced on the bounding object) →
  labeled.
- kernel marker on a non-invariant Constraint cell → labeled
  (`fragment_extraction`; the kernel owns invariant expr cells only
  this Revision).
- invariant cell opted in with a broken grammar → labeled at compile,
  stage `kernel_grammar`.
- double opt-in (`**kernel:**` + `**rust:**`) → labeled.
- three expr-cell languages (prose, citation, kernel) coexist in one
  table; prose stays byte-identical.
- empty-domain quantifiers decide honestly (∀∅ verified, ∃∅
  counterexample — bounded, finite, not vacuous-pass).
- unknown seeds/dangling comparisons honest (never fabricated).

Findings: 4 gaps, all fixed by new pins (12 grammar tests, 4
compile-edge tests).

## Stage 5 — EXCELLENCE (deferred of record, no action this phase)

- `Term::Proj` evaluation scans `morphism_values` linearly per lookup —
  O(n) per term, O(n²) per ∀ body. Termination holds (the D8 gate
  demands termination, not speed); a precomputed projection map in
  `KernelEnv` is a future shrink-only tidy.
- `parse_kernel_str` rebuilds `schema::canonical()` per call — same
  class, same disposition.

## Verdict

**PASS** — converged at stage 4; findings all fixed or pinned: 0
CRITICAL, 0 HIGH open, 2 MEDIUM fixed (F-1, F-2), 4 low edge gaps
pinned, 2 low perf notes deferred of record.

Gate evidence at review close: `just test` green (4 new suites, 31 new
tests); `just lint-specs` 0 issues; `uv run spk lint` on both delta
specs 0 issues; `ah check` pre-existing debt only (23 no-toml +
1 overlay-conflict); `openspec validate add-min-expr-kernel` valid;
fmt + clippy clean on touched files.

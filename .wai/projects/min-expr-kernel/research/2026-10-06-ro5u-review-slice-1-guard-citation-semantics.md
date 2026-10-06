# RO5U Review — add-min-expr-kernel slice 1 (specodelic-txo): guard citation semantics

Scope: `1649fbf` (GREEN) + `444ad1c` (refactor) + `beff19d` (fmt) —
src/compile.rs, src/model_check.rs, src/verify.rs, tests/citation_algebra.rs.
Method: Universal Rule of 5 (DRAFT → CORRECTNESS → CLARITY → EDGE CASES →
EXCELLENCE). Validation: self-reported, confirmed by executing
`cargo test` / scratch-parse probes against the real code (noted per finding).

## Stage 1 — DRAFT (shape/scope)

Overall shape is sound. The grammar + evaluator live in compile.rs beside the
spec language; model_check.rs only wires evaluation callers (re-export keeps
the API reachable); extraction is a pure widening of ModelIr; native/TLC
backends report all-unknown citations (honest, nothing executed);
run_executable discharges cited ids from exec outcomes. Tests are
integration-scope, respecting the model_check.rs file_lines ratchet
(now 2278/2300). Scope discipline holds: no behavior expansion, pretender
untouched, openspec/specs untouched.

## Stage 2 — CORRECTNESS

Kleene algebra verified against the delta scenarios
(openspec/changes/add-min-expr-kernel/specs/model-check/spec.md):
counterexample-dominant ∧, unknown absorbs, ¬ flips keeping unknown,
undischargable → unknown never pass (`evaluate_citation`, compile.rs).
`report_from_facts`: violated id → counterexample; completed-within-bound run
→ verified for the rest; truncated run → unknown. Matches
`run_report_status`. Serde: `#[serde(default, skip_serializing_if)]` keeps
old reports loadable; roundtrip test covers it.
**No correctness defects found.**

## Stage 3 — CLARITY

Doc comments are precise (closed grammar, prose-never-upgraded, honest
unknown). `strip_guard_cell` (introduced in 444ad1c) names the shared cell
normalization. `citation_statuses` doc states the undischargable rule.
**No clarity defects blocking ship.**

## Stage 4 — EDGE CASES (scratch-verified via parse probes)

**F-1 (Medium) — parser accepts `]` inside citation ids.**
- Location: src/compile.rs, `parse_citation_expr`, id guard
  `if id.is_empty() || id.contains('[') { return None; }`.
- Evidence (executed probe): `parse_citation_expr("[[a]b]]")` →
  `Some(Cite("a]b"))`. A citation id containing `]` can never match a real
  Constraints id (ids are filename stems — no `]`), so the cell is a
  malformed citation expression, yet it extracts as a citation and
  permanently evaluates `unknown`. This is asymmetric: `[` is rejected,
  `]` is not — and it contradicts the code's own documented stance that
  malformed cells "stay prose, not a silent upgrade."
- Concrete fix: reject ids containing `]` as well
  (`id.contains('[') || id.contains(']')`) + a pinning test in
  tests/citation_algebra.rs asserting `parse_citation_expr("[[a]b]]").is_none()`.

**F-2 (Low) — silent drop if re-parse fails at evaluation.**
- Location: src/model_check.rs, `citation_statuses` (`filter_map ... parse_citation_expr(cell).map(...)`).
- Evidence: cells were proven parseable at extraction time, so the `None`
  arm is unreachable today; but if it ever fired, the invariant would
  silently vanish from `invariant_statuses` instead of surfacing. Honest-
  unknown would be the safer shape. UNVERIFIED-as-reachable (unreachable by
  construction); fix if cheap: fall back to `ThreeValued::Unknown` instead
  of dropping, with a comment.

**F-3 (Low, informational) — `invariants_checked` vs `invariant_statuses`
partially duplicate the id list.** By design for backward compatibility
(`skip_serializing_if` keeps old consumers working). Track, do not fix in
slice 1.

**F-4 (Low, informational) — double-negation `¬¬[[a]]` parses to None
(stays prose).** Probe-confirmed. Closed slice-1 grammar is single `¬` per
term; conservative prose fallback is the documented behavior. No action —
note for the slice-2 kernel grammar work, where violations become labeled.

## Stage 5 — EXCELLENCE

Fmt/clippy(lib)/pretender clean; commit hygiene held (three atomic commits,
own files only). Nothing further blocking production quality.

## Convergence checks

- After Stage 2: 0 critical, 0 new issues → CONVERGED.
- After Stage 4: 1 new medium, 3 low; no criticals → CONVERGED.
- Final: **CONVERGED** — verdict PASS with findings.

## Verdict

**PASS** — 0 critical / 0 high / 1 medium / 3 low. F-1 (medium) fixed in the
fix-review step; F-2 fixed if cheap; F-3/F-4 tracked (F-4 noted for the
slice-2 kernel grammar increment).

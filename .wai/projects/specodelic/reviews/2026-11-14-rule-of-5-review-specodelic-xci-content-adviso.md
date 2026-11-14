---
reviews: 2026-11-14-green-implemented-content-advisories-in-src-lint.md
verdict: needs-work
severity: {critical: 0, high: 2, medium: 3, low: 4}
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-xci-advisory-warnings-for-placeholder-and-tautology-content, pipeline-step:fix-review]
---

# Rule of 5 Review — specodelic-xci content advisories

Method: rule-of-5-universal (Draft → Correctness → Clarity → Edge Cases → Excellence), applied to the working-tree diff, the new spec, and the touched generator/test surfaces. No project commands were run.

Counts: CRITICAL 0, HIGH 2, MEDIUM 3, LOW 4.

## CRITICAL (0)
None. The advisory channel is correctly separate from `Report::issues`: `lint_content_advisories` only pushes to `report.warnings`, `Report::failures()` counts issues only, and `cmd_lint` exits 0 when `failures == 0`.

## HIGH (2)
1. `tautological_predicate` judges every Constraint kind, not only `invariant`. Spec says “predicate or invariant expr”; code calls `inspect_row_cell` for every constraint without consulting `row.kind`, so effect/advisory/extension_point exprs warn. Fix: thread a `judge_tautology` flag (constraints: `kind == invariant`, properties: true); keep placeholder judging kind-agnostic.
2. The `scaffold_*` id class is unreachable for the real `migrate` sidecar and the spec names the wrong scaffold producer. `spk new` emits ids c1/t1/p1; `migrate` emits `scaffold_constraint/transition/property` with a custom statement and `true` cells, so the current signature gate is false. Fix: signature = placeholder statement OR any content cell placeholder OR tautology; update spec Purpose/`placeholder_classes` to name the real producers; test the migrate sidecar warnings.

## MEDIUM (3)
1. `tautological_predicate` judges intent statements (outside the spec class; only reachable on already-invalid EARS). Fix: remove the tautology branch from `inspect_statement`.
2. The spec Model contradicts the tautology class: a tautology-only corpus matches neither transition, and an invariant-expr tautology transitions to `clean`. Fix guards: `scan_clean` excludes predicate/invariant-expr tautology; `scan_warned` cites placeholder OR tautology.
3. The scaffold-id class has no positive isolated test: the TODO-cell warning message also contains the row id, so the assertion passes even if `warn_scaffold_id` never fires. Fix: isolated stub-signature fixture with an otherwise-clean `scaffold_*` row; assert the exact “carries the `scaffold_` placeholder prefix” message.

## LOW (4)
1. The conformance matrix has no case for either new rule. Fix: one TODO-expr case expecting `warn:linter.placeholder_content`, one `true`-predicate case expecting `warn:linter.tautological_predicate`.
2. `RULE_TABLE`'s `placeholder_content` semantics omits the scaffold-signature gate (and the tautology entry omits the `invariant` qualifier). Fix wording.
3. `TBD`, `FIXME`, and `⊤` have no positive tests. Fix: table-driven placeholder/tautology tests.
4. `spk orchestrate` never surfaces the new advisories because `advisory_findings` runs only observability. Fix: run both advisory families.

Verdict: NEEDS_REVISION — no CRITICAL; fix H1/H2, then M1–M3 and the LOW test/catalog gaps.

---
tags: [pipeline-run:tdd-ro5-2026-11-24-claim-coverage, pipeline-step:plan]
---

PLAN (tdd-ro5): claim coverage for 'spk check' — scope A of openspec/changes/add-check-claim-coverage.

DESIRED BEHAVIOR (from the change's delta + the user's 2026-11-24 rulings):
1. Every judged claim is named by its FILE-QUALIFIED id (<file id>.<claim id>) in the coverage section; never abbreviated, never matched by name/slug/similarity.
2. Placement rule (D8): a row places a claim only when it names the claim directly (model row key 'verify:model:<claim-id>'; oracle row's evaluated_claim_ids) or names a PROPERTY whose derives_from names that claim (one hop, the corpus's declared edge). Rows placing no claim go to the residual bucket (never dropped).
3. Fold (D9): a claim carries exactly ONE class from audit's closed set, the strongest among its placing rows — order contradicted > current > binds_nothing > unchecked; a claim placed by no row is 'unchecked'. EXTENSION (design gap found during planning, to be recorded as a D9 amendment in design.md + the delta): in a --before run the overlay can also produce 'stale' and 'new' rows, which the four-class order does not rank; the fold therefore ranks the full closed set contradicted > stale > new > current > binds_nothing > unchecked (a non-current label outranks 'current'; a contradiction outranks all). Documented in the change's design.md and the delta requirement text.
4. Materialization (D10 + ruling D-2): the section rides the CHECK ENVELOPE, not AuditReport. AuditReport gains NO field and its schema version/field set are unchanged. Concretely: a new payload type CheckPayload { #[serde(flatten)] report: AuditReport, claim_coverage: ClaimCoverage } — the envelope's 'data' therefore carries the audit report fields unchanged PLUS one new sibling key 'claim_coverage'. audit::parse_prior_report still accepts a check-produced envelope as a --before prior (serde ignores the unknown key), so report_interop and prior-diff interop are preserved.
5. Scope statement (D10 + ruling D-1 with amendment): the section carries a MACHINE-READABLE scope field, not prose only: coverage.scope = { id: 'this_run_producers', statement: '...' }. The id is the machine-readable discriminator; the statement is the human text. Both views render it.
6. Non-gating (D4): the section contributes counts only; the exit code is what the evidence rows alone determine. The counts appear beside the existing evidence counts AND on the human view's verdict line (first line), not in a trailing block.
7. Only 'invariant' constraints are listed (D2 narrowing); effect/advisory/extension_point claims are excluded.

OUT OF SCOPE (explicit):
- specodelic-qbox: wiring conform::gate_in_corpus into check (the artifact gate) — a separate bug; check's corpus-awareness is NOT touched here.
- Reading sidecar bindings / the sibling contract surface (scope B, demand-gated; check.no_foreign_authority).
- Any new AuditClass / new vocabulary value (check.one_vocabulary) — scripts/check_evidence_axes.py must pass with no new row.
- Changing AuditReport's schema version or field set; changing spk audit's behavior; changing the exit contract.
- The self-hosting exercise (tasks.md 6.5) — blocked on task 0.1 (specodelic-qbox).
- Stock openspec archive (the change archives only via 'just archive-change id=add-check-claim-coverage' and must ship the layer.md sidecar).

TEST CASES (RED first, one file: tests/check_claim_coverage.rs — library-level, fixtures only, never the corpus):
- T1 (2b) placement through a property: a spec with two declared invariant claims and a property deriving from one of them; a row naming that property places that claim and no other.
- T2 (2b) residual: a row naming neither a claim nor such a property (e.g. a model-level row) appears in the residual bucket and no claim carries it.
- T3 (2c) fold: a claim placed by a current row AND a contradicted row is listed once with 'contradicted'; a claim placed only by a vacuous bind row carries 'binds_nothing'; a claim placed by no row carries 'unchecked'.
- T4 (2d) materialization: the check envelope's data carries the audit report's schema version and field set unchanged (AuditReport deserializes from it), the coverage section rides beside the report, and the scope field is machine-readable (scope.id == 'this_run_producers').
- T5 (2d/4) rendering: the human view's verdict line carries the coverage class counts; the section states its scope in both views.
- T6 (3) vocabulary: the coverage section's class values are exactly audit's closed set (no new value); scripts/check_evidence_axes.py still passes.
- T7 (4) non-gating: a run whose rows are all current and which declares an unchecked claim exits 0 and the unchecked count is reported.
- T8 (1) ordering/stability: the same inputs judged twice produce the same claims in the same order with the same classes (byte-identical envelope).
- T9 (1) file-qualified keys: two files declaring the same claim id keep distinct entries; a near-miss slug places nothing.
- T10 (2) residual row class set: residual rows carry an existing class, no new spelling.

NARROW TEST COMMAND: cargo test --test check_claim_coverage   (then: cargo test --lib)
FULL VERIFICATION COMMAND: just ci  (fmt-check, clippy -D warnings, tests, release build) PLUS: openspec validate add-check-claim-coverage --strict; spk lint openspec (whole tree — the sidecar cannot be linted alone); python3 scripts/check_evidence_axes.py; ah check --changes add-check-claim-coverage.

FILES LIKELY TOUCHED:
- src/claim_coverage.rs (NEW) — the section's types and assembly (placement, fold, residuals, counts, scope).
- src/check.rs — assemble the section after the row fold; emit the check envelope (JSON + human); verdict-line counts.
- src/audit.rs (MINIMAL) — one rendering hook so the human view keeps ONE formatter (the check verdict line + section lines); AuditReport itself untouched.
- src/lib.rs — register the module.
- tests/check_claim_coverage.rs (NEW) — the RED tests.
- docs/src/graded-evidence.md — task 6.1 (a NEW section; the evidence-record axes table stays untouched).
- openspec/changes/add-check-claim-coverage/design.md, tasks.md, specs/check/spec.md — the D9 stale/new amendment, the task checkboxes, and the stale 'two ADDED requirements' text in tasks.md 7.1 (the delta carries five).
- .beads/issues.jsonl — bd export after the ticket update.

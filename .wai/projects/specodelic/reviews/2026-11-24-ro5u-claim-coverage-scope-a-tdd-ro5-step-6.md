# RO5U review — claim coverage (tdd-ro5 step 6, 2026-11-24)

**Work reviewed:** the tdd-ro5 diff — `src/claim_coverage.rs` (new, 285 lines),
`src/check.rs` (CheckPayload + payload/human_view/emit_judged/emit_payload),
`src/audit.rs` (human_view_labelled_extended), `src/lib.rs` (module reg),
`tests/check_claim_coverage.rs` (12 tests), and the change-artifact amendments
(design.md D9 amendment, delta fold order, layer.md claim_coverage_listed,
tasks.md 7.1).

## Stage 1 — DRAFT

Shape GOOD — one new module with a single responsibility
(placement/fold/residuals/counts/scope), the payload type in check.rs (the
envelope is check's), one extended renderer in audit.rs keeping the
single-formatter convention (audit's 2-arg `human_view_labelled` delegates with
empty additions, so `spk audit` output stays byte-identical — pinned by the
audit suites passing). No structural findings.

## Stage 2 — CORRECTNESS

EXCELLENT — 0 CRITICAL, 0 HIGH. Verified against evidence:

- (a) `named_ids` strip/split handles `verify:prop:<id>:<case>` law-case keys
  (first segment), `verify:model:<claim>`, `bind:<owner>`, and the bare
  `verify:model`/empty-name case lands residual per the delta scenario.
- (b) oracle rows place claims only via `evaluated_claim_ids` — scenario-id
  keys are never name-matched.
- (c) `resolve_claim` is exact-match bare or file-qualified, so a resemblance
  cannot place (`linter.unique_id` guarantees no claim/property id collision
  within a file, so the property hop is unambiguous).
- (d) `cell_targets` handles multi-link cells and bare `derives_from` ids
  (rename's bare-target precedent).
- (e) fold via `class_rank` covers the full closed set — the D9 gap
  (stale/new unranked) was found and the design.md/delta/layer amended
  consistently (`openspec validate --strict` green, `spk lint openspec/specs`
  0 issues).
- (f) `report_interop` preserved: `AuditReport` derives Deserialize without
  `deny_unknown_fields`, so a check envelope parses as an audit `--before`
  prior — pinned by test.
- (g) audit byte-identity preserved (460 lib tests + audit_* + check_* suites
  green).
- (h) non-gating: the exit code is computed from the report's rows only — the
  payload is assembled after `run()` returns; pinned by
  `an_unchecked_claim_does_not_fail_the_run`.

## Stage 3 — CLARITY

GOOD.

- **[CLAR-001] LOW** —
  `openspec/changes/add-check-claim-coverage/specs/check/spec.md`, "strongest
  class" requirement: the fold-order edit left an awkward wrap ("A claim placed
  / by no row SHALL carry / `unchecked`."). Fix: rewrap the sentence. Impact:
  readability only; `validate --strict` already green.

## Stage 4 — EDGE CASES

EXCELLENT — no new findings. Checked: zero-invariant-claim file ("claims: none"
summary), zero-row report (all unchecked + empty residuals), duplicate claim ids
across files (the file-qualified test), near-miss property id (residual),
multi-claim `derives_from` cell, exit-1 findings envelope carrying the payload
(audit's `findings_failure` works over any Debug+Serialize payload).

## Stage 5 — EXCELLENCE

- **[EXCL-001] LOW** — `src/claim_coverage.rs` `named_ids` knows the three
  current producer prefixes; a future producer with a new key shape defaults to
  the residual bucket (safe-by-default, never a wrong placement) — document
  this property in `docs/src/graded-evidence.md` when task 6.1 lands (deferred
  to that task, not the diff).
- **[EXCL-002] LOW** — `check::payload` clones the report once per invocation;
  fine at CLI scale, no action.

## Convergence

New CRITICAL 0; new issues stage3: 1, stage4: 0, stage5: 1 (both LOW) —
**CONVERGED at stage 2** (the severity floor never rose).

## TypeSafe verification pass

`TYPESAFE_API_KEY` present; VERIFY_SEVERITIES = CRITICAL,HIGH — this review has
ZERO findings at or above the verification severities, so zero verification
questions are generated; all findings (2 LOW) are below VERIFY_SEVERITIES →
UNVERIFIED by rule, excluded from FP-rate arithmetic. No banner required (the
key was present and no API call failed).

## Verdict

**READY** — 0 CRITICAL, 0 HIGH, 2 LOW (1 fix now: CLAR-001 rewrap; 1 deferred to
task 6.1: EXCL-001).

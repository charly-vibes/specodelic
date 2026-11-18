# Change: Evidence record axes — one vocabulary, checked against the code

## Why

D-1 asked how to reconcile three evidence vocabularies (`meth-research-decision-memo.md` §3): the four
interoperability primitives, the audit class set, and conform's verdicts. The
research answer is that they are **three layers of one record**, not three
competitors. The code already behaves that way — audit rides conform's verdicts
(`src/audit.rs:110`, `AuditClass::of`) and ships computed `stale` (`--before` +
`scope_sha256`), `binds_nothing` as a citation face, and the both-layer record —
but **nowhere states which value belongs to which axis**, and the page that
documents the vocabulary (`docs/src/graded-evidence.md` §"The closed class
vocabulary") can drift from the enums without any gate noticing.

Two facts from the D-1 groundwork constrain the change:

1. **T1 verdict (`.wai/resources/research/2026-11-18-t1-clause-audit.md`):**
   `inconclusive` is a borrow (ISO/IEC 9646-1 = ITU-T X.290 3.3.124, 6.5.1.3),
   and absence ("not executed") is a recorded test-case error/termination
   (6.5.1.4–6.5.1.6), never a verdict.
2. **The memo's proposed code additions have no producer today.** No conform
   path can return `inconclusive` (the walk classifies evaluator kinds; it does
   not evaluate claims three-valued — kernel `Unknown` is an unresolved id, a
   spec defect). `origin` and `binding_kind` have no input source or consumer
   yet; `bound` already exists as `model_check::Bound` in run reports. Adding
   those values now would ship dead enum arms and unpopulated fields.

So the change implements the part that is real today — the **record
specification** and its **enforcement** — and records the reserved values with
their producers instead of shipping placeholders.

## What Changes

- **Normative five-axis table** in `docs/src/graded-evidence.md`: run status,
  spec defect, currency, knowledge absence, scope exclusion; plus the annotation
  fields. Every row names its code source and is marked `implemented` or
  `reserved (trigger: …)`. The table states the `AuditClass::of` mapping
  verdict-by-verdict.
- **Drift gate**: `scripts/check_evidence_axes.py` (+ stdlib unittest) parses
  the closed sets from `src/conform.rs` and `src/audit.rs`, parses the table,
  and fails when the two disagree in either direction, when a documented
  mapping differs from `AuditClass::of`, or when a `reserved` row drops its
  trigger. Fail-closed on unparseable input. Wired into `just ci` and the
  existing `sync-sections-test` discovery.
- **Recorded D-1 decisions** (design.md): `inconclusive` keeps its borrowed
  name and is reserved until a producing runner exists (D-10 `check` fusion or
  the first contract-suite runner); `origin`/`binding_kind` are reserved to the
  changes that introduce their annotation syntax and binding consumer; `bound`
  maps to the existing `model_check::Bound` and is not added to audit records
  until a consumer reads it.
- **No Rust behavior change.** No verdict value, record field, CLI flag, or
  exit-code change.

## Impact

- **Affected specs:** `audit` (one ADDED requirement).
- **Affected code:** `docs/src/graded-evidence.md`; `scripts/check_evidence_axes.py` (new);
  `scripts/test_check_evidence_axes.py` (new); `justfile` (one recipe + `ci` wiring).
- **Not affected:** `conform`/`audit` behavior, report schemas, exit codes,
  `spk lint` rules, packs.
- **Risk if skipped:** the vocabulary stays three undocumented layers, and the
  next change that quotes an axis name has no source of truth to check against.
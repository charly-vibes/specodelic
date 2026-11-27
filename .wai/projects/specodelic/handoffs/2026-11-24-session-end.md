---
date: 2026-11-24
project: specodelic
phase: research
---

# Session Handoff

## What Was Done

qbox session (2026-11-24, tdd-ro5 pipeline): fixed specodelic-qbox — check's
artifact gate lints the judged file within its corpus
(discover_openspec_root + parse_batch, labeled isolated-file fallback), so
corpus capabilities reach verify_model instead of refusing at
artifact_gate. Shipped as 23ebc98b; handoff scaffold committed as 326cbb8b;
pushed (f0748a23 was the run's HEAD at the after-qbox entry). Exhibits: 4
contracts in tests/cli/check.rs (56 passed), clippy clean, just ci exit 0,
ah check --run-tests 758/0.

Post-qbox: authored openspec change add-kani-proof-harnesses (Kani harness
suite over verdict-deciding internals; quint.sh FVT grading 2026-11,
research doc .wai/resources/research/2026-11-18-quint-fvt-tools-grading.md)
— left UNCOMMITTED at session end; committed 2026-11-27 with ticket
specodelic-i9th.

## Key Decisions

- Kani over Verus/Prusti/Creusot: cargo-native, MIT, harness shape fits the
  pure-function internals without a forked toolchain (grading: Kani A/A,
  Stateright A+/A+).
- `just kani` explicitly NOT in `just ci` — Kani runtimes too slow for the
  per-commit gate; run points are pre-push/nightly (design D4/D6).
- Zero-production-change stance: a red harness counterexample is a finding
  to file, not something to silently fix inside this change.

## Gotchas & Surprises

- The wai close scaffold was committed with placeholder prose sections
  unfilled — filled 2026-11-27 during housekeeping; the stale
  `.wai/pipeline-runs/tdd-ro5-2026-11-24-qbox-*.yml` "modified" line in the
  auto git_status had already shipped as 23ebc98b + 326cbb8b.

## What Took Longer Than Expected

(2026-11-24 session ran clean; no multi-attempt steps recorded in the
branch notes for qbox.)

## Open Questions

- None blocking. Backlog hygiene ticket specodelic-cgfm remains the
  standing cleanup decision point.

## Next Steps

1. specodelic-rzwg (P1, parked 2026-11-25T14:52Z): trust-ledger change
   authored + Rule-of-5 reviewed + committed (20c7ab53, 681d8cac);
   implementation NOT started — tasks.md Slice 1 (`src/verify.rs`:
   TrustBase derivation) is the next action.
2. specodelic-i9th: implement add-kani-proof-harnesses (tasks.md Slice 1:
   separation property first, then `just kani` wiring).

## Context

### git_status

```
 M .wai/pipeline-runs/tdd-ro5-2026-11-24-qbox-artifact-gate-lints-the-judged-file-in-corpus.yml
?? .dont/
?? .wai/projects/specodelic/.pending-resume
?? .wai/resources/research/2026-11-18-quint-fvt-tools-grading.md
?? openspec/changes/add-kani-proof-harnesses/
```

### open_issues

```
○ specodelic-an79 P1 Research: shift-left ledger — does spec-first discipline reduce back/forth?
○ specodelic-v64i P2 Fleet compliance: bd-hooks chaining in lefthook + root CHANGELOG.md
○ specodelic-zbzq P2 [bug] docs: book pages for USAGE/STATUS/theory/CHANGELOG 404 on the deployed site
○ specodelic-0xnd P3 import acset.generic as openspec change — schema-parametric incidence core, benchmark-first (external handoff, at/arltificial/renzi)
○ specodelic-20q3 P3 Oracle-validity probes for espectacular contract tests — a green contract test proves scenarios, not that tests would notice a violation (vacuous-pass check)
○ specodelic-2a2g P3 demonstrate staleness on a real evidence pair across one revision (scope A, decision 2026-11-18 section 5.1.3)
○ specodelic-2fp1 P3 docs: preservation recipe — conform twice + verdict diff (simplification review)
○ specodelic-30s5 P3 origin field — authoring-lineage annotation syntax (reserved trigger, add-evidence-record-axes)
○ specodelic-3xc P3 decide: fan-in multiplicity rule — state_view counts (source,field) pairs, traceability_view counts distinct sources (B3)
○ specodelic-5tm7 P3 experiment E3: composition — state growth inlined vs hand-factored
○ specodelic-7e7b P3 decide: meter spk orchestrate wall-clock before changed-scoping work
○ specodelic-b3nv P3 warrant probe: seeded-defect replay on one module — do the tool's drift classes change when known defects are injected? (decision 2026-11-18 section 5.1.5, optional)
○ specodelic-cgfm P3 chore: backlog hygiene — 39 open issues missing template sections, 42 stale 14+ days, orphans heuristic false-positives
○ specodelic-f046 P3 docs: detection-class table + ladder empirical cross-link (catitas C2+C3)
○ specodelic-fem0 P3 docs: kata-workflow page + FAQ entries (catitas C4+C5)
○ specodelic-g1h8 P3 Parked: Wild contract-bridge experiment (design-substrate report §11)
○ specodelic-glbv P3 verify: routed _props.rs path and end-to-end duplicate-stem verify are untested
○ specodelic-igh6 P3 paired-attempt experiment: next comparable feature tickets run one spec-first, one straight-to-implementation (an79 follow-through)
○ specodelic-izsx P3 experiment E1: law substitution — does law_cases pressure simplify without detection loss
○ specodelic-j64w P3 [bug] openspec: add-py/add-ts fragment-emission deltas carry a MODIFIED header absent from the deployed compile spec
○ specodelic-jwjj P3 extract rust generator-strategy half into its own demand-gated change proposal (l8wp decision a)
○ specodelic-l54a P3 docs: per-file ASCII state-machine views — unblocked by gently-hpd closure
○ specodelic-m69q P3 docs: document single-input generator limitation (catitas A6-doc-half)
○ specodelic-mn77 P3 inconclusive producer — verdict for a run that executes and cannot decide (reserved trigger, add-evidence-record-axes)
○ specodelic-pjyn P3 experiment E4: vacuity-guided elimination on the study's final models
○ specodelic-qz7c P3 claim reasons: composed counterexample shapes still carry none
○ specodelic-uyfi P3 docs: bowling head-to-head evidence case study (catitas C1)
○ specodelic-yfib P3 binding_kind consumer — check reading sidecar bindings (reserved trigger, add-evidence-record-axes)
○ specodelic-mngq P4 [bug] investigate ah auto-report issue #12: exit-1 error report with no error text (gh-12)

--------------------------------------------------------------------------------
Total: 29 issues (29 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```


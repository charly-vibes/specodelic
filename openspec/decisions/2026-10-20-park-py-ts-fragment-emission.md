# Decision: park the Python and TypeScript property-emission features (l8l / aby)

*Decision note, 2026-10-20. Rule-of-5 review of the plan of record
(verified by TypeSafe jev-1.13.0: all HIGH findings verified, 0 dropped);
owner instruction to park the same day.*

## What is parked

- **specodelic-aby** (add-ts-fragment-emission) and all children —
  deferred of record, indefinitely.
- **specodelic-l8l** (add-py-fragment-emission) and all children —
  deferred of record, indefinitely, **except** the D5 agreement slice,
  which is split out as specodelic-y5x8 (open, not parked).

The proposals stay in `openspec/changes/` as written (both already said
"Pending approval"); they were never archived, so nothing deployed
changes. `**py:**` and `**ts:**` fragments keep failing compile labeled
(`no_emitter_labeled_failure`) — that failure was always honest, and
stays the behavior of record.

## Why

The format's own positioning test (USAGE.md, Positioning section) decides
this: *will anything mechanical consume the structured rows?*

1. **No consumer exists.** Zero `py:`/`gen:` cells across all 59 deployed
   corpus files; every dogfood predicate is a parameterless helper call,
   so the BREAKING `gen:` opt-in and adequacy policy would apply to zero
   existing rows. Zero external requests across the repo's 22 gh issues
   (which instead report dual-format, fmt-churn, and conform gaps).
2. **The demand signal points elsewhere.** The external reviews' single
   most convergent demand was real-valued/quantitative predicate
   checking — neither proposal delivers that. The py/ts pair answers an
   internal demand (closed-tag-set coherence + deferred-of-record
   follow-ups), not an external one.
3. **TypeScript fails the test outright.** Its stated rationale is
   reuse/symmetry with Python; no TypeScript project exists in scope.
   A second full runner surface (Vitest/fast-check subprocess,
   process-tree cleanup, three-language accounting) cloned from a
   feature that itself lacks a consumer is pure maintenance liability
   (Hypothesis/fast-check version drift, per-runner support forever).
4. **Reversal is cheap.** Nothing was built: the epics were XL plans
   with no implementation commits. Parking now costs only the two
   proposal documents; re-activating later re-reads them.

## What is NOT parked — the split-out slice

**specodelic-y5x8 — Python kernel agreement backend** (from
add-py-fragment-emission design D5, tasks 4.1–4.2): a Python reference
evaluation adapter for the closed kernel vocabulary over the 36n
fixtures, with parity tests for every v0 atomic, Kleene composition,
bounds, empty domain, missing reference and cycle. It cross-checks the
production Rust kernel evaluator with an independent implementation and
needs neither `gen:`, `py:` predicates, nor a PropertiesRunner. It is an
agreement-test backend — not a user-facing emitter, not a production
`--backend` selector, not a Rust-to-Python translator. While y5x8 is
open, l8l.7/l8l.8 (same scope, parked) must not be implemented.

## Re-activation gate

Un-park l8l only when a **real** consumer materializes: a corpus
(specodelic's or a downstream project's) with predicates that want
compiled generated-input properties in Python. The first step of any
re-activation is migrating at least one such predicate to the `gen:`
grammar *before* building emitters — the demand-side proof the current
plan never had.

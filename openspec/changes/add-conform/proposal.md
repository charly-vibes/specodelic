# Change: Add conform — external-oracle trace conformance for spec files

## Why

The pipeline today verifies a spec **against itself**: lint → compile →
model-check → verify all evaluate the spec's own claims and structure
(verification claims are bounded model checking of opted-in invariants;
`exploration_only` is never a clean verdict). Nothing checks a spec against
**reality** — an external scenario corpus or oracle recording of what an
existing system actually does. This is the one direction the format cannot
yet speak, and it is exactly the capability needed to (a) recover a contract
from a legacy SUT without laundering its accidental structure into a new
design, and (b) surface *underspecification* instead of silently
accommodating any observed behavior.

External review grounding (2026-10-09, `~/Downloads/cv` bundle): the
"Specodelic as a Design Substrate" report (§7.2, §13) marks external-oracle
trace conformance as guarantee-ladder level 4 — **PROPOSED**, the first level
not yet backed by any shipped verb — and names its failure criteria: missing
transitions must never read as forbidden without declared closed-world
semantics, and uninterpreted prose must never silently count as checked.
The proposal was reviewed under a Rule-of-5 pass (converged Stage 4,
TypeSafe-verified); its binding corrections are folded in as decisions
D1–D6 in `design.md` — notably the closed-world gate (EDGE-001), the
distinct `underspecified` verdict, and the MVP boundary (read-only
evaluation; no active hypothesis testing).

## What Changes

- **Tool — `spk conform <spec-path> --oracle <scenarios.jsonl>`:** a new
  read-only evaluation verb. It consumes the linted/compiled spec artifacts
  (never the domain implementation, never a generated one) and an external
  JSONL scenario corpus, and classifies each oracle trace with a verdict
  from a closed taxonomy: `permitted`, `forbidden`, `underspecified`,
  `unknown`, `unsupported` — each verdict record carrying its reason and
  the evaluated claim ids.
- **Closed-world gate:** the two prohibition evidence classes are
  separated. A trace that positively violates a declared executable claim
  is `forbidden` in any mode (the same fact model_check reports as
  `counterexample_found`). A trace *no declared claim or Model element
  covers* is `forbidden` only when the run is invoked with `--closed-world`
  (an explicit, recorded declaration that the spec is intended to be
  exhaustive); in the default open-world mode it is `underspecified`,
  never `forbidden`.
- **Verdict taxonomy is closed and distinct:** `underspecified` (no declared
  claim covers the trace) ≠ `unknown` (a claim covers it but the checker
  cannot interpret its content) ≠ `unsupported` (the claim's evaluator kind
  is not executable in this run). Reuses the claim classification and scope
  digest machinery from `specs/model_check.md` — no new status vocabulary.
- **Prose discipline:** verdicts derive only from structural model facts and
  executable claims (kernel, `**rust:**` fragments, citations). Prose-only
  claims contribute `unknown` with reasons — mirroring
  `required_claims_classified`, never implied verified.
- **Report schema:** JSON envelope by default (pipes), `--human` for TTYs;
  per-trace verdict records plus a scope digest binding parsed structured
  content and the consumed scenario corpus — same binding discipline as the
  model_check run report.
- **Spec — `conform` capability:** dual-format capability spec under
  `openspec/specs/conform/` on archive (this change's delta drafts it).
- **Docs:** `spk explain` gains no new topic in this change; the existing
  lifecycle topic is untouched. The docs book's pipeline section gains a
  conform page stating the evidence scope: a finite oracle corpus
  establishes *agreement on those examples*, never behavioral equality.

> **Sequencing note:** `add-py-fragment-emission` and
> `add-ts-fragment-emission` extend the compiler's executable-claim surface;
> conform is a read-only consumer of compile output and must not modify
> `src/compile.rs`'s claim model concurrently with them. If those changes
> are mid-flight, conform consumes the claim classification as-is.

## Impact

- **Affected specs:** new `conform` capability (this delta); no MODIFIED
  requirements on existing capabilities — `model_check`'s claim
  classification and report schema are consumed, not changed.
- **Affected code:** new `src/commands/conform.rs` (or module) + one
  `Commands` variant in `src/main.rs`; consumes existing
  parse/lint/compile/claim-classification types. No changes to the linter,
  compiler, or model checker.
- **Workflow:** conform sits **outside** the artifact lifecycle
  (`draft → … → verified`) — it evaluates evidence, it does not advance or
  gate lifecycle stages; `orchestrate` is unchanged and never runs it.
  If D4 consumption requires visibility changes in model_check types, a
  MODIFIED delta on model-check is filed as a follow-up, never absorbed
  silently.
- **Failure-culture:** this change implements the external report's failure
  criteria verbatim (no missing-transition-as-forbidden without
  completeness rule; no silent prose treatment; agreement ≠ equality), so
  the tool cannot fabricate verdicts from underspecification.
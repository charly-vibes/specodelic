# Decision: rust generator-strategy sequencing (specodelic-l8wp)

**Date:** 2026-11-12 · **Ticket:** specodelic-l8wp (catitas item A6) ·
**Status:** decided · **Follow-up:** specodelic-jwjj

## Question

Generators compile to `Just(name)` constants (`src/compile.rs:917`) — one
input per property. The fix (typed proptest strategies) sits inside the
parked change `add-py-fragment-emission`'s declared scope. Decide how to
sequence it without reopening the parked decision
(`decisions/2026-10-20-park-py-ts-fragment-emission.md`):

- **(a)** extract the rust-strategy half into its own demand-gated change
  proposal;
- **(b)** fold it into the parked change's re-activation.

## Decision

**(a) — extract the rust-strategy half into its own demand-gated change
proposal.**

## Rationale

1. **The rust path is the only executable path today.** Revision 15/16
   landed `**rust:**` fragments as the sole executable tag; `**py:**`/`**ts:**`
   are accepted-then-refused (`no_emitter_labeled_failure`). A typed-strategy
   fix for rust generators is live, useful work on the live path.
2. **Concrete demand signal exists.** The pending corpus property-cell
   migration (specodelic-lf3; 319 `todo_predicate!` rows across 23 files)
   requires adequate generators — its own notes forbid `Just(name)`
   scaffolds as acceptance evidence. Waiting on the parked decision would
   couple that demand to a re-activation the parking decision says must be
   demand-gated on py/ts consumers.
3. **Scope hygiene.** The parked change remains the py/ts record, untouched.
   Extraction keeps the rust work small, independently gateable, and
   reviewable.

## Constraint (EDGE-001, honored explicitly)

**Py/TS emission remains parked and requires its own proposal — no
bundling.** This decision authorizes only the rust-strategy extraction and
explicitly does not re-open `add-py-fragment-emission`, does not add py/ts
emitters, and does not alter the closed tag set.

## Demand gate

The extracted proposal (`specodelic-jwjj`) starts as design-only. It may
be implemented when either condition holds:

- the corpus property-cell migration (specodelic-lf3) is authorized and
  needs real generators (the current expected trigger), or
- another demonstrated consumer of typed rust strategies appears.

Absent either, it stays parked like its py/ts sibling.

## Consequences

- `specodelic-l8wp` closes with this record (the sequencing decision is the
  deliverable; no openspec proposal is created here by design).
- `specodelic-jwjj` tracks the extraction (proposal scaffold + demand gate).
- No code, corpus, or CI change in this decision.
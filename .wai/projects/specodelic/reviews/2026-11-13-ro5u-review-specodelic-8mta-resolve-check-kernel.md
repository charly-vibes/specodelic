---
reviews: 2026-11-13-green-specodelic-8mta-kernelenv-gains-a-field.md
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-8mta-resolve-check-kernel-id-literals, pipeline-step:ro5u-review]
---

# RO5U review — specodelic-8mta (resolve-check kernel id literals)

Scope reviewed: the working-tree diff (`src/kernel.rs` — the declared-id set,
`term_value`/`eq`/`cmp`/`atomic`; `src/guide.md`; `openspec/specs/kernel/spec.md`;
`tests/kernel_literals.rs` (new); `tests/common/kernel_corpora.rs` (the TRAFFIC
fixture); `tests/cli/model_check.rs` (one new CLI test)).

## Critical

None.

## High

None. The change is total and terminating (one `BTreeSet` lookup per literal —
no new recursion, no scan), the Kleene composition is untouched, and the corpus
verdicts are identical before and after (measured: `model-check
openspec/specs/specodelic` — `coverage`, `every_transition_valid`,
`supersedes_acyclic`, `total_refs` all `verified` before AND after).

## Medium

1. **`src/kernel.rs` — `Atomic::Reachable`: the target resolve-check runs only
   AFTER `forward_closure` succeeds**, so a claim with BOTH an unresolvable
   seed and an unresolvable target reports the seed.
   *Evidence:* the traversal arm returns early on `Err`; the target check sits
   inside the `Ok` arm.
   *Assessment:* DELIBERATE — the design of record keeps the seed's existing
   unknown path and its labeled reason untouched, and the target is still never
   read as a refutation.
   *Fix:* none (documented in the code comment).

2. **`src/kernel.rs` — `Atomic::Eq`/`Cmp` thread `ra.or(rb)`**: when the LEFT
   term is indeterminate for a cause-less reason (an unbound `Var` — reachable
   only from a hand-built expression; parse-time row-typing forbids it) and the
   RIGHT term is an unresolved literal, the literal's reason wins and masks the
   unbound var.
   *Evidence:* `ra.or(rb)` takes the left cause only when it is `Some`; a
   cause-less `Unknown` yields `None`.
   *Severity:* LOW in practice — the parser never produces an unbound `Var`
   term.
   *Fix:* none.

3. **`src/kernel.rs` — the shared reason text says "id literal" for
   `reachable`'s TARGET too** (a reachable endpoint, not a term).
   *Evidence:* `unresolved_id_reason` is used by both call sites.
   *Assessment:* the text was prescribed by the design of record and names the
   offending id, so a reader can fix the cell.
   *Fix:* none.

4. **OBSERVATION (NOT fixed — out of scope, reported per the brief):**
   `Atomic::Resolves(m)` is verified VACUOUSLY when the corpus has no values
   for that morphism at all (`morphism_values(m).iter().all(|(_, t)|
   t.is_some())` is vacuously true over an empty vector).
   *Evidence:* `src/kernel.rs`, the `Atomic::Resolves` arm.
   *Assessment:* a second, genuinely different vacuity shape from F1 (an absent
   population rather than an unresolved literal). It predates this ticket and
   is defensible under the constraint's own wording ("no dangling references").
   *Fix:* NONE HERE — not silently widened; reported for the orchestrator to
   triage as a separate ticket.

## Low

- `src/kernel.rs` — `unresolved_id_reason` is a free function declared after the
  `TermVal` enum, below both call sites. Nit; the doc comment states the "ONE
  text" intent. No fix needed.
- `tests/kernel_literals.rs` — the fixture's property row carries the `true`
  predicate the frozen corpora in `tests/common/kernel_corpora.rs` already use
  (`CLEAN`/`BROKEN`/`UNKNOWABLE`/`ABSORBING`). It is an inert corpus fixture
  (the kernel claims are what the tests assert), not a softened assertion. No
  fix needed.

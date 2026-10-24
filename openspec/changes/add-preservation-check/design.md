# Design: add-preservation-check

## Context

The kata studies showed the tool can represent legality exhaustively but
offers no check that a revision preserves or intentionally changes behavior
(O4). The mistral mandate requires preservation obligations to name: original
model, new model, observation boundary, preserved examples, intentionally
changed behaviors, and unproved obligations. `spk conform` already classifies
external traces against a compiled Model with a closed verdict taxonomy and
evidence-scope labeling — the exact machinery a preservation report needs.

## Goals / Non-Goals

- Goals: example/oracle-level preservation classification between two model
  revisions of the SAME capability file; honest labeling; zero new semantics.
- Non-Goals: any claim of behavioral equivalence or refinement; trace-level
  agreement beyond conform's existing walk; automated refactoring; diffing
  anything other than Model relations + oracle verdicts.

## Decisions

- **Decision: diff conform verdicts, do not invent a new comparator.** The
  same oracle corpus runs through the conform walk for both compiled files;
  a scenario whose verdict class is equal is `preserved`, different is
  `intentionally_changed` (declared) or a finding when undeclared.
  - Alternatives considered: comparing compiled TOML bytewise (fragile,
    prose-encoding-coupled — rejected); embedding examples in the spec and
    re-validating them (that is A3's scope, separate change — rejected here).
- **Decision: undeclared verdict changes are findings; declared ones are not.**
  The declaration channel is preserve-owned, NOT conform's oracle record:
  conform's record schema is closed (`deny_unknown_fields`,
  src/conform.rs:245,257 — unknown fields are refused, never ignored), so a
  new field there would be refused and the proposal's "no existing capability
  requirements modified" claim would be false. Instead:
  `spk preserve --declarations <changes.json>` consumes a separate file of
  `{scenario_id, expect_change}` records with its own
  `deny_unknown_fields` struct — a typo'd field fails loudly there,
  conform's schema stays untouched, and an undeclared verdict difference
  remains a finding.
- **Decision: evidence_scope copied verbatim from conform's envelope.**
  "agreement on the supplied corpus; not a proof of behavioral equality" —
  the anti-over-claiming lesson from the corpus (review CORR-001).

## Risks / Trade-offs

- Risk: users read `preserved` as equivalence → mitigated by scope labeling
  in every record + docs page stating the boundary.
- Risk: oracle corpus drift between the two runs → `scope_sha256` of the
  corpus is reported for both sides; a mismatch is a labeled finding.
- Risk: conform's `unknown` (prose guards) dominates → the report says
  `unchecked` per scenario and counts them; the count is the honest signal.

## Open Questions

- Should `preserve` gate a lifecycle transition (archive/rename) later?
  Deferred — v1 is read-only classification, exactly like conform.
- Could the declarations file instead ride inside the oracle corpus under a
  preserve-specific top-level key? Rejected for v1: it would require
  extending conform's parse of the same file, reintroducing the coupling
  this design avoids.

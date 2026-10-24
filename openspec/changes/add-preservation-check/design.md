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
  The delta's declaration channel is the oracle record's optional
  `expect_change: true` field — matching conform's
  `deny_unknown_fields` style, so a typo'd field fails loudly rather than
  silently meaning nothing.
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

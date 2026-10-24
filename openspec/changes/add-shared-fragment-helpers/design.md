# Design: add-shared-fragment-helpers

## Context

Fragments are emitted per-row into `*_props.rs` blocks. Today the only way to
share a helper is to paste it into every row (3× growth observed; two full
copies of the model table in the bowling study). The format has no
composition construct.

## Goals / Non-Goals

- Goals: named, per-file shared definitions usable from any fragment in the
  same file; single emission path; zero effect on files that don't use it.
- Non-Goals: cross-file sharing (corpus-wide name resolution is a later
  decision); changing fragment grammar or the closed tag set; py/ts
  emitters (parked).

## Decisions

- **Decision: inline at use site, not a shared module.** Emitting one shared
  module would change the scratch-crate layout every consumer of artifacts
  assumes; inlining keeps per-row blocks self-contained and the compiled
  artifact contract unchanged.
  - Alternatives considered: `mod helpers;` per file (breaks artifact
    self-containment; rejected); corpus-level helper packs (scope creep;
    rejected).
- **Decision: `def <name>:` fenced bodies inside `## Fragments`.** Extraction
  stays textual; references resolve by name to the single definition; a
  definition referenced by zero rows warns (warnings channel) — dead helper.
- **Decision: collision → labeled failure.** Generated symbols already own a
  prefix namespace; a user definition colliding with one fails labeled
  naming both symbols (never a silent shadow).
- **Decision: hygiene is inherited, imprecision documented.** The substring
  token ban already misfires on names like `unsafe_state` (O14, known);
  helpers inherit the same rule rather than a better one, so behavior is
  uniform; improving the hygiene check is a separate decision.

## Risks / Trade-offs

- Risk: inline duplication re-grows artifacts for many-reference helpers →
  acceptable: artifact size is not spec-authoring size; revisit only with
  evidence (corpus discipline: smallest change that enables a demonstrated
  improvement).
- Risk: format revision bump → the layer sidecar extension carries it;
  lint gate unchanged (`model_present` etc. untouched).

## Open Questions

- Should `## Fragments` definitions be visible to invariant fragments and
  property fragments alike in v1? Proposed: yes — both are `**rust:**
`  bodies with identical hygiene.

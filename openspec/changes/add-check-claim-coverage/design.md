# Design — add-check-claim-coverage

## D1: The claim key is the file-qualified claim id (chosen)

**Chosen:** `<file id>.<claim id>` — e.g. `audit.pack_row_collision_follows_the_same_rule`.

Rejected:

- **The bare claim id.** Not unique: the corpus declares 452 constraint ids
  and 28 kebab-style slugs collide across files (`process-lifecycle` appears
  in 21 capability files). A bare id is ambiguous by construction.
- **A slug or filename, matched by similarity.** Refuted by measurement: of
  643 sidecar files in this workspace, only 5 ids match any constraint slug
  at all, and where they do the match is a coincidence of naming, not a
  declared relation. Inference by similarity is a guess; a guess that binds
  evidence to a claim is worse than no binding.
- **A new generated claim uid (a `uid:` column, a hash).** It would be
  unique, but it would be a *second* name for a claim the corpus already
  names uniquely — and every external binder would have to learn the
  generated form. The file-qualified id is already unique corpus-wide
  (Revision 18), already stable across renames that preserve ids, and
  already what `spk graph` and the corpus-wide reference resolver use.

Consequence: the join key is **decided in scope A**, before any scope-B
consumer exists, so that a future proposal binds to a decided key instead of
inventing one.

## D2: The coverage section reuses audit's closed class set (chosen)

**Chosen:** each claim carries exactly one audit class from audit's existing
closed set `{current, stale, unchecked, contradicted, binds_nothing, new}`.

A claim with no evidence row carries `unchecked` — the class the vocabulary
already reserves for knowledge absence. A first draft of this design
introduced a separate coverage vocabulary (`covered` / `uncovered` /
`contradicted` / `stale`); that was **rescinded**: `check.one_vocabulary`
forbids a fourth vocabulary and a new enum value, and
`scripts/check_evidence_axes.py` (from `add-evidence-record-axes`) fails when
the documented axes and the code enums disagree. Reusing the class set means
the drift gate passes with no new row, and a reader learns one vocabulary.

Rejected: a coverage-specific status set (new vocabulary), and a boolean
"covered" flag (a stored boolean the corpus forbids — `no_boolean_columns`:
superseded-ness and every other such fact is derived, never stored).

## D3: The section is derived at report time, from the rows the run produced

The section is not a new producer and not a new file. It is a projection of
two things the run already has: the judged file's declared claims (from the
parsed spec) and the rows the run produced (keyed by claim id).

Deriving it at report time means:

- no second scan of the corpus, no second evaluator, no new artifact;
- the section cannot drift from the rows — it *is* the rows, re-indexed;
- a run that produced zero rows is still a refusal (exit 2); the section is
  never a way to emit a verdict-free report that reads as a pass.

## D4: The section never gates

An unchecked claim is knowledge absence. The exit contract
(`check.exit_contract`) is already explicit that exit 2 means "no judgement
was emitted" and that unchecked evidence never gates. The coverage section
must not become a back door to exit 1: a run whose evidence rows are all
current and which happens to declare twenty unchecked claims still exits 0,
with the count visible.

The section's value is *visibility*, not enforcement: it makes the gap
countable so a reader can act on it. If the gap should gate, that is a
different decision with a different owner.

## D5: Rows are re-keyed by claim id, not re-spelled

Existing row keys are producer-shaped (`verify:prop:<block-id>`,
`verify:model`). The coverage section needs the *claim* dimension, so a row
carries both: its producer key stays as it is (the report's identity and
interoperability discipline depend on it — `check.report_interop`), and the
section indexes rows by the file-qualified claim id they correspond to.

A row that corresponds to no declared claim of the judged file (for example a
model-level row) is listed in the section's own residual bucket rather than
being silently dropped — the section accounts for every row it did not place.

## D6: What this deliberately does not do

- **No sidecar reading.** `check.no_foreign_authority` forbids it and the
  scope decision keeps it in scope B, demand-gated. The join key is decided
  here precisely so that the scope-B proposal, when its warrant arrives,
  has nothing left to invent.
- **No binding kind.** `binding_kind` and `origin` stay reserved
  (`specodelic-yfib`, `specodelic-30s5`); they name a *binding*, and this
  change is about a *claim*'s evidence status. A claim's coverage class is
  not a binding kind: many bindings may attach to one claim, and a claim
  with no binding is simply `unchecked`.
- **No independence claim.** A claim covered by evidence this repo produced
  itself is not independently evidenced; the section says nothing about
  independence and must not be read as saying it.
- **No derivation of claims.** Claims are declared, never inferred from tests
  or prose.

## D7: Rollout

- The section ships with the report; there is no flag and no new artifact.
  An opt-in flag would make the coverage count disappear exactly when it
  matters (a CI run that does not pass the flag).
- `docs/src/graded-evidence.md` gains the section's shape and the mapping of
  each coverage class to its source; the drift gate must agree.
- The report stays byte-stable for identical inputs — the section's ordering
  is the corpus's declaration order, never a hash-map iteration order.
- Self-hosting: the change lands with the corpus unchanged, and the section
  is exercised against this repo's own capabilities (a file with claims that
  have no evidence rows is the natural fixture — most corpus files today
  would be `unchecked` for most of their claims, which is the honest reading
  and the reason this change is worth making).

## Research lineage

- `.wai/resources/research/2026-11-21-expected-behaviour-knowledge-retention-review.md`
  — the Rule-of-5 review that produced the finding this change answers
  (CORR-001: binding knowledge preserved only as prose in the one foreign
  consumer; CORR-005: the join key undefined).
- `.wai/resources/research/2026-11-16-authority-lattice-goals-and-problems.md`
  — G1 (one authority per claim), acceptance condition 6, the four test
  populations, the rejected "derive specs from tests".
- `.wai/resources/research/2026-11-10-executable-contracts-build-spec-grounding.md`
  — D-1 (the join key is undefined and is the second-source risk).
- `openspec/decisions/2026-11-18-scope-a-specification-tool.md` — scope A is
  the product; scope B is demand-gated.
- `openspec/changes/archive/2026-10-06-add-language-neutral-property-binding/`
  — the binding layer's owner is the code-adjacent side; specodelic builds no
  parallel registry.
- `~/para/areas/dev/gh/ak/catitas/openspec/specs/catitas-classify/spec.md`
  (commit `5b2885c`) and this workspace's 643 sidecar files — the measured
  evidence for D1.

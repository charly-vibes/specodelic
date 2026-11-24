# Design — add-check-claim-coverage

## D1: The claim key is the file-qualified claim id (chosen)

**Chosen:** `<file id>.<claim id>` — e.g. `audit.pack_row_collision_follows_the_same_rule`.

Rejected:

- **The bare claim id.** Not unique: the corpus declares 452 constraint ids
  and 28 kebab-style slugs collide across files (`process-lifecycle` appears
  in 21 capability files). A bare id is ambiguous by construction.
- **A slug or filename, matched by similarity.** Refuted by measurement: of
  643 sidecar files in this workspace, only 5 ids match any **constraint**
  slug at all (613 distinct ids; 5 match a constraint slug, 296 match a
  **property** slug, and only 4 contract files name a constraint of their own
  capability), and where they do the match is a coincidence of naming, not a
  declared relation. Inference by similarity is a guess; a guess that binds
  evidence to a claim is worse than no binding. (The measurement is stated in
  full, both readings, in `reviews/2026-11-23-rule-of-5.md` M3–M6 — the
  narrow "5" is true only for the Constraints table, and the surface it
  describes speaks in *properties*, which is why D8 resolves the property to
  its claim through the corpus's declared `derives_from` edge.)
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

**`unchecked` means "no evidence from THIS RUN's producers" — not "no evidence
exists".** The two are different facts, and scope A deliberately collapses them:
`check.no_foreign_authority` forbids reading a consumer's bindings, so a claim
whose evidence lives in the sibling contract surface (301 of the 643 sibling
contract files name a property of their own capability;
202 of the corpus's 408 invariant claims are reachable that way — measured,
see `reviews/2026-11-23-rule-of-5.md` M5/M6) is reported as `unchecked` by a
run that cannot read that evidence. The class is therefore a statement about
the run, not about the world, and the report says so (D10). Sibling precedent:
`dont` keeps the two absences apart — claim status `unverified` (never
verified) versus evidence outcome `unchecked` (evidence present, not
checkable) — but `check.one_vocabulary` forbids specodelic the same split, so
the honest, vocabulary-free form is one class plus an explicit scope
statement.

**Only `invariant` claims are listed.** The corpus also declares 39 `effect`,
2 `advisory` and 3 `extension_point` constraints; they assert emissions and
judgements rather than behaviours that evidence can contradict, and no
producer can place an evidence row on them. Narrowing is stated here so the
delta's "declared invariant claim" is not read as an accident — and so a
future change that wants effect claims says so explicitly.

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

Grounding, both directions: `dont`'s `prime` counts claims by status and
exits 1 only on the blocking (doubted) ones — absence is visible, contradiction
is fatal, which is this design's shape (`dont/src/main.rs:4856-4960`). The
sibling `espectacular` gates the opposite way: a scenario with no declared
test is a finding that blocks execution
(`espectacular/src/check/structural.rs`, finding kind `no-tests-declared`).
That divergence is deliberate here and is a consequence of
`check.one_vocabulary`: specodelic has no "no-evidence" finding kind to
gate with, so the gap is counted instead. The count must therefore be
unmissable — D10 puts it on the verdict line.

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
- **No claim-count verdict.** The section counts; it never decides. The exit
  code remains the one the evidence rows alone determine (D4).
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
  is exercised against this repo's own capabilities. **Measured baseline
  (2026-11-23, review M7–M10):** of the corpus's 408 invariant claims, 76
  (19%) are placeable by this repo's own producers and 202 (50%) are
  reachable through the sibling contract surface the tool may not read; 470
  of the corpus's 572 properties are still `todo_predicate!` stubs, and
  `spk check` currently judges 0 of the 50 capability files (see D10's
  precondition). The first report therefore covers one capability at a
  time, and a first report for a capability whose properties are stubs is a
  refusal, not a coverage list — the honest reading, and the reason the
  precondition is tracked rather than assumed.

## D8: The placement rule — which row places which claim (decided)

**Chosen:** a row places a claim when it names the claim directly, or names a
**property** whose `derives_from` names the claim. Nothing else places
anything.

- A **model row** names the claim directly: `src/check.rs:123` emits
  `verify:model:<claim-id>` for a claim the model gate could not verify.
- A **property row** names a property, not a claim: `src/check.rs:60-70` sets
  the row's `claims` field to the property id (`verify:prop:<property-id>`),
  so the placement hop is the property's own `derives_from` column — the
  corpus's declared edge from a property to the constraint it derives from
  (`linter.coverage` guarantees every property derives from at least one
  constraint).
- A **vacuous-binding row** names its owner directly: `src/audit.rs:629-641`
  keys it `bind:<invariant-id>`, the owner being the claim itself.

Why this matters: the external surface speaks in **properties**. The corpus
titles its scenarios by property id (`openspec/specs/compile/spec.md:264`,
`#### Scenario: precondition_violation_rejected`, with
`- **VERIFIES** [[compile.precondition_violation_rejected]]`), and 301 of the
643 sibling contract files name a property of their own capability — while
only 4 of 643 name a constraint of their own capability, and only 5 of 613
contract ids match any constraint slug at all (measured: review M3–M6). A
rule that stopped at "the row names a property" would leave every claim
unplaced; a rule that guessed from names would bind evidence to the wrong
claim. The one-hop `derives_from` resolution is the corpus's own declared
edge, so it is neither a guess nor a new vocabulary.

Rejected: matching a row to a claim by name, slug or similarity (D1's
measurement refutes it); reading a binding kind (`binding_kind`/`origin` stay
reserved — `specodelic-yfib`, `specodelic-30s5`); and inferring claims from
tests or prose (the anti-goal).

## D9: The fold — several rows, one class (decided)

**Chosen:** a claim's class is the strongest class among the rows that place
it, in the order `contradicted` > `stale` > `current` > `new` >
`binds_nothing` > `unchecked`. A claim with no placing row is `unchecked`; a
row that places no claim belongs to the residual bucket, not to the section's
class set.

**Amended during implementation (2026-11-24, tdd-ro5 green step):** the
original order listed only `contradicted > current > binds_nothing >
unchecked`, noting that `stale` and `new` "are reachable only in a `--before`
run" without ranking them — an unranked class would make the fold
order-dependent (two implementations could disagree by fold order), the same
two-implementations shape CORR-002 named. The fold ranks the full closed set:
a non-current currency label outranks `current` (a regression is stronger
evidence about the claim than an agreement), `new` outranks `binds_nothing`
but sits below `current` (a novelty label is weaker than an actual
agreement), and a contradiction still outranks everything.

- `contradicted` outranks everything: a claim-level contradiction (a model
  row naming the claim) beats a property-level pass.
- `current` is the ordinary case: a placing row that passed.
- `binds_nothing` is a *vacuous* placement — the row exists but its text
  cites no declared id (`src/audit.rs:653`), so nothing was checked. It
  ranks below `current` and above `unchecked`: the claim was looked at and
  found vacuous, which is not the same as never being looked at.
- `unchecked` is the floor: no placing row.

`stale` and `new` are reachable only in a `--before` run (`src/check.rs:270`
diffs the prior report: same class ⇒ current, worsened ⇒ stale, absent ⇒
new); they are listed in the mapping with their producer path so a reader
does not look for a class that cannot occur in a default run.

Rejected: first-row-wins (order is an implementation detail, and the corpus's
declaration order is not evidence strength); a claim carrying a *set* of
classes (the requirement says exactly one); and a separate
"contradicted-beats-stale" vocabulary.

## D10: Where the section lives, and what it says about itself (decided)

**Chosen:** the claim-coverage section is materialized in the **check
envelope**, beside the report — not as a field of `AuditReport`.
`check.report_interop` freezes the report's schema version and field set so
that either verb's report is accepted as the other's prior; a new field on
`AuditReport` would change that field set and would touch `spk audit` too.
The section is check's own addition to the envelope.

**Chosen:** the section carries a scope statement — "classes describe the
evidence THIS RUN's producers produced; evidence outside this run's authority
is not read" — because `unchecked` otherwise reads as "no evidence exists"
where it means "no evidence in this run" (D2). The statement is part of the
section, not a footnote.

**Chosen:** the section appears in every output view (JSON envelope and human
view), with the class counts surfaced **on the verdict line** of the human
view — a trailing summary block is a block a reader misses, and a missed
count is the same as no count.

**Precondition (measured 2026-11-23):** the change cannot be exercised
against this repo's own capabilities until `spk check` can judge a corpus
capability file at all. Today it cannot: `src/check.rs:242` calls the
corpus-less `conform::gate`, while the corpus-aware `conform::gate_in_corpus`
is wired only for `spk conform` (`src/commands/corpus.rs:888-925`). Measured
over the 50 corpus capability files: 19 refuse at `artifact_gate` (the
isolated lint cannot resolve cross-file references) and 31 at `verify_model`
(no model-check report) — **0 of 50 judged**. The gate wiring is tracked
separately (see the change's tasks §0) and this change's self-hosting claim
depends on it; the section is not expected to make the corpus judgeable.

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

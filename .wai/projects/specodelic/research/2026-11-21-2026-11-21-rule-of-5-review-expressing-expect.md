# 2026-11-21 — Rule-of-5 review: expressing expected behaviour without losing the knowledge

**Status: research/review only (advisory). No code, spec, gate, or ticket changed
by this document.** Method: Rule-of-5 universal review (5 editorial passes), with
the TypeSafe (`jev-latest`) verification pass applied to every CRITICAL/HIGH
finding, per the review skill's conditional validation rule
(`TYPESAFE_API_KEY` was set — the banner in §6 therefore does **not** apply).

**Subject under review.** The proposal that specodelic's format can host
"expected behaviour" while keeping it distinguishable from implementation
detail — specifically the mechanism set proposed in the 2026-11-21 session
answer (constraint-kind split `invariant`/`advisory`/`effect`/`extension_point`;
`**rust:**` executable fragments; `extension_point`+`satisfies`; "prose is
never parsed" as the home for non-normative material; `conform`'s verdict
tiers) — read against every prior investigation of how specifications are
defined in this ecosystem (catitas repo + cotorreo corpus + this repo's
recorded research).

**User goal under review.** "I want to have a way were we express expected
behaviour and also not lose that knowledge."

## Grounding sources (all read this session)

| Source | Path | What it contributes |
|---|---|---|
| Authority-lattice synthesis | `.wai/resources/research/2026-11-16-authority-lattice-goals-and-problems.md` | G1–G15 goals; four drifts; four test populations; the six acceptance conditions; the authority lattice; the rejected alternatives |
| Executable-contracts grounding | `.wai/resources/research/2026-11-10-executable-contracts-build-spec-grounding.md` | C-1..C-8 confirmations, G-1..G-5 corrections, D-1..D-7 open questions (join key, evidence vocabulary, adapter shape) |
| catitas corpus (foreign consumer) | `~/para/areas/dev/gh/ak/catitas/openspec/specs/catitas-classify/spec.md` + commit `5b2885c` | The `binding` column removed by `row_shape_closed`; the pins folded into unparsed `## Purpose` prose |
| catitas failure census | `~/para/areas/dev/gh/ak/catitas/catitas-failure-census/preregistration.md` | Pre-registration discipline: defect classes + decision flip table frozen *before* measurement; explicit `UNEXPECTED` bucket; no post-hoc redefinition |
| cotorreo — specodelic expression/verification gaps | `~/cotorreo/synthesized/_conversations/58cdb04abbbb-…` | Multi-provider consensus that verification coverage is thin (9/536 executable cells — single-provider claim, UNVERIFIED) |
| cotorreo — specodelic vs SDD | `~/cotorreo/synthesized/_conversations/26f999e5409e-…` | claude: "model-check returns empty invariants"; grok: heavy spec-first drives spec drift |
| cotorreo — executable contracts & independence | `~/cotorreo/synthesized/_conversations/6c25e587eae1-…`, `2d11ff979a5a-…` | "Contract/test evidence independence" divergence; the gap that no evidence-backed framework for judging suite normativity exists |
| cotorreo — specodelic as human artifact | `~/cotorreo/synthesized/_conversations/8effb8698a08-…` | B grade — strong as machine-checkable artifact, weaker as human thinking artifact (facts findable, reasoning less so) |
| This repo's format sources | `src/spec.rs`, `src/model_check.rs`, `openspec/specs/kinds/spec.md`, `openspec/specs/kernel/spec.md`, `openspec/specs/specodelic/spec.md` | The mechanism set under review |
| In-flight changes | `openspec/changes/add-component-realization`, `add-evidence-record-axes`, `add-evidence-audit` (shipped) | Realization edges (G5/G7); evidence vocabulary (D-1/D-2) |

`wai search "expected behaviour implementation detail contract binding"` →
no prior artifact for this exact question; `wai search "binding"` → the
`kernel.binding` opaque carrier family (`specodelic-bf5`, `binding_tidy`) and
the Nov 2 "realization" concept.

---

## STAGE 1: DRAFT

**Assessment.** The proposal is a faithful map of the *format's* surfaces and
a poor map of the *ecosystem's* answer. It answers "what can a specodelic file
say?" correctly, and then slides into answering "how do we express expected
behaviour and keep the knowledge?" — a question the investigations place
squarely in a different artifact family (contract suites, evidence records,
binding carriers). The subject is well-shaped as a description and
mis-shaped as a prescription.

### Major issues

**[DRAFT-001] CRITICAL — the proposal conflates claim-home, normativity, and
evidence-binding into one answer.**
Location: proposal §4 ("Parsed shape vs prose") + §5 ("Evidence tiering").
Description: The format owns *where a claim lives* and *whether it is
normative* (kind set + Reference Typing). It explicitly refuses the third
question — *what pins this claim*. The proposal answers all three with the
kind split, then adds "prose is never parsed" as if unparsed prose were a
*home*. Unparsed prose is not a home; it is the absence of one. The
investigations' central prohibition (acceptance condition 6: "No additional
authoritative copy of existing decisions, contracts or evidence is
introduced"; G1 "one authority per claim") is what a prose home for
behaviour-bearing knowledge violates.
Recommendation: split the answer explicitly into three artifacts — claims
(spec), bindings (opaque carriers), evidence (graded records) — and state
which one the proposal is answering.

**[DRAFT-002] HIGH — the recipe's promotion path has no trigger, no ladder,
and no decay.**
Location: proposal §4 + "Practical recipe" ("keep implementation detail in
prose … until someone needs to depend on it").
Description: The investigations already answered this and the proposal
re-invents it: "No honest exploration tier. Markdown sketches are implicitly
normative; there is no `exploration_only` default, no sketch → Model →
contract ladder" (authority-lattice, symptom 6), answered by G11
("Sketch → Model → tier 0 → tier 1 … Models demand-gated by a recorded
formalization trigger") and by the Nov 10 resolution that promotion happens
"only by replacement — writing the contract — never by elevation."
Impact: a prose sketch that is never promoted stays in the corpus forever
looking normative — the comprehension-drift generator the corpus already
condemns.
Recommendation: adopt the recorded ladder verbatim; drop the "until someone
needs to depend on it" heuristic, which has no owner and no date.

**[DRAFT-003] MEDIUM — `advisory` is miscast as the implementation-detail
bucket.**
Location: proposal §1 and the recipe ("put non-binding guidance in
`advisory`").
Description: `advisory` is a *soft obligation* — it carries no gating duty,
but it is still a Constraint row, still `traces_to` an intent, still subject
to `coverage` and `single_root_reachable`. It is not a "this is an
implementation detail" marker, and using it as one will make every internal
detail look like a first-class (if soft) promise.
Recommendation: keep `advisory` for guidance-with-a-reason; put
implementation detail outside the corpus (code) or in an `extension_point`
contract when others must depend on it.

---

## STAGE 2: CORRECTNESS

**[CORR-001] CRITICAL — in the one foreign consumer, the "which test pins
this behaviour" knowledge survives only as unparsed prose, so it is preserved
textually and lost mechanically.**
Location: `~/para/areas/dev/gh/ak/catitas/openspec/specs/catitas-classify/spec.md`
`## Purpose` (lines 8–22) and commit `5b2885c` ("binding column folded into
Purpose prose (c1-c4 test pins preserved as node ids)").
Description: catitas carried a `binding` column on its constraint rows
(`pytest::tests/test_classify_queue.py::test_queue_items_selects_downloaded_unclassified`).
The format's `row_shape_closed` rule rejected the column; the pins were moved
into `## Purpose` prose. `src/spec.rs:5` — "Prose is never parsed". Consequence:
no checker can detect a renamed or deleted test, a stale pin, an invariant
with no pin, or a pin that now asserts something else. The knowledge exists in
the file; nothing can lose it, and nothing can notice that it has.
Evidence: commit diff (`id|kind|expr|traces_to|binding` → `id|kind|expr|traces_to`,
pins relocated into Purpose); `src/spec.rs:5`.
Impact: this is precisely the user's goal failing in the only real-world
consumption of the format. "Expected behaviour" is expressed (c1–c4 read as
behaviour); the *knowledge that it is pinned* is one refactor away from
silently rotting.
Recommendation: give the binding a typed, checkable home that is not the
Constraints table — either the sanctioned opaque `kernel.binding` cell
(`openspec/specs/kernel/spec.md`, `kernel_binding_opaque`: "the per-constraint
kernel.binding cell is an opaque claim carrier … extracted and surfaced
verbatim and never interpreted") or a pack-added `## Bindings` section whose
rows point at constraint ids and carry an opaque target. Either way: a
*pointer*, never a copy, so no second authority is introduced.

**[CORR-002] HIGH — fragments are an implementation-pinning channel, and
using them as the carrier of "expected behaviour" makes the spec check
itself.**
Location: proposal §2 (executable fragments).
Description: The investigations separate four test populations, and classify
generated fragments as "projection of a source — never evidence" and
"spec → generated tests → implementation is a tautology loop". A fragment
lives in the *same* artifact as the claim it checks, so it cannot be
independent evidence of that claim; it is the sharpest possible
implementation-detail binding and the weakest possible contract binding.
Evidence: authority-lattice §3.2 population table (line 200) and the grill
quote (line 210); layer table line 113 "Compiled artifacts … Projections —
never evidence".
Recommendation: keep fragments exactly where they are useful — pinning a
claim to *this* implementation's behaviour, and as a migration witness. Do not
count a fragment as evidence that the specified behaviour holds.

**[CORR-003] HIGH — the proposal's own claim is right but its consequence is
not drawn: "expressed" ≠ "checked", and the toolchain says so loudly.**
Location: `src/model_check.rs:18-24` ("Fragment-less input honestly reports
`invariants_checked: []` and never fabricates a counterexample — and a
completed exploration reports `exploration_only`, never `no_counterexample`"),
`:97-99`, `:358-363` (`invariants_checked: vec![]`).
Description: attaching a fragment to a *prose* constraint still yields zero
invariant predicates executed: the fragment binds to a property predicate, and
a prose guard/exec evaluates to nothing under the citation algebra. The
outcome label is `exploration_only` — "the space WAS explored exhaustively
within the bound, but zero invariant predicates were executed".
Impact: a reader of the proposal could reasonably conclude "add a fragment →
behaviour is now verified". It is not; the honest label is exploration. The
corroborating external signal is consistent: claude across the cotorreo SDD
evaluation reports "model-check returns empty invariants", and the
multi-provider expression audit found only 9 of 536 expression cells
executable (single-provider count — UNVERIFIED, but directionally confirmed
by the implementation above).
Recommendation: whenever a spec claims behaviour, state the tier it is
verified at (sketch / model / tier 0 / tier 1) — the format already has the
vocabulary; the proposal just does not use it.

**[CORR-004] MEDIUM — the kind split is orthogonal to the question.**
Location: proposal §1.
Description: `invariant`/`advisory`/`effect`/`extension_point` decide
*obligation strength and observability*, not *behavioural expressiveness*.
None of them makes a constraint checkable against reality; none of them
records what pins it. The proposal's headline mechanism contributes nothing
to "express expected behaviour and not lose the knowledge" beyond saying
"some rows are soft".
Recommendation: keep the kind split for what it is (the format's typing
strength), and stop treating it as the answer to the user's question.

**[CORR-005] MEDIUM — the join key between a claim and its evidence is still
undefined, so the knowledge cannot be re-linked even if it were typed.**
Location: Nov 10 grounding D-1 ("the join key (scenario slug ↔ property id) is
undefined and is exactly the 'second source of behavioral truth' risk"),
D-2 (three evidence vocabularies).
Description: a constraint id (`catitas.classify.c1`), a sidecar scenario slug,
an openspec scenario id, and a pytest node id are four different names for
one fact. Until one canonical form is chosen, any typed binding section will
be a fourth naming scheme.
Evidence: Nov 10 grounding, D-1/D-2; in-flight `add-evidence-record-axes`.
Recommendation: fix the join key first, and make it the *file-qualified
constraint id* (`<file id>.<constraint id>`), which the corpus already
guarantees unique corpus-wide (Revision 18) — sidecars and test node ids
become targets, not keys.

---

## STAGE 3: CLARITY

**[CLAR-001] MEDIUM — "implementation detail" is used for two different
things.** Location: proposal §1 vs §4. §1 means "non-normative guidance";
§4 means "code structure". The investigations keep these apart: G7
("Architecture/C4 realization — components and realization edges; views
generated from one snapshot") versus the non-normative tier.
Recommendation: name them **guidance** (non-binding, corpus-resident) and
**realization** (structure: which component realises which obligation) —
and note that realization is exactly what the in-flight
`add-component-realization` change is scoped to, so the proposal should
point at it rather than invent a home.

**[CLAR-002] MEDIUM — "not lose that knowledge" has no noun.** Location:
throughout. The proposal lists five mechanisms but never says which artifact
*holds* the knowledge. Readers cannot tell whether the answer is "the spec",
"the sidecar", or "the evidence record".
Recommendation: state the three-artifact split in one line — **claims** (spec
rows, normative) → **bindings** (opaque pointers) → **evidence** (graded
records) — and say which one answers the user.

**[CLAR-003] LOW — "expected behaviour" is used for both the claim and its
verification.** Recommendation: use the recorded vocabulary: *claim*,
*contract*, *evidence*, *verdict* — the authority-lattice's own distinction,
and the one the `check` verdict vocabulary already implements.

### Convergence check (after Stage 2)

```
New CRITICAL issues: 1 (CORR-001)
Total new issues: 5
New issues vs Previous Stage: +400% (5 vs 1)
Estimated false positive rate: 0% measured (0 dropped / 1 verified at gate)
Status: CONTINUE
```

---

## STAGE 4: EDGE CASES

**[EDGE-001] HIGH — a never-promoted sketch has no decay rule.**
Scenario: a `## Purpose` paragraph states a behaviour that never becomes a
constraint row and never becomes a contract. It stays in the corpus, is read
by reviewers as normative, and is never checked. This is symptom 6 of the
authority-lattice (sketches implicitly normative) reproduced by the proposal's
own recipe.
Impact: comprehension drift plus false confidence — the worst combination.
Recommendation: either adopt the tier ladder (G11) so sketches are labelled
`exploration_only` until promoted, or record the promotion trigger per file.
The format's own `exploration_only` outcome vocabulary already exists at the
model-check level (`specodelic-len`) and can be borrowed for the prose tier.

**[EDGE-002] MEDIUM — supersede chains drop pins.** Scenario: constraint `c4`
is superseded by `c5`; the prose pin for `c4` disappears with the row, and
`c5` inherits no pin.
Impact: a behaviour change silently loses its evidence binding.
Recommendation: require that a superseding row carries the binding forward
(either as a typed row, or — if prose remains the only option — by requiring
the successor's Purpose to name the predecessor's pin).

**[EDGE-003] MEDIUM — multi-language repositories lose bindings first.**
Scenario: a repo whose behaviour is pinned by pytest (catitas) or by a
cross-language contract suite; the fragment mechanism is Rust-only, and
`add-py-fragment-emission` / `add-ts-fragment-emission` are parked.
Impact: the language with the *least* fragment support is the one where the
binding would live only in prose.
Evidence: Nov 10 grounding D-4 (the adapter is the sidecar's runner column);
the two parked fragment-emission changes.
Recommendation: the binding must be language-agnostic (an opaque target
string), with resolution delegated to the runner that owns it — which is
precisely the shape `kernel.binding` already specifies.

**[EDGE-004] LOW — a binding target can point at a test that asserts the
*opposite*.** Recommendation: when bindings become typed, record the
asserted direction (positive/negative expectation) as part of the opaque
payload, so the runner can detect contradiction rather than only absence.

### Convergence check (after Stage 3)

```
New CRITICAL issues: 0
Total new issues: 3 (1 HIGH, 2 MEDIUM)
New issues vs Previous Stage: -40% (3 vs 8 cumulative)
Estimated false positive rate: 0% measured
Status: CONTINUE
```

---

## STAGE 5: EXCELLENCE

**[EXCL-001] LOW — the proposal never says what it is for.** Add one sentence
at the top: "This describes what a specodelic file can *say*; it does not
describe how behaviour is *pinned* or *verified*."

**[EXCL-002] LOW — the strongest available precedent is unused.** The catitas
failure-census pre-registration is the one mechanism in the whole ecosystem
that demonstrably preserved knowledge across a measurement: defect classes,
severity tags, exclusions and the decision flip table were **frozen before
any run was classified**, surprises were routed to an explicit `UNEXPECTED`
bucket, and "no post-hoc redefinition of classes" was written into the
protocol. That is the transferable answer to "express expected behaviour and
don't lose it": *pre-register the expected classes and the decision rule
before you look at reality*, then let the unexpected land in a named bucket
instead of being rationalised into the expected one.
Recommendation: carry the pre-registration discipline into the contract-suite
direction; it is the discipline the authority lattice's "promotion by
replacement" and the census share.

**[EXCL-003] LOW — the review should not have needed four documents to learn
one rule.** Suggest a one-page "Where does this go?" card in the repo
(`docs/` or `openspec/USAGE.md`): claim → spec row; soft guidance →
`advisory`; observable → `effect`; published contract → `extension_point`;
pointer to evidence → binding carrier; evidence record → `check` report.

### Excellence assessment

- Structure: GOOD — the proposal is well-organised; the problem is scope, not shape.
- Correctness: GOOD — every mechanism it names exists as described (verified file:line); one inference is wrong (CORR-002 → fragments as contract evidence) and one consequence is undrawn (CORR-003 → exploration_only).
- Clarity: GOOD — mostly; the "implementation detail" ambiguity and the missing noun for "knowledge" are the two real gaps.
- Edge Cases: GOOD — the promotion/decay and multi-language cases are the ones that bite.
- Overall: GOOD.

**Production Ready: WITH_NOTES** — the format-mechanism description is
accurate and safe to use as background; the *prescription* it ends with
("keep implementation detail in prose until someone depends on it") should not
be adopted without the corrections below.

---

## 6. Verification log (TypeSafe / jev-latest)

`TYPESAFE_API_KEY` was set, so the skill's conditional verification pass
replaced self-reported validation for every CRITICAL/HIGH finding. Stage A
mechanical pre-gate passed for all five (locations exist; quoted evidence
present verbatim in the artifact).

| Finding | Verdict | Confidence | Disposition |
|---|---|---|---|
| DRAFT-001 | verified | 0.69 | kept, **REVIEW_REQUIRED** (< 0.8 gate) |
| DRAFT-002 | verified | 0.73 | kept, **REVIEW_REQUIRED** (< 0.8 gate) |
| CORR-001 | verified | 0.60 | kept, **REVIEW_REQUIRED** (< 0.8 gate) |
| CORR-002 | verified (2nd pass) | 0.96 | kept, counts toward convergence |
| CORR-003 | verified | 0.15 | kept, **REVIEW_REQUIRED** (< 0.8 gate) |

**Disclosure — one pass was malformed.** CORR-002's first pass returned
`unsupported` because the artifact I assembled omitted the cited lines
(authority-lattice 200 and 210); the excerpt was a construction defect on my
side, not a false positive in the subject. Re-run with the correct excerpt:
`verified` at 0.96. It is therefore **not** counted as a measured false
positive.

**Measured false-positive rate: 0%** (0 dropped / 1 gate-verified; the four
REVIEW_REQUIRED findings are kept, not dropped).

**Escalation.** Four CRITICAL/HIGH findings sit below the 0.8 confidence gate
and are routed to human ratification, per the skill's criteria. They are not
in doubt on the evidence — each was read directly in the repo (file:line
above) — but the review will not self-certify them. **Overall verdict:
ESCALATE_TO_HUMAN for those four; the rest of the review stands.**

---

## 7. Final report

**Work reviewed:** proposal/mechanism set — "express expected behaviour vs
implementation detail in a specodelic repo" (2026-11-21 session answer),
grounded against the catitas repo, the cotorreo corpus, and this repo's
recorded research.
**Convergence:** Stage 5 (substantively converged; formally CONTINUE on the
<10% rule because Stage 5 contributed 2 LOW items over a 3-item baseline).

### Summary

| Severity | Count | IDs |
|---|---|---|
| CRITICAL | 2 | DRAFT-001, CORR-001 |
| HIGH | 4 | DRAFT-002, CORR-002, CORR-003, EDGE-001 |
| MEDIUM | 6 | DRAFT-003, CORR-004, CORR-005, CLAR-001, CLAR-002, EDGE-002 |
| LOW | 5 | CLAR-003, EDGE-003, EDGE-004, EXCL-001..003 |

### What's working well

1. **The format is honest about its own limits** — `src/model_check.rs:21-22`
   reports `invariants_checked: []` and `exploration_only` rather than a fake
   clean verdict. That honesty is the reason the corpus can be reviewed at
   all, and it is stronger than most tools' behaviour.
2. **The kind split is real typing, not convention** — `advisory` cannot
   gate a transition because of the Reference Typing table, not because of a
   README sentence (`kinds.md`, `specodelic.md` Reference Typing).
3. **The `kernel.binding` opaque carrier already exists** —
   `kernel_binding_opaque` gives an external checker a pointer to a claim
   without the format learning the checker's language, and without building a
   registry. The right shape for "express behaviour, don't lose the binding",
   already specified.
4. **The catitas failure census preserves knowledge** by construction —
   pre-registered classes, frozen flip table, explicit `UNEXPECTED` bucket,
   "no post-hoc redefinition". This is the ecosystem's best answer to the
   user's question and it lives outside the format entirely.

### Top 3 findings

1. **[CORR-001] The binding knowledge is prose in the only real consumer.**
   Location: `catitas/openspec/specs/catitas-classify/spec.md` `## Purpose`
   + commit `5b2885c`.
   Impact: the "which test pins this behaviour" link cannot be checked, so it
   rots silently — the user's "don't lose that knowledge" fails today.
   Fix: a typed binding home that is not the Constraints table (opaque
   `kernel.binding` cell, or a pack-added `## Bindings` section of pointers),
   with the join key fixed to the file-qualified constraint id.
2. **[DRAFT-001] The proposal answers three questions with one mechanism.**
   Impact: "prose is never parsed" is offered as a home; it is an absence of
   one, and filling it with behaviour-bearing knowledge risks the second
   authoritative copy the acceptance conditions forbid.
   Fix: split the answer — claims / bindings / evidence — and say which
   artifact holds the knowledge.
3. **[CORR-002] Fragments are not evidence of the contract.**
   Impact: pinning expected behaviour to a `**rust:**` fragment makes the spec
   judge itself (tautology loop) and binds the contract to one implementation.
   Fix: fragments stay implementation-pinning; contracts need independently
   authored suites (Nov 11 decision, step 1 of 5 shipped).

### Recommended actions (ordered)

1. **Answer the user's question with the three-artifact split** — claims in
   spec rows, bindings as opaque pointers, evidence as graded records — and
   put it in one place (`docs/` card; EXCL-003).
2. **Fix the join key first** (CORR-005): file-qualified constraint id
   (`catitas.classify.c1`) as the canonical name; sidecar scenario slugs and
   test node ids become *targets*.
3. **Propose (do not yet build) the typed binding section** as a standard
   pack in the shape of `packs/svc-contracts.md`: a `## Bindings` table
   (`constraint_id | kind | target | origin`), a checker that verifies each
   `invariant` row has ≥1 binding and each target path exists, and no
   interpretation of the target payload (mirroring `kernel_binding_opaque`).
   This is format-legal without touching the base schema.
4. **Adopt the ladder** (G11) with the honest tier labels already in the
   codebase (`exploration_only`), so a prose sketch is never silently
   normative.
5. **Carry the census pre-registration discipline** into the contract-suite
   direction as the "don't lose the knowledge" process rule.
6. **Do not** promote fragments to evidence, and do not build a second
   binding home in the Constraints table.

### Verdict

**NEEDS_REVISION** for the prescription (the recipe's prose-home step must be
replaced by the three-artifact split plus a typed binding home);
**READY** for the mechanism description (accurate as background, with the
CORR-003 caveat that "expressed" never means "verified").

**Rationale.** Every mechanism the proposal names exists as described, but
the one thing the user asked for — expressing expected behaviour *and keeping
the knowledge* — is not a property of the kind split at all: it is a property
of the binding layer, which today lives in unparsed prose in the one foreign
consumer and has no typed home in the format.

---

## 8. Explicitly not decided here

- Whether a `## Bindings` section should be a pack, a base-format section, or
  a `kernel.binding` widening. (D-1/D-4 territory; the `add-component-realization`
  change is the live precedent for "declared vs computed".)
- Whether the specodelic-side binding supersedes or coexists with
  `.espectacular/` sidecars. (D-12 naming collision; cross-tool boundary is
  read-only by standing policy.)
- Any change to `check`'s verdict vocabulary. (`add-evidence-record-axes` owns it.)
- No ticket was filed by this document.


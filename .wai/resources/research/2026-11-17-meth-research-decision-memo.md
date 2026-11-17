# 2026-11-17 — Cross-topic decision memo: evidence vocabulary (D-1), acceptance experiments (D-9), and the goal set after eight research passes

- **Inputs:** the eight topic syntheses of the meth program and their evidence audits
  (`~/cotorreo/synthesized/_conversations/meth-*.md` + `.audit.md`):
  M0 rival-baseline, M1 executable-contracts, M2 authority-models, M3
  design-substrate, M4 composition-architecture, M5 evidence-assurance,
  M6 adoption-review, M7 agent-authored. Evidence base: 41 usable provider
  reports across 8 blind dispatches (2026-11-16 – 2026-11-17); dispatch
  attempts were higher — collection failures are listed per topic in the
  syntheses.
- **Question set:** the two open decisions the Nov 16 design synthesis called
  first — **D-1** (reconcile three evidence vocabularies) and **D-9** (map the
  six acceptance conditions to E1–E4) — plus a status update for **G1–G15**.
- **Design under test** (as stated, not as validated): contract suite normative;
  markdown tier non-normative; Model as adapter #0; projections never evidence;
  existing tests evidence with recorded origin; evidence graded and
  revision-bound; composition typed and computed; formalization demand-gated;
  one binary plus markdown.
- **One-line test applied throughout:** does a proposal add a second
  authoritative copy, or only a projection, an adapter, or evidence?

---

## 1. How to read this memo

Seven of the eight syntheses carry a MEDIUM reliability rating from their audits;
one (M7) is MEDIUM at the upper end. Every synthesis was written from provider
reports produced by the same model lineage against a prompt written in the
design's own vocabulary. Agreement therefore measures a **shared prior**, not
independent replication — the syntheses say this themselves, and the audits
found inflated counts wherever a synthesis forgot it.

Confidence vocabulary below:

| Label | Meaning |
|---|---|
| **High** | ≥4 providers, or a mechanism documented in a primary source; audit `VERIFIED` or benign `PARTIAL` |
| **Medium** | 2–3 providers, or one provider plus a code/document fact |
| **Low / UNKNOWN** | single-provider, provisional quorum, or an explicit research gap |

No citation, clause number or percentage in this memo was independently
verified. Where a provider's own tag said `NORMATIVE`, the tag is preserved and
the clause still marked unverified.

**Quorum status of the inputs.** M0 7/7 usable · M1 4/7 attempted (Z.ai's
report recovered from its 23-page PDF; mistral-work's capture aborted in its
work-plan line; gemini and claude produced no report) · M2 6/6 · M3 5/5 ·
M4 4/6 · M5 4/9 · M6 5/8 · M7 6/8. M4 and M5 were synthesized below quorum and
are cited provisionally. M1 v2 (2026-11-17) covers four providers and is cited
here; its three-provider predecessor is superseded.

---

## 2. Headline findings

1. **Nothing already suffices wholesale, and nothing in the lattice is
   unprecedented as a mechanism.** Six of seven providers state BORROW-MOST;
   the seventh's verdict is inferred from a truncated capture. The residue is
   packaging, an explicit non-normative tier, and the orchestration/provenance
   boundary. Confidence high on direction, low that the residue is novel (M0).
2. **Authority is a checkable artifact plus the mechanised relation binding it
   to that artifact's revision** — never prose, never the artifact alone (5/6;
   M2 K1–K3). No surveyed approach implements per-claim authority end to end,
   and **no study measures the defect or drift delta attributable to authority
   placement** (6/6; M2). The consensus is architectural, not empirical.
3. **Independence is structural, not social, and it is untested for agents.**
   Two artifacts from one author or one model family are a copy, not a second
   authority (6/6; M7 C13; M1 v2 C12 4/4: two suites from one specification are
   one evidence). The design's distinguishing link — an "independently
   authored" suite — is its least-supported assumption. M1 v2 also shows the
   panel is **not unanimous** on rejecting the strong line: 2 keep / 1 reject /
   1 conditional.
4. **Vacuity and spec weakening are the dominant threat, and the current
   grading cannot see them** (6/6; M7 C7). A checker-green contract that no
   longer encodes intent grades "executed" today. DeepSeek states the gap
   directly: the lattice "does not include a mechanism that prevents the
   contract suite itself from being agent-edited into vacuity."
5. **Syntactic success far exceeds semantic fidelity for every measured
   formal language** (6/6; M7 C2). TLA+ from natural language: 26.6% syntactic,
   8.6% semantic in one curated study — a *ceiling*, not a rate, and one
   benchmark (Grok, ChatGPT share the source).
6. **Evidence is a multi-axis annotation, not one enum.** Run status, spec
   defect, currency, knowledge absence and scope exclusion are different kinds
   of statement; merging them is the error the standards corpus documents
   (M5; audit-corrected). `stale` is a relation over a revision pair, not a
   verdict value (M5 C2 3/4; M2 K11).
7. **The migration sequence is partly ratifiable.** Step 2 (one
   foreign-consumer slice) is precedented 5/5; step 1 is conditional (3/5
   negative or conditional); step 3 is the drift risk — only 2/5 separate dual
   *execution* from dual *authority*; step 4 (invert self-hosting) has no
   method-level or small-team precedent (4/5, Mistral dissenting); step 5
   (coverage-gated demotion) is 2 reject / 1 conditional / 2 endorse and
   unresolved (M6).
8. **Composition is a sequence of typed checks; realization is structural
   only.** The discharge record — who discharges which assumption, at which
   revision — is the bottleneck, not the operator (M4 3/4; audit: F-1 was a
   paraphrase).
9. **No measured effect exists anywhere in the corpus**: no cost data, no
   controlled comparison of authority placements, no measured maintenance
   ratio for a contract suite versus a curated test suite (M2 6/6; M5; M6).

---

## 3. D-1 — Reconcile the evidence vocabularies

### 3.1 What the research settles

The three vocabularies are not three competitors. They are **three layers of
one record**, and the current code already says so: `AuditClass::of(verdict)`
rides conform's verdicts "never re-derives them", `binds_nothing` is documented
as "deliberately NOT derivable here: it is evidence-level vacuity from citation
analysis", and `stale`/`new` are diff overlays a fresh build never produces
(`src/audit.rs:60-140`). The research supplies the missing vocabulary discipline
and two concrete defects.

**Recommendation: do not merge the vocabularies into one enum.** Adopt a
five-axis record plus three fields, with each axis closed and tested separately.
This is M5's explicit recommendation (the corpus documents merging as the
failure mode), constrained by M2 (one Relation record) and M4 (evidence as an
annotation, never a peer concept).

| Axis | Closed values | Where it comes from | Delta from today |
|---|---|---|---|
| **Run status** | `permitted`, `forbidden`, `inconclusive`, `not-executed` | conform's verdicts, minus the absence values; `inconclusive` borrowed from ISO/IEC 9646 (audit A3: three-valued, pass/fail/inconclusive) — M5 | **Add `inconclusive`**; `unknown`/`unsupported` are not substitutes (a run that cannot reach a decision is not an absence) |
| **Spec defect** | `underspecified`, `binds_nothing` | conform already carries `underspecified`; `binds_nothing` is the record-level citation face — M5, M2 K2 | Keep both; define `binds_nothing` as a citation test over declared model elements, never as a verdict |
| **Currency** | `current`, `stale` — **computed** | a `(revision, scope, timestamp)` relation compared against a prior report; event-driven, never TTL — M5 C2, M2 K11 | **`stale` becomes computed, never stored or asserted**; no provider reports a time-based staleness signal |
| **Knowledge absence** | `unknown` | conform; the honest "we do not know", distinct from "not run" — M5 | Keep; forbid using it for a run that executed |
| **Scope exclusion** | `unsupported`, `N/A` with justification | conform's `unsupported`; PICS/PIXIT-style declared scope (DeepSeek, single-source) — M5 | Keep; require the justification text for `N/A` |
| **Field: origin** | `authored-from-intent`, `characterizing-observed`, **`authored-from-same-agent`** (the third class M1 v2 C13 4/4 prescribes) | design for the first two; M2 CG+CL, M7 C4; M1 v2 | Promote from a comment to a required field — an `origin`-less record cannot be graded; decide whether the third class is a value or a flag |
| **Field: binding kind** | `generated`, `manifest`, `mapping`, `attested` | M4 (binding kind is a separate axis from grade; fusing mapping-conformance with rule-conformance reproduces a known collapse) | New; prevents the adapter from becoming the authority |
| **Field: bound / assumption set** | model-checking bound, constants, fairness and vacuity assumptions; declared scope | M3: a bounded model result is "supported at bound N, not guaranteed"; CL binds `checked` to constants/depth/fairness/vacuity; CG uses an eight-value completeness scale | **New and load-bearing**: a bare `pass` on a model-checked claim is a category error (M3, 5/5) |

**Code grounding for the verdict change.** `Verdict` today is
`{Permitted, Forbidden, Underspecified, Unknown, Unsupported, Agree}`
(`src/conform.rs:48-70`) — no `inconclusive`, and `Agree` is a structural-only
pass whose doc comment already says it "conforms to the declared transition
relation, but a covering guard's expr is prose". `Agree` is therefore a bounded
pass with no bound attached: the single strongest argument for the bound field.

**The four primitives (Reference / Relation / Observation / Assessment) are the
record's structure, not a verdict layer.** M2's implementable primitive is the
typed relation: `{kind ∈ generated|conforms|refines|describes, checker identity,
authority revision digest, evidence revision digest, origin grade, outcome}`.
The audit class set and conform's verdicts are then derived views over that
record. `describes` (or `unchecked`) must be a typed state graded `unknown`,
never a prose label (M2, CL B2).

### 3.2 What this buys, and what it costs

- **Buys:** one record, three views, no fourth vocabulary; `stale` cannot become
  a second stored truth (G4); `binds_nothing` starts catching the vacuity class
  the corpus says nothing currently catches (M7 C7); `inconclusive` stops a
  non-decision from being reported as a pass.
- **Costs:** a conform verdict addition (schema-level, one value), a computed
  staleness pass, and a citation test. No new tool, no new file kind.
- **Evidence limit:** the axis set is a composition, not a standard (M5 3/4
  "composition, not a re-invention"); every clause behind it is unverified.
  Treat the axis set as `PROPOSED` and the first run over real examples as the
  test — that is M5's T1 clause audit, not an experiment.

### 3.3 Rejected alternatives

| Alternative | Why rejected | Evidence |
|---|---|---|
| One merged enum | Collapses five kinds of statement into one; the standards corpus shows the categories are different and only the *names* overlap | M5 (D-1 recommendation); audit A2–A4 |
| Keep `stale` as a verdict value | Staleness is a configuration-management delta; a stored `stale` is a second copy of the revision relation | M5 C2 3/4; M2 K11 |
| Grade independence organisationally (I0–I3 style) | Grades the org, never the authoring lineage — the axis an agent pipeline actually varies | M5 C5/C8 (audit: C8 3/4, C9 1/4) |
| A five-grade ladder as the primary vocabulary | `checked ≠ verified`; the ladder's top rung is unwarranted by any standard in the corpus | M5 C4, X6; M7 C12 |
| Numeric confidence / Likert grades | Single-provider, and contradicted where checked (C9 is 1/4) | M5 |

---

## 4. D-9 — Acceptance conditions mapped to E1–E4

The six acceptance conditions (Nov 2) and the four filed experiments: E1 law
substitution, E2 kernel coverage, E3 composition, E4 vacuity-guided
elimination. **One condition has no experiment at all (1), two have only a
structural answer (3 and 5), and one is testable but unproven as a property
(6).** That is the gap list this decision asked for.

| # | Acceptance condition | Experiment | What the research says the experiment can and cannot show | Status |
|---|---|---|---|---|
| 1 | All relevant source files accounted for or marked unsupported | **none** (E2 covers kernel coverage only) | M4: reflexion-style views derive intended-vs-actual, but the mapping is authored input, not extracted; "all files accounted for" needs an extraction-completeness measure no provider shows | **GAP** |
| 2 | Cross-tool references resolve to real, versioned artifacts | **E3** (composition) + M0 K1 document audit | M4 K11: no surveyed vocabulary carries a *versioned compatibility obligation* — borrowing cannot satisfy this; Pact-style revision pinning is the nearest mechanism (DS verbatim) | partial — mechanism known, no experiment |
| 3 | The architectural violation is detected | **E3** | M4: reach is structural only; behavioral (shared-state) violations are explicitly undetectable; T2 (reflexion vs ArchUnit, seeded violations) is the test | **partial** — structural violations testable, behavioral ones not |
| 4 | The behavioral verification result retains its actual evidence limits | **E4** (+ E1) | M5 supplies the vocabulary that keeps a verdict honest (`inconclusive`, declared scope, N/A-with-justification); M7 supplies the failure it must catch (vacuity, 6/6); M2 supplies the binding (revision digest) | **covered** if §3's axes land |
| 5 | A generated C4 view and an impact explanation trace to the same snapshot | **E3** (computed interpretation) | M4 K9: a computed interpretation plus revision scope gives "same snapshot"; C4 is a view language, not a calculus; levels 3–4 go stale | **partial** — computable for levels 1–2 only |
| 6 | No additional authoritative copy is introduced | **E1** (law substitution) + M2's controls | M2: enforced only when the copy's acceptance is conditional on a machine check bound to the authority's revision; M6 C5: incremental migration is *not* automatically low-risk — two representations create a permanent synchronisation obligation | **covered as a test**, unmet as a property |

**Mapping summary.** E4 is the only experiment that tests condition 4 directly
and the only one that can test the vacuity class. E3 reaches conditions 2, 3 and
5, but only structurally. E1 and E2 test substitution and coverage — neither has
an acceptance condition of its own, so they must be scored against 4 and 1
respectively. Conditions 1 and 6 need experiments that do not exist:
extraction completeness (1) and copy-prevention under incremental migration (6).

**Recommended D-9 decision:** file the six-condition × four-experiment matrix as
the acceptance table; add two experiments — **E5 extraction-completeness** (file
accounting against a real subproject, borrowing vampiro's completeness
reporting) and **E6 second-copy audit** (apply M2's five mechanical controls to
one slice and count surviving authoritative copies). Without them, "all source
files accounted for" and "no additional authoritative copy" stay unmeasured
claims, and the Nov 2 investigation's own two hardest conditions remain
untested.

---

## 5. G1–G15 after the research

Status legend: **supported** (research backs the goal as stated), **amend**
(the goal survives, its wording or its metric does not), **at risk** (research
contradicts an assumption the goal rests on), **unchanged** (no research input).

| # | Goal | Verdict | What changes |
|---|---|---|---|
| G1 | One authority per claim | **amend** | Authority is claim-scoped *within a scope*, not artifact- or file-scoped (M2 CG verbatim, 5/6). "One file" was already rejected; state "one owner per claim, one artifact per claim-set". |
| G2 | Independence | **at risk** | Independence is structural — different author, different lineage, intent before code (M2; M7 C13 6/6; M1 v2 C12 4/4 "two suites from one spec are one evidence"). An agent authored the same way as the code is a copy, and M1 v2 records the design's own case as a third origin class (C13 4/4). The untested probe is implementation-blind, cross-family authoring (M7 3/6). |
| G3 | Honest evidence limits | **amend** | Add the vacuity status (`binds_nothing` as a first-class record) and `inconclusive`; make `stale` computed (§3). Without them the goal is honest about *what ran*, not about *what it means*. |
| G4 | Drift impossible by construction | **amend** | Mechanism-level truth: drift is made **mechanically visible or refused**, not impossible (M2 G4; K10). "Impossible" overclaims; the enforceable statement is "no second authoritative copy, and every copy's acceptance is revision-bound". |
| G5 | Compositional typed structure | **supported** | Typed relations with checker identity and revision digest are the implementable graph primitive (M2 CL; M4 B3). Composition is a per-edge rule yielding a residual assumption set, not a `composes=true` flag (M4). |
| G6 | Specs where meaning is authoritative; bindings near code | **amend** | Co-location removes the copy by construction for data-shaped, finite-instance claims (DS, GM F3); it is contested for universal/concurrency claims (CL's DO-178C separation; CG dissents that co-location "does not necessarily eliminate" the copy). Split the rule by claim class. |
| G7 | Architecture/C4 realization | **amend** | Realization is a typed, revision-scoped **relation**, structurally checkable only; C4 levels 3–4 go stale and behavioral violations are out of reach (M4 K7/K9). "Derived from source facts" holds for levels 1–2. |
| G8 | Extension without editing | **unchanged** | No research input; pack machinery is shipped. |
| G9 | Simulate before code | **amend** | Model-as-adapter is 2/3 after audit (M6; a Grok attribution was removed). SOFA is the ancestor (M4 DS); a state machine that is not the contract renames the second model rather than eliminating it. Keep the goal, drop the novelty claim. |
| G10 | Pressure without implementation | **supported** | The pressure mechanism has a documented control: withhold the implementation (M7 3/6). The oracle is the global invariant, not the composition operator (M4 T3). |
| G11 | Progressive formalization | **supported** | Demand-gated formalization is the one adoption finding with EMPIRICAL support (M6 C1 4/5 + partial; AWS/seL4). `checked` is not yet a real tier — see G3. |
| G12 | Reviewability | **at risk** | An unbound prose tier drifts 5/5 (M6 C6); it survives only as a generated or ID-linked projection. Domain-expert comprehension is training-gated (M6 C8 4/5; M7 6/6 human review necessary, unreliable in practice). |
| G13 | Agent loop | **amend** | Viable only bounded and externally gated: verifier-in-loop 6/6, bounded repair 3/6 (3–4 iterations), evaluator isolation 4/6, escape-hatch bans single-provider (M7; audit corrections). |
| G14 | Small surface | **unchanged** | One binary conceded by all seven providers, five calling it packaging (M0). No contradiction. |
| G15 | Migration safety: demotion earned by measured coverage | **at risk** | Coverage is not an adequacy measure (M7 C5 5/6; M6 C9 5/5; M5). "Demotion earned by measured coverage" is the one phrase the research contradicts outright. Re-gate on behavioural-guarantee preservation plus mutation score, seeded-bug replay and differential execution (M6; M7; M5 T4). |

**The goal set survives; three wordings do not.** G4's "impossible", G15's
"coverage", and G2's unqualified "independence" are the three claims the
corpus does not support. Fixing those three is the cheapest alignment between
the ratified goals and the evidence.

---

## 6. What this research does not establish

- **No controlled study.** Neither the authority placement, the contract-suite
  superiority, nor the migration sequence has a measured comparison. Every
  confidence in this memo is architectural or documentary.
- **Lineage.** Eight dispatches, one model family line, prompts written in the
  design's vocabulary. Convergence is a prior worth testing, not replication;
  several syntheses had to have counts reduced for exactly this reason.
- **Citations.** No clause, DOI or percentage was verified. Clause-level claims
  (`NORMATIVE`) are unverified everywhere, including where a provider tagged
  them confidently and where an audit found a plausible fabrication.
- **Provisional inputs.** M4 (4 of 6) and M5 (4 of 9) are below quorum; M1 v2
  covers four of seven attempted channels. A memo built on them inherits their
  provisionality.
- **Not measured here.** Corpus counts, ticket statuses and shipped-surface
  claims in this memo are as recorded in the source documents; nothing was
  re-derived for this pass.

---

## 7. Recommended next actions

1. **Answer D-1 with a change proposal, not a design doc.** Scope: add
   `inconclusive` to conform; make `stale` computed from the `(revision, scope)`
   relation; define `binds_nothing` as a citation test; document the five-axis
   record and the three fields. Acceptance: one mapping table in the docs that
   equals the code, and `spk lint`/`audit`/`conform` agreeing on the same
   record (§3).
2. **Answer D-9 by filing the matrix, then the two missing experiments.** E5
   extraction-completeness, E6 second-copy audit (§4). Condition 1 has no
   experiment, conditions 3 and 5 are structural-only; that is the finding, not
   a scheduling detail.
3. **Run M5's T1 clause audit before building anything on the axis names.**
   It is a document read, not an experiment, and it decides whether
   `inconclusive` is a borrow or a renaming.
4. **Fix the three goal wordings (G4, G15, G2) before the next proposal.** Every
   downstream proposal quotes these; leaving "drift impossible" and "demotion
   earned by coverage" in place invites work the research already rejected.
5. **Use M1 v2, and close its two collection gaps.** The revised synthesis
   covers four of seven attempted channels (Z.ai recovered from its PDF;
   mistral-work aborted; gemini and claude absent) and adds the third origin
   class `authored-from-same-agent-as-implementation`. Before independence is
   cited in a design record, decide whether that third class belongs in the
   origin field — it is the design's own case (G2).
6. **Ratify the migration sequence as: step 2 first, step 1 conditional, step 3
   behaviour-only with a sunset, step 4 removed as a milestone, step 5
   redefined.** This is M6's reading of the adoption record and matches the
   design's own "submit to real problems" starting point.
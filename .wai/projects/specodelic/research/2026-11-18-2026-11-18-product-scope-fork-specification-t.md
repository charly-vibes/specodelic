# 2026-11-18 — Product scope fork: specification tool vs authority lattice (review brief)

**Status: decision brief for owner review. Nothing is decided by this
document.** It records the fork, the evidence on both sides, the current state
of the code, and a recommendation. The decision note that follows (if any)
belongs in `openspec/decisions/`.

**Why this exists.** The 2026-11-11 decision ratified the executable-contracts
direction ("authority lattice") and a five-step migration. The research program
(M0–M7, T1) then found no measured benefit for that direction, while the goal
amendments of 2026-11-18 (G2, G4, G15) softened exactly the claims that
required it. The owner's question — *"if what we want is a specification tool,
why touch wild?"* — exposes that two different products were fused into one
roadmap. This brief separates them.

---

## 1. Executive summary

1. **Two distinct products were fused.** (A) a specification tool that makes
   drift *visible and graded* inside one repo; (B) an authority lattice that
   moves normative truth to a *consumer's* contract suite so drift is *refused*.
2. **The objective as stated by the owner is A.** Verbatim: "something that
   checks spec drift against implementation" (Oct 14); "a substrate that helps
   thinking about the system" (Nov 10); "any agent with the tool was able to
   create markdown specs and verify it didn't break anything — keep that
   flexibility" (Nov 11).
3. **B was an escalation, not a requirement.** It entered on 2026-11-10 and was
   ratified on 2026-11-11 before any warrant existed. The research that followed
   (6/6, M2) found no study measuring a defect or drift delta attributable to
   authority placement; M6 rates the self-hosting inversion as having no
   small-team precedent.
4. **A foreign consumer (wild) is only needed to validate B.** Wild is a
   *sample*, not a dependency: any consumer would do, and under A none is
   needed.
5. **Recommendation: take A now; keep B demand-gated** with a written entry
   condition. This keeps the original question answered, drops the unbuilt
   machinery, and leaves the stronger claim available if a real consumer or a
   measured benefit appears.

---

## 2. The objective, reconstructed (owner's words, dated)

| Date | Source | Words |
|---|---|---|
| 2026-10-14 | pi session | "do we have something for quantifying spec drift? or control that?" — "well I meant something that checks spec drift against implementation" |
| 2026-10-07 | codex session | "agents produce code fast and cheap… something deterministic that mines [contracts]" (the economic premise) |
| 2026-11-10 | pi grill | "a substrate that helps thinking about the system and how it should work" — "not every change or feature required a model" |
| 2026-11-11 | pi grill | "any agent with the tool was able to create markdown specs and verify it didn't break anything, how can we keep that flexibility and ease of use without requiring a bunch of things?" |
| 2026-11-16 | omp session | "tests are the new single source of truth… help me understand what happened" → "why we decided not to enforce it?" |
| 2026-11-16 | omp session | "what research should we trigger with catitas to ensure we have clear understanding of existing methodologies, standards, processes like the one we want to build" |

Distilled objective: **make "what the system should do" and "what the system
does" unable to stay silently divergent, while any agent can author and review
the spec with nothing but the binary and markdown.**

Four requirements follow: prevention-or-visibility of drift; a design surface
usable before code; agent ergonomics; honest evidence.

---

## 3. The chain that produced the ratified direction

| Date | Artifact | Outcome |
|---|---|---|
| 2026-10-07 | reverse-derivation evaluation (7 providers + pi session) | No extractor exists; mutation test on pybreaker: 1 of 5 behaviors pinned → derive-from-tests rejected. Note: the evaluation had **zero empirical runs** (qwen failed; "all seven evaluation questions remained unanswered") |
| 2026-11-02 | ecosystem correspondence investigation (ChatGPT + grounding) | Four drifts (spec, architecture, comprehension, evidence), six acceptance conditions, four primitives, "no second authoritative copy"; progress recorded as theory 90%, audit 80%, prototype 25%, **real cross-tool validation 0%** |
| 2026-11-10 | executable-contracts grill + external build spec | The lattice: contract suite normative, markdown non-normative, Model as adapter #0, projections never evidence. Rejected: derived spec layer, all-tests-as-truth, derive-from-tests |
| 2026-11-11 | `openspec/decisions/2026-11-11-executable-contracts-direction.md` | Direction and five-step migration ratified; **deployed behavior unchanged**; adapter registry, verdict mapping, C4 home left open |
| 2026-11-16 | triggering session (`01a1e4d9`) | Owner asks why the tests-as-truth shift stalled → problems/goals synthesis → meth program |
| 2026-11-16/17 | meth program M0–M7 (8 topics, 41 reports) | Borrow-most; independence is the weakest link; vacuity is the dominant unseen threat; **no measured effect anywhere** |
| 2026-11-17 | decision memo | D-1 (five-axis record), D-9 (experiment gaps), G1–G15 status |
| 2026-11-18 | T1 clause audit | 9646 = pass/fail/inconclusive (X.290 3.3.124, 6.5.1.3); absence is a recorded test-case error, not a verdict; no per-item staleness grade in any standard read |
| 2026-11-18 | goal amendments | G2 lineage-scoped; **G4 "impossible" → "visible or refused"**; G15 coverage → behavioural preservation |
| 2026-11-18 | `add-evidence-record-axes` proposal | Five-axis record + drift gate; `inconclusive`/`origin`/`binding_kind`/`bound` reserved (no producer) |

---

## 4. The fork

| | **A. Specification tool** | **B. Authority lattice** |
|---|---|---|
| What is normative | the markdown spec (EARS + Model + claims) | the contract suite |
| Tests are | evidence, graded and dated | truth |
| Answer to drift | detected: `contradicted`, `stale`, `unchecked`, `binds_nothing` | refused by construction (no shared artifact) |
| Repo scope | one repo, any repo | producer + at least one consumer |
| Extra machinery | none | adapter registry, contract runner, per-language bindings, corpus demotion, independence discipline |
| Honest claim | "drift is visible at every revision" | "no second authoritative copy exists" |
| Owner's stated need | matches | exceeds |

### 4.1 What A already has

- `conform` — closed verdict taxonomy over oracle traces (`src/conform.rs:48`).
- `audit` — six lifecycle classes, computed `stale` via `--before` + `scope_sha256`, `binds_nothing` citation face, ladder rung.
- `verify` / `model-check` — bounded claims, honest outcomes (`timed_out`, `exploration_only`).
- `graph` — typed references, closures, views.
- `new` / `lint` — agent-authorable markdown with instant structural verdict.
- Existing in-repo evidence bindings: `.espectacular/` sidecars with **both** `cargo` and `pytest` runners configured.

### 4.2 What A lacks (the real gap)

- **Evidence record incompleteness**: `inconclusive`, `origin`, `binding_kind`, `bound` are specified but not produced (`add-evidence-record-axes`, pending approval).
- **No single judge verb**: `verify`, `conform`, `audit` overlap; `check` does not exist.
- **Architecture/composition**: no component/realization concept; C4 levels 1–2 only (G5/G7).
- **No drift *measurement*** — the tool reports classes, not whether a class change corresponds to a real defect (a seeded-defect replay would answer this).

### 4.3 What B requires beyond A

- A contract runner and adapter registry (decided empirically — the reason wild entered).
- Independent authoring (M1 v2 C12/C13: two suites from one spec are one evidence; an agent authored beside the code is a copy).
- Corpus demotion (step 5), gated on coverage → now on behavioural preservation (G15), which needs a mutation/replay/differential harness that does not exist.
- Version skew handling across consumers (wild pins `spk 0.5.1`; sidecars say `authored_with = 0.12.0`; current is `0.14.0`).

### 4.4 What B was supposed to buy, and what the research found

- Buy: drift impossible by construction. Found: mechanism-level truth only — "drift is made mechanically visible or refused" (M2 K10; G4 amended accordingly).
- Buy: contract suites catch what ordinary tests miss. Found: no controlled comparison anywhere (M2 6/6; M5; M6).
- Buy: independence. Found: "independently authored" is the design's least-supported assumption (M7 C13 6/6; M1 v2 C12 4/4); the design's own case is the third origin class.

---

## 5. Recommendation

**Adopt A as the product scope. Keep B as a demand-gated option with a written
entry condition.**

Proposed entry condition for B (all three, or an owner override):

1. a real external consumer asks to bind its own suite (not a sample we pick), **or**
2. a measured benefit appears on one module (seeded-defect replay or equivalent), **and**
3. the preservation harness exists (so demotion is measurable).

Rationale: A answers the Oct 14 question with code that already ships; B adds
five new subsystems and three unproven claims before any measurement exists.
The research program was created to test claims before paying for them; the
reverse order is what the fork currently proposes.

### 5.1 What A's work program looks like (concrete)

1. **Evidence record** — implement `add-evidence-record-axes` (`re29`): axis
   table + drift gate; `origin: characterizing-observed` for existing evidence
   bindings; keep reserved rows reserved.
2. **One judge verb** — propose `check` as a fusion of `verify` + `conform` +
   `audit` over *existing* evidence (no runner, no registry). Define the verdict
   mapping and exit contract; `inconclusive` enters only if a real producer
   appears (a run that executes and cannot decide).
3. **Staleness on real evidence** — use the in-repo sidecars (authored at
   `0.9.2`) across one revision to demonstrate `stale` on a genuine pair, not a
   fixture.
4. **Composition gap** — scope the component/realization question (G5/G7) as a
   spec-tool feature: what is declared, what is computed, what a view may claim.
5. **Optional warrant probe (A-side)** — seeded-defect replay on one module:
   inject known defects, measure whether the tool's classes change. This is
   the honest version of "does it detect drift", and it needs no consumer repo.

### 5.2 What A explicitly does not include

Contract runner, adapter registry, corpus demotion, self-hosting inversion,
independence claims, foreign-repo coupling, the wild slice. Wild is not
required anywhere under A — it was only ever the nearest *sample* for B.

---

## 6. Consequences if A is approved

| Artifact | Change |
|---|---|
| `openspec/decisions/2026-11-11-executable-contracts-direction.md` | Supersede or amend: direction becomes demand-gated; steps 2–5 become conditional on the B entry condition; step 1 (`add-evidence-audit`) stands as shipped |
| Goals doc §2.2 | Status column: G1/G3 keep direction/partly-shipped (A honors them); G5–G7 re-scoped as spec-tool work; G9/G10/G11 stay but lose the "adapter #0 is normative" framing where it implies B |
| Meth memo §7 | Item 6 (ratify migration sequence) becomes "ratify scope: A now, B gated" |
| Tickets | `re29` unchanged; no wild ticket is filed; the stage-2 wild slice is dropped from the plan, not deferred as a phase |
| Docs | `docs/src/graded-evidence.md` states the tool's claim: drift is *visible and graded*, not impossible. No "no second authoritative copy" claim in user-facing docs |

**What is lost under A, stated plainly:** the strong single-source promise and
any claim of independence for agent-authored evidence. The tool must say so in
the vocabulary and the docs.

---

## 7. Decisions requested from the reviewer

1. **Scope:** A (specification tool, B demand-gated) — recommended — or B (authority lattice, foreign slice first), or A+B with B's entry condition relaxed.
2. **If A:** approve amending/superseding the 2026-11-11 decision note as described in §6.
3. **If A:** approve `add-evidence-record-axes` for implementation (`re29`), and authorize the `check`-as-judge proposal next.
4. **If B is retained in any form:** confirm the entry condition in §5 (three clauses) or state the override.

Defaults if no answer: A, with the 2026-11-11 decision left untouched (no
amendment) — so the record keeps the ratified direction while the code follows
A. That combination is inconsistent and is **not** recommended; it is only the
safe no-action state.

---

## 8. Evidence index

| Claim | Source |
|---|---|
| Owner's objective quotes | omp session `01a1e4d9-2cb1`; pi `01a1c762`, `01a117f7`; codex `01a14695` |
| Four drifts, six acceptance conditions, 0% cross-tool validation | `.wai/resources/research/2026-11-02-ecosystem-correspondence-investigation-grounding.md` |
| Lattice design, rejected alternatives | `.wai/resources/research/2026-11-10-executable-contracts-design-resolutions.md`; goals doc §3 |
| Direction + five-step migration ratified | `openspec/decisions/2026-11-11-executable-contracts-direction.md` |
| Borrow-most; independence weakest; vacuity unseen; no measured effect | `.wai/resources/research/2026-11-17-meth-research-decision-memo.md` §2, §3, §5 |
| 9646 verdict set; no per-item staleness grade | `.wai/resources/research/2026-11-18-t1-clause-audit.md` |
| G2/G4/G15 amendments | `openspec/decisions/2026-11-18-goal-wording-amendments.md` |
| Current evidence vocabulary and gaps | `src/audit.rs:70-140`; `src/conform.rs:48-72`; `docs/src/graded-evidence.md:33`; `openspec/changes/add-evidence-record-axes/` |
| In-repo adapters and authoring skew | `.espectacular/config.toml`; `.espectacular/parse/export-full-ir.toml`; wild `.espectacular/config.toml` (`authored_with 0.12.0`, `spk 0.5.1`) |

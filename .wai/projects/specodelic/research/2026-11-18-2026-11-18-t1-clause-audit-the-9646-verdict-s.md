# 2026-11-18 — T1 clause audit: the 9646 verdict set and per-item staleness grading

**Status: research artifact. No code, spec, or gate changes.** This is M5's
T1 (`meth-evidence-assurance.md:243`) executed as a document read, per its own
falsification rule (`:275`): *"run T1 first. If 9646-1 confirms
pass/fail/inconclusive and no in-scope standard grades staleness as a verdict,
the design proceeds with a four-value verdict enum + (revision, scope) binding
+ origin field, and drops the five-grade ladder."*

**Why this ran before D-1:** every provider report in the meth program was
written without reading a standard (M5 audit §15), and the two load-bearing
attributions disagreed — DeepSeek assigned the verdict set to ISO/IEC 9646-1
with "exact clause UNKNOWN"; OpenRouter assigned `inconclusive` to
`ISO/IEC/IEEE 29119-4 §6.3` and omitted 9646 entirely; Mistral invented a
DO-178C clause. A change proposal that bakes an axis name into `conform`
should not rest on an unread citation.

**Method and scope.** Primary text was read where it is freely available
(ITU-T X.290, IEEE Std 828-2012, Kelly & McDermid 2001); part titles were
verified from ISO/IEC/IEEE catalogue pages; paywalled clause text
(DO-178C, UL 4600, 29119-2/-3) is marked `UNVERIFIED` rather than inferred.
Negative findings are bounded to the instruments actually read — this is not
a claim that no standard anywhere grades staleness.

---

## 1. Question (a): is the 9646 verdict set exactly pass / fail / inconclusive?

**VERIFIED — yes, and absence of a verdict is not a verdict.**

ITU-T Recommendation X.290 (04/95) states its own identity: *"This
Recommendation is also published as ISO/IEC 9646-1:1994"* (X.290 §1, p. 3
note). All citations below are to X.290, i.e. ISO/IEC 9646-1.

| Citation | Text | What it settles |
|---|---|---|
| **3.3.124** | "(test) verdict: A statement of 'pass', 'fail' or 'inconclusive', as specified in an abstract test case, concerning conformance of an IUT with respect to that test case when it is executed." | The definition enumerates exactly three values. |
| **3.3.63** | "pass (verdict): … the observed test outcome gives evidence of conformance to the conformance requirement(s) … and when no invalid test event has been detected." | `pass` is evidence of conformance, not proof. |
| **3.3.34** | "fail (verdict): … non-conformance … or contains at least one invalid test event." | `fail` covers both contradiction and invalidity. |
| **3.3.44** | "inconclusive (verdict): A test verdict given when the observed test outcome is such that neither a pass nor a fail verdict can be given." | Verbatim the design's `inconclusive` axis: executed, undecided. |
| **6.5.1.2** | "A foreseen test outcome … always results in the assignment of a test verdict to the test case." | Verdict assignment is tied to a *foreseen* outcome. |
| **6.5.1.3** | "The test verdict will be pass, fail or inconclusive: a) Pass … b) Fail … c) Inconclusive …" | Normative, exhaustive list. |
| **6.5.1.4** | "An unforeseen test outcome … always results in the recording of a test case error or an abnormal test case termination for the test case." | No verdict is assigned when the outcome was not foreseen. |
| **6.5.1.6** | "An abnormal test case termination is recorded if the execution of the test case is prematurely terminated by the test system for reasons other than test case error." | Absence has its own record type. |
| **6.5.2** | SCTR/PCTR "include a summary of the verdicts assigned" | Reports summarize verdicts; they do not add a fourth value. |

**Conclusion (a).** `inconclusive` is a **borrow**, not a rename: the axis name
should stay `inconclusive`. The complete status space is *three verdicts plus
recorded absence* — absence being a **test case error** (3.3.108,
6.5.1.5) or an **abnormal termination** (6.5.1.6), not a verdict value.
Two corrections follow:

1. The M5/DeepSeek formulation "9646 'none' verdict implied" is **unsupported**:
   X.290 defines no `none` verdict and no `none` term in the verdict
   definitions; absence is recorded, not graded. The design's `not-executed`
   is therefore a **recording class**, and must not be modeled as a fourth
   verdict value.
2. OpenRouter's attribution `ISO/IEC/IEEE 29119-4 §6.3` is **mistaken at the
   part level**: 29119-4 is *Test techniques* (verified from the IEEE SA
   catalogue entry). Any result-status clause belongs to 29119-2 (*Test
   processes*) or 29119-3 (*Test documentation*); the exact clause remains
   `UNKNOWN`. 29119's status triad (pass/fail/inconclusive/not-executed) is
   plausible from the M5 matrix but **unverified**.

## 2. Question (b): does any standard grade staleness per evidence item?

**No verdict-grade precedent found; the concept exists as a change trigger and
as a practice method, not as a per-item grade.**

### 2.1 Configuration management: per-item lifecycle state, never evidence-vs-revision staleness

IEEE Std 828-2012 (read in full text) is the clearest counterexample to any
claim that CM grades staleness:

| Citation | Text | Reading |
|---|---|---|
| **10.2.1** | data elements "shall include: a) The status (such as being changed, stable, archived) of each of the current configuration items" | A **lifecycle status per CI**, declared and maintained — not a judgement about whether the item still supports a current claim. |
| **10.2.3** | each CI "shall be findable … with: … f) The CI's parent and successor CIs (for traceability)" | Supersession is recorded as **linkage** (parent/successor), not as a grade. |
| **14.2.6** | release tracking shall capture "e) What release is superseded by this release" | Supersession is a **release record field**, not an evidence verdict. |
| **Annex I.3** | status accounting "keeps track of each of the current CIs, the current configuration of the product as a whole, past versions, and the status of changes" | Currency is **reported**, not graded per evidence item. |

So CM answers "what state is this item in and what replaced it", not "does this
evidence still bear on this claim at this revision". The design's `stale` is
the second question; 828 supplies its nearest neighbor (parent/successor
linkage + status) and no precedent for the grade.

### 2.2 Safety standards: a change *trigger*, with the method left to practice

Kelly & McDermid, *A Systematic Approach to Safety Case Maintenance* (University
of York, SAFECOMP 2001) documents both halves:

- **Standards mandate re-examination on change, not grading.** Quoting MoD
  Safety Standard 00-55 §4.7.1: *"After the preparation of the operational
  Safety Case, any amendments to the deployment of the system should be
  examined against the assumptions and objectives contained in the Safety
  Case."* And U.K. Railways (Safety Case) Regulations 1994 reg. 6(1): *"A
  Person who has prepared a safety case … shall revise its contents whenever
  it is appropriate…"* (§1).
- **The assessment itself was informal and subjective.** The paper's problem
  statement (§2) is the absence of *"a systematic and methodical approach by
  which to examine the impact of change on safety argument"*, and it lists
  *"insufficient information recorded to support the change process"* as a
  root cause.

This is **documented prior art for the relation** — a change obliges
re-examination of the evidence that supports an argument — and **not** prior
art for the *mechanized grade*. `stale` as a computed `(revision, scope,
timestamp)` relation is therefore the design's **mechanization of a known
obligation**, and should be cited that way. It is not an invention of the
relation, and no standard is found that stores it as a verdict.

### 2.3 The remaining cells are unverified, and one is a known fabrication

| Instrument | Status | Note |
|---|---|---|
| UL 4600 "continual assurance" / evidence currency | **UNVERIFIED** | No primary text read. M5 C2 records it as a process obligation; do not cite a clause until read. |
| DO-178C §6.4.10 = "Outdated" (Mistral) | **UNVERIFIED / likely fabricated** | No primary text located; the M5 audit already judged this citation invented. Do not propagate. |
| ISO/IEC/IEEE 29119-2/-3 result statuses | **Term plausible, clause UNKNOWN** | Part titles verified; 29119-4 attribution corrected above. Even if the triad is normative there, it is a **per-result status**, not a per-item staleness relation. |
| ISO 10007:2017 / ECSS-M-ST-40 configuration status accounting | **Not read** | Definitions match 828's status-accounting shape; treat as CM-side, i.e. same conclusion until read. |

## 3. Outcome against the falsification rule

The rule has two branches; this audit lands on the first:

1. **9646 confirms pass/fail/inconclusive** — done, clause-level
   (3.3.124, 6.5.1.3), so `inconclusive` is a borrow with a citable home.
2. **No in-scope standard grades staleness per item** — no counterexample
   found in the instruments read (X.290, IEEE 828-2012, Kelly & McDermid,
   plus the M5 matrix's negative evidence); the closest artifacts are CM
   lifecycle status/linkage and safety-case change triggers.

**Decision consequences for D-1:**

- Keep the axis name `inconclusive`; add it to `conform`'s verdict vocabulary.
- Model `not-executed` as **recorded absence**, not a fourth verdict (same
  shape as X.290 6.5.1.4–6.5.1.6).
- Keep `stale` **computed**, never stored or asserted.
- Drop the five-grade ladder (as M5 recommended); no clause supports it.
- **Cite prior art instead of claiming invention** for the change-trigger
  relation: MoD 00-55 §4.7.1 (via Kelly & McDermid §1), CM status accounting
  (IEEE 828-2012 §10), and Kelly & McDermid's impact-assessment method. The
  defensible novelty is the *mechanization over revisions*, not the concept.

## 4. Residual uncertainty

- 29119's exact status clause; UL 4600's currency clauses; DO-178C's CM and
  lifecycle-data clauses. Reading any of them could add a *status* precedent,
  but none as currently described would make `stale` a **verdict**.
- This audit does not assess whether `stale` *works*: no measurement of
  recomputation cost, false-stale rate, or maintenance burden exists anywhere
  in the corpus. That is E-side work, not T1.
- Bounded negative claim: no standard **read in this audit** grades staleness
  per evidence item. Broadening the claim requires reading the three
  `UNVERIFIED` instruments above.

## 5. Sources

| Source | How read | Used for |
|---|---|---|
| ITU-T X.290 (04/95) = ISO/IEC 9646-1:1994 | Full text PDF (`tmit.bme.hu` mirror of the ITU edition) | 3.3.34/44/63/108/124, 6.5.1.2–6.5.1.6, 6.5.2, identity note |
| IEEE Std 828-2012 | Full text PDF | §10.1, §10.2.1–10.2.4, §14.2.6, Annex I.3 |
| Kelly & McDermid 2001, *A Systematic Approach to Safety Case Maintenance* | Full text PDF (SCSC) | §1 standards quotes (00-55 §4.7.1; Railways regs 6(1)); §2 problem statement |
| IEEE SA / ISO catalogue pages | Web | 29119-2 = Test processes, -3 = Test documentation, -4 = Test techniques |
| M5 synthesis + audit (`meth-evidence-assurance.md` / `.audit.md`) | Local | C2, C4, X4, the fabrication flags, the T1 definition and rule |

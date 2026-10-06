# Tasks: add-min-expr-kernel

Each phase is one red→green→refactor cycle; the failing test is written
and observed to fail before the implementation that makes it pass.
Tidying commits are separate from feature commits. Phase 1 is the HITL
gate (no code); implementation phases do not open until it lands.

## 1. HITL gate — D7 joint sign-off (no specodelic code)

- [x] 1.1 **GATE**: present the kernel-first sequencing to the sibling
      project and obtain countersignature (design D7). Record the
      decision in both projects' decision records. If the sibling
      prefers pack-first, re-scope this change to slice 1 only and
      update proposal.md before any later phase opens.
      → **DONE 2026-10-06**: kernel-first countersigned; recorded in
      both `.wai/projects/*/designs/matrix/decision.md`; proposal D7 row
      = decided.
- [x] 1.2 **GATE**: confirm the sequencing note with `add-graph-views`
      (gre): kernel atomics ground in acset traversal, not the graph
      projection; record in both proposals (EDGE-004).
      → **DONE 2026-10-06**: note present in this change's proposal
      (§Sequencing) and added to `add-graph-views/proposal.md`.

## 2. Slice 1 — guard citation semantics (approach-02 increment)

- [x] 2.1 **RED**: model-check tests — an artifact whose invariant
      Constraint cites `[[a]] ∧ [[b]]` evaluates the citation under
      the kernel's citation algebra with a three-valued status;
      an undischargable citation reports `unknown`, never pass.
      Run `just test` — new tests fail (citations are inert today).
      ✅ 1649fbf — tests/citation_algebra.rs (9 tests incl. compile.rs
      extraction + rust-guard pinning); RED evidence in
      .wai/projects/min-expr-kernel/research/2026-10-06-red-slice-1-*.md
- [x] 2.2 **GREEN**: implement citation-algebra evaluation in the
      model-check path (citation refs resolve against compiled
      properties; ∧/¬ composition; honest `unknown` propagation).
      Executable guard fragments still fail labeled
      (`fragment_guard_rejected` stands — design D3).
      ✅ 1649fbf (parse/evaluate + guard_citations, Kleene composition)
      + c2bc2fc (RO5U F-1: bracket-in-id rejected, malformed stays prose);
      fragment_guard_rejected pinned by guard_rust_marker_still_rejected_labeled
- [x] 2.3 **GREEN**: persist per-invariant status in run reports
      (backend, bound, status per invariant; `unknown` persisted, not
      coerced).
      ✅ 1649fbf (RunReport.invariant_statuses across native/TLC/exec;
      exec ids discharge from run facts) + c2bc2fc (F-2: re-parse failure
      → honest unknown, never silent drop); serde back-compat kept
- [x] 2.4 **TIDY**: extract citation-resolution helpers; re-run
      `just test` and `just lint-specs`.
      ✅ 444ad1c (strip_guard_cell helper, unused parse binding dropped)
      + beff19d (fmt); just test 23/23 suites ok, just lint-specs 0
      issues, model_check.rs 2282/2300 ratchet; RO5U PASS (0c/0h/1m/3l)

## 3. Tier B — kernel core over the acset substrate

- [ ] 3.1 **RED**: grammar tests — kernel expressions in expr cells
      parse under the closed atomic set; an unknown atomic fails
      labeled naming the closed set (sd1 discipline); a prose expr
      cell compiles byte-identically (pure widening).
- [ ] 3.2 **GREEN**: implement the kernel grammar and row-typing over
      `src/acset` (Schema-as-data supplies 𝒦-typed instances `I(k)`;
      traversal primitive supplies the quantifier domain).
- [ ] 3.3 **GREEN**: implement the v0 atomics with per-atomic grounding
      (design D1 grounding table): `acyclic`/`reachable` on graph
      traversal, `unique`/`resolves` on acset traversal, `==`/
      comparisons/bounded ∀/∃ on model_check bounded evaluation.
      An atomic without a grounding entry cannot ship.
- [ ] 3.4 **GREEN**: three-valued status chain end-to-end —
      `verified`/`counterexample`/`unknown` with propagation rules;
      `unknown` never coerces to pass (dl/1 Kleene absorb).
- [ ] 3.5 **TIDY**: predicate-registry seam for later pack registration
      (dl/1 absorb, gated by D8's decidability gate — registration
      code exists, no pack predicate registers here).

## 4. Backend agreement (D2 interim)

- [ ] 4.1 **RED**: shared-fixture harness — the same fixture corpus
      evaluates through both backends with identical three-valued
      status for every cell.
- [ ] 4.2 **GREEN**: wire the agreement property into `just ci`
      (rust-only until l8l lands: the property asserts rust status
      equals the fixture's expected status; the cross-backend
      assertion activates with the py emitter and must not be
      commented out).
- [ ] 4.3 **TIDY**: fixture corpus under a shared module; document the
      promotion path to a `law_requires_cases`-shaped Property row in
      the l8l change (deferred of record — proposal Deferred).

## 5. Tier C — opaque binding bridge

- [ ] 5.1 **RED**: extraction tests — an invariant-kind Constraint with
      a `kernel.binding` cell extracts the claim as an opaque string;
      specodelic never interprets its contents; a constraint claimed
      by an external checker surfaces in contract-TOML `flags` binding
      (node id / `-k` / `[[tests.shell]]`).
- [ ] 5.2 **GREEN**: implement `kernel.binding` extraction and the
      claim-carrier surface (no registry — design D4).
- [ ] 5.3 **TIDY**: document the claim path next to the existing
      espectacular binding docs (language-tag change precedent).

## 6. Corpus migration (specodelic-corpus edits, gated per-file)

- [ ] 6.1 **RED→GREEN**: migrate `specs/USAGE.md` examples to kernel
      expressions where they assert data-dependent facts; each file
      stays lint-clean before the next migrates (`just lint-specs`
      gate on every commit; design D5).
- [ ] 6.2 **RED→GREEN**: migrate `specs/specodelic.md` invariants
      (equational and bounded-quantified cells) the same way.
- [ ] 6.3 **TIDY**: no new advisory class introduced; if a migration
      would need one, the migration is wrong — record the decision.

## 7. Discipline, contracts, closure

- [ ] 7.1 **CHORE**: FORMAT_REVISION bump — `src/guide.rs:20` + all
      `specodelic.md Revision N` literals (`tests/cli/model_check.rs`
      ×3, `tests/cli/parse_misc.rs:787`, doctor/init skew sites) +
      regenerate compiled artifacts, as one chore commit (turu
      specodelic-6sb).
- [ ] 7.2 **GREEN**: per-scenario contract TOMLs for every scenario
      this change deploys, authored in the same phase (espectacular
      gate: `ah check` green; lefthook ah-check gate runs on every
      commit — EDGE-001).
- [ ] 7.3 **GREEN**: `delta_self_contained` verification per capability
      delta — each delta restates the full requirement set; no orphan
      contracts, no scenarios without contracts (14b7a75 discipline).
- [ ] 7.4 **TIDY**: `openspec validate --strict` green; `just ci` green;
      archive via `just archive-change id=add-min-expr-kernel`.

# 2026-11-02 — Ecosystem correspondence investigation: grounding + open design questions

**Status: research/design only. No implementation is authorized or planned from
this document.** The purpose is (a) to preserve the external investigation's
conclusions, (b) to record how they check out against the current
implementation, and (c) to enumerate the design questions that must be
resolved before any of it becomes proposals or tickets.

## Provenance

- Source: ChatGPT conversation "Specodelic architecture discussion",
  `6ae7f1e9-827c-83e9-887a-351a8e5a2d84`, 2026-11-01/02, nine user turns.
  Raw transcript (708 KB, 14.9k lines):
  `~/para/areas/dev/gh/ak/catitas/catitas/data/raw/_conversations/chatgpt/6ae7f1e9-827c-83e9-887a-351a8e5a2d84/transcript.md`
  (indexed in cotorreo, content_hash `37df0dc467d805e0`, captured 2026-11-02;
  artifacts: specodelic 0.12.1 binary + source zip — re-syncs of the
  conversation must be diffed against that hash before citing this doc).
- Grounding target: this repo @ `a2ca04f` (v0.12.1, 2026-10-24) — as of this
  document's writing, **identical to the snapshot the conversation audited**,
  so there was zero version skew at review time; every discrepancy below is
  the investigation's own misread or omission, not drift.
- Review method: Rule-of-5 (5 stages, converged at stage 4) with TypeSafe
  verification of CRITICAL/HIGH findings. Outcomes over the 7 verified
  findings: 1 verified ≥0.8 gate; 1 dropped as `unsupported` (measured false
  positive — its mechanical evidence was preserved anyway as design question
  D-3 for human re-judgment); 5 verified below the 0.8 gate → REVIEW_REQUIRED,
  escalated to the maintainer (the four G-findings below, plus the
  policy-ownership edge finding that became D-2). MEDIUM/LOW findings
  UNVERIFIED per cost cap. Measured FP rate 1/7 ≈ 14%.
- Notation: **G-n** = grounding corrections from the review; **D-n** = open
  design questions. The dropped id-grammar finding is carried as D-3; the
  policy-ownership finding as D-2.
- Canonical copy: this file. `wai add research --file` also created a
  registration copy under `.wai/projects/domain-specific-extensions/research/`;
  treat that as a snapshot — edit here, then refresh the registered copy
  (first `--file` import in this repo; no prior convention).

## What the investigation concluded (synthesis)

1. Turn 1 (diagnosis): specodelic expresses behavior but not C4-style system
   structure; internal spec coherence ≠ implementation conformance. Missing
   concept named **realization** (typed binding of obligations to components).
   Recommended hybrid: independent specs for external behavior, code-adjacent
   contracts for local obligations, tests as linked evidence. Design
   principle: *specs live where their meaning is authoritative; implementation
   bindings live as close to code as possible.*
2. Turns 2–4 (theory): user-supplied adjointness investigation (spec→code
   underdetermines, code→spec overdetermines; round trips relate to identity,
   are not it); Naur theory-building / comprehension debt; single-source-of-
   truth with projections into views.
3. Turns 5–7 (audit → prototype → test): nine tools audited; concluded
   substantial overlap — no new ADR store / evidence lifecycle / test-impact
   engine. Built a small Python architecture-correspondence sidecar; 4/4
   controlled tests, 1 injected violation caught, **0 real cross-tool tests**.
4. Turn 8 (recap): four drift problems (spec, architecture, comprehension,
   evidence). Progress: theory 90%, audit 80%, semantic integration design
   45%, prototype 25%, **real cross-tool validation 0%**. Milestone: one real
   question-to-evidence chain, acceptance gated by six conditions (preserved
   verbatim below).
5. Turn 9 (consolidation): ~145k lines Rust across 8 tools; **Genesis already
   is the shared infrastructure layer** (don't create another kernel crate);
   minimal interoperability vocabulary of four primitives — **Reference,
   Relation, Observation, Assessment** — as a *computed* graph, never a stored
   second source of truth; extract common components where semantics match;
   do not overgeneralize graphs (specodelic ACSet ≠ testaruda Datalog);
   reconsider the four mandatory layers (typed, optional semantics — Contract
   · Component · StateMachine · Decision · Property), tested against the
   existing kind/pack mechanisms first. Bottom line: *"The simplest successful
   outcome may be no Specodelic successor at all: just a smaller Specodelic …"*

### The six acceptance conditions (verbatim, turn 8 §7)

1. All relevant source files are accounted for or explicitly marked unsupported.
2. Cross-tool references resolve to real, versioned artifacts.
3. The architectural violation is detected.
4. The behavioral verification result retains its actual evidence limits.
5. A generated C4 view and an impact explanation trace back to the same snapshot.
6. No additional authoritative copy of existing decisions, contracts or evidence is introduced.

## Grounding review — how the claims check out

### Confirmed by the implementation

- **Architecture gap is real.** No realization/C4/component concept anywhere
  in the corpus — 47 `spec.md` files under `openspec/specs/` (the directory's
  49 entries include two non-spec files); only an incidental mention in
  `openspec/specs/acset-writer/spec.md`, and graph-views' wiring views collapse
  rows to intents, not components. Beads checked under
  architecture/realization/component/C4/docstring/extraction/correspondence
  terms: no open ticket.
- **Genesis as shared infra is real.** `genesis-vibes = "0.13"`
  (`Cargo.toml:27`); 39 `genesis::` call sites across 10 src files; the repo's
  own AGENTS.md convention ("Genesis baseline") matches the rule the
  investigation quotes.
- **Four mandatory layers are real.** `src/lint/rules.rs:171–173` enforces
  `model_present` (States list + Transitions table; empty-but-present beats
  absent). The investigation's caveat — test typed-optional semantics against
  the existing kind/pack mechanisms first — is correct: packs exist
  (svc-contracts, skill, numeric-predicates, empirical-registry,
  bioimage-data, data-lineage, quant-finance) with `pack_shape` narrowing
  rules already in the managed lint block.
- **Espectacular derivation split is real.** `espectacular/src/sync.rs`:
  five sync-owned derived fields with a drift fingerprint
  (`derived_fingerprint -> [String; 5]`, line 75), humans own `tests`/`status`;
  `ah check` reports contract-stale findings. The investigation's "another set
  of artifacts that can become stale" is accurate but incomplete — see G-3.

### Corrected / contradicted (the material grounding errors)

- **G-1 (CRITICAL): verdict taxonomy misstated.** The conversation's
  adjointness investigation calls conform's verdicts "three-valued". Ground
  truth: the corpus closes the set at **five** — `{permitted, forbidden,
  underspecified, unknown, unsupported}`
  (`openspec/specs/conform/spec.md:28`, `verdict_taxonomy_distinct`), plus the
  `--structural-only` agree mode (`src/conform.rs:48–72`; `Agree` added in
  `14f682b`). Consequence: the investigation's own recommendation D
  ("distinguish satisfied/violated/underspecified/unknown/unsupported instead
  of PASS") **is already the shipped taxonomy**. The missing piece is only the
  cross-tool envelope, not verdict semantics. [TypeSafe: verified, 0.36 <
  gate → REVIEW_REQUIRED]
- **G-2 (CRITICAL): vampiro omitted from the audit.** `charly/vampiro` — beta,
  prove/check shipped, seven seam axes (call, module, effect, law, retry,
  resource, trust) — is a shipped local implementation of exactly the
  investigation's highest-priority workstream ("trustworthy extraction").
  Its known Rust gap ("data-flow edges are partial → advisory", pinned in this
  repo's AGENTS.md sibling-tool constraints) **is** the extraction-
  completeness problem the conversation's prototype hit. The proposed first
  experiment ("extract trustworthy source facts from Pretender") duplicates
  vampiro's charter. [TypeSafe: verified, 0.39 → REVIEW_REQUIRED]
- **G-3 (HIGH): ignores the repo's own Oct 24 simplification pass.** Commit
  `a2ca04f` (seven days before the conversation) already ran a Rule-of-5
  consolidation: three catitas wave tickets (fqte/va3q/4m3y, plus the
  model-binding half of h5ks) superseded as scenarios of a new
  **`add-evidence-audit`** proposal with closed graded-evidence class set
  `{current, stale, unchecked, contradicted, binds_nothing, new}`; three
  proposals (preservation, doctor-advice, changed-scoping) parked
  demand-gated; falsifiable experiments **E1–E4** filed (E1 law substitution,
  E2 kernel coverage, E3 composition, E4 vacuity-guided elimination — names
  from the commit message). The conversation's evidence-envelope
  recommendation reinvents this vocabulary without reconciling class sets.
  [TypeSafe: verified, 0.45 → REVIEW_REQUIRED]
- **G-4 (HIGH, with a dropped twin): espectacular staleness framing.** The
  claim "another set of artifacts that can become stale" omits that staleness
  is already fingerprinted and detected (G-3's sync.rs evidence; the
  `contract-stale` finding kind in `ah check`). Derive-at-check-time is
  therefore a **tradeoff** (loses the persisted stale signal that
  byte-identical-rerun discipline relies on), not the pure win the turn-9 text
  implies. [TypeSafe: verified, 0.51 → REVIEW_REQUIRED] Related edge finding
  EDGE-001 (Reference syntax vs the corpus's dual id grammar — file stem `-`⇔
  `.`, directory-derived ids, `_` literal — and spec-integration's "one spec
  vocabulary" rule) was TypeSafe-dropped as `unsupported`; mechanically the
  evidence still looks real, so it is preserved as a design question (D-3)
  for human re-judgment rather than arithmetic.

## Open design questions (resolve before any implementation)

Each question lists candidate answers and what evidence would decide it.
None has a proposal yet; none should get one until the question is answered.

- **D-1 — Reconcile the three evidence vocabularies.** The investigation's
  four primitives (Reference/Relation/Observation/Assessment), the repo's
  `add-evidence-audit` class set `{current, stale, unchecked, contradicted,
  binds_nothing, new}`, and conform's five-verdict taxonomy are 80% the same
  idea at three scopes. Candidate: one mapping note stating which tool owns
  which class and which classes are projections of others (e.g. is
  `binds_nothing` the cross-tool face of conform `underspecified`?). Decider:
  side-by-side table over real examples from this repo + one dont claim + one
  espectacular contract. This is the prerequisite for every other question.
- **D-2 — Who owns Assessment policy?** The investigation quotes genesis's
  rule ("keep domain-specific engines in their respective tools") then floats
  the interoperability contract as "a modest addition to Genesis" — those
  collide. Architecture-policy evaluation (forbidden dependency detected) is
  domain logic. Candidates: (a) a thin `ah`-style companion (the ecosystem
  already has this pattern); (b) vampiro itself (policy = declared seams vs
  observed edges); (c) specodelic pack — but sibling-tool read-only walls
  (espectacular over openspec/, vampiro advisory over src/) constrain what a
  pack may gate. Decider: write the policy grammar for ONE rule ("risk engine
  must not depend on storage") in each candidate and compare.
- **D-3 — Reference grammar for cross-tool ids.** Any interoperability
  Reference must round-trip the dual id rules (`-`⇔`.`, directory-derived
  spec.md ids, `_` literal), openspec path identity, and spec-integration's
  one-vocabulary discipline. Candidate: reuse `spk parse --json` id emission
  as the canonical form plus a normative scheme→grammar resolver table.
  Decider: can every id in this corpus round-trip through one resolver?
  (Mechanically evidenced; TypeSafe-dropped; needs human re-judgment.)
- **D-4 — Where do architecture declarations live?** Candidates: (a) beside
  wai design artifacts (investigation's preference); (b) a specodelic domain
  pack (`architecture-pack`, using existing pack mechanics — no new top-level
  kind); (c) openspec-side with a layer.md sidecar (spec-integration
  machinery exists for exactly this). (a) risks a third vocabulary;
  (b) respects packs but stretches "behavior" semantics; (c) reuses the
  proven dual-format recipe. Decider: draft the same component declaration in
  all three homes and check lint/openspec-validate/archive-companion behavior.
- **D-5 — Espectacular: derive-at-check-time vs pinned digest.** Full
  derivation deletes five persisted fields and the stale finding with them.
  Candidate middle path: keep ONE pinned digest per contract (hash of derived
  fields), drop the five fields — most of the simplification, none of the
  signal loss. Decider: measure `ah check` recompute cost on this corpus and
  diff pinned-digest vs full-persistence staleness behavior. Constraint:
  espectacular stays read-only over `openspec/` (AGENTS.md, ADOPTED-WITH-
  SEQUENCING); contract TOMLs live in `.espectacular/` — inference: the
  pinned read-only constraint names only `openspec/`, so contract TOMLs sit
  outside it (boundary source: the 2026-10-01 adoption decision).
- **D-6 — Vampiro re-scope of the source-facts experiment.** Candidates:
  (a) source-facts becomes a vampiro Rust-frontend-parity milestone (its
  README/design docs own the seam model); (b) an explicit written waiver +
  parallel extractor in Pretender (requires the waiver mechanism AGENTS.md
  reserves for deny-level changes); (c) defer — accept advisory-grade
  observations for the first slice. Decider: scope one real extraction task
  ("list observed depends_on edges for one component") in vampiro and measure
  completeness reporting (analyzed/skipped/unresolved/unsupported) against
  the six acceptance conditions' condition 1.
- **D-7 — The "smaller Specodelic" question.** Typed-optional semantics
  (Contract · Component · StateMachine · Decision · Property) vs the
  four-mandatory-layer shape is the largest possible format change
  (Revision 20-class). The investigation itself says to test against
  kinds/packs first. Candidate: express Component as a pack-carried kind +
  `realizes` as a typed reference column (the ACSet schema already models
  typed references as data — `src/acset/schema.rs`) before touching the
  mandatory-layer rule. Decider: prototype one pack (not a format revision)
  and run `spk lint` over a hybrid corpus; the `linter.model_present`/
  `pack_shape` narrowing rules will either permit or name the blocker.
- **D-8 — First vertical slice target.** All charly-family tools are Rust,
  but vampiro's Rust edges are advisory until frontend parity — an all-Rust
  slice makes every Observation advisory-grade (condition 4 becomes
  self-undermining). Candidates: (a) a Python/TS subproject (stronger adapter
  coverage); (b) a Rust slice explicitly scoped advisory; (c) the synthetic
  prototype corpus — already rejected by the conversation itself, since "0
  real cross-tool tests" passing is precisely the gap this slice must close).
  Decider: pick the repo after D-6 resolves.
- **D-9 — Unify the two acceptance gates.** The conversation's six conditions
  and the repo's experiments E1–E4 are parallel acceptance programs for the
  same consolidation. Candidate: file the six conditions as one bd issue and
  map each E-experiment to the conditions it partially satisfies. Decider:
  the mapping table itself; if two conditions map to nothing, that is the
  real gap list.

## Relationship to in-flight work (do not collide)

- `add-evidence-audit` (in changes/, staged as 4 slices) owns the graded-
  evidence verb for specodelic-internal evidence. D-1 must layer the
  investigation's cross-tool envelope ON it, not beside it.
- Parked demand-gated proposals (preservation, doctor-advice,
  changed-scoping) — the investigation's proposals must respect the same
  demand-gating discipline rather than reopening them.
- `specodelic-3a8` (decouple dual-format metadata from core parse — adapter
  layer) is a prerequisite-shaped ticket: the investigation's "semantic model
  independent of Markdown" points the same direction.
- Espectacular sequencing: per the 2026-10-01 adoption decision, wiring is
  adopted, full per-scenario contract authoring is sequenced follow-up —
  D-5 must not jump that queue.
- Vampiro blocker: AGENTS.md records "no integration ticket exists" for
  vampiro — D-6 is the natural moment to file it.

## Explicit non-goals for this document

- No format revision, no pack, no new command, no companion tool is proposed.
- The Python correspondence sidecar from the conversation is NOT adopted;
  its tests may inform D-6/D-8 but the implementation was rated
  replace-me-by-design (extraction blind spots are known).
- Merging repositories and a universal semantic kernel are both recorded as
  rejected strategies (investigation turn 9 §10; this review concurs).

## Suggested next steps (design work, not implementation)

1. D-1 reconciliation table (smallest, unblocks the rest) — one working
   session, output appended here or as a successor research doc.
2. D-9 mapping note + file the six acceptance conditions in bd.
3. D-4 three-homes draft (pure markdown experiments, no tooling).
4. D-6 vampiro scope spike → file the vampiro integration ticket it was
   always going to need.

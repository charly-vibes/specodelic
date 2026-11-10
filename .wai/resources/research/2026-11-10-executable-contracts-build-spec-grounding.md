# 2026-11-10 — Executable contracts build spec: grounding + open design questions

**Status: research/design only. No implementation is authorized or planned from
this document.** Purpose: (a) preserve the newest external proposal's design,
(b) record how each claim checks out against the current implementation, (c)
enumerate the design questions that must be resolved before any of it becomes
proposals or tickets. Companion to
`2026-11-02-ecosystem-correspondence-investigation-grounding.md` (its D-5 and
D-7 questions intersect directly; see §Relationships).

## Provenance

- Source: ChatGPT conversation **"Executable Specification Strategy"**,
  `6aedeb14-e2d4-83e9-9428-ff6fff6fcf37`, 2026-11-10, five user turns.
  Raw transcript (6,271 lines / 312,517 bytes, sha256 `508eb0bc51efd41d…`):
  `~/cotorreo/raw/_conversations/chatgpt/6aedeb14-e2d4-83e9-9428-ff6fff6fcf37/transcript.md`.
- Inputs the conversation worked from: the specodelic source/binary snapshot
  **@ `8b54086`** (zip dated 2026-11-03), the **wild-main** zip (its
  `.espectacular/` sidecars — "the contract parts we already have"), and an
  external working-notes document, **"Adjointness between specifications and
  implementations"** (2026-11-01), pasted inline at turn 3.
- Same-day companion conversation: "Feature Usability Evaluation"
  (`6aed377e`) — produced a *bajan* "Knowledge Composition Contract v0.1"
  (40 EARS requirements / 12 properties / 16 BDD scenarios), explicitly
  marked not-yet-specodelic-linted. Context only; not grounded here.
- Grounding target: this repo @ `62c5e02` (2026-11-05, pretender gate-debt
  fix). The audited snapshot `8b54086` is **two commits behind**; the two
  intervening commits are a docs-only brief and the pretender 0.11 gate-debt
  fix — no change to any command, format, or artifact surface this document
  grounds, so every file:line citation below is current.
- Review method: claim-by-claim grounding against HEAD with file:line
  evidence, single pass (not a Rule-of-5 cycle — cost-capped like the Nov 2
  doc's MEDIUM/LOW tier). Notation: **G-n** = grounding corrections;
  **D-n** = open design questions.
- Canonical copy: this file. Register with
  `wai add research --file` (same convention as the Nov 2 doc).

## What the investigation concluded (synthesis)

The conversation evolved across five turns:

1. **T1 (diagnosis)** — the spec/test/implementation mismatch; proposal:
   tests become the executable specification, openspec specs become
   reading drafts ("docstrings of each test").
2. **T2 (strategy)** — specodelic becomes *the authoritative source of
   executable verified truth*; openspec keeps change-proposal management;
   specs are *synthesized from* accepted contracts; the test system needs DI
   to run against different implementations.
3. **T3 (theoretical correction)** — grounding against the adjointness
   document: `compile` is the spec→artifact direction; `conform` is the
   external-evidence→spec direction but sits outside the verification
   lifecycle; **tests compiled from specs are one projection of a single
   source and are not independent evidence** — independence comes only from
   the oracle. Therefore: elevate *independently authored behavioral
   contracts*, not the generated proptest artifacts.
4. **T4 (inventory)** — "for the contract parts we already have": inspection
   of wild's `.espectacular/` sidecars as the existing contract→test binding
   mechanism.
5. **T5 (build spec)** — "Executable Contracts and Implementation
   Conformance — Build Specification": six phases, each with an exit
   condition —
   - **P1 Contract suite**: observable-behavior assertions independent of
     implementation internals.
   - **P2 Adapter + runner**: one suite, one implementation.
   - **P3 Independent implementation**: same suite, second adapter.
   - **P4 Evidence and traceability**: stable contract-to-test associations
     ("extend `.espectacular` or its existing successor"), revision-scoped
     verification evidence, coverage + stale-evidence reporting.
   - **P5 OpenSpec projections**: normative spec sections generated from
     accepted contracts; semantic change summaries; reviewable proposals;
     human-authored rationale preserved.
   - **P6 Broader adoption**: second wild subsystem; process/cross-language
     adapters only on demonstrated need.
   Design decision summary (verbatim intent): *"A contract suite describes
   observable behavior. An adapter makes an implementation available to that
   suite. The harness evaluates the contract and records evidence. OpenSpec
   manages changes to the accepted contract corpus."*
   Eight open questions are carried (§Open questions, mapped to D-n below).
   Final caveat: no new TOML fields or APIs prescribed yet; next artifact
   named by the conversation itself is a **requirement-to-repository
   traceability matrix** for `wild-core`.

## Grounding review — confirmed by the implementation

| # | Conversation claim | Verdict | Evidence |
|---|---|---|---|
| C-1 | `compile` provides the spec→executable-artifact direction (TOML / TLA+ / proptest) | **Confirmed** | `src/compile.rs`; committed self-hosted artifacts `specodelic/*.toml`, `*.tla`, `*_props.rs`, `*.check.json` |
| C-2 | `verify` checks the compiled specification and its evidence, hash-bound | **Confirmed** | `src/verify.rs:10` (stale `.check.json` refusal), `:350-352` (`artifact_sha256` must match current `.tla`; foreign evidence is a rerun, never an acceptance), `:416-447` (`scope_sha256` scope fingerprint, duplicate/missing/extra claim-record rejection) |
| C-3 | `conform` classifies external oracle traces and is deliberately outside the verification lifecycle | **Confirmed** | `src/conform.rs:1-20` — closed five-valued verdict taxonomy (D1), closed-world evidence gate (D2), scope digest, *"Read-only: it evaluates, it never generates"* |
| C-4 | The test system needs DI to run against different implementations | **Already exists (Rust)** | `src/verify.rs:868` `pub trait PropertiesRunner { run_blocks(&self, props_path, blocks) -> Result<Vec<BlockResult>, RunnerError> }`; production `CargoRunner` (scratch crate, shared `CARGO_TARGET_DIR`, wall-clock timeout, `specodelic-xx1`); test `FakeRunner` at `:1719` |
| C-5 | Stable contract-to-test associations need a home ("extend `.espectacular` or its existing successor") | **Home already exists** | `.espectacular/<capability>/<scenario>.toml`: `id`, `description`, `archetype`, `status`, `superseded_by`, `authored_with`, `[[tests.cargo]] flags` (this repo); wild uses `[tests] pytest = [{flags = …, timeout_seconds = …}]` (e.g. `wild-core/prototype-accretion-pipeline-reports-round-1-verdicts.toml`) |
| C-6 | Semantic change detection can reuse the graph machinery | **Confirmed for closures** | `src/merge.rs:126-132` — blast radius **is** `acset::query::blast_radius` (mixed forward/backward closure); `src/graph.rs:1130-1132` — fan-in/out via `acset::query`; byte-stable projections (`edges_tsv`, dot/mermaid/wiring views) |
| C-7 | Stable identifiers + traceability without schema changes (their open Q1) | **Mostly yes** | Property rows carry unique ids (per-file `linter.unique_id`; file-qualified ids corpus-wide, Revision 18); `derives_from`/`traces_to` build own-file reachability; cross-file typed edges (`satisfies`, `observes`, guard citations) are outbound leaves by `single_root_reachable` — referenceable, never reachability paths. A one-file contract suite needs zero schema change; a cross-file suite must respect the leaf convention |
| C-8 | Independent evidence precedent | **Exists for the checkers** | `tests/conformance_matrix.rs` — data-driven case table, expectations *derived from the checker specs, never from current tool behavior* (`specodelic-vv8`) — the exact discipline P1 wants for application contracts |

## Corrected / contradicted (the material grounding errors)

- **G-1 (MEDIUM): `conform` is not user-reachable.** The T3 table lists
  `conform` among "existing mechanisms", but its own module header states
  *"no CLI wiring yet — the `spk conform` command lands in phase 4; the gate
  and emit layers here are the testable API surface"*. Any P2/P4 harness
  design that assumes `spk conform` exists today is wrong; it is a library
  surface only (`specodelic-f4sx` tracks its single-file gate friction).
- **G-2 (HIGH): "the existing tests are often compiled from specifications"
  conflates two test populations.** True for the **self-hosted corpus**
  (`specodelic/*_props.rs` etc. are compile artifacts re-run by verify);
  false for the tool's acceptance layer — `tests/` (conformance matrix, CLI
  suites) is hand-authored with spec-derived expectations. The build spec's
  own "two kinds of executable knowledge" (behavioral contracts vs generated
  projections) is therefore **already the de facto split** in this repo:
  generated fragments check the spec against itself; hand-authored,
  sidecar-bound tests are the independent evidence. P1 does not need to
  invent the distinction — it needs to *name* it and move it from discipline
  to artifact.
- **G-3 (HIGH): P4's premise understates what sidecars already do.** "Extend
  `.espectacular` … to express stable contract-to-test associations" — the
  associations exist *now* (C-5), in two runner flavors (cargo, pytest).
  What is actually missing is only: revision-scoped **evidence persistence**,
  coverage aggregation, and stale-evidence reporting for those bindings.
  Additionally, `.espectacular/` is espectacular's managed dir and
  espectacular is **read-only over `openspec/`** per this repo's AGENTS.md —
  a specodelic-side generator writing sidecars touches a sibling tool's
  contract and needs an explicit agreement, not a quiet extension.
- **G-4 (MEDIUM): "reuse existing graph and diff capabilities" (their open
  Q7) — there is no diff capability.** What exists: closure primitives
  (C-6), graph views, merge's blast radius. A semantic diff would be *built
  on* `acset::query` closures, not reused. The wording risks scoping P5
  against a capability that doesn't exist.
- **G-5 (LOW): snapshot skew.** The conversation audited `8b54086`; HEAD is
  `62c5e02` (two commits, docs + gate-debt only). No claim below is affected,
  recorded so future re-syncs diff against the right base.

## Open design questions (resolve before any implementation)

- **D-1 — Who owns the contract-suite ↔ property ↔ test join?** The three
  candidate homes each already hold half the truth: specodelic property rows
  (behavioral claims, machine-checkable), `.espectacular` sidecars (test
  bindings, `[[tests.cargo]]`/`[tests] pytest`), and the scenario blocks
  under openspec requirement headers (espectacular's conformance target).
  The build spec defers this ("declared in the existing specification
  format, in sidecar metadata, or derived from both" — their Q3). Grounding
  leans sidecar-side (bindings live there today; spec rows stay the
  behavioral source), but the join key (scenario slug ↔ property id) is
  undefined and is exactly the "second source of behavioral truth" risk
  their Q8 names.
- **D-2 — Evidence vocabulary: fourth entry forbidden.** Verify carries
  `artifact_sha256`/`scope_sha256`/claim records; conform carries schema
  version/`evidence_scope`/scope digest; the in-flight `add-evidence-audit`
  change defines `{current, stale, unchecked, contradicted, binds_nothing}`.
  P4 must consume one of these (the audit verdict set is the natural
  candidate), not mint a parallel one. This extends the Nov 2 doc's D-1
  (reconcile the three evidence vocabularies) — same question, one more
  entrant.
- **D-3 — Independence authorship.** `tests/conformance_matrix.rs` shows the
  discipline (expectations from specs, never behavior) for the checker
  surface; the oracle argument (T3 §7.6) shows why compiled fragments don't
  count. For application contracts (wild-core), D-3 asks: who authors the
  suite, against what oracle, and where is the falsifying statement that the
  suite is *not* a projection of the implementation? Without an answer, P1
  quietly re-creates the single-source problem the whole proposal exists to
  close.
- **D-4 — Cross-language adapter shape.** `PropertiesRunner` is
  Rust-artifact-shaped (`*_props.rs` + cargo). Wild's contracts are pytest
  bindings that live entirely in sidecars — outside specodelic's runner
  entirely. Their Q2 ("smallest adapter interface that supports the
  wild-core properties") therefore has a grounded answer candidate: for
  non-Rust implementations the adapter is *the sidecar's runner column*,
  and specodelic's role shrinks to evidence recording, not execution. P6's
  deferral of cross-language adapters is supported by this grounding.
- **D-5 — Generation vs the sibling-tool boundaries (P4+P5).** Writing
  sidecars touches espectacular's contract; generating OpenSpec proposals
  must emit **plain deltas + `layer.md` sidecars** (Revision 19 discipline;
  the stock archive destroys the specodelic layer — see the 2026-09-28
  dual-format spike findings) and openspec writes stay human-gated. The
  build spec's P5 exit condition ("a contract change can be reviewed and
  accepted without manually maintaining a competing behavioral definition")
  is achievable *through the existing archive-companion recipe*; it must not
  open a second write path into `openspec/specs/`.
- **D-6 — First vertical slice.** Their Q4/Q5 (evidence fields; applicability
  after change) have local candidates: scope-digest the sidecar + bound test
  files (the conform/verify `scope_sha256` pattern applied to bindings), and
  reuse `add-evidence-audit`'s `stale` verdict for re-classification after
  revision. The smallest slice the grounding supports: **one wild-core
  scenario, its existing sidecar binding, evidence recorded and
  staleness-checked** — no schema change, no new runner (D-4 shape), reusing
  C-2/C-3 machinery.
- **D-7 — Suite scope vs corpus constraints.** Cross-file suites must respect
  `single_root_reachable`'s leaf convention (C-7); corpus-wide compile over a
  suite directory additionally runs into `specodelic-523w` (no corpus compile
  scope guard — later same-stem file silently wins). One-file-per-suite
  sidesteps both.

## Relationship to in-flight work (do not collide)

- **`openspec/changes/add-evidence-audit/`** — graded-evidence verb with
  staged slices; P4 should be a *consumer* of its verdict set (D-2), and its
  stale-diff (`--before`) machinery is the natural staleness carrier for
  sidecar evidence (D-6).
- **`openspec/changes/add-preservation-check/`, `add-changed-scoping`,
  `add-doctor-advice`** — orthogonal lifecycle work; no overlap beyond
  shared evidence plumbing.
- **`add-py-fragment-emission` / `add-ts-fragment-emission`** — parked
  (2026-10-20 decision). Relevant only if P6 ever promotes cross-language
  *fragment* execution; D-4's sidecar-runner shape is the alternative and
  keeps the park in place.
- **`specodelic-ynfv`** (rustfmt-clean `*_props.rs`) — matters the moment
  compiled fragments are consumed outside this repo (consumer fmt gates);
  P2/P3 artifact distribution inherits it.
- **`specodelic-523w`** (corpus compile scope guard) and **`specodelic-1ey4`**
  (verify artifact_paths via `citation_corpus::artifact_dir`) — both sit on
  the path P4 evidence recording would walk.
- **Nov 2 grounding doc** — its D-5 (derive-at-check-time vs pinned digest)
  *is* P4's evidence model question; its D-7 ("smaller Specodelic") is the
  strategic frame this build spec either honors or quietly violates; its
  D-9 (unify the two acceptance gates) is the same unifying instinct as the
  build spec's design-decision summary. Resolve jointly, not separately.

## Explicit non-goals for this document

No implementation, no ticket creation, no schema proposal, no verdict on
whether the build spec *should* be built — that is D-1…D-7's outcome, not
this document's. Nothing here overrides the repo's sibling-tool constraints
(espectacular read-only over `openspec/`; pretender hook ownership; vampiro
advisory-only) or the commit-hygiene and gate-drill policies.

## Suggested next steps (design work, not implementation)

1. Answer D-1/D-2 jointly with the `add-evidence-audit` design review — one
   evidence vocabulary, one join-key decision.
2. Draft the **requirement-to-repository traceability matrix** for one
   `wild-core` scenario (the conversation's own named next artifact) as the
   concrete D-6 probe: sidecar binding + property row + evidence record,
   on paper first.
3. Reconcile with the Nov 2 doc's D-5/D-7/D-9 in a single design session —
   three questions, one sitting, both documents updated.

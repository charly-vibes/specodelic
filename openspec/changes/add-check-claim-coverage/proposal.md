# Change: Claim coverage — name every judged claim, list every expected behaviour

> Scope A (the ratified product scope: `openspec/decisions/2026-11-18-scope-a-specification-tool.md`).
> Grounded in the review
> `.wai/resources/research/2026-11-21-expected-behaviour-knowledge-retention-review.md`
> (CORR-001, CORR-005). Sidecar consumption stays scope B, demand-gated — see
> `## Out of scope`; `specodelic-yfib` (`binding_kind` consumer) stays reserved.

## Why

The corpus already expresses expected behaviour: an `invariant` row is a
claim, and it is typed, linted, and citable. What it does not do is say
**whether anything stands behind that claim**, and it does not give the claim
a name anything outside the corpus can cite.

Two measurements from this repo settle the second point:

- The corpus declares **452 constraint ids across 50 capability spec files**
  (`ls openspec/specs/*/spec.md | wc -l`; 408 invariant, 39 effect,
  2 advisory, 3 extension_point), and
  **28 kebab-style slugs collide** across files — `process-lifecycle` alone
  appears in 21 files. Any join key derived from a name is ambiguous by
  construction.
- The workspace carries **643 sidecar files**, and only **5** of their ids
  match any constraint's slug at all. The join that exists today is by
  *test name*, not by claim.

So a claim has no citable name, and the binding between "what we said should
happen" and "what was actually checked" is carried in whatever artefact
happens to sit next to it — prose in one consumer, a test name in another.
When the pin dies, nothing notices: the run stays green because a green run
measures the evidence that exists, never the evidence that is missing.

Baseline, stated so the first run reads honestly: today most declared claims
have no evidence rows at all, so a first report lists most claims as
`unchecked` and still exits 0. That is the point — the count makes the gap
visible, and a falling `unchecked` count is progress a reader can see.

The `check` verb is the right place to close this, because `check` is
already the judge over the evidence a revision actually has
(`openspec/specs/check/spec.md`), and it already refuses to fabricate a
verdict for evidence it did not obtain (`check.verdict_mapping`). What it
cannot yet say is "this declared claim had no evidence at all" — the
knowledge-absence case is present in the vocabulary (`unchecked`) but
attached to evidence *units*, never to *claims*.

## What Changes

- **`check` names every judged claim by its file-qualified claim id**
  (`<file id>.<claim id>`) — the form the corpus already guarantees unique
  corpus-wide. No row abbreviates a claim to its bare id; nothing is matched
  by name, slug, filename, or similarity.
- **`check` gains a claim-coverage section**: every declared invariant claim
  of the judged file is listed with exactly one class from audit's existing
  closed set (`current`, `stale`, `unchecked`, `contradicted`,
  `binds_nothing`, `new`). A claim with no evidence row carries `unchecked`
  — the class the vocabulary already reserves for knowledge absence —
  listed, counted, and never silently absent.
- **The section never gates.** An unchecked claim is knowledge absence, not a
  contradiction: the exit code stays the one the evidence rows alone
  determine, and the count is surfaced next to the existing counts.
- **No new vocabulary, no new producer, no new file format.** The statuses
  reuse `audit`'s existing classes; the section is derived at report time
  from the rows the run already produced.

The effect: "expected behaviour" becomes a list anyone can read off a
report, and "behaviour with nothing behind it" becomes a number instead of
an absence. That is the scope-A half of "express expected behaviour and do
not lose the knowledge".

## Out of scope

- **Reading sidecar bindings.** `check.no_foreign_authority` forbids it, and
  the scope decision keeps contract-suite consumption in scope B, demand-gated
  (a warrant — a real external consumer or a measured benefit — plus a
  preservation harness). This change touches neither. The join key is
  *decided* here (file-qualified claim id) so that a future scope-B proposal
  starts from a decided key rather than inventing one.
- **A binding registry, an adapter registry, or a runner.** Nothing new
  executes.
- **Deriving claims from tests or from prose.** The anti-goal stands
  (`openspec/changes/archive/2026-09-30-add-spk-migrate/proposal.md:24`).
- **A claim's coverage class is not a binding kind.** Many bindings may
  attach to one claim; a claim with no binding is simply `unchecked`.
- **`binding_kind` / `origin` producers.** They stay reserved
  (`specodelic-yfib`, `specodelic-30s5`); this change gives the *claim* a
  citable name, not the binding a kind.

## Impact

- `openspec/specs/check/spec.md` — two ADDED requirements; the layer
  sidecar carries three new constraints and three new properties.
- `src/check.rs` — report assembly gains the coverage section; no new
  verdict, no new audit class, no exit-code change.
- `docs/src/graded-evidence.md` — one row per new status mapping (the
  coverage section reuses the existing class set; the drift gate
  `scripts/check_evidence_axes.py` must agree).
- `spk check --help` — no new flag; the section is part of the report.
- No change to `verify`, `conform`, `audit`, `model-check`, or the compiler.

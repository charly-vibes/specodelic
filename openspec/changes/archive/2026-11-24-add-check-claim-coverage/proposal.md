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
  match any **constraint's** slug at all. The join that exists today is by
  *test name*, not by claim.
  The full measurement, both readings, is in
  `reviews/2026-11-23-rule-of-5.md` (M3–M6): 5 of 613 distinct sidecar ids
  match a constraint slug, 296 match a **property** slug, and 301 of the 643
  sidecar files name a *property* of their own capability while only 4 name
  a constraint of it. The external surface speaks in **properties** — the
  corpus titles its scenarios by property id
  (`openspec/specs/compile/spec.md:264`, `#### Scenario:
  precondition_violation_rejected`, with `- **VERIFIES**
  [[compile.precondition_violation_rejected]]`) — which is why the placement
  rule (design D8) resolves a property to its claim through the corpus's
  declared `derives_from` edge, and why the key (D1) is a claim key at all.

So a claim has no citable name, and the binding between "what we said should
happen" and "what was actually checked" is carried in whatever artefact
happens to sit next to it — prose in one consumer, a test name in another.
When the pin dies, nothing notices: the run stays green because a green run
measures the evidence that exists, never the evidence that is missing.

Baseline, stated so the first run reads honestly, and measured rather than
assumed (2026-11-23):

- Of the corpus's **408 invariant claims**, **76 (19%)** are placeable by
  this repo's own producers, and **202 (50%)** are reachable through the
  sibling contract surface the tool may not read — so `unchecked` means "no
  evidence from this run", never "no evidence exists" (design D2).
- **470 of the corpus's 572 properties are still `todo_predicate!` stubs**,
  and `spk check` currently judges **0 of the 50** corpus capability files
  (19 refuse at `artifact_gate` because the isolated lint cannot resolve
  cross-file references, 31 at `verify_model` for an absent model-check
  report). The section's first appearance is therefore one capability at a
  time, and the gate wiring it depends on is tracked as a precondition
  (design D10, tasks §0).

A first report is consequently a small, honest list — most of it `unchecked` —
and it counts rather than blocks (D4). That is the point: the count makes
the gap visible, and a falling `unchecked` count is progress a reader can
see.

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
- **The placement rule is decided**: a row places a claim when it names the
  claim directly (a model row) or names a property whose `derives_from` names
  the claim; a claim's class is the strongest class among the rows that
  place it (`contradicted` > `current` > `binds_nothing` > `unchecked`).
  Without this, two conforming implementations would disagree — and scope B
  would inherit the ambiguity instead of inheriting a decision.
- **The section says what it is about**: it describes the evidence *this
  run's* producers produced; evidence outside this run's authority is not
  read. It lives in the check envelope, not in the audit report, so the
  report's schema version and field set stay untouched.
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

- `openspec/specs/check/spec.md` — five ADDED requirements; the layer
  sidecar carries four new constraints and four new properties.
- `src/check.rs` — report assembly gains the coverage section; no new
  verdict, no new audit class, no exit-code change. The section is
  materialized in the check envelope; `AuditReport` and its schema version
  are untouched.
- `docs/src/graded-evidence.md` — one row per new status mapping (the
  coverage section reuses the existing class set; the drift gate
  `scripts/check_evidence_axes.py` must agree).
- `spk check --help` — no new flag; the section is part of the report.
- No change to `verify`, `conform`, `audit`, `model-check`, or the compiler.
- **Precondition, tracked separately**: `spk check` cannot yet judge a
  corpus capability file (its artifact gate lints the file in isolation —
  `src/check.rs:242` calls the corpus-less `conform::gate`, while the
  corpus-aware `conform::gate_in_corpus` is wired for `spk conform` only,
  `src/commands/corpus.rs:888-925`). The change does not fix that; its
  self-hosting exercise does.

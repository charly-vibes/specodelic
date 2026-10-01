# Spike — contract-wiring checker: spec-level interface impedance (specodelic-x30)

**Date:** 2026-10-01 · **Ticket:** specodelic-x30 (exploration; no production code
in this ticket) · **Deliverable:** decision note — feasible / not / partial per
pattern. **Companion:** `2026-10-01-wiring-view-decision/` (specodelic-5qj)
decides the *view* half (projections into `add-graph-views`); this note covers
the *detection* half only.

## Decision

**Partial — no new checker pattern.** One small follow-up candidate
(message shape), one already-covered pattern, one already-derivable pattern,
one rejected with evidence:

| # | Pattern | Verdict |
|---|---------|---------|
| 1 | Published-unconsumed (`extension_point`, zero inbound `satisfies`) | **Derivable today** from published graph JSON (`external_boundaries` + `fan_in`); real corpus signal = 0. Advisory-at-best; fold into any future work, do not build a checker for it now. |
| 2 | Consumed-unpublished (`satisfies` → nonexistent/unpublished contract) | **Half-covered.** Wrong-kind → interface-shaped typing violation (good). Unresolved target → generic dangling message (not interface-shaped). Follow-up candidate: interface-shaped dangling message for consumption edges. |
| 3 | Emitted-unobserved (`emits` → effect, zero `observes`) | **Covered.** `linter.observability` (landed 2026-09-30) already warns corpus-wide on exactly this — 34 warnings on the real corpus, same universe, same facts. Do not duplicate (ticket's own instruction, confirmed empirically). |
| 4 | Unlinked expectation (consumer constraint describing contract shape, no `derives_from` to producer) | **Rejected.** No structural signal distinguishes it from any other constraint — evidence below. Requires reading prose/semantics, which `prose_untouched` forbids. |

## Evidence

### Real corpus (HEAD 882d139, `spk graph -j specs`)

- 21 files, 909 edges, 0 dangling, 0 typing violations.
- 3 `extension_point` rows (all in `errors.md`: `envelope_error_kind`,
  `exit_code_mapping`, `remediation_hint_present`) — **all consumed** by
  27 `satisfies` edges across 18 tool files. Pattern 1 fires zero times:
  the errors contract is fully wired. The one apparent hit
  (`USAGE.symmetry_contract`) is an artifact — USAGE.md is frontmatter-less
  (exempt), invisible to the graph; docs tables are not contract rows.
- All 34 effects receiving `emits` have **zero** `observes` edges — and
  `spk lint specs` warns on every one of them via `linter.observability`
  (advisory, exit 0). Same facts, same universe: no graph-level gap beyond
  the lint rule.

### Per-pattern fixtures (run with `spk graph -j <dir>`)

Not committed: `spk lint openspec` runs in the pre-commit hook and any
frontmatter-bearing fixture under `openspec/` is linted as a spec file
(14 spurious issues verified). Shapes are documented inline; the fixture
markdown is reproducible from them in seconds. When a proposal lands,
fixtures live in `tests/cli.rs` (per the ticket's review notes).

- **p1** (producer publishes `hook_contract` `extension_point`, no consumer):
  graph output already carries the fact — `external_boundaries: ['producer']`,
  zero `constraints.satisfies` edges. A checker would wrap a fan-in query that
  any consumer can run today:
  `spk graph -j specs | jq '.data.edges | map(select(.kind=="constraints.satisfies")) ...'`.
  Semantics caveat: an unconsumed extension_point is *legitimate OCP design*
  (a contract awaiting its first consumer) — this pattern cannot distinguish
  "published and waiting" from "published and forgotten". Advisory-tier at best.
- **p2a** (`satisfies → [[producer.nope]]`, unpublished): reported as
  `dangling: ['consumer → [[producer.nope]]']` — **generic**, no interface
  framing. A reader cannot tell "typo'd row id" from "consuming a contract
  nobody published" from the message alone (structurally they are the same
  fact, but the *remediation hint* differs: fix the id vs. publish the row).
- **p2b** (`satisfies` → local `invariant` row): typed violation with a good
  message — `satisfies must resolve to an extension_point Constraint
  (Reference Typing); target is a Constraint (kind 'invariant')`. Already
  interface-shaped.
- **p4** (consumer-side invariant restating the contract's shape, no link):
  zero signal in graph output — `dangling: [], violations: [], satisfies: 0`.
  The row is indistinguishable from any healthy invariant. The format's own
  mechanism for expressing consumer-side contract interest IS the typed
  `satisfies` edge; a constraint that *should* link but doesn't is a semantic
  judgment over expr text, which `prose_untouched` and the checker contract
  forbid. **Rejected**: undetectable at the spec level without semantics.
  The hard precondition stands: wiring must be declared in structured cells
  (bajan's `extraction.claims→eval.claims` prose wiring — CORR-001 — is
  invisible by design); "impedance proper" (shape divergence) additionally
  needs both sides modeling contract shape as structured rows, which no
  corpus does today.

### Meter discrepancy (recorded, not blocking)

Ticket meter said `expect 38` violations at base_commit 3f9b25d via
`d['data']['graph']['violations']`. Run verbatim: at that commit the envelope
has **no `data.graph` nesting** (keys are directly under `data`) and reads
**0 violations / 0 dangling**. The 38 figure predates the corpus cleanup
between ticket authoring (09-29) and the error-contract landing (09-30); the
command as written crashes at base_commit. Current HEAD baseline: 0/0.

## Follow-up candidates (for maintainer go/no-go)

1. **Interface-shaped dangling messages for consumption edges** (small,
   lint/graph tier): when a `satisfies` (and by symmetry `observes`) target
   fails to resolve, say what the edge *means* — e.g. `consumer.hooks_installed
   (satisfies) → [[producer.nope]]: no published contract row `producer.nope`
   exists — publish it in the producer's file or fix the id`. Cheap, improves
   every consumer's first-contact error. *This is message polish, not a new
   pattern — would ride the existing dangling machinery.*
2. **Published-unconsumed advisory**: only if dogfood shows contracts sitting
   silently unconsumed matter. Current corpus: 0 hits. The pattern is
   already answerable from published JSON; a lint advisory would be a
   convenience, not new capability.

## Fixture shapes (per pattern, `spk graph -j <dir>`)

- **p1** — `producer.md` (frontmatter id producer) with one `hook_contract`
  `extension_point` row, no other files. Output: `external_boundaries:
  ['producer']`, 0 satisfies edges, 0 dangling/violations.
- **p2a** — producer + `consumer.md` whose `hooks_installed` effect row has
  `satisfies: [[producer.nope]]` (row does not exist). Output:
  `dangling: ['consumer → [[producer.nope]]']` — generic, not
  interface-shaped.
- **p2b** — `consumer_wrong_kind.md` whose effect row satisfies a local
  `hook_shape` **invariant**. Output: typed violation — `satisfies must
  resolve to an extension_point Constraint (Reference Typing); target is a
  Constraint (kind 'invariant')` — already interface-shaped.
- **p4** — producer + `consumer_copy.md` with a `hook_shape` invariant
  restating the contract's shape with no `derives_from`/`satisfies` link to
  the producer. Output: `dangling: [], violations: [], satisfies: 0` — zero
  signal; the row is indistinguishable from any healthy invariant.

## Decision point (HITL)

Per the ticket, maintainer decides go/no-go on proceeding to a change
proposal. Spike recommendation: **no change proposal for a contract-wiring
checker**; optionally file the message-shape follow-up as its own small
ticket. The visualization half proceeds separately per specodelic-5qj
(folded into `add-graph-views`).
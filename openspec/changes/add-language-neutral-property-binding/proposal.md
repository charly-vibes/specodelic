# Change: Add language-neutral property binding — closed fragment language tags and espectacular pytest/property wiring

## Why

The Properties leg of the `Set^𝒦` functor is Rust-bound at five points:
the `**rust:**` marker makes predicate cells verbatim Rust fragments
(specodelic Rev 15), `properties_to_proptest` emits `proptest!` blocks,
`spec_gen` generators yield constant Strings, `CargoRunner` needs the
Rust toolchain on PATH, and shrinking comes from proptest. Only the
first is *conceptual* coupling — the rest is emission and execution
machinery that no spec author ever sees.

A Rule-of-5 review of the generalization discussion (converged Stage 5;
CRITICAL/HIGH findings TypeSafe-verified, measured FP rate 0%) produced
a corrected three-layer ownership split and rescinded the obvious first
draft:

- **Rescinded:** building a language-runner *registry inside specodelic*.
  The repo family already ships one — espectacular's contract TOMLs
  carry `[[tests.cargo]]` ×157 and `[[tests.shell]]` ×3 runner blocks,
  `.espectacular/config.toml` has a `[runners]` section, and `ah doctor`
  already detects `pytest` and `property` capabilities and recommends
  enabling them. A parallel registry would fragment the capability story
  (CORR-002, verified).
- **Decided:** the migration fork (EDGE-001, verified). "Move the prop
  tests into espectacular/ah" is *allowed* exactly as far as contract
  binding to non-cargo property tests goes; it is **prohibited** as
  fragment-compilation or `specs/`-corpus enforcement — espectacular
  stays read-only over `openspec/` and never enforces over the corpus
  (sibling-tool constraints, AGENTS.md).
- **Corrected:** the claim that the metadata-comment convention is the
  neutral ABI (CORR-001, verified). There are two binding layers with
  different jobs: the `// id:` / `// case:` comments key per-block
  verdicts *inside a specodelic artifact*; contract-TOML `flags` bind
  scenario-derived contracts to *existing tests* in any runner. This
  change touches only the second layer's documentation; the first stays
  as-is.

The actionable subset this change implements: widen the fragment marker
to a closed language-tag set (format grammar, stays in specodelic), and
wire + document the binding path espectacular already has — enabling
pytest/property capability adoption with a hand-written hypothesis
exemplar, zero new espectacular code (EDGE-003: the generic
`[[tests.shell]]` escape hatch and the documented `[[tests.pytest]]`
form mean the seam needs no construction, only adoption).

## What Changes

- **Format (specs/specodelic.md, specs/compile.md — new Revision):**
  the `**rust:**` marker widens to a closed language-tag set —
  `**rust:**`, `**py:**`, `**ts:**` — under a new `fragment_language_closed`
  invariant. The fragment-position rule (specodelic-sd1: marker only in
  fragment position, mid-span occurrences are mentions) carries over
  verbatim per tag. `**rust:**` semantics are unchanged — pure widening.
- **Compile:** a fragment's tag is extracted alongside the fragment; a
  tag from the closed set *without an emitter* (`py`, `ts` — emitters are
  a follow-up change) is a **labeled extraction failure with a remediation
  hint**, never a silent fall-through to Rust emission and never prose.
- **Verify (specs/verify.md, no behavior change):** `PropertiesRunner`
  stays the seam; `CargoRunner` stays the only adapter; per-language
  adapters land *with* their emitters in the follow-up — no registry is
  built here (D3).
- **Espectacular (config + contracts only, no openspec delta):**
  `ah doctor --enable pytest --enable property`; authoring guidance for
  binding property-derived contracts to non-cargo property tests
  (`[[tests.pytest]]` node ids / `-k`, `[[tests.shell]]` escape hatch);
  a hand-written hypothesis exemplar bound to one deployed scenario
  proves the seam end-to-end.
- **Boundary restated in docs:** espectacular traces (`property-untraced`
  is bookkeeping) but never enforces spec semantics; the dual-format
  corpus and `specs/` remain specodelic's alone.

## Deferred (follow-up changes, not tasked here)

- **Per-language artifact emission:** a `py` emitter (pytest + hypothesis
  scratch scaffold) and a pytest adapter behind the existing
  `PropertiesRunner` seam. The grammar accepts `**py:**` now so consumer
  corpora can declare intent; compile's labeled failure tells them what
  to wait for.
- **Generator vocabulary:** the hybrid option (language-neutral core
  vocabulary in-format + named project-level registries) is recorded as
  the direction (D5) but deferred — the exemplar binds hand-written
  tests that need no scaffold, so nothing here is blocked by it, and the
  vacuous-`Just(name)`-generator problem deserves its own change.

## Sequencing

- `specodelic-lf3` (migrate corpus cells from `todo_predicate!` to
  executable `**rust:**` fragments) lands first or concurrently: this
  change's grammar widening must not force a second edit of the same
  predicate cells. It is *not* a correctness dependency — the widening is
  purely additive (unknown tags fail only when present) — it is a churn
  dependency.
- The pytest exemplar depends only on espectacular 0.9.2's shipped
  capabilities and one deployed scenario; it can land immediately.

## Capabilities

### ADDED

- `compile`: closed language-tag fragment opt-in — the tag set, the
  unknown-tag failure mode, and the no-emitter gate.
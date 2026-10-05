---
date: 2026-10-05
project: domain-specific-extensions
phase: design
---

# Session Handoff

## What Was Done

<!-- Summary of completed work -->

## Key Decisions

<!-- Decisions made and rationale -->

## Gotchas & Surprises

<!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->

## What Took Longer Than Expected

<!-- Steps that needed multiple attempts. Commands that failed before the right one. -->

## Open Questions

<!-- Unresolved questions -->

## Next Steps

<!-- Prioritized list of what to do next -->

## Context

### open_issues

```
○ specodelic-0jg P2 tap: publish homebrew/scoop formulae — tap serves 0.3.1, now 2 releases behind (0.4.0, 0.5.0); TAP_GITHUB_TOKEN absent (HITL)
○ specodelic-g17 P2 refactor: tighten pretender gate ratchet (drive violations DOWN, thresholds with them)
○ specodelic-3a8 P3 refactor: decouple dual-format (openspec) metadata from core Spec parse — adapter layer
○ specodelic-814 P3 corpus: spec-doc sync — guard-typing row doesn't mention the **rust:** executable-fragment escape hatch (Rev 15 transition)
○ specodelic-gre P3 implement add-graph-views — edge-list projection, guide --json, native dot/mermaid, wiring view, transform prototype (openspec change approved 2026-09-29, D4 revised 2026-10-04 per specodelic-hya: schema view now derives from the lint-gated acset Schema value)
○ specodelic-lf3 P3 ci: wire spk verify specs into just ci — blocked on corpus cells migrating todo_predicate! to executable **rust:** fragments
○ specodelic-75m P4 corpus: revision-upgrade tooling — no spk migrate path across format revisions for user corpora
○ specodelic-mlc P4 corpus: consolidate restated outbound-leaf carve-outs into one normative statement

--------------------------------------------------------------------------------
Total: 8 issues (8 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)

```


## Decision Matrix

- Problem: How should the specodelic format enable first-class domain-specific extensions (domain packs) while preserving append-only evolution, prose_untouched, closed-kind discipline, and self-hosting?
- Decision: 02-profile-intent — Profile intent + manifest section wins. Evidence: only approach green on pack-namespacing, capability-negotiation, invariant-preservation, core-minimality and discoverability while keeping migration cost zero for the existing corpus. Decided under the strict thin-core position (D7): core grows once for the mechanism itself (kind: profile as an explicit Revision delta), everything else rides packs as Grothendieck fibers one level up (theory.md precedent) — closed sets become fiber-relative and stop growing. D1 v2 (Ro5-reviewed, TypeSafe-verified on the HIGH findings): pack = four-layer kind: profile spec + six per-facet manifest tables (Sections/Kinds/References/Checkers/Floors/Requires) + corpus-scan discovery (no config file) + advisory-first vocabulary-triggered opt-in + draft→published→deprecated lifecycle. 04-external-manifest rejected: violates the self-hosting ethic every vendor preserved. 03-row-level-extension-point rejected: the edge cannot carry section/kind/floor registration (its column shows red on floors, time, EARS, invariants). 01-status-quo rejected: global closed sets collide (2 vendors claimed tolerance on different layers; two mistral proposals collided on ## Data). D6: pilot pack = bioimage-data (most mechanically complete vendor design: spec-first checkers, outbound-leaf validated_against proof, rank-≤6 handoff unification; exercises data/lineage + numeric predicates + empirical-kind standard packs at once); quant second. Status quo's zero-migration advantage is preserved by design: packs are optional, files without packs lint identically.
- Matrix: .wai/projects/domain-specific-extensions/designs/matrix/ (60/60 cells filled)
- Design doc: .wai/projects/domain-specific-extensions/designs/2026-10-02-profile-intent.md

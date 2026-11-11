---
date: 2026-11-11
project: specodelic
phase: research
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

### git_status

```
?? .wai/projects/specodelic/reviews/
?? .wai/projects/specodelic/runs/
?? .wai/resources/pipelines/.last-run
```

### open_issues

```
○ specodelic-04c P2 Decide kernel.binding vocabulary enablement: orphan_vocabulary labels files with the kernel.binding column until a kind:profile pack in namespace 'kernel' is discovered (src/packs.rs scans column headers). Found by specodelic-bf5 — its CLI fixture needed a minimal kernel pack as the rule's sanctioned remediation. Decide at §6 corpus migration / archive: ship kernel vocabulary enablement in specs/, or a linter carve-out. Out of specodelic-bf5 scope.
○ specodelic-1ey4 P2 verify: route verify stage artifact_paths through citation_corpus::artifact_dir — flat-path residual after yl39
○ specodelic-39w P2 [bug] acset instance builder panics on model.state anchor links — uninterned pseudo-node
○ specodelic-3yt9 P2 audit: bind:<owner-id> row key can collide with an oracle corpus id in one report
○ specodelic-4und P2 experiment E2: kernel coverage of fragments — gates add-shared-fragment-helpers
○ specodelic-523w P2 [bug] compile: commands/corpus.rs (spk compile over a corpus) has no scope guard — later same-stem file silently wins
○ specodelic-5iez P2 conform: no explain topic — oracle JSONL schema, verdict taxonomy, permitted-requires-executable-claims rule undocumented (gh-18)
○ specodelic-7c7 P2 lint: model_present 'empty-but-present beats absent' wording promises a differential that does not exist
○ specodelic-d8c P2 lint: Reference Typing (ref_kind_compatible) is declared but enforced by no rule
○ specodelic-f4sx P2 [bug] conform: single-file lint gate dangles corpus-wide refs; prose-dominated unknowns; undocumented trace schema (gh-17)
○ specodelic-gixd P2 audit: dedupe audit test fixtures cross-file (l6md RO5U L8 residual)
○ specodelic-h5ks P2 lint: vacuity advisories — extension of specodelic-xci (catitas A2)
○ specodelic-l54a P2 docs: per-file ASCII state-machine views — unblocked by gently-hpd closure
○ specodelic-oeuz P2 docs: no gate on format-summary prose — 'four tables'/'Frontmatter'-for-'Intent' drift shipped to the site
○ specodelic-q7uo P2 compile: unescape \| before fragment emission (catitas A1)
○ specodelic-wq7n P2 [bug] audit: deployed spec Purpose is the archiver TBD placeholder
○ specodelic-xci P2 lint: advisory warnings for placeholder and tautology content (TODO cells, trivially-true predicates)
○ specodelic-ynfv P2 [bug] compile: emit rustfmt-clean *_props.rs — recompile churn trips consumer fmt gates (gh-15)
○ specodelic-0mn1 P3 [bug] ah sync refuses self-hosting corpora: per-file lint can never be clean (total_refs is corpus-wide)
○ specodelic-0xnd P3 import acset.generic as openspec change — schema-parametric incidence core, GATED on adjacency-graph benchmark (external handoff, at/arltificial/renzi)
○ specodelic-18mt P3 conform: optional expect field on scenario records — negative probes classify as agreements (gh-18 item 7)
○ specodelic-2fp1 P3 docs: preservation recipe — conform twice + verdict diff (simplification review)
○ specodelic-3a8 P3 refactor: decouple dual-format (openspec) metadata from core Spec parse — adapter layer
○ specodelic-3xc P3 decide: fan-in multiplicity rule — state_view counts (source,field) pairs, traceability_view counts distinct sources (B3)
○ specodelic-5ae1 P3 [bug] pretender: vendor role lacks function-metric limits — vendored mermaid.min.js breaches 88 global thresholds when present locally, breaking tests/pretender_gate.rs head_passes_gate
○ specodelic-5tm7 P3 experiment E3: composition — state growth inlined vs hand-factored
○ specodelic-60e P3 [bug] model-check: citation-counterexample claim records carry no reason (claim_report_schema F4/F6 follow-up)
○ specodelic-7e7b P3 decide: meter spk orchestrate wall-clock before changed-scoping work
○ specodelic-7nu4 P3 deps: migrate off serde_yaml_ng or track its unsafe-libyaml drop (RUSTSEC-2024-0375 currently ignored in deny.toml)
○ specodelic-814 P3 corpus: spec-doc sync — guard-typing row doesn't mention the **rust:** executable-fragment escape hatch (Rev 15 transition)
○ specodelic-cbh P3 decide: kernel-grammar gate placement (lint vs compile) + derive TYPED_REFERENCE_COLUMNS from schema (F5 + Grok risk)
○ specodelic-dzn P3 Unify the struct-shape mirror: citation_corpus::ResolvedStatus vs kernel::CorpusClaimStatus
○ specodelic-f046 P3 docs: detection-class table + ladder empirical cross-link (catitas C2+C3)
○ specodelic-fem0 P3 docs: kata-workflow page + FAQ entries (catitas C4+C5)
○ specodelic-g1h8 P3 Parked: Wild contract-bridge experiment (design-substrate report §11)
○ specodelic-izsx P3 experiment E1: law substitution — does law_cases pressure simplify without detection loss
○ specodelic-l8wp P3 decide: rust generator strategies sequencing vs parked add-py-fragment-emission (catitas A6)
○ specodelic-lf3 P3 ci: wire spk verify specs into just ci — blocked on corpus cells migrating todo_predicate! to executable **rust:** fragments
○ specodelic-m69q P3 docs: document single-input generator limitation (catitas A6-doc-half)
○ specodelic-nkg P3 graph: text-projection exit codes still silent for dangling refs and supersedes cycles (JSON exits 1) — follow-up to specodelic-0zk
○ specodelic-pjyn P3 experiment E4: vacuity-guided elimination on the study's final models
○ specodelic-uyfi P3 docs: bowling head-to-head evidence case study (catitas C1)
○ specodelic-y5x8 P3 Python kernel agreement backend (split from specodelic-l8l D5)
○ specodelic-75m P4 corpus: revision-upgrade tooling — no spk migrate path across format revisions for user corpora
○ specodelic-i51 P4 hygiene: orphan zero_files fixture, orchestrate help omits parse stage, duplicated warnings, vacuous-claim note (F10)
○ specodelic-mlc P4 corpus: consolidate restated outbound-leaf carve-outs into one normative statement
○ specodelic-mngq P4 [bug] investigate ah auto-report issue #12: exit-1 error report with no error text (gh-12)

--------------------------------------------------------------------------------
Total: 47 issues (47 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```


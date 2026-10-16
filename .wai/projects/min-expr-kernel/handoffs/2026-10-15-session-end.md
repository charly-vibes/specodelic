---
date: 2026-10-15
project: min-expr-kernel
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

### git_status

```
 M .wai/pipeline-runs/epic-orchestrator-2026-10-16-specodelic-t0aj-spk-explain-conform-topic-gh-18.yml
 M .wai/resources/pipelines/.last-run
```

### open_issues

```
○ specodelic-04c P2 Decide kernel.binding vocabulary enablement: orphan_vocabulary labels files with the kernel.binding column until a kind:profile pack in namespace 'kernel' is discovered (src/packs.rs scans column headers). Found by specodelic-bf5 — its CLI fixture needed a minimal kernel pack as the rule's sanctioned remediation. Decide at §6 corpus migration / archive: ship kernel vocabulary enablement in specs/, or a linter carve-out. Out of specodelic-bf5 scope.
○ specodelic-1ey4 P2 verify: route verify stage artifact_paths through citation_corpus::artifact_dir — flat-path residual after yl39
○ specodelic-39w P2 [bug] acset instance builder panics on model.state anchor links — uninterned pseudo-node
○ specodelic-523w P2 [bug] compile: commands/corpus.rs (spk compile over a corpus) has no scope guard — later same-stem file silently wins
○ specodelic-6aj0 P2 follow-up: archive add-conform via dual-format recipe + author per-scenario contract TOMLs (conform spec, 17 no-toml) + RO5U M2 single-rendering-path tidy
○ specodelic-6bq2 P2 [bug] ci: pretender version drift — local 0.7.0 reports reds at ratchet-max files (parse_typing_table cyclo 20, cmd_model_check 17, graph build 26, kernel atomic 16, lint family fns, section_sync check 28) that CI's pinned pretender passes; pin pretender version in repo (like the CI matrix) or document the sanctioned local pin
○ specodelic-7c7 P2 lint: model_present 'empty-but-present beats absent' wording promises a differential that does not exist
○ specodelic-bhp P2 Rollout: tambor/espectacular contract re-key + archive-companion write-path decision (mcy steps 4+6)
○ specodelic-d8c P2 lint: Reference Typing (ref_kind_compatible) is declared but enforced by no rule
○ specodelic-dlf P2 openspec: MODIFIED change for migrate mirror_byte_identical — mixed-delta aggregation supersedes ADDED-only mirror (follow-up to specodelic-54v)
○ specodelic-oeuz P2 docs: no gate on format-summary prose — 'four tables'/'Frontmatter'-for-'Intent' drift shipped to the site
○ specodelic-q596 P2 [epic] gh-bug backlog implementation — 4 verified bug fixes via epic-orchestrator (nvs9, w9ju, yl39, 1v30)
○ specodelic-xci P2 lint: advisory warnings for placeholder and tautology content (TODO cells, trivially-true predicates)
○ specodelic-0mn1 P3 [bug] ah sync refuses self-hosting corpora: per-file lint can never be clean (total_refs is corpus-wide)
○ specodelic-18mt P3 conform: optional expect field on scenario records — negative probes classify as agreements (gh-18 item 7)
○ specodelic-3a8 P3 refactor: decouple dual-format (openspec) metadata from core Spec parse — adapter layer
○ specodelic-3xc P3 decide: fan-in multiplicity rule — state_view counts (source,field) pairs, traceability_view counts distinct sources (B3)
○ specodelic-5ae1 P3 [bug] pretender: vendor role lacks function-metric limits — vendored mermaid.min.js breaches 88 global thresholds when present locally, breaking tests/pretender_gate.rs head_passes_gate
○ specodelic-60e P3 [bug] model-check: citation-counterexample claim records carry no reason (claim_report_schema F4/F6 follow-up)
○ specodelic-7nu4 P3 deps: migrate off serde_yaml_ng or track its unsafe-libyaml drop (RUSTSEC-2024-0375 currently ignored in deny.toml)
○ specodelic-814 P3 corpus: spec-doc sync — guard-typing row doesn't mention the **rust:** executable-fragment escape hatch (Rev 15 transition)
○ specodelic-cbh P3 decide: kernel-grammar gate placement (lint vs compile) + derive TYPED_REFERENCE_COLUMNS from schema (F5 + Grok risk)
○ specodelic-dzn P3 Unify the struct-shape mirror: citation_corpus::ResolvedStatus vs kernel::CorpusClaimStatus
○ specodelic-g1h8 P3 Parked: Wild contract-bridge experiment (design-substrate report §11)
○ specodelic-l8l P3 [epic] Python properties verify real generated inputs
├── ○ specodelic-l8l.1 P3 Marked generators give Rust predicates real typed values
├── ○ specodelic-l8l.2 P3 Keep generator metadata identical across emitters
├── ○ specodelic-l8l.3 P3 Python unit properties pass or report a shrunk failure
├── ○ specodelic-l8l.4 P3 Keep Rust and Python artifact emission consistent
├── ○ specodelic-l8l.5 P3 Partial or inadequate property execution cannot verify
├── ○ specodelic-l8l.6 P3 Keep property failures identical across reporting paths
├── ○ specodelic-l8l.7 P3 Rust and Python agree on every declared kernel fixture
├── ○ specodelic-l8l.8 P3 Keep backend agreement fixtures shared without skipped cases
├── ○ specodelic-l8l.9 P3 A runnable example distinguishes spec checks from application tests
└── ○ specodelic-l8l.10 P3 Python verification retains legacy contracts and migration guidance
○ specodelic-lf3 P3 ci: wire spk verify specs into just ci — blocked on corpus cells migrating todo_predicate! to executable **rust:** fragments
○ specodelic-nkg P3 graph: text-projection exit codes still silent for dangling refs and supersedes cycles (JSON exits 1) — follow-up to specodelic-0zk
○ specodelic-75m P4 corpus: revision-upgrade tooling — no spk migrate path across format revisions for user corpora
○ specodelic-aby P4 [epic] TypeScript properties share complete verification
├── ○ specodelic-aby.1 P3 TypeScript predicates receive the shared declared value domains
├── ○ specodelic-aby.2 P3 Keep TypeScript property manifests aligned with other languages
├── ○ specodelic-aby.3 P3 Mixed Rust Python and TypeScript runs verify only when complete
├── ○ specodelic-aby.4 P3 Keep mixed-language execution outcomes consistent during cleanup
└── ○ specodelic-aby.5 P3 TypeScript support preserves the deployed Python and kernel contracts
○ specodelic-i51 P4 hygiene: orphan zero_files fixture, orchestrate help omits parse stage, duplicated warnings, vacuous-claim note (F10)
○ specodelic-mlc P4 corpus: consolidate restated outbound-leaf carve-outs into one normative statement
○ specodelic-mngq P4 [bug] investigate ah auto-report issue #12: exit-1 error report with no error text (gh-12)

--------------------------------------------------------------------------------
Total: 47 issues (47 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```


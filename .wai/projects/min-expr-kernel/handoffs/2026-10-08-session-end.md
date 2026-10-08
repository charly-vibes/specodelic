---
date: 2026-10-08
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
 M .wai/resources/pipelines/.last-run
 M .wai/resources/reflections/orchestrator-pi-subagents.md
?? .wai/pipeline-runs/epic-orchestrator-2026-10-08-specodelic-gre-9-docs-graphs-wiring-and-primer-tasks-3-1-3-4.yml
?? .wai/projects/domain-specific-extensions/briefs/specodelic-gre.3.md
?? .wai/projects/domain-specific-extensions/briefs/specodelic-gre.4.md
?? .wai/projects/domain-specific-extensions/briefs/specodelic-gre.5.md
?? .wai/projects/domain-specific-extensions/briefs/specodelic-gre.6.md
?? .wai/projects/domain-specific-extensions/briefs/specodelic-gre.7.md
?? .wai/projects/domain-specific-extensions/briefs/specodelic-gre.8.md
?? .wai/projects/domain-specific-extensions/briefs/specodelic-gre.9.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-flake-note-pipeline-gate-s-just-test-showed-model.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-green-2-1-2-2-a-src-guide-rs-gained-pub-fn-va.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-green-2-1-2-2-src-guide-rs-gained-pub-fn-value.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-green-new-scripts-traceability-view-py-render-t.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-green-no-production-change-needed-the-character.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-green-scripts-graph-views-py-gained-the-states-su.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-green-specodelic-gre-3-task-1-5-renderers-are-pur.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-green-src-graph-rs-gains-no-wiring-note-wiringro.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-refactor-extracted-shared-rendering-helpers-into.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-refactor-extracted-slice-topic-source-topic-hel.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-refactor-kept-the-green-change-is-already-minim.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-refactor-no-structural-duplication-left-states-t.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-refactor-reviewed-the-green-diff-for-tidy-opportu.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-refactor-specodelic-gre-3-task-1-5-none-needed.md
?? .wai/projects/domain-specific-extensions/designs/2026-10-08-refactor-tidy-pass-folded-into-the-ratchet-driven.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-behavior-spk-graph-view-wiring-projects-constra.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-artifact-for-gre-7-run.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-artifact-gre-6.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-4-saved-wai-projects-domain-specific-ex.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-5-artifact-placeholder-pending-write.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-5-written-briefs-specodelic-gre-5-md-tas.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-5.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-6-to-be-written-next.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-6-written-briefs-specodelic-gre-6-md-tas.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-7-next.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-7-written-briefs-specodelic-gre-7-md-tas.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-8-next.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-8-written-briefs-specodelic-gre-8-md-tas.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-9-next.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-gre-9-written-briefs-specodelic-gre-9-md-tas.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-record-for-gre-8-run.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-record-gre-9.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-brief-wai-projects-domain-specific-extensions-br.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-gre-5-plan-desired-1-spk-guide-no-args-emits.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-gre-5-run-plan-artifact-for-epic-orchestrator-2026.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-gre-7-task-2-5-plan-file-level-traceability-view.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-green-specodelic-gre-3-task-1-5-cargo-test-test.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-orient-specodelic-gre-3-task-1-5-native-dot-mermai.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-quality-ledger-specodelic-gre-3-task-1-5-scope.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-red-specodelic-gre-3-task-1-5-command-cargo-test.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-ro5u-fixes-specodelic-gre-3-task-1-5-medium-findi.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-ship-specodelic-gre-3-task-1-5-commit-d61cbcd-on.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-specodelic-gre-3-task-1-5-native-dot-mermaid-pro.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-specodelic-gre-6-plan-behavior-python3-scripts.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-specodelic-gre-8-task-2-7-behavior-view-outputs.md
?? .wai/projects/domain-specific-extensions/plans/2026-10-08-specodelic-gre-9-tasks-3-1-3-4-desired-behavior.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-artifact-for-gre-7-run.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-artifact-gre-6.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-gre-5-artifact-none-apply.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-afk-lane-feature-slice-no-hit.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-afk-lane-feature-slice-task-1.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-for-gre-3-afk-lane-feature-sli.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-for-gre-4-afk-lane-feature-slice.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-for-gre-5-afk-lane-feature-slice.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-gre-6-afk-lane-no-hitl-gate.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-gre-7-afk-lane.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-gre-8-afk-lane-cleanup.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-none-apply-gre-9-afk-lane-docs-wiring.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-record-for-gre-8-cleanup-ticket.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-gates-record-gre-9.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-green-evidence-python3-m-unittest-discover-s-sc.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-green-implemented-docs-graphs-recipe-justfile-b.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-green-resolved-tests-cli-model-check-rs-explain.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-orient-gre-5-tasks-2-1-2-2-2-3-schema-2-6-of-a.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-orient-gre-7-task-2-5-file-level-traceability-v.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-orient-specodelic-gre-4-add-graph-views-task-1.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-orient-specodelic-gre-6-per-file-state-machine.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-orient-specodelic-gre-8-add-graph-views-task-2.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-orient-specodelic-gre-9-tasks-3-1-3-4-just-doc.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-quality-ledger-changed-scripts-traceability-vie.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-quality-ledger-gre-5-tasks-2-1-2-2-2-3-schema-2.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-quality-ledger-gre-9-changed-justfile-docs-gr.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-quality-ledger-ro5u-status-complete-no-critica.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-quality-ledger-specodelic-gre-6-what-was-built.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-quality-ledger-specodelic-gre-8-task-2-7-comma.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-red-1-python3-m-unittest-discover-s-scripts.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-red-characterization-command-python3-m-unittest.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-red-cli-surface-command-target-debug-specode.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-red-command-cargo-test-test-ci-wiring-5-new.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-red-command-cargo-test-test-cli-wiring-9-1.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-red-command-python3-m-unittest-discover-s-scri.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-red-command-python3-m-unittest-discover-s-scrip.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ro5u-fixes-1-medium-scripts-test-graph-views.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ro5u-fixes-both-medium-findings-fixed-in-place-du.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ro5u-fixes-high-1-docs-yml-ci-regeneration-defe.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ro5u-fixes-high-fixed-test-graph-views-py-docst.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ro5u-fixes-no-critical-high-findings-the-one-med.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-run-pointer-gre-5-select-complete.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ship-epic-complete-specodelic-gre-closed-all-10.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ship-gre-2-landed-as-2c31628-c5a889b-6984df5-on-m.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ship-gre-4-4770c14-landed-on-main-pushed-deviati.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-spawn-gate-artifact-gre-5.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-spawn-verify-gre-3-d61cbcd-on-gre-dot-mermaid-d.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-spawn-verify-gre-5-a9da2d5-on-gre-schema-diagrams.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-spawn-verify-gre-6-c554b82-states-view-from-gra.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-spawn-verify-gre-7-e44bdeb-traceability-view-d.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-spawn-verify-gre-8-42407ec-679a0ba-prose-pertur.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-spawn-verify-gre-9-977d8cc-just-docs-graphs-rec.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-gre-4-next-slice-unblocked-after-gre-3-maps.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-gre-5-claim-artifact.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-gre-5-next-slice-unblocked-after-gre-4-map.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-gre-6-next-unblocked-after-gre-2-and-gre-5.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-gre-7-next-unblocked-after-gre-6-task-2-5.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-gre-8-next-unblocked-after-gre-7-task-2-7.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-gre-9-next-unblocked-after-gre-8-tasks-3-1.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-specodelic-gre-3-next-slice-unblocked-a.md
?? .wai/projects/domain-specific-extensions/research/2026-10-08-ticket-specodelic-gre-4-next-slice-unblocked-a.md
?? .wai/projects/domain-specific-extensions/reviews/
```

### open_issues

```
○ specodelic-04c P2 Decide kernel.binding vocabulary enablement: orphan_vocabulary labels files with the kernel.binding column until a kind:profile pack in namespace 'kernel' is discovered (src/packs.rs scans column headers). Found by specodelic-bf5 — its CLI fixture needed a minimal kernel pack as the rule's sanctioned remediation. Decide at §6 corpus migration / archive: ship kernel vocabulary enablement in specs/, or a linter carve-out. Out of specodelic-bf5 scope.
○ specodelic-0jg P2 tap: publish homebrew/scoop formulae — tap serves 0.3.1, now 2 releases behind (0.4.0, 0.5.0); TAP_GITHUB_TOKEN absent (HITL)
○ specodelic-7c7 P2 lint: model_present 'empty-but-present beats absent' wording promises a differential that does not exist
○ specodelic-d8c P2 lint: Reference Typing (ref_kind_compatible) is declared but enforced by no rule
○ specodelic-efb P2 [bug] acset instance builder panics on member-path (label-qualified) link targets
○ specodelic-vpq P2 docs: positioning — state the spec-first/disposable-code assumption and when NOT to use the format
○ specodelic-xci P2 lint: advisory warnings for placeholder and tautology content (TODO cells, trivially-true predicates)
○ specodelic-3a8 P3 refactor: decouple dual-format (openspec) metadata from core Spec parse — adapter layer
○ specodelic-814 P3 corpus: spec-doc sync — guard-typing row doesn't mention the **rust:** executable-fragment escape hatch (Rev 15 transition)
○ specodelic-dzn P3 Unify the struct-shape mirror: citation_corpus::ResolvedStatus vs kernel::CorpusClaimStatus
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
○ specodelic-75m P4 corpus: revision-upgrade tooling — no spk migrate path across format revisions for user corpora
○ specodelic-aby P4 [epic] TypeScript properties share complete verification
├── ○ specodelic-aby.1 P3 TypeScript predicates receive the shared declared value domains
├── ○ specodelic-aby.2 P3 Keep TypeScript property manifests aligned with other languages
├── ○ specodelic-aby.3 P3 Mixed Rust Python and TypeScript runs verify only when complete
├── ○ specodelic-aby.4 P3 Keep mixed-language execution outcomes consistent during cleanup
└── ○ specodelic-aby.5 P3 TypeScript support preserves the deployed Python and kernel contracts
○ specodelic-mlc P4 corpus: consolidate restated outbound-leaf carve-outs into one normative statement

--------------------------------------------------------------------------------
Total: 30 issues (30 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```


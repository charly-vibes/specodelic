---
tags: [pipeline-step:select-ticket, pipeline-run:epic-orchestrator-2026-11-12-specodelic-814-guard-typing-rust-escape-hatch, pipeline-run:epic-orchestrator-2026-11-12-specodelic-lf3-verify-gate-in-just-ci, pipeline-step:land-gates]
---

TICKET: specodelic-814 - corpus spec-doc sync: the guard row in openspec/specs/specodelic/spec.md:76 still says a prose-only guard is machine-uninterpreted (invariants_checked: []) with no mention of the **rust:** executable-fragment hatch (Revision 15). Selected now: independent, small, sanctioned direct-revision-note path; only canonical site is specodelic/spec.md (docs/src/specs and docs/book are generated/gitignored); schema_matches_typing_table compares field rows via acset::doc::parse_typing_table (prose cell edit is safe); Revision 19 exists -> append Revision 20 heading + FORMAT_REVISION bump + spk init --force catalog.

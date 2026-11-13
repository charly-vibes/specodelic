---
tags: [pipeline-run:epic-orchestrator-2026-11-13-specodelic-1nhe-law-kind-per-case-fragment-bodies, pipeline-step:land-gates]
---

GATES: none apply — no HITL sign-off blocks specodelic-1nhe.

Why none apply:
- Ticket specodelic-1nhe carries no gate task / sign-off requirement in bd; its scope is fully specified (per-case fragment bodies for law rows, compile emission one block per case, law_cases interaction; anti-goals: no weakening law_cases labels, no single-body-per-row semantics).
- Format Revisions are ordinary in-session workflow under openspec/specs/AGENTS.md ("fold it back into specodelic.md as a new Revision in the same session"), and epic specodelic-ez9a carries the owner's standing authorization for format completion. Precedents Revision 19 (archive recipe) and Revision 20 (guard-row rust hatch, ticket specodelic-814) landed inside epic runs without a separate sign-off.
- scoped-first authorization (the only owner decision in this area) was already given and is being honored; 1nhe extends the same mechanism rather than re-deciding scope.
- No credential/deploy/push gate: orchestration does not push until the ship step, and pushing remains the orchestrator's call per standing rules (subagents never push).

Judgment call worth recording: the Revision must bump the format revision counter, add a CHANGELOG entry (newest first), and update STATUS.md inventory — this is acceptance work, not a gate.

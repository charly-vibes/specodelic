---
tags: [pipeline-step:select-ticket, pipeline-run:epic-orchestrator-2026-11-12-specodelic-7c7-model-present-differential, pipeline-step:select-ticket]
bead: specodelic-7c7
---

TICKET: specodelic-7c7 - model_present finding message and rule-table semantics promise an empty-but-present-beats-absent differential that does not exist: absent Model and empty-but-present Model produce the identical finding, message, and exit 1 (verified on v0.5.2 per bd). Selected now: top of ez9a queue, complexity:s, no HITL gate. Owning corpus spec is linter-model_shape whose model_sections_paired constraint is symmetric and promises no differential; the phrase exists only in code (src/lint/mod.rs:91 semantics feeding the spk-init AGENTS.md catalog, src/lint/rules.rs:174 finding message) so the fix is code-side wording plus pinned fixtures for both cases.

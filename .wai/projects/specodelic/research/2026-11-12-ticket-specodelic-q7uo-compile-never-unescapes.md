---
tags: [pipeline-step:select-ticket, pipeline-run:epic-orchestrator-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-run:epic-orchestrator-2026-11-12-specodelic-7c7-model-present-differential]
---

TICKET: specodelic-q7uo - compile never unescapes \| in fragments: the parser splits table cells on unescaped pipes and keeps the escape, so ||, |=, and closures cannot appear in **rust:** fragments emitted into _props.rs / .toml / model-check scratch crate. Selected now: second in ez9a queue, complexity:s, independent of 7c7; fixes a real expressiveness gap with a byte-stability constraint (legacy artifacts identical for inputs without \|). Surface: fragment extraction in src/compile.rs (FRAGMENT_MARKER **rust:**, fragment.text consumers: properties_to_proptest, constraints toml doc, model_to_tla manifest comment) + scratch_run_source in src/model_check.rs; src/packs.rs:154-161 protected handling is FORBIDDEN to touch.

---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-8mta-resolve-check-kernel-id-literals, pipeline-step:orient]
---

ORIENT: specodelic-8mta (E2 F1) — kernel id literals are opaque strings (src/kernel.rs term_value Term::Id arm), so an unqualified/typo'd literal compares against nothing and yields verified-vacuous claims (measured: ¬(∃ t: t.to == both_go) = verified). Design of record (orchestrator 2026-11-13): resolve-check id literals at evaluation time -> unknown with a labeled reason; add a BTreeSet declared-id lookup built in KernelEnv::from_specs; thread the reason out of term_value so eq/cmp surface the literal's cause; resolve-check reachable's target id too (seed path unchanged); Kleene discipline unchanged; no grammar/parser change. Corpus blast radius: the four **kernel:** claims in openspec/specs/specodelic/spec.md use bound vars, projections, ⊥ and cardinalities only — no bare id literals — so their verdicts must not change.

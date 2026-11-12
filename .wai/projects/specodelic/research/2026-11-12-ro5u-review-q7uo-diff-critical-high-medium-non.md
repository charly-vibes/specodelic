---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-q7uo-compile-pipe-unescape, pipeline-step:ro5u-review]
---

RO5U review (q7uo diff): Critical/High/Medium: NONE. Correctness: unescape at the single Fragment.text producer => props, .tla manifest, scratch crate, audit/conform evidence all share one string; emptiness/hygiene checks run pre-unescape on trimmed raw text, unaffected (no banned token contains a pipe). Byte-stability: replace is identity without the escape, pinned by frag_plain literal-byte test incl. toml/tla. Scope: packs.rs, spec.rs, model_check.rs, lint/graph untouched. Low informational, no fix: (1) an intentional backslash-pair before a pipe inside a fragment string literal would lose one backslash — documented in the helper doc, same accepted semantics as packs.rs's ESC protection, unreachable at expression position; (2) IR guard_citations/emits_values keep raw text — citation grammar uses ∧/∨/¬ not pipes; ticket scopes fragment emission only.

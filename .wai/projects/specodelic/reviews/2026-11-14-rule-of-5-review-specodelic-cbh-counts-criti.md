---
reviews: 2026-11-14-green-1-decision-recorded-in-openspec-decisions.md
verdict: needs-work
severity: {critical: 0, high: 0, medium: 2, low: 2}
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-cbh-kernel-grammar-gate-placement-derive-typed-reference-columns-from-schema, pipeline-step:ro5u-review]
---

# Rule of 5 review — specodelic-cbh

Counts: CRITICAL 0, HIGH 0, MEDIUM 2, LOW 2.

Derivation verified sound: the derived set is exactly the previous seven columns; `from`/`to` are owned by the transitions walk, `uses` is omitted from graph total_extraction and is an outbound leaf per packs.md; LazyLock is deterministic and private; graph output is byte-identical; the pin test fails on a new unexempted column.

MEDIUM 1 — normative prose claimed `spk verify` compiles first. It never re-compiles; grammar is reached via `spk compile` and `spk model-check`'s kernel_grammar pre-check. Fixed in kernel.md Notes and the decision note.

MEDIUM 2 — the new test insertion accidentally deleted the pinned mermaid `classDef violation` assertion from `render_wiring_labels_empty_views`. Restored.

LOW 1 — the coverage loop was tautological by construction. Replaced with `exempt.is_subset(&schema_cols)`, which catches bogus/stale exemptions; the exact-set pin remains the new-column gate.

LOW 2 — decision note attributed the `uses` exemption to graph.md's total_extraction; corrected to graph.md omission + packs.md `uses_is_outbound_leaf`.

All fixed; no remaining findings.

---
tags: [pipeline-run:tdd-ro5-2026-11-14-specodelic-cbh-kernel-grammar-gate-placement-derive-typed-reference-columns-from-schema, pipeline-step:refactor]
---

REFACTOR: none needed — the derivation is one LazyLock static + one accessor, the exemption list is a named const with documented rationale, and the gate test is one focused test. No duplication or dead weight surfaced; behavior unchanged.

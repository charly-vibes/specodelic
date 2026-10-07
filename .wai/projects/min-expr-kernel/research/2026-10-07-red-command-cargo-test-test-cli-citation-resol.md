---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-hb4-add-min-expr-kernel-ss3-6-citation-resolution-in-commands, pipeline-step:red]
---

RED: command='cargo test --test cli citation_resolution' — 13 tests, 12 fail / 1 regression pin (multi-file lint) passes. Intended failures verified: (a) baseline divergence reproduced manually — cbdemo report shows c2 bare [[c1]]=verified, c3 qualified [[cbdemo.c1]]=unknown (29f81ef class); CLI JSON lacks invariant_statuses entirely; (b) scope tests fail because isolated_scope_required/duplicate_corpus_identity labels do not exist; (c) cycle/missing reasons absent from output. Fixture lints corrected (linter.coverage deriving properties per constraint). Evidence file: .wai/projects/min-expr-kernel/research/2026-10-07-red-hb4.md

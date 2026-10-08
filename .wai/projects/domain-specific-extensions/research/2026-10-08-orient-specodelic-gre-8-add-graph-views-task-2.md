---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-gre-8-derived-diagrams-stable-under-prose-perturbation-task-2-7, pipeline-step:orient]
---

ORIENT: specodelic-gre.8 = add-graph-views task 2.7 only (TIDY). Two legs: (1) views_from_artifact_only characterization test — perturb prose (frontmatter statement, prose under headings, trailing prose) of a copy of tests/fixtures/typing_violations, regenerate artifacts with target/debug/specodelic, assert states+trace view outputs byte-identical; verified manually that current code passes (TSV + envelope data identical). (2) TIDY: move genuinely shared rendering helpers (mermaid_escape/names, parse_edges_tsv, owning_file, scope gate stack, fan-in, violation lines, no-output note) into view_common.py; semantics untouched. Characterization already run by hand — binary needed because the chain is corpus→artifacts→views.

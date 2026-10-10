---
tags: [pipeline-run:tdd-ro5-2026-10-09-specodelic-eczv-1-conform-phase-1-verdict-engine-core, pipeline-step:red]
---

RED: command=TMPDIR=/var/tmp/specodelic-eczv just test-smart; expected failure=tests/conform_classification.rs fails to compile — no Verdict/classify_claims/classify_traces/parse_corpus/ScenarioTrace in specodelic::conform (module is header-only stub); observed=exactly that, E0425/E0432, all 10 phase-1 tests absent from the binary. Failure is the missing verdict-engine API, not setup breakage.

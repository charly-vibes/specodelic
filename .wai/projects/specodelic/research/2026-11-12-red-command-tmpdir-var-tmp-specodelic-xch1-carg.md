---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-7c7-model-present-differential, pipeline-step:red]
---

RED: command='TMPDIR=/var/tmp/specodelic-xch1 cargo test --test cli model_present -- --nocapture'; expected failure=assertion on differential claim; actual='absent: no differential claim allowed: Model section incomplete: 0 states, 0 transitions — both sections must exist (empty-but-present beats absent)' — fixture A (no Model) produced exactly one linter.model_present finding with exit 1 as expected; failure is purely the wording, not setup

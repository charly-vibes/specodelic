---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-3-conform-phase-3-input-gate, pipeline-step:plan]
---

eczv.3 phase 3: (1) RED tests/conform_gate.rs — stale_artifacts_refused (on-disk .tla compiled from older structured content -> gate refuses, hint names spk compile, zero verdict records) + lint_dirty_refused (file with lint findings refused, hint names spk lint, zero records). (2) GREEN conform::gate in src/conform.rs reusing orchestrate's stage discipline (lint stage + currency via compile output vs on-disk .tla; pub(crate) visibility tweak allowed). (3) RED corpus-validation refusals: malformed JSONL, missing id, duplicate ids, unknown fields -> ConformError with remediation hints. (4) RED->GREEN empty corpus -> valid report, zero records, evidence_scope intact, Ok. Scope: src/conform.rs, tests/conform_gate.rs, tasks.md 3.x checkoffs only. Narrow: TMPDIR=/var/tmp/specodelic-eczv just test-smart. Full: just test + just lint + just pretender-check.

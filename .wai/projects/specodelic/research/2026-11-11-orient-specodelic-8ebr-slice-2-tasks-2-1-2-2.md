---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-8ebr-pack-check-rows, pipeline-step:orient]
---

ORIENT: specodelic-8ebr slice 2 (tasks 2.1-2.2) — pack-check audit rows. Slice 1 landed at 3c754b8 (src/audit.rs has taxonomy on conform verdicts + ensure_diffable/overlay_class). Delta spec scenario 'Empty checked-set is unchecked' + 'Unchecked never fails the run' are the pins. Brief scope guard: only src/audit.rs, tests/audit_oracle_rows.rs, tasks.md 2.1-2.2. bd metadata lists src/packs.rs but brief overrides — pack-row surface designed inside audit.rs. Supersedes va3q.

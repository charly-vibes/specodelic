---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-7c7-model-present-differential, pipeline-step:green]
---

GREEN: minimal change = reword two strings. rules.rs firing message -> 'Model section incomplete: {n} states, {m} transitions — both a States list and a Transitions table must exist' (counts+remediation kept, differential dropped); mod.rs RULE_TABLE semantics -> 'the Model section must contain both a States list and a Transitions table'. RED test now green; full --lib lint suite (93) + --test cli (311) green — no other test asserted the old message

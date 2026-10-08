---
tags: [pipeline-run:epic-orchestrator-2026-10-08-specodelic-68m-5-claim-blockers-report-parity, pipeline-step:ship]
---

GREEN specodelic-68m.5: claim parity landed — CheckedFile carries claims/expected/unchecked (orchestrate.rs), CLI checked entries + model_gate_json expose them (commands/model_check.rs), ModelGateState::NotClean carries blocking_claims/unchecked_claim_ids and verdict names them (verify.rs), human::model_check/verify render the claim view via claims_lines (human.rs). Docs synced: README verify bullet + verification-vs-application-testing note, docs/src/status.md capability row + version 0.5.2, openspec/project.md verification-claims bullet, specs/STATUS.md claims paragraph, embedded guide lifecycle topic (src/guide.md) + guard unit test (src/guide.rs). RED→GREEN observed; docs fixture RED at 0.5.0 vs 0.5.2.

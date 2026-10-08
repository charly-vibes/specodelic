---
tags: [pipeline-run:epic-orchestrator-2026-10-08-specodelic-68m-5-claim-blockers-report-parity, pipeline-step:write-brief]
---

ORIENT specodelic-68m.5: users see the same claim blockers in every report — governed by define-verification-claim-gates D2 (aggregate rules), D3 (report version/freshness), D5 (explain assurance levels). Tasks 3.1 RED parity fixtures (JSON/persisted/human claim counts agree; docs check fails when version literal or capability status conflicts with Cargo metadata) and 3.2 GREEN (views surface evaluated/unchecked/blocking claims consistently; sync README, docs/src/status.md, openspec/project.md, specs/STATUS.md, src/guide.rs; document verification vs application-test distinction). 3.3 TIDY is separate ticket specodelic-68m.6. Prior tickets 68m.1-68m.4 landed claim classification, aggregate rules, versioned scope-bound reports.

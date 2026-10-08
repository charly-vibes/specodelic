---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-4v1-failures-ride-success-shaped-envelopes-f1, pipeline-step:fix-review]
---

RO5U-FIXES specodelic-4v1: fixed both MEDIUM findings — (1) model-check not_clean next_step hint generalized from 'refuted claim(s)' to 'not-clean outcome(s) named in .data.checked' (timed_out/exploration_only fire the same branch); (2) docs/src/commands.md exit-codes paragraph rewritten: envelope_kind error + ok:false whenever the run is not clean (was 'nonzero-by-invocation'); model-check section now states the aggregate→exit/envelope mapping incl. CHANGELOG #116; verify section gained the F7 scope-digest prose (digest binds inputs, not result statuses — forged statuses rejected, report never rewritten). Targeted suites green, fmt clean, docs build clean. LOW findings noted-only (helper transitive coverage; genesis stderr convention; file-local fixture duplication).

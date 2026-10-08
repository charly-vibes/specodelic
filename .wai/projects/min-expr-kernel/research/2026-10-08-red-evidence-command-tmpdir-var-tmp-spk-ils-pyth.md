---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:red]
---

RED evidence: command=TMPDIR=/var/tmp/spk-ils python3 -m unittest discover -s scripts -p 'test_mermaid_escape.py' + -p 'test_hint_routing.py'. Expected failures confirmed for the MISSING BEHAVIOR, not setup: (1) escape: 5/8 fail — arrow (a --> b unescaped), hash (# raw), %% (raw), & (raw), newline (unflattened); quote/safe-text/determinism pass as existing-contract controls. (2) hint routing: test_lint_envelope_without_issues_list_gets_artifact_hint fails with stderr 'artifact_invalid: lint envelope carries no issues list — fix the reported invariants...' — exactly the B1 bug (ArtifactInvalid wrongly gets LINT_HINT); 3 controls pass. (3) B4 meter: grep -c 'marker-free whole-cell opt-in' tests/kernel_grammar.rs == 1 (must be 0).

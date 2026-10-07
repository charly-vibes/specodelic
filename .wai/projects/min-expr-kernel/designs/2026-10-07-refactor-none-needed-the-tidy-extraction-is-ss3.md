---
tags: [pipeline-run:tdd-ro5-2026-10-07-specodelic-mui-ss3-7-kernel-claims-across-the-complete-invocation-corpus, pipeline-step:refactor]
---

REFACTOR: none needed — the TIDY extraction is §3.8's own separate work item (specodelic-k3h: shared identity/evaluation helpers), deliberately out of scope here per the tasks.md amendment. The §3.7 change keeps kernel.rs's corpus pass self-contained with the two output shapes mirroring ResolvedStatus; cargo fmt + clippy -D warnings clean; just test green after refactor step.

---
tags: [pipeline-run:tdd-ro5-2026-10-08-specodelic-ils-view-scripts-b1-hint-routing-b2-mermaid-escaping-b4-docstring, pipeline-step:refactor]
---

REFACTOR: cargo fmt clean; clippy -D warnings forced one collapse in the rust mirror (.replace(['\r','\n'], " ") replacing the two single-char replaces) — no behavior change, tests re-run green (rust 1/1, python 109 OK). The pre-existing 'specodelic vs spk dual bin target' manifest warning is unrelated (present before this change). No further refactoring: the diff is already minimal (two escapers + a type-based hint branch + a comment fix).

---
tags: [pipeline-run:tdd-ro5-2026-10-15-specodelic-w9ju-compile-rustfmt-clean-props-rs-emission-gh-15, pipeline-step:green]
---

GREEN: rustfmt_clean (stdin->stdout, edition 2021) invoked in write_artifacts before fs::write; missing/failing rustfmt = graceful fallback (unformatted bytes, warning string returned via new Written{paths,warnings}); warnings threaded into existing warnings vecs in cmd_compile and orchestrate; todo_predicate macro now multi-line by construction. Narrow test green (3/3).

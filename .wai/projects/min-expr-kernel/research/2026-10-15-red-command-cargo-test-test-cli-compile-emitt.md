---
tags: [pipeline-run:tdd-ro5-2026-10-15-specodelic-w9ju-compile-rustfmt-clean-props-rs-emission-gh-15, pipeline-step:red]
---

RED: command='cargo test --test cli compile::emitted' expected failure='emitted _props.rs not rustfmt-clean' actual=fail with rustfmt diff (one-lined macro_rules! todo_predicate, stray blank line in spec_gen module, trailing blank line) — matches gh-15 churn classes 1-3, not setup breakage.

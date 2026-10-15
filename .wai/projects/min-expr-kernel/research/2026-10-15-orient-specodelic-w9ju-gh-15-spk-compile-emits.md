---
tags: [pipeline-run:tdd-ro5-2026-10-15-specodelic-w9ju-compile-rustfmt-clean-props-rs-emission-gh-15, pipeline-step:orient]
---

ORIENT: specodelic-w9ju (gh-15): spk compile emits non-rustfmt-clean *_props.rs; fix = rustfmt pass in write_artifacts (src/compile.rs:1341) with graceful fallback when rustfmt binary missing; red test in tests/cli/compile.rs asserting rustfmt --check over emitted artifacts.

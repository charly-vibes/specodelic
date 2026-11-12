---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-d8c-enforce-reference-typing-ref-kind-compatible-as-a-lint-rule, pipeline-step:green]
---

RED: command=cargo test --lib ref_kind + cargo test --test cli lint_ref_kind. Expected failure=10 unit firing tests + CLI labeled-finding test fail with 0 linter.ref_kind_compatible hits (rule absent from RULE_TABLE; finding_carries_semantics panics on rule_semantics lookup — rule missing from catalog). Pass-side controls (7) green trivially; clean baseline lints clean. Not setup breakage: existing suites untouched.

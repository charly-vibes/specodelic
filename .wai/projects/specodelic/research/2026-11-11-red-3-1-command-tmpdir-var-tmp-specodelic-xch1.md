---
tags: [pipeline-run:tdd-ro5-2026-11-11-specodelic-edrv-binds-nothing, pipeline-step:red]
---

RED 3.1: command='TMPDIR=/var/tmp/specodelic-xch1 cargo test --lib audit::binds_nothing_tests' => compile error: 4x cannot find function bind_rows, 7x no variant AuditClass::BindsNothing (the missing behavior — new API). CLI pin 'cargo test --test audit_binds_nothing' compiles (existing API only) and FAILS at runtime: binds_nothing_flags_on_the_warnings_channel_exit_zero (missing behavior — no bind row emitted). Scaffolding fix during red: removed an unused test helper with a lifetime error (setup breakage, not behavior). Failures are for the expected missing behavior, not setup breakage.

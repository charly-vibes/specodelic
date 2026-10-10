---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-4-conform-phase-4-cli-wiring, pipeline-step:red]
---

RED: command='cargo test --test cli conform' expected='spk conform accepted; report envelope + exit-code contract' failure='error: unrecognized subcommand conform (6 tests fail; clap exit 2, empty stdout) — the missing verb is the missing behavior, fixture setup itself is sound (compile of fixture succeeds)'

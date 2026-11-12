---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-75m-revision-upgrade-tooling, pipeline-step:refactor]
---

REFACTOR: the migrate verb's arguments moved into a MigrateArgs struct (the Commands::Migrate arm is one line again — run() is back under the 209-line function ratchet; the pretender gate is green), and the per-file upgrade moved out of plan_upgrade into upgrade_file(), which returns Ok(None) for a file that is not a corpus spec file instead of routing a skip through the error type. Behavior is unchanged: cargo test --lib migrate 24/24, cargo test --test cli migrate 19/19, pretender check exit 0.

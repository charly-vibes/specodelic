---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-75m-revision-upgrade-tooling, pipeline-step:red]
---

RED: command=TMPDIR=/var/tmp/specodelic-xch1 cargo test --test cli migrate_upgrade expected failure=9/9 new integration tests fail with clap 'unexpected argument --to-revision found' — the migrate verb has no --to-revision/--from-revision/--write flags and no revision registry yet, so the upgrade path is missing (not a setup breakage: the tests compile, the fixtures build, the binary runs and rejects the unknown flag). The OpenSpec change openspec/changes/add-revision-upgrade/ (proposal.md, design.md, tasks.md, specs/migrate/spec.md delta, specs/migrate/layer.md sidecar) validates clean with 'openspec validate add-revision-upgrade --strict', and the sidecar lints with zero findings via 'spk lint openspec/changes/add-revision-upgrade/specs/migrate/layer.md'.

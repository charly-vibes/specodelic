---
tags: [pipeline-step:select-ticket, pipeline-run:epic-orchestrator-2026-11-12-specodelic-75m-revision-upgrade-tooling, pipeline-run:epic-orchestrator-2026-11-12-specodelic-814-guard-typing-rust-escape-hatch]
---

TICKET: specodelic-75m - no spk migrate path across format revisions for user corpora: the format grows append-only revision-gated variants (Rew 2..20 in openspec/specs/specodelic/spec.md; FORMAT_REVISION 20 after 814) and spk exposes format_revision in --version --json, but src/migrate.rs (542 lines) only wraps openspec deltas into dual-format skeletons. Selected now: last independent implementation ticket (lf3 parked on user decision); spec-first (migrate.md delta defining a thin core revision registry + upgrade contract; bulk rewriter beside rename/refactor), then verb extension. Meter: corpus fixture at Revision r upgrades to r+1 via the specced path with spk lint 0 and byte-stable output for unchanged files; just ci green.

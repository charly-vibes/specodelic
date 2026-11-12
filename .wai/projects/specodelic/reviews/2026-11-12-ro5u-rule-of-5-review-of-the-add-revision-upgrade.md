---
reviews: 2026-11-12-orient-plan-specodelic-75m-revision-upgrade-too.md
verdict: needs-work
severity: {critical: 0, high: 0, medium: 2, low: 4}
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-75m-revision-upgrade-tooling, pipeline-step:ro5u-review]
---

RO5U: rule-of-5 review of the add-revision-upgrade diff (src/migrate.rs registry + planner, src/main.rs migrate verb wiring, tests/cli/migrate_upgrade.rs, openspec/changes/add-revision-upgrade/).
CRITICAL: none. HIGH: none.
MEDIUM:
- [M1] src/main.rs::cmd_migrate_verb — a flag conflict is silently resolved: 'spk migrate --rekey --to-revision 18 --write <file>' runs the upgrade and ignores --rekey (evidence: the run output shows mode=upgrade, the file re-keyed through the IdLaw step, exit 0). Fix: refuse the combination with exit 2 and a labeled message (--rekey and --to-revision are different modes).
- [M2] src/main.rs::cmd_migrate_upgrade (write loop) vs openspec/changes/add-revision-upgrade/design.md D6 — the planning phase is atomic (a refusal writes nothing), but the write loop writes file by file: an IO failure at file k leaves files 1..k-1 upgraded while the command exits 2. The spec laws are planning-scoped ('when any file in the supplied set cannot be upgraded, every file stays unchanged') so they hold; the design doc overclaims. Fix: state the caveat in D6 (the plan is atomic, the write loop is not; a re-run is idempotent) — or write through a temp file + rename.
LOW:
- [L1] src/migrate.rs::plan_upgrade — edit artifact: the comment '// The steps to cross, in ascending revision order.' ended up trailing the closing brace of the downgrade 'if' block. Fix: move it above the 'let steps' binding.
- [L2] src/migrate.rs::apply_id_law — a text with frontmatter but no closing fence maps to NoDerivableId ('the real id cannot be derived'), a misleading message for a different fault. It is unreachable today (the step only applies when pending() saw an id: line, which requires a fence). Fix: map it to its own error variant or an 'unreadable frontmatter' message.
- [L3] openspec/changes/add-revision-upgrade/design.md D2 — the detection is only as sharp as the registered step boundaries: a corpus older than Revision 18 is indistinguishable from a Revision-17 corpus (both satisfy Revision 17). Fix: state the limit in D2 so the contract is not read as a full version detector.
- [L4] src/main.rs::cmd_migrate_upgrade — an unreadable file (a dangling symlink) refuses the whole upgrade with exit 2 and writes nothing (verified against /tmp). Not covered by a test; the NoDerivableId path is. Fix: none required; optionally a unix-only dangling-symlink test.
VERIFIED GOOD: the registry's shape and thinness (only Revision 18 carries a step), the shape-guarded steps, the byte-stability (CRLF, no final terminator), the scope confinement, the refusal taxonomy, the exit codes, the envelope's genesis Output path, the MigrateArgs refactor that keeps run() under the 209-line ratchet (pretender check exit 0), and the RED-first test file (9 tests, all green).
Convergence: stage 4 (edge cases); stage 5 adds no new findings.

---
tags: [pipeline-run:tdd-ro5-2026-11-12-specodelic-75m-revision-upgrade-tooling, pipeline-step:quality-ledger]
---

QUALITY LEDGER: specodelic-75m — corpus revision-upgrade tooling (add-revision-upgrade).
CHANGED:
- src/migrate.rs — the core-owned revision registry (REVISIONS: revisions 2..20 with date, one-line change note, optional step), the upgrade seam (file_revision, plan_upgrade, upgrade_file, UpgradeError) and the Revision 18 id-law step (frontmatter id re-key + [[spec…]] ref re-key, byte- and EOL-preserving); 9 unit tests.
- src/main.rs — migrate-verb wiring only: the MigrateArgs struct (paths, --dry-run, --rekey, --to-revision, --from-revision, --write/--force), cmd_migrate_verb (mode dispatch + flag-conflict refusals) and cmd_migrate_upgrade (collect, plan, write only the changed files, genesis Output envelope, exit-2 refusals with remediation hints).
- tests/cli/migrate_upgrade.rs — 12 RED-first CLI integration tests (old-revision corpus upgrade + lint-clean, dry-run writes nothing, scope confinement, unknown target/source refusals, target-below-current refusal, failed upgrade writes nothing, declared source, idempotent second run, byte-identical unchanged file, both flag-conflict refusals).
- tests/cli/main.rs — registers the new test module.
- openspec/changes/archive/2026-11-12-add-revision-upgrade/ — the archived change (proposal.md, design.md, tasks.md all ticked, specs/migrate/spec.md delta, specs/migrate/layer.md sidecar).
- openspec/specs/migrate/spec.md — the deployed capability after the archive (the upgrade constraints, the upgrading/upgraded states, the three upgrade transitions, the upgrade properties and the eleven new scenarios).
- .espectacular/migrate/*.toml — eleven contract TOMLs for the new scenarios, bound to the new test flags (staged under .espectacular/changes/add-revision-upgrade/migrate/ while the change was open, moved into the deployed location by 'ah archive add-revision-upgrade').
VERIFIED:
- TMPDIR=/var/tmp/specodelic-xch1 cargo fmt --check — clean; cargo clippy -- -D warnings — clean (the CI gate's clippy invocation).
- cargo test — full suite green (423 lib tests; 324 in the biggest integration target; 22 in the cli target's migrate slice).
- cargo test --test cli migrate — 22 passed, 0 failed; cargo test --lib migrate — 24 passed, 0 failed.
- just ci — exit 0 (fmt-check, clippy, tests, release build, openspec-validate 53/53, lint-deltas, model-check-specs, sync-sections, doc checks, lint-baseline, pretender-check, kernel/sibling guards, gate-drill lock, ah check --run-tests: 638 passed, 0 findings).
- just lint-specs — 48 files, issues: [] (warnings are pre-existing advisories).
- openspec validate --all --strict — 53 passed, 0 failed; 'openspec validate add-revision-upgrade --strict' — valid (before the archive).
- ah check — 0 findings (deployed); ah check --changes add-revision-upgrade --run-tests — 0 findings, 637 passed (before the archive).
- pretender check — exit 0; src/migrate.rs, src/main.rs and tests/cli/migrate_upgrade.rs all green under the existing ratchets (no threshold was raised).
- just archive-change add-revision-upgrade — openspec_ran true, restored openspec/specs/migrate/spec.md, sidecar archived; 'ah archive add-revision-upgrade' moved the eleven staged contracts; 'spk init --force' reported block updated with an EMPTY AGENTS.md diff (catalog-only: the regenerated block is byte-identical).
REVIEW:
- RO5U (universal rule of 5) ran on the diff: 0 critical, 0 high, 2 medium, 4 low. Both medium findings are fixed (M1: --rekey with --to-revision is now refused instead of silently ignored, locked by a test and stated in the delta and the layer; M2: the write-phase atomicity caveat is stated in design.md D6 — the planning phase is atomic, the write loop is not, a re-run is idempotent). Low findings: L1 (comment placement) fixed, L2 (misleading NoDerivableId for an unterminated frontmatter) fixed with the new UnreadableFrontmatter variant, L3 (detection sharpness stated in D2) fixed, L4 (an unreadable file refuses the whole upgrade — verified manually, tracked, no platform-dependent test added). No review finding remains open.
RISKS:
- The write phase is not atomic against an I/O failure (documented in design.md D6): the files written before the failure stay upgraded; a re-run is idempotent, so the residue heals.
- The revision detection is only as sharp as the registered step boundaries: a corpus older than Revision 18 reads as Revision 17 (documented in design.md D2). Crossing revisions that need no corpus rewrite is byte-stable, so the outcome is the same.
- A concurrent orchestrator run (specodelic-814) reorders .genesis/tools.toml in the shared worktree; that file is left unstaged as a foreign artifact.
NEXT:
- Commit the authored paths only (src/migrate.rs, src/main.rs, tests/cli/main.rs, tests/cli/migrate_upgrade.rs, openspec/specs/migrate/spec.md, openspec/changes/archive/2026-11-12-add-revision-upgrade/**, .espectacular/migrate/*.toml) with no push, no beads update and no 'wai close' — the ticket forbids them; the pipeline's ship-close step is completed up to the commit only.

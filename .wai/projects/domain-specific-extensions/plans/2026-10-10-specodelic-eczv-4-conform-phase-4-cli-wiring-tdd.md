---
tags: [pipeline-run:tdd-ro5-2026-10-10-specodelic-eczv-4-conform-phase-4-cli-wiring, pipeline-step:plan]
---

specodelic-eczv.4 conform phase 4 CLI wiring (TDD plan)

Desired behavior:
- 'spk conform <file> --oracle scenarios.jsonl' evaluates a compiled spec against an oracle corpus, READ-ONLY.
- '--closed-world' flag accepted, recorded in report header AND every verdict record.
- Exit-code contract: 0 = no forbidden/unsupported verdicts; 1 = any forbidden/unsupported; 2 = invocation error or gate refusal with ZERO verdict records emitted.
- 'open_world_never_forbidden' asserted at CLI level: any forbidden verdict in non-closed-world mode is a bug (CLI-level assertion).
- 'spk explain' untouched; no changes to src/conform.rs internals beyond a tiny CLI helper if needed.

Out of scope:
- src/conform.rs logic changes (phases 1-3 landed), explain topics, specs corpus, openspec/specs, .espectacular, pretender.toml.
- Push, wai close, bd tickets.

Test cases (tests/cli/conform.rs, registered in tests/cli/main.rs):
1. happy path: valid spec + oracle with permitted claims -> exit 0, stdout has verdict counts, records carry closed_world flag.
2. forbidden verdict: oracle contradicting a claim -> exit 1, forbidden count > 0.
3. unsupported verdict -> exit 1.
4. underspecified/unknown in open-world mode -> exit 0 (never failing), counts surfaced in output.
5. closed-world: absence-of-coverage -> forbidden, exit 1; report header shows closed_world=true.
6. invocation error: missing file / missing --oracle -> exit 2, NO verdict records on stdout.
7. gate refusal: dirty lint -> exit 2, zero records.
8. help text states READ-ONLY evaluation.

RED: write tests/cli/conform.rs, register module, cargo test --test cli conform -> fails (no Conform verb).
GREEN: add Conform variant to Commands enum (src/main.rs), dispatch arm cmd_conform shim using conform::gate/run/emit_report + exit-code mapping.
TIDY: ratchet check src/main.rs, just lint, just pretender-check, openspec validate --all --strict, tasks.md 4.1-4.3 checkoffs.

Narrow test command: export TMPDIR=/var/tmp/specodelic-eczv; cargo test --test cli conform
Full verification: just test && just lint && just pretender-check && openspec validate --all --strict

Files likely touched: src/main.rs, tests/cli/conform.rs (new), tests/cli/main.rs, openspec/changes/add-conform/tasks.md

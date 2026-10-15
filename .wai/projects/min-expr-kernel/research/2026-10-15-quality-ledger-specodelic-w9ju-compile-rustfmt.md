---
tags: [pipeline-run:tdd-ro5-2026-10-15-specodelic-w9ju-compile-rustfmt-clean-props-rs-emission-gh-15, pipeline-step:quality-ledger]
---

QUALITY LEDGER — specodelic-w9ju (compile rustfmt-clean *_props.rs emission, gh-15)

Changed:
- src/compile.rs — write_artifacts now formats the *.rs artifact with rustfmt (edition 2021, stdin→stdout) before fs::write; graceful fallback writes unformatted bytes + warning when rustfmt is unavailable/failing; new Written{paths,warnings} return; todo_predicate! macro emitted multi-line by construction; rustfmt_clean helper.
- src/commands/corpus.rs, src/orchestrate.rs — thread the warnings/paths split into the existing warnings vecs (minimal deviation from allowed-files list; both callers already owned a warnings channel).
- tests/cli/compile.rs — RED→GREEN contract test: emitted *_props.rs passes rustfmt --check --edition 2021, plus byte-stability across re-run.
- specodelic/*.rs — one-time reformat of 22 corpus artifacts via corpus-wide `spk compile` regen (CORPUS_DIRS) + one-off rustfmt on the legacy orphan quant-finance_props.rs (source tree retired pre-rename; unreachable by the generator).

Verified:
- cargo test / just test → 38 suites green
- meter: spk compile CORPUS_DIRS exit 0 && rustfmt --check --edition 2021 specodelic/*.rs exit 0; re-run byte-identical
- just ci → exit 0 (includes fmt-check, clippy -D warnings, openspec-validate, ah check --run-tests 547 passed, pretender-check, guards)
- ah check → no issues; openspec validate --all --strict → 51 passed, 0 failed

Review:
- RO5U pass; Medium-1 fixed (stdin writer thread, no deadlock window); Medium-2 documented (envelope payload carries semantic props string, disk is fmt-clean). No remaining critical/high.

Risks:
- rustfmt resolved from PATH; absence = graceful fallback with warning (by design)
- rustfmt version drift could change canonical bytes across toolchains (byte-stability contract is per-environment re-run, not cross-toolchain)
- legacy quant-finance.* artifacts are orphans (source retired); formatted in place once

Next: orchestrator verifies, commits incl. specodelic/ reformat, pushes.


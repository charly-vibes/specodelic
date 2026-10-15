---
tags: [pipeline-run:epic-orchestrator-2026-10-15-specodelic-1v30-conform-gate-corpus-scope-unknown-flooding-gh-17, pipeline-step:spawn-subagent]
---

SPAWN: session=subagent:specodelic-1v30:implement report=landed 14f682b — corpus-scoped input gate (discovers corpus root, verdict keys on target file's own findings, partial-corpus + isolated fallback as labeled warnings, force-include target outside tree) + --structural-only agree mode with structural_agree_prose_guard_unchecked reason naming the guard; cmd_conform moved verbatim to commands/corpus.rs, main.rs carries the clap flag (net -158 lines); 6 red-green CLI tests; gates: cargo test all pass, clippy 0, pretender baseline byte-identical, ah check ok, openspec 51/51; dogfood: acset-core cross-file-ref conform exits 0

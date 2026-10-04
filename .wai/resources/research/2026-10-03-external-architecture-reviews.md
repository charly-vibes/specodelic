# 2026-10-03 — External Architecture Reviews: Findings Register

Four independent AI design reviews of `charly-vibes/specodelic`, produced the
same day from the same four-phase brief (supplied by the maintainer), grounded
against the codebase and verified 2026-10-03. This register is the canonical,
deduplicated artifact; the raw packages are the evidence.

## Provenance (raw packages)

| Reviewer | Repo state reviewed | Source package |
|---|---|---|
| GPT-5.6 Luna (OpenAI) | `main`, branch-level pin; flags README v0.4.0 vs docs.rs 0.5.0 vs crates.io 0.1.0 | `~/Downloads/cv/specodelic_architecture_review.zip` (+ `chat-Specodelic Repository Design Review.txt` raw chat, `ai-handoff.md`) |
| Grok 4.5 (xAI) | v0.4.0 main (no commit pin) | `~/Downloads/cv/specodelic-design-review.zip` |
| Vibe / GLM `glm-5-latest-short` (Mistral infra) | `d055583` | `~/Downloads/cv/mistral` |
| GLM `x-preview-l` (chat.z.ai) | `ad4a2c0` (stale, v0.2.0-era) | `~/Downloads/cv/zai-architectural-health-review-of-specodelic.{md,json}`, `zai` (share URL) |
| Vibe / GLM `glm-5-latest-short` (Mistral infra) — 2nd generation | `3e76089` (v0.5.0) | `~/Downloads/cv/specodelic-v0-5-0-technical-architecture-audit-self-contained.md` (+ raw chat `specodelic-tech-audit.{md,json}`, zips) |

All four are static-read only (no build, no test run). Method for grounding:
every finding re-validated against HEAD `a6278d6` before ticketing; CRITICAL/HIGH
findings verified via TypeSafe (`jev-1.13.0`), measured — 0 fabricated,
2 contradicted, 2 below confidence gate.

## Consensus (unanimous, 4/4)

Engine core (dumb parser, derived typed reference graph, check-before-write,
honest verification) is sound — preserve. The weight is hand-coded wiring and
parallel encodings, not the four-layer format.

## Register

| # | Finding | Verification | Disposition |
|---|---|---|---|
| F1 | Typing/ownership knowledge drift-prone: `REFERENCE_TYPING` (src/guide.rs:49) is doc-only; enforcement is a separate match (src/graph.rs:163 `typing_violation`); Checker Ownership table (specs/specodelic.md:81) vs hand-wired order (src/orchestrate.rs:6) | TypeSafe verified 0.98 | **Already covered** by add-acset-core task 2.3 (`schema_matches_typing_table`) — corroboration noted on specodelic-84i, no new work |
| F2 | OpenSpec/dual-format metadata woven into core `Spec` + parser (src/spec.rs:72-75, 265-308) | TypeSafe verified 0.99 | **specodelic-3a8** (P3, blocked-by 84i.1) |
| F3 | Second markdown-table parser: src/packs.rs:126 `table_after` | TypeSafe verified 0.54 (below gate — human call; likely deliberate) | Note added to specodelic-p5b (decide during writer slice) |
| F4 | Outbound-leaf carve-out restated across 8 spec files (specodelic.md ×10) | TypeSafe verified 0.84 | **specodelic-mlc** (P4, sequenced after 84i) |
| F5 | Version inconsistency: Cargo.toml 0.5.0 vs README.md:11 v0.4.0 | verified by direct inspection | **Fixed 2026-10-03** (README → v0.5.0) |
| F6 | lint.rs monolith (3,586 lines; registry exists at lint.rs:199, partially mitigated) | TypeSafe verified 0.95 | Refactor during acset slices; no standalone ticket |
| — | "3+ parallel lifecycle FSMs" (Mistral) | **TypeSafe contradicted 0.91** — corpus delegates (specs/graph.md:53), not restates | Dropped |
| — | "Demote dual-format gate to plugin" (Z.ai) | **TypeSafe contradicted 0.99** — gate ADOPTED 2026-10-01 (AGENTS.md); review saw stale repo state | Dropped |
| — | "Reference Typing is a hard-coded match" (Grok) | Partially stale — it IS a data table; real issue is the doc-only consumer split (= F1) | Merged into F1 |
| F7 | README claims stale vs code: pipeline verbs "not implemented yet" (all ship) and "no executable predicate language yet" (Rev 15 `**rust:**` fragments landed) — found by the 2nd-gen Mistral audit, re-verified against HEAD `7e4df13` | Direct inspection | **Fixed 2026-10-04** (README pipeline paragraph + Status table; same class as F5) |
| F8 | No corpus revision-upgrade tooling: `spk migrate` wraps openspec deltas only; a user corpus written at Revision r has no `spk migrate --from` path across format revisions (audit §2.4) | Direct inspection (`src/migrate.rs` header) | **specodelic-75m** (P4, filed 2026-10-04) |
| — | Constraints+Properties merge (Z.ai only) | Rejected by all current-state reviewers; loses teaching value | Rejected |

## Meta-observation

The 3/4 current-state reviews independently arrived at schema-as-data — the
same direction as the in-flight acset epic — from public sources alone. This
is decision-grade external validation for `add-acset-core` and is recorded on
the epic.

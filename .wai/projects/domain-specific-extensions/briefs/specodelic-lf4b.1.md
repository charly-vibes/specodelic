---
tags: [pipeline-run:epic-orchestrator-2026-10-09-specodelic-lf4b-1-fix-explain-topic-drift-across-readme-index-md-installation-md, pipeline-step:spawn-subagent]
---

# TDD-BRIEF: specodelic-lf4b.1 — Fix explain-topic drift across README, index.md, installation.md

## Run context

- Epic: specodelic-lf4b — Human-facing documentation improvements from external reviews (beads-only epic, no openspec change dir). You are run ticket 1 of 11; only P1 in the epic.
- Staleness gate status: **DISCHARGED (pre-spawn)** — steps 1–2 of this run re-verified the claim against current main (commit 66e03a4):
  - `spk explain` live topics = **9**: format, ears, kinds, references, lifecycle, lint-rules, dual-format, packs, graph-views (source of truth: `src/guide.rs` `TOPICS`, append-only — topics 7–9 are newer than the counts claimed in the three pages).
  - Drift confirmed: `README.md` lines ~122–123 name **6 topics**; `docs/src/index.md` lines ~49–50 say "eight topics" with an inline list of 8 (missing graph-views); `docs/src/installation.md` line ~51 says "seven topics".
  - Acceptance criteria in the ticket are verbatim machine meters (the numbered list below) — no HITL sign-off applies; epic has no gate tasks. Any staleness re-check by you (e.g. `spk explain` again post-red) is belt-and-suspenders, not a gate.
- If the pre-spawn verification is wrong when you re-check (topic count ≠ 9, or the "seven/eight topics" strings no longer exist), STOP, do not implement, and report back: this brief's meters would be stale and the orchestrator must re-derive them.

## Task (from the ticket description)

Three docs pages claim stale counts of guide topics served by `spk explain` — README says 6, index.md says 8, installation.md says 7 — while the binary now serves **9**. Update all three to the correct count **and** the full topic list, keeping each page's own framing. Update the cross-reference *count* only; pages with topic *lists* must also enumerate all 9 topics.

## RED: failing check before any fix

Write and run a failing anti-drift guard script that would catch this drift — then implement against it.

- New script `scripts/check_explain_topic_docs.py` (stdlib only, mirroring `scripts/check_section_sync.py` style): parse topic ids from `src/guide.rs` `TOPICS` (regex over the quoted first elements), then for each of `README.md`, `docs/src/index.md`, `docs/src/installation.md`:
  1. fail if the page's stated count of topics (any prose like "six topics", "seven topics", "eight topics", "9 topics" — allow digit or word counts) ≠ len(TOPICS);
  2. fail if the page names fewer than all TOPICS ids (topic lists/pages must enumerate every served topic);
  3. fail if a quoted count phrase exists that matches no TOPICS-derived count (rejects silent drift of the word-count fallback).
- RED evidence = script exit non-zero against current main, output showing the exact drift lines (README six, index eight, installation seven).
- Add a `just` recipe `check-explain-topic-docs` (justfile, near the other check_* recipes) and wire it into the `ci:` guard chain after `summary-completeness`. Also add a stdlib unittest for the script itself (mirror `check_section_sync_test.py` / `sync-sections-test` pattern): a temp file with "nine topics" + all ids passes, "seven topics" or a missing id fails.
- Tests for the script itself = python stdlib unittest (mirror sync-sections-test pattern). No Rust tests co-modify this scope — the binary's `spk explain` serving (the thing the pages claim about) is untouched; `TOPICS` is read as source, not re-parsed at runtime by the binary.

## GREEN: the docs fix

- `README.md` (~lines 122–123): count → 9 topics, topic list gains `dual-format`, `packs`, `graph-views` with their one-line framings matching `TOPICS` titles (see src/guide.rs:126–137; graph-views = "Entity-relationship and ER-graph views — format-adjacent corpus analysis").
- `docs/src/index.md` (~lines 49–50): "eight topics" → "nine topics"; inline list gains `graph-views`.
- `docs/src/installation.md` (line ~51): "seven topics" → "nine topics".
- Keep each page's own framing/wording (reference list vs cross-reference sentence); do not copy one page's prose into another.
- Meters that must pass (these are the ticket's verbatim acceptance criteria):
  1. `grep -c "seven topics\|eight topics" README.md docs/src/index.md docs/src/installation.md` == 0;
  2. `grep -c "nine topics" docs/src/index.md docs/src/installation.md` == 1 each;
  3. `spk explain | grep -c "^format$\|^ears$\|^kinds$\|^references$\|^lifecycle$\|^lint-rules$\|^dual-format$\|^packs$\|^graph-views$"` == 9;
  4. each of the three pages contains all 9 topic names.

## Scope (guard-drift discipline)

- **Modifiable files:**
  - README.md — count + topic list only
  - docs/src/index.md — count + inline list only
  - docs/src/installation.md — count only
  - scripts/check_explain_topic_docs.py — new anti-drift guard
  - scripts/check_explain_topic_docs_test.py — new stdlib unittest for the guard (mirror check_section_sync_test.py naming/style)
  - justfile — new recipe + ci: wiring
- **Forbidden paths:**
  - docs/src/commands.md, docs/src/SUMMARY.md, docs/src/release.md (generated, gitignored — never edit), docs/src/llm.txt (generated stamp_llms.py artifact — count here is derived at build; stamp_llms.py only edits if release.md/llm.txt are the source, docs/src/llm.txt is generated from the book, so no edit)
  - src/guide.rs, src/**, tests/**, specs/**, openspec/**, .espectacular/**, CHANGELOG.md
  - If src/guide.rs `TOPICS` appears to need edits for meters to pass → scope violation; STOP and report.

## Verification (before reporting back)

1. `python3 scripts/check_explain_topic_docs_test.py` — pass (test the test).
2. `python3 scripts/check_explain_topic_docs.py` — pass (green evidence against the fixed pages).
3. Meters 1–4 above — pass.
4. `spk explain --topic graph-views` serves — sanity that count 9 is the right claim.
5. `just ci` with TMPDIR set to an existing dir (gotcha: nonexistent TMPDIR = 101 phantom failures; use TMPDIR=/var/tmp/specodelic-lf4b-1). Expect GREEN, not a baseline regression signal (pin baselines can print red lines but exit 0 — not a failure).
6. Per-file commits on main (specodelic is a single-worktree repo; no branch/worktree slot for this epic):
   - commit 1: `chore(docs): new anti-drift guard scripts/check_explain_topic_docs.py + just recipe + ci wiring + stdlib unittest` (the RED infrastructure, may commit even if the guard is failing against unfixed pages — report the exit honestly)
   - commit 2: `chore(docs): fix explain-topic drift — nine topics across README, index.md, installation.md (specodelic-lf4b.1)`
   - Per-file commit hygiene (specodelic-0e2): guard script, test, justfile in commit 1; the three docs pages in commit 2. TDD order: write failing guard (RED) → fix pages (GREEN) → then commit in the hygiene order above.
7. Push each commit to origin/main before reporting back (remote `origin`).

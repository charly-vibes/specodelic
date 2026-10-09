---
tags: [pipeline-run:epic-orchestrator-2026-10-09-specodelic-lf4b-1-fix-explain-topic-drift-across-readme-index-md-installation-md, pipeline-step:spawn-subagent]
---

# CONTINUATION brief: specodelic-lf4b.1 — Fix explain-topic drift (run 2; prior spawn aborted)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`
(Rust CLI `specodelic`, alias `spk`). An orchestrator has claimed ticket
`specodelic-lf4b.1` and is hands-off: you own the implementation end-to-end.
The orchestrator will verify with real gates — never claim work the diff
doesn't contain.

A PRIOR subagent spawn for this ticket aborted mid-run (upstream model error).
It left TWO UNCOMMITTED files: `scripts/check_explain_topic_docs.py` and
`scripts/test_check_explain_topic_docs.py`. Inspect them first: verify they
match the spec below, run them, fix or extend as needed — they are yours to
own now. Do not assume they are correct.

## State (verified by orchestrator at pre-spawn)

- HEAD = 03ca451; working tree has only the two uncommitted scripts above.
- Live `spk explain` serves **9** topics: format, ears, kinds, references,
  lifecycle, lint-rules, dual-format, packs, graph-views
  (source of truth: `src/guide.rs` `TOPICS`, append-only).
- Drift still present: `README.md` ~122–123 names 6 topics;
  `docs/src/index.md` ~49–50 says "eight topics" (list of 8, missing
  graph-views); `docs/src/installation.md` ~51 says "seven topics".
- No docs edits, no justfile edits, no commits, no pushes have happened yet.

## Remaining work (RED is partially done; do GREEN + ship)

1. **Finish RED**: make the guard + its stdlib unittest correct and run them
   against the current drifted pages — script must exit 1 naming the three
   drift lines (README six, index eight, installation seven). Guard spec:
   parse topic ids from `src/guide.rs` `TOPICS`; for each of README.md,
   docs/src/index.md, docs/src/installation.md fail if (a) stated count
   (digit or word) != len(TOPICS), (b) any served topic id is absent from
   the page, (c) a count phrase matches no TOPICS-derived count. Fail-closed
   on unparseable/empty TOPICS. Unittest (stdlib unittest, mirror
   `scripts/check_section_sync_test.py` style): temp page with "nine topics"
   + all ids passes; "seven topics" or missing id fails.
2. **GREEN — the docs fix** (each page keeps its own framing; do not delete
   the sentences, correct them):
   - README.md ~122–123: 9 topics; list gains `dual-format`, `packs`,
     `graph-views` (titles per src/guide.rs TOPICS entries).
   - docs/src/index.md ~49–50: "nine topics"; list gains `graph-views`.
   - docs/src/installation.md ~51: "nine topics".
3. **justfile**: add recipe `check-explain-topic-docs` near the other
   check_* recipes; wire into the `ci:` chain after `summary-completeness`;
   add a `check-explain-topic-docs-test`-style invocation or fold the test
   run into the recipe if the sync-sections-test pattern does.
4. **Verify**: unittest passes; guard exits 0 on fixed pages;
   `grep -rn "seven topics\|eight topics" README.md docs/src/index.md docs/src/installation.md` empty;
   `spk explain --topic graph-views` serves.
5. **Commits** (per-file hygiene — stage only files you authored, never bare
   `git add`):
   - commit A: `chore(docs): anti-drift guard for explain-topic docs claims (specodelic-lf4b.1 RED)` — scripts/check_explain_topic_docs.py, scripts/test_check_explain_topic_docs.py, justfile
   - commit B: `chore(docs): fix explain-topic drift — nine topics across README, index.md, installation.md (specodelic-lf4b.1 GREEN)` — the three docs pages
6. **Push**: `git pull --rebase && git push` (yes — for this continuation run
   YOU push; the orchestrator's verify step re-checks remotely). Then report.

## Hard scope guard

- Allowed files ONLY: `scripts/check_explain_topic_docs.py`,
  `scripts/test_check_explain_topic_docs.py`, `justfile`, `README.md`,
  `docs/src/index.md`, `docs/src/installation.md`,
  `.beads/issues.jsonl` (only if you close the ticket — do NOT, orchestrator
  closes it).
- Never edit: `src/**`, `tests/**`, `specs/**`, `openspec/**`,
  `.espectacular/**`, `docs/src/commands.md`, `docs/src/SUMMARY.md`,
  `docs/src/release.md` (generated, gitignored), `pretender.toml`,
  `.wai/resources/**`, this brief. If you believe `src/guide.rs` needs an
  edit for the meters to pass — that is a scope violation; STOP and report.

## Repo facts (boilerplate)

- **Commit hygiene**: before ANY `git commit`, `git status --short`, stage
  only files YOU authored (`git add <paths>`, never bare `git add`);
  unstage foreign files. Attribute the message only to what the diff contains.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang.
- **TMPDIR gotcha**: if you run cargo tests, TMPDIR must EXIST first
  (TMPDIR=/var/tmp/specodelic-lf4b-1, mkdir -p) or tests fail 101 en masse.
- **Do NOT close the bd ticket, do NOT run `wai close`** — orchestrator owns both.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- guard RED evidence (pre-fix) → <result>
- guard GREEN (post-fix) → <result>
- unittest → <result>
- `git push` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">

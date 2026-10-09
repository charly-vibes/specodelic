---
tags: [pipeline-run:epic-orchestrator-2026-10-09-specodelic-lf4b-3-readme-post-install-verification-block, pipeline-step:spawn-subagent]
---

# Subagent brief: specodelic-lf4b.3 — README post-install verification block

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`.
The orchestrator has claimed `specodelic-lf4b.3` and is hands-off.

## Task

README.md `## Install` section (~line 86) ends with curl/cargo commands and no
verification step. Add a short "verify it works" block to README.md ONLY:
`spk --version` and `spk explain` (both work offline, no repo access needed).
Mirror the Z.ai recommendation (post-install check + first command).

## Meters (ticket acceptance criteria)

1. README Install section contains both `spk --version` and `spk explain`.
2. Both commands run successfully when copy-pasted on a clean machine
   (verify locally: run them in a throwaway dir with the repo-built binary).
3. Anti-goal: do NOT duplicate the book's existing-workspace `spk doctor`
   content — instead link `docs/src/installation.md` for the full quickstart.

## Hard scope guard

- Allowed files: `README.md` ONLY. Never edit anything else.
- lf4b.1/lf4b.2 touched README/docs today — pull fresh before editing.

## Repo facts

- Commit hygiene: stage only files you authored, never bare `git add`.
- Non-interactive flags (-f/-rf/-y) always.
- One commit on main: `docs(readme): post-install verify block — spk --version + spk explain (specodelic-lf4b.3)`,
  then `git pull --rebase && git push` (you push this run).
- Do NOT close the bd ticket; do NOT run `wai close`.

## Report (end with this)

## Report
**Commits** / **Gates run** (meter 1-3 results) / **Deviations** / **Next**

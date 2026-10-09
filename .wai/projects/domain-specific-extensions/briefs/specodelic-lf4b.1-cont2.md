---
tags: [pipeline-run:epic-orchestrator-2026-10-09-specodelic-lf4b-1-fix-explain-topic-drift-across-readme-index-md-installation-md, pipeline-step:spawn-subagent]
---

# BOUNDED FIX brief: specodelic-lf4b.1 — verify-stage remediation (run 3)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/specodelic`.
The orchestrator verified run 2's work (commits 56df12d, 406efff, both
pushed) and found exactly ONE violation. Fix only this. Do not refactor
anything else.

## The violation

Ticket anti-goal: "do not introduce a second count statement elsewhere in
the file" (docs/src/installation.md). Current state has TWO "nine topics"
statements there: line 51 (the corrected original comment) and lines 55-57
(an enumeration sentence run 2 added because the guard's rule (b) demanded
every page enumerate all 9 topic ids). The enumeration sentence must go;
installation.md is a cross-reference page — count-only.

## The fix (3 files, nothing else)

1. `docs/src/installation.md`: DELETE lines 55-57 (the sentence "The guide
   serves nine topics (... graph-views) straight from the binary." plus its
   wrap). Keep the line-51 count fix exactly as is. Result: exactly ONE
   "nine topics" statement in the file, no topic enumeration.
2. `scripts/check_explain_topic_docs.py`: change the per-page policy so the
   enumerate-all-served-ids requirement applies ONLY to pages that carry a
   topic enumeration in their framing: `README.md` and `docs/src/index.md`
   must enumerate every served id; `docs/src/installation.md` is
   COUNT-ONLY — it must state the correct count and must NOT be required to
   enumerate ids. Implement as a per-page policy map (e.g. PAGES =
   (("README.md", "enumerate"), ("docs/src/index.md", "enumerate"),
   ("docs/src/installation.md", "count-only"))) — keep fail-closed parsing
   of TOPICS and the count-phrase rules unchanged.
3. `scripts/test_check_explain_topic_docs.py`: update/add stdlib unittests —
   count-only page with correct count and NO ids passes; count-only page
   with wrong count fails; enumerate page missing an id still fails. Keep
   all existing passing tests passing (adjust only what the policy change
   breaks).

## Verify before committing

- `python3 scripts/test_check_explain_topic_docs.py` → all pass
- `python3 scripts/check_explain_topic_docs.py .` → exit 0
- `grep -c "nine topics" docs/src/installation.md` == 1
- `grep -rn "seven topics\|eight topics" README.md docs/src/index.md docs/src/installation.md` → no matches
- `just check-explain-topic-docs` → exit 0
- `just sync-sections-test` (or the recipe that discovers scripts/test_*.py) → all pass

## Commit + push

One commit, staged by exact paths:
`git add docs/src/installation.md scripts/check_explain_topic_docs.py scripts/test_check_explain_topic_docs.py`
Message: `chore(docs): installation.md count-only policy — drop second count statement, enforce per-page enumeration policy (specodelic-lf4b.1 verify fix)`
Then `git pull --rebase && git push`.

## Hard scope guard

Allowed files ONLY: the three above. Never edit: anything else. No bd/wai
state changes.

## Report format (end with this)

## Report

**Commits**
- `<hash>` <message>

**Gates run**
- <each verify command> → <result>

**Deviations** (or "none")

**Next**

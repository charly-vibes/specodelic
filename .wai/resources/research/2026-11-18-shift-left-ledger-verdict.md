# Shift-left ledger — first honest read & verdict (specodelic-an79, acceptance items 3–5)

Date: 2026-11-18 · Inputs: the validated detector
([validation report](2026-11-18-shift-left-detector-validation.md)), the labeling
rubric, mechanical extracts archived in `sources/an79/`
(`ledger_purpose.json`, `ledger2.json`, `fire_labels.json`).

## The ledger (repo × axis)

Correction rate = C1 events ÷ human user messages in **implementation sessions
only** (design point 1 dilution control; purpose via documented heuristic:
subagent-brief/pipeline markers → orchestration, edit/write tool calls →
implementation, else deliberation; transcript/no-human sessions excluded).

| repo | era | impl msgs | impl C1 | impl C1 rate | orch/delib C1 | cycle proxy (median open→closed) | decision-amendments |
|---|---|---|---|---|---|---|---|
| testaruda | post (all data is post-2026-07-07) | 533 | 2 | 0.38% | 0 (31+3 msgs) | 4.6 h (n=269 closed) | N/A |
| tambor | post (all data is post-2026-10-06) | 98 | 1 | 1.02% | 1 orch (111 msgs) | 4.7 h (n=138) | N/A |
| wanna | post (all data is post-2026-10-06) | 16 | 1 | 6.25%† | 0 orch (90 msgs) | 6.0 h (n=89) | N/A |
| canticos | all (no adoption era) | 38 | 0 | 0% | 0 (12+9 msgs) | 1.1 h (n=31) | N/A |
| coffee | all (no adoption era) | 112 | 1 | 0.89% | 0 (17+12 msgs) | 1.3 h (n=153) | N/A |

† n=16 — one event; not a rate, a anecdote. Purpose heuristic has ±1-session
noise (two probes disagreed on one testaruda fire session; the overall C1
counts are unaffected).

## What the three axes honestly say

1. **Correction rate** — spec repos weakly higher (0.38–1.02% vs plain
   0–0.89% within implementation sessions), on **six C1 events total across
   1,095 human messages**. Noise scale; no citable contrast.
2. **Decision-doc amendments** — structurally empty: **no study repo has
   `openspec/decisions/` or `.wai/resources/research/` at all**; the artifact
   class exists only in specodelic (excluded as study subject). Recorded as
   N/A, not zero — the absence itself is the observation: study repos do not
   yet externalize design decisions into checkable docs.
3. **Cycle proxy** — spec-repo median ticket open→closed is 3–5× the plain
   repos' (4.6–6.0 h vs 1.1–1.3 h). Confounded by ticket-granularity
   differences (bd ticket cultures differ per repo) and by plain repos'
   small closed counts; cannot distinguish "process overhead" from "honest
   decomposition".

**Era axis infeasible**: every store begins at (or after) its repo's adoption
date — no pre-adoption session data exists to compare against. Design point
2's within-repo pre/post contrast cannot be computed retrospectively; only
prospective paired capture (point 4) can produce it.

**Classification wrinkle**: coffee (classified "plain control") has carried
openspec change artifacts since ~2026-09-20 (visible in fire #13's context).
The plain/spec dichotomy is fuzzy at the margins; noted, not resolved.

## Verdict (acceptance item 4)

**Inconclusive.** The founding value claim — spec-first discipline reduces
decision back/forth — remains unmeasured after this first honest pass: the
retrospective record is too thin (6 C1 events), two of the three ledger axes
are structurally empty or infeasible, and the one signal that moved (cycle
proxy) is confounded. The crude first-pass "against the hypothesis" reading
does not survive validation (the coffee control flipped, canticos's
denominator was 82% noise), but nothing positive survived either.

**Priority call**: unchanged. Neither check-as-judge nor specodelic-re29 is
promoted or demoted by this evidence; re29 stays blocked per 9p7c's owner
gate. The value claim should be tested prospectively before any capability
work is justified by it.

## Prospective capture (acceptance item 5) — the only path to an answer

- **bd reopen logging**: bd JSONL persists no status history, so reopens are
  lost. Going forward: a one-off weekly snapshot of `status` per issue id
  (script + diff against the prior snapshot; cron optional). One-off scripts,
  not a subsystem — the snapshot diff *is* the mechanism.
- **Paired attempts**: run the next 2–3 comparable feature tickets in a spec
  repo (testaruda/tambor/wanna) as deliberate pairs — one spec-first, one
  straight-to-implementation, same repo, similar size — and diff the ledger
  (C1 events, decision churn, cycle time, reopen count) between them. That
  is the smallest experiment that can actually move the verdict.

# Shift-left ledger — labeling rubric (specodelic-an79, acceptance item 1)

Date: 2026-11-18 · Status: working draft applied to the 30-message detector-validation
sample (item 2). This document is the labeling oracle: labels are assigned by this
rubric, not by vibe. Owner-auditable — every label in the validation sample is
recorded with its evidence text.

## Units

- **Session** — one JSONL file in a repo's agent store (pi: `~/.pi/agent/sessions/--…-<repo>--`, omp: `~/.omp/agent/sessions/-…-<repo>`).
- **User message** — a `role=user` entry with non-empty human text, after wrapper exclusion (E1).

## Session purpose classes (dilution control — design point 1)

| Class | Decision rule (objective evidence) |
|---|---|
| **implementation** | Dominant activity is producing/changing code or repo artifacts under direct human direction. Signal: edit/write tool calls on `src/`/`tests/`, user issues feature/bug tasks. |
| **deliberation** | Design/research talk with no primary code-change goal: weighing options, proposals, decision docs, research notes. Signal: proposal/skill artifacts, discuss-verbs, few or no src/tests edits. |
| **orchestration** | The session drives *other* agent sessions: epic-orchestrator/pipeline runs, subagent spawn/monitor, gate checks, `/renew` handoffs. |
| **mixed** | ≥25% of turns belong to a second class. Label dominant + secondary; analyses compare within dominant class only or drop the session. |
| **unknown** | Insufficient signal. Never guessed; counted as unknown. |

Tie-break: torn between implementation and deliberation, the presence of
`src/`/`tests/` edit calls decides **implementation**.

## Message-level correction classes (the detector's target)

The study measures **behavioral-design back/forth**: the human correcting the
agent's grasp of system behavior (states, guards, invariants, edge cases, file
semantics). Product-strategy churn is excluded (ticket boundary condition).

- **C1 — behavioral correction**: the message negates or replaces agent-produced
  behavior understanding: wrong behavior description, wrong state/guard/invariant,
  missed edge case, wrong file semantics, "you missed X", "that breaks Y",
  "revert — it should Z", mid-loop "still broken" after an agent-claimed fix.
  Includes partial acceptance + partial rejection.
- **C2 — other correction**: corrections outside study scope — product strategy,
  taste/naming, process violations, priority calls. Counted separately, never in
  the study rate.
- **N — not a correction**: new work, questions, confirmations, continuations
  ("ok", "continue"), handoffs/skill invocations, context dumps.

Tie-breaks: a message that both confirms and corrects is labeled by its correcting
part. A correction of the agent's *restatement of the user's own words* is still C1
when it concerns behavior.

**Task-opening bug reports are N (subtype `bug-report-task`)** — reporting
pre-existing breakage is task *input*, not loop back/forth. Only corrections
*responding to agent work in the same session (or the immediately-preceding
exchange)* are C1. This is the rubric's most consequential rule: the crude
first-pass probe counted opening markers regardless of position in the loop.

## Detector definition (opening-marker probe — the thing item 2 validates)

`detected :=` trimmed user message begins, case-insensitively, with a marker in P,
after stripping synthetic wrappers (E1). Initial pattern set P:

> `no`, `nope`, `not quite`, `actually`, `wait`, `wrong`, `incorrect`,
> `that's not`, `thats not`, `it should`, `should be`, `revert`, `undo`,
> `you broke`, `you missed`, `you removed`, `that broke`, `that's wrong`,
> `stop`, `scratch that`, `i meant`, `i said`, `still broken`, `still fails`,
> `doesn't work`, `doesnt work`, `didn't work`, `didn't worked`, `broken`,
> `ok the`

(`hmm` deliberately excluded — ambiguous between thinking-out-loud and objection.
`ok the` added by the 2026-11-18 validation (FN: "ok the screenshots are all
scrambled" — an `ok`-opened defect report on just-produced output); it costs one
precision point in the strict framing and closes the recall gap.)

## Edge cases

- **E1 synthetic wrappers**: harness-injected "user" text is excluded from both
  numerator and denominator — skill invocations (`<!-- installed-from`,
  `/skill:`, "Use the installed … skill"), command XML, system-context blocks,
  `Caveat:` prefixes, orchestrator subagent briefs (`# Subagent brief: …`),
  and orchestration-driver continuation prompts ("Almost done — you were
  interrupted again …"). Both extensions were forced by validation-sample
  leaks (items 20/23 and 14).
- **E2 language**: label in the language used (canticos sessions are often
  Spanish; Spanish correction markers belong in a per-language P extension).
- **E3 empty/whitespace/echo messages**: excluded.
- **E4 quoted markers**: a marker inside quotes or prose ("don't write 'no'…")
  is not an opening marker; only markers *addressed at the agent* at position 0.
- **E5 multi-line openers**: the marker must be in the first line, but the
  *label* reads the whole message.
- **E6 transcript-dump sessions** (added by validation): sessions whose "user"
  messages are speech-to-text dumps (`[00:00:12] (speaker N) …` — e.g. canticos
  interview pipelines) are not human→agent direction at all; the whole session
  is excluded from the denominator, not just the message.

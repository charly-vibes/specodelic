# Shift-left ledger — detector validation (specodelic-an79, acceptance item 2)

Date: 2026-11-18 · Companion to
[the labeling rubric](2026-11-18-shift-left-session-labeling-rubric.md) ·
Sample + labels archived under `sources/an79/` (seeded, reproducible).

## What was validated

The opening-marker correction detector (rubric §Detector definition) against a
30-message hand-labeled sample drawn from the five measured repos' session
stores. Hand labels follow the rubric (C1 behavioral / C2 other / N not-a-
correction / EXCLUDE wrapper-or-transcript); the detector is the regex-level
probe whose unvalidated first-pass numbers are recorded (uncited) in the
ticket.

## Sample design

- Repos: testaruda (79 sessions), tambor (99+6), wanna (66+2), canticos
  (28+2), coffee (54+2) — pi + omp stores.
- Per repo: 2 messages sampled from the marker-candidate pool (enrichment)
  + 4 seeded-random from the rest (`seed=79`; canticos has **zero**
  marker-candidates in 324 human messages — its 2 enrichment slots went
  unfilled and the total was topped up elsewhere, `seed=7907`).
- Human-message extraction excludes wrappers (E1); the sample then leaked
  two wrapper classes the first-pass filter missed — see findings.
- Total human user messages across the five stores: 1,393. Marker-candidate
  pool: 13 ≈ **0.9% base rate** — the enrichment design means precision
  numbers below are conditional on this mix, not field rates.

## Results (24 non-excluded items; 6 excluded per E1/E6)

| Framing | TP | FP | FN | P | R |
|---|---|---|---|---|---|
| vs C1 strict — C2-fires count as FP (study framing) | 3 | 4 | 1 | 0.43 | 0.75 |
| vs C1 two-class — C2 ignored | 3 | 1 | 1 | 0.75 | 0.75 |
| vs any correction (C1+C2) | 6 | 1 | 1 | 0.86 | 0.86 |

After the pattern fix forced by the FN (adding `ok the` to P):

| Framing | TP | FP | FN | P | R |
|---|---|---|---|---|---|
| vs C1 strict (study framing) | 4 | 4 | 0 | 0.50 | 1.00 |
| vs any correction | 7 | 1 | 0 | 0.88 | 1.00 |

The strict framing is the one that matters for the study: the detector feeds
a **C1 rate**, so process corrections (C2) it catches are contamination, not
credit — three of the four strict-FPs are real corrections of the wrong
class. The one pure FP is #11, a `wait`-opened *status question*; no
mechanical discriminator separates it from #9's `wait`-opened challenge
without reading the exchange.

## Findings (what validation was for)

1. **Wrapper leaks**: orchestrator subagent briefs (`# Subagent brief: …`,
   items 20/23) and orchestration-driver prompts ("Almost done — you were
   interrupted again…", item 14) pass a naive wrapper filter and land in the
   denominator as "user" messages. Rubric E1 extended. Spec-heavy repos run
   more orchestrators — this leak *deflates* spec-repo correction rates
   mechanically; it is a confounder in the first-pass direction.
2. **Transcript-dump sessions**: canticos "user" messages include speech-to-
   text interview dumps (`[00:00:12] (speaker N)…`) — not human→agent
   direction at all (rubric E6, whole-session exclusion).
3. **The FN class**: `ok`-opened defect reports on just-produced output
   ("ok the screenshots are all scrambled") are C1 and were missed; `ok` is
   ambiguous (continuation vs lead-in to a complaint). `ok the` added; a
   bigger sample will likely surface more `ok`-class variants.
4. **C2 inflation**: `wait`-opened *process* corrections (wrong repo for
   tickets, session-naming practice, wrong fix target) fire the detector but
   are outside the study's behavioral scope. The study rate needs the C1/C2
   split, which is exactly the part that stays human-labeled.
5. **canticos zero-marker result replicated structurally**: 0 marker
   candidates in 324 messages is consistent with the crude 0/320 — but the
   crude denominator included transcripts and wrappers, so the honest rate
   needs the re-run with E1/E6 filtering before anyone cites either number.

## Threats to validity

- **Single labeler**: labels were assigned by the agent (this session) from
  the rubric; the labeled table below is the audit surface — owner review
  invited, disagreements are rubric amendments.
- **Small denominator**: 24 effective items; recall estimates rest on 4 C1
  positives. This validates the *detector*, not the rates.
- **Enrichment**: precision is conditional on the 2-in-6 marker-enriched mix.
- **Language**: the sample drew no Spanish correction instances (canticos
  draws were transcripts); E2's per-language marker extension is still
  unvalidated.

## Verdict + next step

The opening-marker probe is usable **as a high-recall screening pass only**:
after the fix it catches all C1 positives in the sample (R = 1.00) but at
strict precision 0.50 — half of what it flags is a question or an
out-of-scope correction class. Per the ticket's design point 5, every rate
that enters the ledger must come from **detector-fire → human label → C1
count**, never raw fire counts. Next step (remaining acceptance items): fix
the pattern set as validated, re-run across all repos with E1/E6 filtering
to produce honest denominators, then build the repo × era ledger.

## Labeled sample (the audit surface)

Generated mechanically from `sources/an79/sample.json` + `labels.json`;
`detected` = opening-marker probe on the sampled text; full message text and
exchange context are in the archived sample.

| # | repo | message (first ~70 chars) | label | detected | rationale |
|---|---|---|---|---|---|
| 0 | tambor | `are the changes being done ok? seems like a lot of time` | N | no | pace question, no negation of agent output |
| 1 | tambor | `ok` | N | no | acknowledgment after explanation |
| 2 | testaruda | `continue` | N | no | continuation |
| 3 | tambor | `.` | N | no | noop continuation |
| 4 | testaruda | `continue` | N | no | continuation |
| 5 | testaruda | `continue` | N | no | continuation |
| 6 | canticos | `[00:00:12] (speaker 0) trabajando, que me cuentes un poco tu día a día` | EXCLUDE (E6 transcript-dump session) | no | speech-to-text interview, not human-to-agent direction |
| 7 | coffee | `it should be built in the ghpages from the cta` | C1 | yes | corrects build/serve behavior of dashboard (should be ghpages from cta), mid-loop |
| 8 | tambor | `no, make just test the most obvious one` | C1 | yes | rejects test recipe choice, redirects to obvious target |
| 9 | wanna | `wait why are you constructing this code???` | C1 | yes | challenges unrequested code construction; agent concedes 'I leapt' |
| 10 | canticos | `.` | N | no | noop continuation |
| 11 | wanna | `wait so the code is done? how does it work? is all the specs covered i` | N | yes | status/verification question, corrects nothing |
| 12 | testaruda | `wait should all be in espectacular??` | C2 | yes | process correction: tickets filed in wrong repo; agent agrees |
| 13 | canticos | `[00:00:01] (speaker 1) Todo bien. Acá, agarrando unas cositas. [00:00:` | EXCLUDE (E6 transcript-dump session) | no | interview transcript |
| 14 | wanna | `Almost done — you were interrupted again after committing a930631. Rem` | EXCLUDE (E1 orchestration-driver prompt) | no | harness continuation prompt, not human typing |
| 15 | tambor | `wait you should always use named sesions so we can have the trace see ` | C2 | yes | process correction: session naming practice |
| 16 | coffee | `yes` | N | no | confirmation |
| 17 | testaruda | `the gap-filling ones` | N | no | task selection fragment |
| 18 | tambor | `is it running the tests through testaruda?` | N | no | question |
| 19 | coffee | `ok let continue` | N | no | continuation |
| 20 | wanna | `# Subagent brief: wanna-uvw — Implement recovery-contract You are a pi` | EXCLUDE (E1 subagent brief) | no | orchestrator-injected brief |
| 21 | coffee | `can we create a cronjob that runs the analysis everynight and push it ` | N | no | new feature request |
| 22 | coffee | `now orchestrate the implementation` | N | no | task instruction (orchestrate) |
| 23 | wanna | `# Subagent brief: wanna-mwy — Implement evaluation-telemetry You are a` | EXCLUDE (E1 subagent brief) | no | orchestrator-injected brief |
| 24 | coffee | `Sesión de verificación v2-budget-variance. Ticket: bd show coffe-bz9 (` | N | no | session-opening task brief |
| 25 | canticos | `[00:29:04] (speaker 1) para que modifiquen el código alimentario eso e` | EXCLUDE (E6 transcript-dump session) | no | interview transcript |
| 26 | testaruda | `wait, you need to fix the testaruda specs not the evaluation ticket` | C2 | yes | wrong-target correction (fix specs not evaluation ticket); process class |
| 27 | wanna | `ok orchestrate the implementation` | N | no | task instruction (orchestrate) |
| 28 | testaruda | `ok, can we run pretender check and ah check and start putting stricter` | N | no | new request: stricter gates |
| 29 | tambor | `ok the screnshots are all scrambled` | C1 | no | defect in just-produced screenshots; agent output correction |

## Full re-run: honest rates (detector-fire → human label → C1 count)

Re-run 2026-11-18 (same session, design point 5) with the validated pattern set,
E1/E6 filtering, both stores, across the five measured repos. **All 15 fires
hand-labeled** — the complete loop, no sampling. Labels archived with evidence
(`sources/an79/fire_labels.json`); summary in `rerun_summary.json`.

| repo | human msgs (E1/E6-filtered) | C1 events | C1 rate |
|---|---|---|---|
| testaruda | 567 | 2 | 0.35% |
| tambor | 222 | 2 | 0.9% |
| wanna | 106 | 1 | 0.94% |
| canticos | 59 | 0 | 0.0% |
| coffee | 141 | 1 | 0.71% |

Class split of the 15 fires: **6 C1, 5 C2, 4 N**.

### What the honest numbers change vs the crude first pass

- **coffee (plain control) is no longer 0**: one C1 (0.71%). The crude pass
  reported 0/139 — its denominator missed fires, and the marker pool it scanned
  was wrapper-filtered differently.
- **canticos's denominator was 82% non-human**: 320 crude → 59 human messages
  (transcript-dump sessions + wrappers). Its 0% now has a real denominator.
- **testaruda drops**: 1.35% crude → 0.35% C1 — three of its fires are C2
  (process corrections), one is an informational `ok the` confirmation.
- **Direction**: spec repos 0.35–0.94% vs plain 0–0.71% — still weakly higher,
  but on **six C1 events total across 1,095 messages**. That is not a citable
  contrast; it is one ledger signal (correction rate) at noise scale.

### Detector defect found by the re-run

`no` raw-prefix-matches `now` (fire 14: "now orchestrate the implementation").
Pattern matching needs word boundaries (`\b`-anchored), not `startswith`.
Screening role absorbed it (hand-labeled N), but the fix belongs in the
pattern spec before any future re-run.

### Standing verdict (acceptance item 4, pending the remaining signals)

On the correction-rate axis alone: **inconclusive** — all repos sit under 1%,
differences rest on 6 events, and the ledger's other two signals (decision-doc
amendments, cycle proxies) plus the pre/post-era split are not yet computed.
No priority call on check/re29 should move on this evidence yet.

### Fire-level labeled table (audit surface)

| # | repo | label | rationale |
|---|---|---|---|
| 0 | testaruda | C2 | wrong-target redirect (fix specs not evaluation ticket) |
| 1 | testaruda | C1 | corrects agent's confusion about what the evaluation document is (artifact semantics) |
| 2 | testaruda | C2 | ticket placement process (should all be in espectacular?) |
| 3 | testaruda | C1 | synthetic fixtures vs real cloned repos — behavioral validation gap; agent reworks backend |
| 4 | testaruda | C2 | repo-boundary overstep (editing testimonial, should ticket it) |
| 5 | testaruda | N | informational confirmation (ok the latest version is there) |
| 6 | tambor | C2 | session-naming process practice |
| 7 | tambor | N | pacing instruction (stop after this ticket) |
| 8 | tambor | C2 | focus redirect (look at this repo, not that one) |
| 9 | tambor | C1 | defect in just-produced screenshots (rendering broken) |
| 10 | tambor | C1 | rejects test-recipe choice — build/test semantics |
| 11 | wanna | C1 | challenges unrequested code construction; agent concedes the leap |
| 12 | wanna | N | status/verification question |
| 13 | coffee | C1 | build/serve behavior of dashboard should differ |
| 14 | coffee | N | task instruction; fired because 'no' prefix-matches 'now' — DETECTOR BUG, needs word-boundary match |

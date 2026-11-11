# Graded evidence — the audit verb

The pipeline's own verbs grade a spec against itself: `lint` checks shape,
`model-check` explores the declared model, `verify` runs the opted-in
claims. But the toolchain also *touches* evidence it never re-validates:
recorded example corpora checked once by `conform`, pack-checker results
over possibly-empty checked sets, fragments and claims that may cite no
declared model element at all, and the audit reports of previous runs.
`spk audit` is the one verb that computes the graded-evidence discipline
uniformly over all of it. The failure modes it exists for are all one
shape — *evidence whose scope silently widened*: stale examples read as
green, an empty checked-set read as a pass, a vacuous fragment read as
verified.

audit is read-only like conform: it never compiles, never writes
artifacts, never advances a lifecycle transition. It consumes, classifies,
and reports — and refuses ungated inputs with a remediation hint.

## Usage

```console
$ spk audit <spec-file> --oracle <corpus.jsonl> [--before <report.json>]
            [--declare-expectation] [--closed-world] [--out-dir <dir>]
```

The oracle corpus is JSONL — one recorded example per line, the same
shape conform consumes: `{id, setup?, trace: [{action, observations?}]}`.
Unknown fields, malformed lines, and duplicate ids are refused with a
remediation hint, never ignored. The gate is conform's, unchanged: the
file must be lint-clean with current compiled artifacts in `--out-dir`
(audit never re-compiles).

## The closed class vocabulary

Every evidence row receives exactly one audit class from a closed set.
The set is audit-local and closed: a row's class is data you can count,
never prose you must interpret.

| Class | Meaning |
|-------|---------|
| `current` | the row's evidence agrees with the current model at least as strongly as in the prior report (or there was no prior report and the walk classified it) |
| `stale` | the row's class dropped or worsened since the prior report — the prior good verdict no longer holds against this revision |
| `unchecked` | the row was never truly checkable (underspecified, prose-only, unsupported evaluator, or agree) — absence of a passing verdict, never a pass |
| `contradicted` | the current model positively contradicts the recorded example |
| `binds_nothing` | the row's evidence unit (a fragment or claim) cites no declared model element — green yet vacuous |
| `new` | the current run graded a row the prior report never saw |

**The both-layer record.** Every record carries *both* layers: the audit
class and the underlying conform verdict with its reason. audit rides
conform's verdict taxonomy — `{permitted, forbidden, underspecified,
unknown, unsupported}` — it never re-derives or reinterprets it; the
audit class is the evidence-lifecycle layer on top. In the report a row
looks like this (from a real run):

```text
reserve-holds: current (permitted) — permitted: trace through `confirm`
  explainable by the declared Model's transition relation
overbook-refused: current (underspecified) — uncovered_trace_never_forbidden:
  step 1 observation `overbooked` names no declared Model state or effect
wrong-obs: contradicted (forbidden) — contradiction_forbidden_any_mode:
  step 1 observed state `open` but transition `confirm` declares target `held`
```

## Honesty channels — countable, never a pass, never a failure

`unchecked` and `binds_nothing` ride the warnings channel. They are
countable in the summary, they never count as a pass, and they never
affect the exit code:

- **Empty checked-set is unchecked.** A pack checker that activates
  vocabulary but produces an empty checked-set yields `unchecked` rows —
  countable in the summary, never a silent pass.
- **Unbound evidence is flagged; sparsity is respected.** A fragment or
  claim citing no declared model element yields `binds_nothing` — but a
  legitimately sparse phase-model whose declared elements are simply not
  exercised stays unflagged. The check is citation-vs-declared-ids,
  never coverage-of-declared-ids.
- **Unchecked never fails the run.** A report where every row is
  `unchecked` or `binds_nothing` still exits 0, with counts surfaced in
  both output views.

The summary and one flagged row from a real run (five records total —
the other three are the oracle rows shown in the previous section):

```text
audit: 5 record(s) — binds_nothing 2, current 1, unchecked 2
  evidence_scope: agreement on the supplied corpus; not a proof of behavioral equality
  ladder_rung: 2 — well-modeled: the Model section is present (states + transitions declared); no current model-check run
  bind:locked: binds_nothing (agree) — fragment `locked` cites no declared model element —
    a vacuous binding (green yet vacuous): nothing was checked; warnings-channel advisory,
    never a failure, and a model whose declared elements are simply not exercised is never flagged
```

The fixed `evidence_scope` statement rides every report in both views —
it is data, not documentation, so no reformatting can strip it.

## Evidence diff across reports

`spk audit --before <report.json>` compares per-row verdict classes with
a prior report: same class ⇒ `current`, dropped or worsened ⇒ `stale`,
unseen by the prior ⇒ `new`. This makes revision preservation a
by-product — re-running the audit after a spec edit classifies exactly
which evidence went stale.

Both reports carry a `scope_sha256` digest of the oracle corpus. A
mismatch is a labeled refusal with zero classification emitted — audit
never silently diffs evidence across corpus versions:

```text
before_diff: scope_sha256 mismatch between the reports (prior 60071a…, current 929123…)
  — the oracle corpus changed between the runs and per-row comparison would silently
  diff different evidence; never silently diff across corpus versions
  (re-run the baseline against the same corpus: spk audit <file> --oracle <corpus.jsonl> --declare-expectation)
```

`--declare-expectation` records a report as a declared expectation
baseline: when it is later used with `--before`, `stale` rows fail the
run — otherwise staleness is informational.

## The ladder-rung aggregate

The summary reports, per file, the highest
[verification-ladder](verification-boundaries.md) rung for which
`current` evidence exists — derived at report time from evidence actually
present, never stored, never exceeding the evidence. The rung is stated
with its basis, so the number is never an undifferentiated PASS:

- A file lint-clean but only model-checked, with no verify pass ⇒ **rung
  3**, never 4 or 5.
- No audit, conform, or verify evidence at all ⇒ **rung 1** (lint-clean
  at most), stated as such — the derivation's floor; through the CLI
  this state is unreachable (see the second limitation below).
- `unchecked` and `binds_nothing` rows are countable, never verified —
  they cannot lift the rung, by construction.
- A current `counterexample_found` model-check run is the ladder's
  better-design rung ⇒ **rung 5** — the refuted claim names the design
  lesson.

For the rung-by-rung exposition — what each rung's gate assures and what
it does not — see [verification boundaries](verification-boundaries.md);
this page treats the ladder as the audit's output vocabulary, not as a
tutorial.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | no `contradicted` row (and no `stale` row under a declared expectation) — including runs that are entirely `unchecked`/`binds_nothing` |
| `1` | any `contradicted` row; or any `stale` row when the `--before` baseline was recorded with `--declare-expectation` |
| `2` | invocation error or gate refusal (lint-dirty file, stale artifacts, digest mismatch, zero records — an audit over no evidence is never a pass) |

## Known limitations

Each limitation below cites a runnable command; run it yourself — the
outputs shown were captured at this HEAD. The commands use the demo
spec and oracle corpus from [the example at the end of this page](#a-runnable-example-end-to-end).

### Rung 4 is unreachable through `spk audit` in v1

`spk verify` persists no report — its verdict is re-evaluated live every
run ("verified is not cached" is verify's own hint), so the properties
blocks' pass results exist nowhere on disk for the audit to read. The
run's evidence collector never sets the verify input; a rung without its
backing evidence is impossible, so the aggregate never claims rung 4.
Run the sequence yourself:

```console
$ spk compile demo.audit.md --out-dir out
$ spk model-check demo.audit.md --out-dir out --json
$ spk verify demo.audit.md --out-dir out        # reports status: verified
$ spk audit demo.audit.md --oracle oracle.jsonl --out-dir out --human
```

verify says `verified`; the audit summary immediately after still says:

```text
  ladder_rung: 3 — checked-within-bounds: current model-check run reported no_counterexample
```

and `ls out/` shows no verify report — only the compile, model-check,
and proptest artifacts. A future persisted verify report is one
collector branch away, not a redesign; until then rung 3 is the honest
ceiling through this verb.

### The rung-1 floor is CLI-unreachable

Rung 1 (lint-clean at most) exists in the derivation core, but no CLI
run can land on it: the audit gate is conform's, and its lint-clean
requirement implies the Model section is present (`linter.model_present`
is one of the gating checkers), which is rung 2's own evidence. Try to
audit a model-less file and the gate refuses before any rung is derived:

```console
$ spk lint demo.nomodel.md        # fires linter.model_present: "Model section incomplete"
$ spk audit demo.nomodel.md --oracle oracle.jsonl --out-dir out
{"ok":false,"envelope_kind":"error", ... "artifact_gate: the file has lint findings —
 conform requires a lint-clean spec, verdicts are never derived from invalid artifacts ..."}
```

So through the CLI the floor is rung 2; rung 1 is pinned where it is
honest — at the pure derivation core over an empty evidence mix, stated
as "at most", never a claim.

## A runnable example, end to end

The demo spec (a two-state reservation model with one executable
invariant), its oracle corpus, and every command, in order:

```markdown
---
id: demo.audit
kind: intent
statement: "WHEN a reservation is requested THE demo spec SHALL hold it
only while confirmed"
---

## Constraints

| id | kind | expr | traces_to |
|---|---|---|---|
| locked | invariant | `**rust:** state != "overbooked"` | [[demo.audit]] |

## Model

### States
- `open`
- `held`

### Transitions

| id | from | to | guard |
|---|---|---|---|
| confirm | open | held | [[demo.audit.locked]] |

## Properties

| id | kind | derives_from | generator | predicate |
|---|---|---|---|---|
| p | unit | [[demo.audit.locked]] | `g()` | `**rust:** true` |
```

```jsonl
{"id":"reserve-holds","setup":{"state":"open"},"trace":[{"action":"confirm","observations":{"state":"held"}}]}
{"id":"overbook-refused","setup":{"state":"open"},"trace":[{"action":"confirm","observations":{"state":"overbooked"}}]}
{"id":"undeclared-action","setup":{"state":"held"},"trace":[{"action":"reopen"}]}
```

```console
$ spk compile demo.audit.md --out-dir out          # the gate needs current artifacts
$ spk audit demo.audit.md --oracle oracle.jsonl --out-dir out --json > before.json
$ spk audit demo.audit.md --oracle oracle.jsonl --out-dir out --before before.json --human
```

The first run classifies: the happy trace `current` (permitted), the
uncovered observation and undeclared action `unchecked`
(`underspecified` — spec gaps, not refutations), the two vacuous
fragments `binds_nothing` on the warnings channel — exit 0. The second
run diffs against the baseline: every class held, so every row stays
`current` — and if you now edit the model so an old example no longer
holds, re-running flags exactly that row `stale`.

conform's verdict taxonomy, the walk machinery audit reuses, and the
lifecycle audit deliberately stays outside of are on the
[conform page](conform.md); what each ladder rung assures is on
[verification boundaries](verification-boundaries.md); the class names
and jargon are in the [glossary](glossary.md).

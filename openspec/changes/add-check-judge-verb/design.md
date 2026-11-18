# Design — `check` as the single judge verb

## Context

D-10: *"Exact contract-verdict mapping when `check` consumes the audit
vocabulary. No fourth vocabulary; write the mapping when the fusion change is
proposed."* This change is that proposal, so this document is D-10's answer.

Grounding facts (verified at HEAD `87f0e6d1`):

| Fact | Where |
|---|---|
| `Verdict` closed set: `permitted`, `forbidden`, `underspecified`, `unknown`, `unsupported`, `agree` | `src/conform.rs:48-72` |
| `AuditClass` closed set: `current`, `stale`, `unchecked`, `contradicted`, `binds_nothing`, `new`; `AuditClass::of` is the only verdict→class derivation | `src/audit.rs:70-150` |
| One both-gates evaluation path shared by `verify` and orchestrate | `verify::evaluate_file_verdicts` (`src/verify.rs:300-330`) |
| A properties runner timeout is *"no verdict at all"*, deliberately distinct from `Failed` | `PropsGateState::TimedOut` (`src/verify.rs:130-138`) |
| The ladder derivation accepts `verify_pass`; the IO collector never sets it, so rung 4 is unreachable through `audit` | `src/audit/rung.rs` `EvidenceMix`; graded-evidence.md §Known limitations |
| `parse_prior_report` accepts the report object (schema-versioned, rows sorted) | `src/audit.rs:641+` |
| `spk audit` already exits 0/1/2 with contradict/stale/refusal semantics | `audit::exit_code` (`src/audit.rs:756`) |

## D1 — One verb, one spec, existing stages

Surface: `spk check <spec-file> [--oracle <corpus.jsonl>] [--before <report>]
[--declare-expectation] [--closed-world] [--structural-only] [--out-dir <dir>]
[--timeout-secs <n>]` — audit's shape plus verify's bound.

One spec per invocation. A fused report's row layer and `scope_sha256` are
per-file; multi-spec invocations would need per-file digests and would silently
redefine audit's diff. Corpus-wide judging already has a home (`spk orchestrate`,
the `just` gates).

Pipeline (in-process, no subprocess chaining):

1. **Gate** — conform's/audit's gate, unchanged: the file must be lint-clean
   with current compiled artifacts in `--out-dir`. `check` never compiles.
2. **Verify stage** — `verify::evaluate_file_verdicts` over the single file as
   its own corpus with the real `PropertiesRunner`, bounded by
   `--timeout-secs`. The model gate reads the persisted `<stem>.check.json` and
   recomputes the required-claim set, exactly as `spk verify` does.
3. **Conform stage** — only when `--oracle` is supplied: conform's walk over the
   corpus, unchanged.
4. **Audit stage** — row assembly (`verify:*`, oracle rows, `pack:*`,
   `bind:*`), `AuditClass::of` classification, `--before` diff, rung derivation.

Rejected: chaining `spk verify && spk conform && spk audit` as subprocesses —
three exits cannot compose into one verdict, three report families cannot merge
rows, and the rung input (verify's live verdict) never crosses the process
boundary.

Rejected: multi-file `check` in v1 — a decision that would need an oracle-per-
file protocol and a new scope digest definition, neither of which this change
owns.

## D2 — "Existing evidence" means no new producer

The decision's "no runner, no adapter registry" is scope B's boundary: no
consumer contract suite is executed, no adapter is instantiated, no sidecar
binding is read, no corpus is demoted. The one execution `check` performs is
verify's own compiled property blocks, through verify's existing bounded
runner — machinery that already exists, with a bound that already exists.

Rejected: consuming only persisted artifacts. `verify` persists nothing, so
that variant is `audit` plus a rung-3 ceiling — an alias, not the judge verb
G13/G14 name.

The runner's termination semantics carry over unchanged: a run killed at the
bound produces **no verdict** (`PropsGateState::TimedOut`), so `check` records
a labeled termination and refuses (exit 2). It never substitutes a verdict for
a run that did not finish.

## D3 — One evidence vocabulary (D-10's answer)

No new closed set. Every row carries **one `Verdict` spelling** (the existing
six) and **one `AuditClass`** (the existing six). Rows from producers other
than conform map onto the existing verdict spellings by the table below; the
class is then derived by `AuditClass::of` exactly as for oracle rows, except
`binds_nothing`, which stays citation-derived and not verdict-derivable. The
capability spec encodes this matrix as one scenario per row — `openspec
validate --strict` rejects requirement bodies over 500 characters, and the
scenarios are the checkable form of the same content.

Why these mappings are honest:

- A failing compiled property block **is** a positive contradiction with a
  declared executable claim — `forbidden` is that verdict's definition, not a
  borrowed label.
- A clean model run with every required claim verified asserts nothing is
  contradicted within the stated bound — `permitted` is the closest existing
  spelling, and the row's reason carries the bound.
- `exploration_only` and an unresolved required claim are **knowledge
  absence** — `unknown`, already defined as an advisory-tier record that never
  gates. Mapping them to `permitted` would claim a check that did not happen.
- `inconclusive` is never emitted. It stays reserved: T1 established that
  absence and abnormal termination are *recording* classes (ISO/IEC 9646-1
  6.5.1.4–6.5.1.6), never a third verdict result; a producer would have to be a
  completed run that executes and still cannot decide.

### The normative producer-outcome mapping

| Producer outcome | Row key | Verdict | Audit class | Exit |
|---|---|---|---|---|
| property block passed | `verify:prop:<block-id>` | `permitted` | `current` | 0 |
| property block failed (shrunk counterexample reported) | `verify:prop:<block-id>` | `forbidden` | `contradicted` | 1 |
| model report `no_counterexample`, every required claim `verified` | `verify:model` | `permitted` | `current` | 0 |
| model report `counterexample_found` | `verify:model:<claim-id>` | `forbidden` | `contradicted` | 1 |
| model report: required claim `refuted` | `verify:model:<claim-id>` | `forbidden` | `contradicted` | 1 |
| model report: required claim `unknown` | `verify:model:<claim-id>` | `unknown` | `unchecked` | 0 |
| model report `exploration_only` (zero predicates executed) | `verify:model` | `unknown` | `unchecked` | 0 |
| model report `timed_out` (completed, did not finish) | `verify:model` | `unknown` | `unchecked` | 0 |
| oracle trace explainable by the declared model and executable claims | corpus id | `permitted` | `current` | 0 |
| oracle trace contradicts an executable claim | corpus id | `forbidden` | `contradicted` | 1 |
| oracle trace `underspecified` / `unknown` / `unsupported` / `agree` | corpus id | unchanged | `unchecked` | 0 |
| pack checker produced violations | `pack:<pack>:<subject>` | `forbidden` | `contradicted` | 1 |
| pack checker empty checked-set | `pack:<pack>:<subject>` | `agree` (carrier) | `unchecked` | 0 |
| evidence unit cites no declared model element | `bind:<owner-id>` | `agree` (carrier) | `binds_nothing` | 0 |
| row worsened since the prior report | unchanged | unchanged | `stale` | 0 (1 under a declared expectation) |
| row absent from the prior report | unchanged | unchanged | `new` | 0 |

Rows that *improved* since the prior report (prior `contradicted`, now
`agree`, or prior `stale`, now current) keep their verdict-derived class
unchanged: v1 mints no improvement marker, and the only trace that a row was
ever broken is the diff against the user-persisted prior report. That is a
named limitation, not an accident — see D8 and task 4.2.

**Non-judgements — no row, no verdict, exit 2:**

| Condition | Why it is not a judgement |
|---|---|
| lint findings, stale/missing compiled artifacts | conform's gate refuses before any evidence exists |
| `*_props.rs` stale, missing, or uncompilable | nothing executed; absence is not a verdict |
| properties runner unavailable or killed at the bound | abnormal termination (T1 recording class), not a verdict |
| `<stem>.check.json` missing, stale, unparseable, or scope-mismatched | no current model evidence; a stored report is never rewritten to manufacture it |
| oracle corpus parse failure; `--before` digest or schema mismatch | the evidence set identity is unknown or differs |
| zero evidence rows | an evidence-less pass is impossible; a run over no evidence is a refusal |

The evidence-axes table in `docs/src/graded-evidence.md` keeps its 13
implemented + 3 reserved values untouched; only the `inconclusive` row's
trigger text is corrected (see D8), because this change narrows rather than
widens the producer question.

## D4 — One exit contract

| Code | Meaning |
|---|---|
| 0 | a judgement was emitted and no `contradicted` row exists (and no `stale` row under a declared expectation) — including runs where every row is `unchecked` or `binds_nothing` |
| 1 | a judgement was emitted and ≥1 `contradicted` row exists, or a `stale` row exists under a declared expectation |
| 2 | **no judgement was emitted** — gate refusal, unavailable or terminated evidence, digest/schema refusal, zero rows, invocation error |

Rules stated as invariants, not conventions:

- Exit 2 never reads as a pass and never as a contradiction; it is the absence
  of a verdict, labeled with its cause and a remediation hint.
- `unchecked`/`binds_nothing` are countable, never a pass, never a failure.
- The class-to-exit mapping is a pure function of the merged report plus the
  prior declaration — one function, both output views.
- **The contradicted set is classified before the `--before` overlay.** The
  overlay relabels same-rank rows `current`, so a contradiction that persists
  across two reports would otherwise vanish from the exit predicate. `check`
  carries the pre-overlay fact into the exit; the report still shows the
  overlay's currency classes (one report family). `spk audit` still has that
  hole in diff mode — filed as specodelic-f1by, not silently changed here.
- `verify`, `conform`, and `audit` keep their own exit codes and status
  strings. `check` changes none of them; it routes their refusals to 2 and
  their judgements to 0/1 by the mapping above.

## D5 — The report is an audit report

The check report carries exactly `AuditReport`'s fields and
`report_schema_version` (1): `evidence_scope`, `scope_sha256`,
`expectation_declared`, `verdict_counts`, `class_counts`, `ladder_rung`,
`records` (sorted by `scenario_id`). Consequences:

- `spk audit --before <check-report>` and `spk check --before <audit-report>`
  both work through the existing `parse_prior_report` + diff, because the row
  layer and digest discipline are identical.
- No new persisted artifact and no `check`-named file: `<stem>.check.json`
  remains the *model-check* run report. `check` writes its report to stdout
  only; a baseline is recorded by redirecting the JSON envelope, exactly as
  with `audit`.
- Re-running over identical inputs yields a byte-identical report (sorted
  rows, no timestamps, no iteration-order leakage).

No-oracle runs: `scope_sha256` is the digest of the empty evidence corpus (a
well-defined constant), and `evidence_scope` states that only the spec's own
evidence was consumed — the honesty statement is never weaker than audit's
fixed sentence.

Rows from the verify stage are keyed `verify:...` and their reasons lead with
the producer and the gate they rest on, so a reader never mistakes a property
row for an oracle trace.

Rejected: a `CheckReport` type with a stage block (a second report family and a
forked diff); a schema-version bump (the row layer did not change).

## D6 — Rung derivation

`rung_from_evidence` already defines rung 4 as a verify pass and documents that
no IO collector sets `verify_pass`. `check` sets it iff this run's verify
verdict is `verified` (both gates green, required claims verified). Therefore:

- a fully green run reports rung 4 with the stated basis;
- a failed/refused verify stage never lifts the rung (a refusal emits no rows
  and no judgement at all);
- `exploration_only`/`unknown` rows cannot lift it, by the derivation's own
  honesty intersection;
- rung 5 (`counterexample_found`) still outranks a verify pass — a refuted
  claim is the design lesson, and `check` does not override the ladder.

`spk audit` keeps its documented ceiling (rung 3) until a verify report is
persisted; that is verify's change to make, not this one.

## D7 — One failure class, one labeled error

Every refusal is the same class: *evidence unavailable*. One failure terminal
state (`evidence_failed`) emits one effect Constraint,
`check.evidence_unavailable(detail)`, satisfying the published error contract
rows (`errors.envelope_error_kind`, `errors.exit_code_mapping`,
`errors.remediation_hint_present`). Splitting gate refusals from artifact
refusals from termination would be modeling the same decision ("we cannot
judge") three times; the `detail` names the cause and the hint names the rerun.

Model: `requested → collected → judged → reported`, with
`requested → evidence_failed` under the negation of the collection guard
(gate admitted ∧ stages reused ∧ execution bounded ∧ evidence present ∧ no
foreign authority).

## D8 — Scope boundary and non-goals

- **No scope-B machinery:** no contract-suite execution, no adapter registry, no
  sidecar bindings, no corpus demotion, no self-hosting inversion. B's entry
  condition (a real consumer or a measured benefit, plus a preservation
  harness) is untouched.
- **No independence claim:** the report says drift is visible and graded; it
  never claims independent evidence for agent-authored corpora (G2/G4/G15
  amendments).
- **No `--draft`:** G13's instant structural verdict needs a no-toolchain
  evidence path that does not exist; it would be a different producer, not a
  flag.
- **No verify persistence:** `check` reads the live verdict it produced; it
  writes nothing for `audit`.
- **No new vocabulary:** `inconclusive`, `origin`, `binding_kind` stay reserved;
  the reserved-row triggers are corrected only where this change narrows them.
- **No improvement attribution:** a row that was `contradicted` in the prior
  report and is `agree` now keeps its derived class; the report mints no
  marker for fixed-since-last-run (no new class value, no schema-v1 field).
  Improvement is visible only by diffing the persisted prior report; a
  dedicated improvement class is the first candidate for a future
  `report_schema_version` 2. (Method lesson, antithesis-skills inventory
  2026-11-18: a sweep that cannot distinguish "always agreed" from "agreed
  after a fix" invites circular re-keying — recorded as a named limitation,
  never silently accepted.)

## Risks

| Risk | Mitigation |
|---|---|
| Row-level `permitted` read as "the whole model is correct" | rows are per producer unit and carry the bound/gate in the reason; the rung states its evidence basis; docs state the claim of record (drift visible and graded, not proven absent) |
| `check` becomes a de-facto contract runner later | the capability spec's scope-boundary requirement plus a unit property asserting no adapter/registry path exists |
| Report interop breaks if audit's schema changes | check pins `AUDIT_SCHEMA_VERSION` and a test asserts `parse_prior_report` accepts a check report; a future schema change must handle both producers explicitly |
| Runner cost/failure modes surprise users | the bound is verify's (`--timeout-secs`, default 600s), and every runner failure is a labeled exit-2 refusal with the rerun hint |
| Name collision with `<stem>.check.json` | docs name the distinction explicitly; `check` writes no file |

## Verification plan

- Matrix tests: one case per mapping-table row plus every refusal row
  (expected record set and exit code).
- Interop test: check report accepted as an audit `--before` prior; row-for-row
  diff equality; byte-identical re-run.
- Ladder test: verified run ⇒ rung 4 with basis; failed properties ⇒ no lift.
- `openspec validate add-check-judge-verb --strict`, `just lint-deltas`,
  `just ci`.
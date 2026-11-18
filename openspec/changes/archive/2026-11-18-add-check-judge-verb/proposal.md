# Change: `check` — the single judge verb (verify + conform + audit over existing evidence)

> **Scope:** authorized by
> [`openspec/decisions/2026-11-18-scope-a-specification-tool.md`](../../decisions/2026-11-18-scope-a-specification-tool.md)
> §4 (brief §5.1.2; ticket specodelic-8hjn). Scope A only — no contract
> runner, no adapter registry, no corpus demotion, no independence claim.

## Why

The tool has three judging verbs that overlap in audience but not in shape:
`verify` reads the spec's own gates, `conform` reads an external oracle, and
`audit` re-validates recorded evidence and grades it. Each has its own report
family, its own exit predicate, and one honest gap: `verify` persists nothing,
so `audit` can never see a verify pass and the ladder's rung 4 ("matches-oracle")
is unreachable through the audit verb (documented in
`docs/src/graded-evidence.md` §Known limitations).

G14 already names the target shape — *three verbs: `check` (judge), `graph`
(relate), `doctor` (advise) — fuse, don't accrete* — and G13 names `check`
as the agent-loop verb. D-10 left the open question this change answers: **the
exact verdict mapping when `check` consumes the audit vocabulary, with no
fourth vocabulary.**

The fusion is possible without new machinery because the machinery already
exists and is already layered: `verify::evaluate_file_verdicts` produces the
both-gates outcome, conform's walk produces per-trace verdicts, `AuditClass::of`
maps those verdicts to lifecycle classes, the rung derivation accepts a
`verify_pass` input the IO collector never sets, and `parse_prior_report`
already diffs reports by row key. `check` composes these; it does not replace
them.

## What Changes

- **New verb `spk check <spec> [--oracle <corpus.jsonl>] [--before <report>]
  [--declare-expectation] [--closed-world] [--structural-only] [--out-dir <dir>]
  [--timeout-secs <n>]`** — one invocation runs conform's artifact gate,
  verify's both-gates evaluation (compiled property blocks executed through
  verify's existing bounded runner; model gate read from the persisted
  `<stem>.check.json`), conform's walk when an oracle corpus is supplied, and
  audit's classification/fold/rung/diff.
- **One evidence vocabulary** — every row carries exactly one conform verdict
  spelling and exactly one audit class from the existing closed sets
  (`Verdict` × `AuditClass`). No fourth vocabulary, no new enum value,
  `inconclusive` never emitted (it stays reserved with no producer). D-10 is
  answered by a normative producer-outcome mapping table in the `check`
  capability spec.
- **One exit contract** — audit's 0/1/2, with the explicit rule that exit 2
  means *no judgement was emitted* (unavailable or terminated evidence, gate
  refusal, digest/schema refusal, zero rows) and never reads as a pass or a
  contradiction.
- **Report interoperability** — the check report *is* an audit report (same
  schema version, same fields, same row keys), so `spk audit --before` accepts
  a check report and `spk check --before` accepts an audit report. No new
  persisted artifact, no check-local schema, byte-identical re-runs.
- **Rung 4 becomes reachable** — the rung derivation receives this run's verify
  verdict (`EvidenceMix.verify_pass`, the input the collector documents but
  never sets), so a fully verified run reports rung 4 and nothing claims a rung
  without its evidence.
- **Docs** — a `spk check` section in `docs/src/commands.md`; mapping and exit
  contract pointers in `docs/src/graded-evidence.md`; the ladder's rung-4 note
  in `docs/src/verification-boundaries.md`; goals-doc §2.2 status for G3/G13/G14.

## Impact

- **Affected specs:** `check` (new capability, ADDED requirements). No MODIFIED
  deltas — `verify`, `conform`, and `audit` keep their behavior, schemas, status
  strings, and exit codes; `check` is a composition over them.
- **Affected code:** `src/commands/check.rs` (new, orchestration only),
  `src/main.rs` (one subcommand), `src/audit/rung.rs`'s collector call site
  (pass the live verify verdict), CLI/docs tests.
- **Not affected:** `Verdict`, `AuditClass`, `AuditReport`, the evidence-axes
  table's value set (one reserved-row trigger wording is corrected), pack
  behavior, `spk lint` rules, scope-B machinery.
- **Risk if skipped:** the judge verb stays three half-verbs with three
  reports; the audit rung keeps an undocumented ceiling; the agent loop keeps
  needing three invocations and three readings to answer one question.
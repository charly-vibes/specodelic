# Design — evidence record axes (D-1)

## Context

Three vocabularies coexist and describe different statements about one evidence
row:

| Layer | Where it lives today | What it answers |
|---|---|---|
| Run status | `Verdict` (`src/conform.rs:48`) | what the run concluded about the trace |
| Lifecycle class | `AuditClass` (`src/audit.rs:70`) | what this row's evidence is worth in the lifecycle |
| Structural vocabulary | four primitives (Nov 2) | what the record *is* (reference/relation/observation/assessment) |

M5's finding is that merging them is the error; M2's is that the relation is the
implementable primitive. D-1's deliverable is therefore a **record
specification**, not a merged enum.

Code facts this design is built on (verified, not assumed):

- `AuditClass::of` maps every verdict into a lifecycle class; `binds_nothing`
  is deliberately not derivable from a verdict (citation analysis, `src/audit.rs:110`).
- `stale`/`new` are diff overlays produced only by `--before`, guarded by
  `scope_sha256` (`src/audit.rs:695`); a fresh run never produces them.
- `AuditRecord` carries `{scenario_id, class, conform_verdict, reason,
  evaluated_claim_ids, closed_world}` — the both-layer record already exists.
- `model_check::Bound {max_depth, max_states, timeout_secs}` exists in run
  reports (`src/model_check.rs:77`); audit/conform records carry no bound.
- No conform code path evaluates a claim three-valued; the walk classifies
  evaluator kinds and checks the transition relation. Kernel `Unknown` means an
  unresolved id, i.e. a spec defect, not "undecided".

## Goals / Non-Goals

- **Goal:** one documented record whose every axis row is traceable to code, and
  a gate that fails when the two drift.
- **Goal:** decide, on the record, which memo-proposed values are reserved and
  what produces them.
- **Non-goal:** changing conform or audit behavior; adding fields or verdicts
  whose producer does not exist; measuring whether the record works (that is
  E-side work).

## Decisions

### D1 — The axis table is normative documentation, enforced by a guard

`docs/src/graded-evidence.md` gains a table with one row per value, naming the
axis, the code source, and a status of `implemented` or
`reserved (trigger: <change or ticket>)`. `scripts/check_evidence_axes.py`
parses:

1. `Verdict` variants from `src/conform.rs`;
2. `AuditClass` variants and every `AuditClass::of` arm from `src/audit.rs`;
3. the table rows from `docs/src/graded-evidence.md`.

It fails when a code value has no row, a `implemented` row has no code value, a
documented mapping differs from the `of()` arm, or a `reserved` row lacks its
trigger. Unparseable input is a failure with a remediation hint — never a
silent pass.

**Alternatives considered.** (a) Generate the table from Rust into the page
(a second write path and a generated-artifact convention for a 20-row table);
(b) hand-maintained docs with review only (this is the drift the change exists
to stop); (c) a `--print-vocabulary` CLI (grows the verb surface for a
developer-only need). Chosen: parse-and-compare, consistent with
`check_pack_doc_counts.py` / `check_section_sync.py`.

### D2 — `inconclusive` is reserved, not shipped

`inconclusive` is defined in the table with its borrowed home — ISO/IEC 9646-1
(ITU-T X.290) 3.3.124 and 6.5.1.3: *"neither a pass nor a fail verdict can be
given"* — and marked reserved. It enters `Verdict` when a **producer** exists:
a runner that executes a contract and cannot decide it (D-10 `check` fusion, or
the first contract-suite runner). Adding the variant now would create an arm no
code path can reach — a dead value that reads as capability.

Same rule for absence: `not-executed` is documented as a **recording** class
(the standard's test-case error / abnormal termination, 6.5.1.4–6.5.1.6), never
a fourth verdict. `undefined`/`unknown` are not substitutes.

### D3 — `origin`, `binding_kind`, `bound` are specified, then deferred to their producers

| Field | Reserved because | Producer that unblocks it |
|---|---|---|
| `origin` | no corpus record carries it; the Nov 11 decision records origin annotation syntax as design-level only, landing "with the change that first needs it" | the annotation-syntax change; third class (`authored-from-same-agent`) is the design's own case (M1 v2 C13) |
| `binding_kind` | no consumer distinguishes generated/manifest/mapping/attested; espectacular bindings are the only current binding site | the change that makes `check` consume sidecar bindings (D-10/D-12) |
| `bound` | already recorded for model-check runs (`model_check::Bound`); audit/conform passes are structural-only by construction | the fusion change that reads a bound alongside a verdict; until then the table points at the existing field rather than duplicating it |

No optional-with-default field is added "to be useful later": an always-populated
constant or an always-null field is noise that consumers learn to ignore.

### D4 — `agree` stays `unchecked`, and the table says why

`agree` is a pass **on the transition relation**, not on the covering claim: the
guard's expr was prose and was never interpreted. Audit grades evidence *for a
claim*, so an `agree` row is correctly `unchecked` ("no passing verdict on the
claim"), not `current`. The table states this semantics explicitly so the
mapping is not read as a bug. Reclassification would claim more than happened;
the memo's "bounded pass" observation is what the future `bound` field will
express, not a class change.

### D5 — `stale` is documented as a computed relation, never a stored value

The table's currency row names `stale`/`new` as **diff overlays** produced only
by `--before` over two reports with equal `scope_sha256`; a fresh run cannot
produce them. The prior-art note is corrected too: the change-trigger relation
is not novel (MoD 00-55 §4.7.1 via Kelly & McDermid 2001; CM status accounting,
IEEE 828-2012 §10); the mechanization over revisions is what this project adds.

## Risks / Trade-offs

- **Regex brittleness over Rust source.** Mitigated by fixture tests (drift in
  each direction, unparseable enum, missing trigger) and by fail-closed
  behavior; the parsed shapes are stable (`pub enum` blocks, `Verdict::X =>`
  match arms).
- **False confidence.** The gate proves presence and mapping, not semantics; a
  row can be documented wrongly with a matching code arm. Stated non-goal.
- **Reserved rows rot.** Each reserved row carries a trigger; a trigger naming
  a closed ticket is itself drift the reviewer can catch. The guard only
  enforces that the trigger exists.

## Migration Plan

Single change, no data migration: author the table from the current code in the
same commit as the guard, so the gate starts green. Rollback is removing the
recipe from `ci` and the two files — no behavior depends on it.

## Open Questions

- Should `inconclusive` ever gate an exit code? Default: no (a non-decision must
  not read as failure); revisit with D-10, where the contract runner defines the
  exit contract.
- Is `authored-from-same-agent` a value of `origin` or a flag beside it? Deferred
  to the annotation change (memo §3 leaves it open).
- Does the four-primitive vocabulary get a row here or its own table? This change
  keeps it out of scope: it is a Nov 2 candidate, not adopted (G5).
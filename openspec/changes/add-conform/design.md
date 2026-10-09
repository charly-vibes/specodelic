# Design: add-conform

## Decisions

### D1 — Verdict taxonomy is closed, five-valued, and distinct

`permitted | forbidden | underspecified | unknown | unsupported`.

- `permitted` — the trace is explainable by the declared Model and
  executable claims over it.
- `forbidden` — the trace contradicts a declared, executable claim **and**
  the run declared closed-world mode (D2).
- `underspecified` — no declared claim or Model element covers the trace.
  Distinct from `unknown` because the remediation differs: author a claim
  vs. make a claim interpretable.
- `unknown` — a claim covers the trace but its content is not interpretable
  by this run (prose-only expr, uninterpretable guard).
- `unsupported` — the covering claim's evaluator kind exists in the format
  but is not executable in this run (mirrors model_check's claim status).

Rejected alternative: a binary pass/fail with warnings — it collapses the
ladder (report §13: "never collapse … into one PASS label") and erases the
authoring remediation signal.

### D2 — Evidence classes and the closed-world gate (EDGE-001; review
R2 CORR-001 correction)

The two prohibition evidence classes are separated; the closed-world flag
gates only one of them:

1. **Positive contradiction** — a trace violates a declared, executable
   claim → `forbidden` in any invocation mode. This is the same fact
   model_check reports as `counterexample_found`; no exhaustiveness
   declaration is needed to report it.
2. **Absence of coverage + declared closed-world** — a trace no declared
   Model element or claim covers, with `--closed-world` given →
   `forbidden`. The flag is the recorded exhaustiveness declaration the
   source report's failure criterion requires.
3. **Absence of coverage, open-world** — → `underspecified`, never
   `forbidden`.

`--closed-world` remains an explicit flag, recorded in every verdict
record and the report header — never inferred. The flag is auditable: the
reviewer can see which rule produced each forbidden verdict (rule 1 or
rule 2) from the record's `closed_world` field and reason.

### D3 — Scenario corpus schema (MVP): one JSON object per line

```json
{"id": "expiry-confirmation-race", "setup": {"state": "held"},
 "trace": [{"action": "expire"}, {"action": "confirm", "observations": {"status": "confirmed"}}]}
```

- `id` (required, unique), `setup` (object; opaque to conform — it names
  the initial state and free variables), `trace` (ordered list of
  `{"action", "observations"?}` records). The key is named `observations`
  — not `observes` — to avoid colliding with the format's own typed
  reference field of the same name.
- conform interprets actions as transition ids/names and observations as
  state/effect references — normalization is mechanical (string identity
  after trimming), never semantic. A trace referencing an undeclared
  state/transition is `underspecified` (open-world) or `forbidden` only
  under `--closed-world` **plus** an executable exhaustiveness claim —
  undeclared names alone never produce `forbidden` (D2).
- Anything richer (concurrency, timing, expected-outcome fields) is out of
  MVP scope; unknown fields are refused with a remediation hint rather
  than silently ignored (envelope discipline: errors carry remediation).

### D4 — Reuse model_check's claim machinery, do not fork it

Classification of a file's invariant claims into required/unchecked comes
from the same types model_check uses (`required_claims_classified`);
executable invariants are evaluated over the compiled model with the same
scratch-crate path; the report's `scope_sha256` binds parsed structured
content **and** the consumed scenario corpus bytes. No new claim status
vocabulary is introduced; conform statuses map onto the existing
evaluator-kind set. Rationale: two taxonomies for "what the checker can
and cannot interpret" would drift — the model_check spec already fixed the
semantics (`specs/model_check.md`, `claim_aggregate_governs`).

### D5 — Input gate: lint-clean file, current artifacts

conform requires the target file to parse and lint clean and its compiled
artifacts to be current (same gate discipline `orchestrate` applies between
stages); it refuses otherwise with a remediation hint (`run spk lint` /
`spk compile`). Rationale: verdicts derived from stale or invalid artifacts
would misattribute failures to the spec.

### D6 — Evidence scope is stated in the output, not just the docs

The human report and JSON envelope both carry a fixed
`evidence_scope` field: "agreement on the supplied corpus; not a proof of
behavioral equality" (report §7.2). This is data, not documentation, so no
consumer can strip it by reformatting.

## Open questions

- **OQ1:** Should `conform` accept multiple spec files (corpus-level
  conformance) in MVP, or exactly one file like `compile`? Proposal assumes
  one file per invocation; corpus-level is a follow-up.
- **OQ2:** Do `law`-kind properties (identity/associativity cases) ever
  contribute to trace classification in MVP, or only invariant claims and
  the Model's transition relation? Proposal assumes invariant claims +
  Model only; laws evaluate over generators, not traces.

## Explicitly out of scope (parked, per review EXCL-002)

- Active conformance testing (report §7.3 — propose discriminating
  scenarios between interpretations).
- Refinement/composition checks (ladder levels 5–6) and cross-language
  adapters (level 7).
- Closed-world exhaustiveness *checking* (verifying the author's declared
  closed-world claim is actually exhaustive) — the flag is honored as
  declared; auditing it is a separate capability.
- Wild contract-bridge experiments (report §11) — parked ticket only.
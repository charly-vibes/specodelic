# Design: explicit claim scope and acceptance

## Test shape first

CLI fixtures exercise compile → model-check → verify, then inspect the
persisted report and envelope. Cover: mixed passing Rust/false kernel;
passing Rust/unknown citation; passing Rust/false negated citation; all
claims true; prose only; missing/duplicate statuses; old report; cross-file
source or scope mutation; timeout; property failure. Assert exit status,
aggregate outcome, exact qualified claim set, and a remediation hint.

## D1 — Required claims and unchecked prose

Required claims are invariant-kind Constraints explicitly opted into Rust,
kernel, or whole-cell citation evaluation. A citation is evaluated against
same-run invariant evidence, not a Property result. Prose-only invariants
are enumerated separately as unchecked. Constraint names, prose wording,
and merely having a deriving Property never imply verification. Keep
advisory/effect/extension-point rows out of the required invariant set.

## D2 — Aggregate rules

After scope validation: a refuted required claim yields counterexample_found;
otherwise exhausted time/state/depth budget yields timed_out; otherwise
any unknown, missing, or unsupported required claim prevents clean and
reports exploration_only with reasons. An empty required set also reports
exploration_only. A nonempty complete set of verified claims with completed
bounded exploration yields no_counterexample. Missing/duplicate/conflicting
report entries encountered by verify are invalid evidence, not inferred
unknown success. Kernel counterexamples carry claim evidence, not invented
state traces. Ordinary exploration_only keeps model-check's existing CLI
exit convention; verify rejects every non-clean aggregate with exit 1.

## D3 — Report version and freshness

Add claim_schema_version=1 and canonical qualified claim records containing
id, evaluator kind, status, and reason/evidence where not verified. Retain
legacy report fields for readers, but never key cross-file evaluation by a
bare local ID. Add expected_claim_ids, unchecked_claim_ids, and scope_sha256.

The scope digest is SHA-256 of a deterministic canonical serialization of
all parsed structured input content and the sorted compiled artifact hashes
consumed for that invocation. Sort files by canonical intent ID and maps
by key; preserve meaningful row/state order. Exclude prose, absolute paths,
byte spans, timestamps, and CLI file order. Include pack/generator registry
inputs when consumed. Version the encoding with the claim schema. The
compiler/model-checker must check artifacts against live structured input;
verify recomputes scope and the required set rather than trusting a stored
expected_claim_ids list. Changing a contributing file or dropping it from
the invocation invalidates the report. Ordinary corpus intent IDs must be
unique. Dual-format `id: spec` is a file-local identity, not a corpus-wide
identifier: model-check, verify and orchestrate accept such a file only as
the sole parsed input. If an invocation contains it plus any other file,
fail before compilation, evaluation or report writes with
isolated_scope_required and a hint to run each file separately using
separate artifact/report directories. This restriction applies to command
evaluation only; multi-file dual-format lint remains valid. An ordinary
corpus with duplicate intent IDs fails as duplicate_corpus_identity.

A single dual-format file's claims keep spec.<row> IDs and resolve citations
only within that file. No filesystem discovery or cross-file borrowing is
allowed. Separate runs may have identical claim names; their scope digests
bind their structured contents, not their paths. Two files differing in
invariant content must have different digests; verify must reject a report
copied from the other run. Identical structured inputs intentionally have
identical digests. Multi-file dual-format evaluation is deferred rather
than inventing path-qualified public claim IDs in this change.

Old/unrecognized schema versions, absent scope fingerprints, missing claims,
and mismatched input sets require model-check to be rerun. Do not rewrite
stored reports to manufacture evidence. This is deliberately stricter than
the old acceptance path; publish the rerun instruction with the release.

## D4 — One aggregation implementation

Model-check, verify, and orchestrate share claim classification and aggregate
rules. Verify still executes every generated property block and requires
both gates. A syntactically valid forged JSON file is not cryptographically
trusted evidence; this feature detects stale/incomplete evidence, not
malicious report forgery. No proof of application correctness is inferred.

## D5 — Explain assurance levels

A capability table states implemented behavior, prerequisites, unchecked
content, and pending changes. Versions derive from Cargo metadata rather
than independently maintained literals. The worked example is owned by the
Python emitter proposal, including an external application-test binding.
Guard-fragment execution and richer application state remain deferred.

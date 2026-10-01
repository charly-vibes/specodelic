---
id: batch.resume
kind: intent
statement: "WHEN a batch run starts and the manifest marks some items done, THE processor SHALL invoke processing only on items whose content fingerprint differs from the manifest's record."
---

# Batch Resume

A processor consumes a collection of files and records per-item progress in
a sidecar manifest, so an interrupted run resumes without reprocessing
unchanged items. The subject of this spec is the batch processor itself —
per USAGE.md §0, nothing here constrains or mentions any checking tool.

## Constraints

| id                              | kind      | expr                                                                                                                                                      | traces_to        |
|----------------------------------|-----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|------------------|
| done_iff_output_durable          | invariant | `manifest[item].status == done → output(item) was durably written before the marker was written`                                                            | [[batch.resume]] |
| resume_touches_only_unprocessed  | invariant | `a resumed run invokes process(item) only if fingerprint(item) ≠ manifest[item].fingerprint`                                                                | [[batch.resume]] |
| changed_input_invalidates_marker | invariant | `fingerprint(item) ≠ manifest[item].fingerprint → the item is treated as pending regardless of its recorded status`                                         | [[batch.resume]] |
| failed_is_not_done               | invariant | `manifest[item].status == failed → item is reprocessed on resume; status == partial → processing resumes from manifest[item].offset`                        | [[batch.resume]] |
| manifest_is_sidecar              | invariant | `the manifest lives outside every output artifact; no output contains manifest rows`                                                                        | [[batch.resume]] |
| offset_version_contract          | invariant | `a recorded offset is valid only for the processor version that wrote it; a version mismatch demotes the item to pending`                                   | [[batch.resume]] |
| resumed_equivalent_to_fresh      | invariant | `result(resume(run interrupted at t)) == result(uninterrupted rerun over the same inputs)`                                                                  | [[batch.resume]] |
| map_is_parallel_unconditional    | invariant | `deterministic per-item processing runs over any partition without coordination — map parallelism is licensed by determinism alone, never gated on merge algebra` | [[batch.resume]] |
| reduce_sound_iff_monoid          | invariant | `partitioned results may be combined by merge iff merge is associative with an identity; partition order is free iff merge is additionally commutative — no reduce-parallelism claim exists apart from these laws` | [[batch.resume]] |
| process_is_deterministic         | invariant | `process(item) is a pure function of (item content, offset) — no reads of wall-clock, randomness, or unrecorded external state` | [[batch.resume]] |
| emits_survive_replay             | invariant | `every emitted artifact is a pure function of (item, offset), so reprocessing after a crash rewrites identical bytes; emissions that cannot be rewritten (notifications, appends) are deduplicated by (item fingerprint, offset) before taking effect` | [[batch.resume]] |
| manifest_merges_idempotently     | invariant | `concurrent workers checkpoint into per-worker manifests combined by union of per-item markers; marker ∪ marker is associative, commutative, and idempotent, so any merge order and any checkpoint frequency yields the same manifest` | [[batch.resume]] |

## Model

### States
- `pending`
- `in_progress`
- `partial`
- `done`
- `failed`

### Transitions

| id             | from        | to          | guard                                                                        |
|----------------|-------------|-------------|------------------------------------------------------------------------------|
| start          | pending     | in_progress | `item unmarked ∨ fingerprint(item) ≠ manifest[item].fingerprint`              |
| complete       | in_progress | done        | [[batch.resume.done_iff_output_durable]]                                     |
| checkpoint     | in_progress | partial     | `output-so-far written ∧ offset recorded`                                    |
| fail           | in_progress | failed      | `¬[[batch.resume.done_iff_output_durable]] ∧ ¬(output-so-far written ∧ offset recorded)` — cites the union of success siblings per guard_negation_total |
| resume_partial | partial     | in_progress | `recorded offset version matches the current processor`                      |
| requeue_failed | failed      | pending     | [[batch.resume.failed_is_not_done]]                                          |
| invalidate     | done        | pending     | [[batch.resume.changed_input_invalidates_marker]]                            |

## Properties

| id                                  | kind | derives_from                                      | generator                                        | predicate                                                                                          |
|-------------------------------------|------|---------------------------------------------------|--------------------------------------------------|----------------------------------------------------------------------------------------------------|
| done_marker_implies_output_readable | unit | [[batch.resume.done_iff_output_durable]]          | `crash_between(write_output(item), write_marker(item))` | `no resumed run reports done for an item whose output is missing`                            |
| resumed_run_invocation_count        | unit | [[batch.resume.resume_touches_only_unprocessed]]  | `manifest with k done items, n−k changed items`  | `invocation_count(process) == n − k`                                                                |
| edited_item_reprocessed             | unit | [[batch.resume.changed_input_invalidates_marker]] | `edit an item after its marker was written`      | `the item is processed on the next resume`                                                          |
| failed_item_reprocessed             | unit | [[batch.resume.failed_is_not_done]]               | `mark an item failed, then resume`               | `invocation_count(process(item)) == 1`                                                              |
| outputs_contain_no_manifest_rows    | unit | [[batch.resume.manifest_is_sidecar]]              | `arbitrary completed run`                        | `manifest rows appear in no output artifact`                                                        |
| version_mismatch_invalidates_partial| unit | [[batch.resume.offset_version_contract]]          | `bump processor version with stale partial offsets recorded` | `partial items reprocessed from 0, never from the stale offset`                          |
| crash_resume_equivalent             | unit | [[batch.resume.resumed_equivalent_to_fresh]]      | `arbitrary crash-injection schedule over the run`| `result(resumed_run) == result(uninterrupted_run)`                                                  |
| resume_composition                  | law  | [[batch.resume.resumed_equivalent_to_fresh]]      | `arbitrary manifests and crash schedules`        | `**associativity:** resume(resume(m, a), b) == resume(m, a ++ b)  **identity:** resume(empty_manifest, items) == process_all(items)` |
| results_merge_monoid                | law  | [[batch.resume.reduce_sound_iff_monoid]]   | `arbitrary triples of partial results (a, b, c)` | `**associativity:** merge(merge(a, b), c) == merge(a, merge(b, c))  **identity:** merge(e, a) == a == merge(a, e)` — fails ⇒ reduce is inherently sequential, chunked execution unsound (map stays parallel per map_is_parallel_unconditional) |
| results_merge_commutative           | unit | [[batch.resume.reduce_sound_iff_monoid]]   | `arbitrary pairs of partial results`             | `merge(a, b) == merge(b, a)` — optional case, feeds the tier table below: holds ⇒ workers need no ordering; fails ⇒ merge is a non-commutative monoid (e.g. ordered append), reduce still parallel-sound, merge order fixed |
| manifest_merge_semilattice          | law  | [[batch.resume.manifest_merges_idempotently]]     | `arbitrary triples of per-worker manifests`      | `**associativity:** (m ∪ n) ∪ o == m ∪ (n ∪ o)  **identity:** ∅ ∪ m == m  **idempotence:** m ∪ m == m` — idempotence is what makes double-checkpointing harmless |
| map_chunks_uncoordinated            | unit | [[batch.resume.map_is_parallel_unconditional]]    | `arbitrary item partition across workers`        | `per-item results match the sequential run regardless of partition boundaries` |
| deterministic_replay                | unit | [[batch.resume.process_is_deterministic]]         | `same item processed twice under any interleaving` | `outputs are byte-identical` |
| replay_dedupes_emissions            | unit | [[batch.resume.emits_survive_replay]]             | `crash after emit, before marker; resume`        | `output artifact state equals an uninterrupted run's — no duplicated appends, no replayed notifications` |

## Notes

**Transducer framing.** `process` is a transducer: a step function over
(accumulator, item) → (accumulator, emits). The Model section *is* that
transducer per item. Transducers compose; **chunkability is a property of
the accumulator's algebra, not of transducer-hood** — a transducer whose
step is associative parallelizes like a scan (parallel prefix); one whose
state entangles order does not. So the design-raising questions this
spec forces are exactly three:

> is the accumulator a monoid — and is the process deterministic, with
> replay-safe emissions?

Three answers, three architectures, all derived rather than asserted
(map parallelism needs none of these — only determinism; the algebra
gates the *reduce*):

| accumulator algebra | architecture the law row licenses |
|---|---|
| deterministic, emissions replayable | parallel **map** over any partition — no algebra required (`map_is_parallel_unconditional`, `emits_survive_replay`) |
| results form a monoid (`results_merge_monoid`) | + chunked parallel **reduce**; merge in tree order |
| + commutative (`results_merge_commutative`) | + workers need no ordering at all; any shuffle, any reduce tree |
| + idempotent (semilattice, `manifest_merge_semilattice`) | + concurrent workers checkpoint freely; duplicates vanish |
| results not a monoid (e.g. running average, ordered state machine) | reduce is inherently sequential — still streamable, never chunkable; say so here rather than discovering it in production |

The resume design of the Constraints table is itself an instance: the
manifest is a semilattice, which is *why* resume composes
(`resume_composition`) and *why* concurrent checkpointing is safe.

The three decisions the constraints force, in dependency order:

1. **Marker placement** (`manifest_is_sidecar`): the manifest is a sidecar
   (e.g. `.state.jsonl`), never embedded in outputs — half-written markers
   must not be readable by downstream consumers.
2. **Fingerprints, not paths** (`changed_input_invalidates_marker`): items
   are keyed by content fingerprint; renames between runs keep their
   progress, edits discard it.
3. **Partial is a contract, not a bookmark** (`offset_version_contract`):
   once offsets are recorded, the processor version is part of their
   meaning; alternatively, promote sub-steps to items and keep markers
   boolean-shaped (see `no_boolean_columns` in `specodelic.md` for why the
   status vocabulary is named variants, not flags).

`done_iff_output_durable` exists to make the worst interleaving — crash
after marker, before output — unreachable; `emits_survive_replay` closes
the mirror-image hole (crash after emit, before marker: at-least-once
replay must not duplicate side effects — the semilattice protects
*manifests*, not *outputs*). `crash_resume_equivalent` is the property a
test suite must actually check to prove resume is just a cache in front
of a deterministic rerun — which is why `process_is_deterministic` is a
constraint in its own right, not an implicit assumption. Failure effects (a
`kind = "effect"` row per error, per the corpus's error contract) are
omitted here; add them if the processor exposes an error surface.

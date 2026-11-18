---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/specodelic
commit: c76e2b0c
updated: 2026-11-18
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: The suite evaluation that grouped specodelic's laws into three clusters; this file refines those clusters per property.
  - path: /var/home/sasha/para/areas/dev/gh/charly/specodelic/.wai/resources/research/2026-11-18-antithesis-skills-method-inventory.md
    why: Method reference for clustering by shared evidence and dominance; its warning against importing harness machinery scopes the relationships to local properties.
  - path: https://hegel.dev/
    why: Executor whose state-machine and generator features determine which properties can share a harness.
---

# Property relationships

Slugs are canonical IDs; each appears in `property-catalog.md` and has an
evidence file under `properties/`.

## 1. Rename algebra and its writer realization

*Properties:* `rename-identity-noop`, `rename-composition-associative`,
`rename-collision-refused`, `rename-kind-preserved`,
`rename-write-set-realizes-target`, `rename-writer-parity`,
`rename-success-reachable`, `writer-apply-faithful-roundtrip`,
`writer-edit-composition`.

Shared mechanism: `rename::run` is a thin planning/verification shell over
`writer::apply`. Identity and collision are the algebra's units/preconditions;
composition and `writer-edit-composition` are the same equation at two
altitudes; `rename-write-set-realizes-target` and
`writer-apply-faithful-roundtrip` are the same reparse oracle at two altitudes.

Suspected dominance:

- `writer-span-coherence` and `writer-untouched-bytes-preserved` dominate this
  cluster's byte claims: if spans are approximate or splicing is not local, the
  rename laws fail for reasons the rename layer cannot fix.
- `rename-kind-preserved` **dominates** `writer-apply-faithful-roundtrip`: the
  roundtrip oracle is blind to non-id cell corruption, so the faithful property
  can be green while the kind property is red. Implement the kind property
  first.
- `rename-collision-refused` is a precondition for composition: composition is
  stated only for fresh ids.
- `rename-writer-parity` is **invalidated** and dominates nothing. Do not spend
  generator budget on it.

## 2. Span fidelity and byte preservation

*Properties:* `writer-span-coherence`,
`writer-emit-identity-byte-exact`, `writer-untouched-bytes-preserved`,
`writer-span-failure-labeled`.

Shared mechanism: `verify_spans` + span-local splicing.

- `writer-emit-identity-byte-exact` is near-tautological and is **dominated by**
  `writer-span-coherence`: `emit` returns the input string, so the only signal
  is whether the spans verify.
- `writer-untouched-bytes-preserved` is the substantive half of the writer's
  identity promise: it constrains what an *edit* may change.
- `writer-span-coherence` and `writer-apply-faithful-roundtrip` are easily
  confused but check different halves: span coherence verifies recorded spans
  against the **unedited** parse; the roundtrip verifies the **edited** text
  against the target instance. A span can be coherent while the roundtrip
  oracle is blind to non-id cells (see `rename-kind-preserved`), so neither
  implies the other.
- `writer-span-failure-labeled` is the failure-side complement: the same
  verification must produce a labeled refusal rather than a panic.
- All three share one generator: a parser-accepted full-family corpus with
  padding/backticks/CRLF variants. Build it once.

## 3. Merge verdict composition

*Properties:* `merge-verdict-symmetry`, `merge-collision-correctness`,
`merge-blast-radius-intersection`, `merge-modify-delete-flagged`,
`merge-relint-gates-verdict`, `merge-textual-conflict-flagged`,
`merge-needs-review-reachable`.

Shared mechanism: five passes collect `Finding`s; verdict precedence maps finding
kinds to `failed` / `needs_review` / `merged`; the union tree is the only
content the relint pass sees.

Suspected connections and dominance:

- `merge-verdict-symmetry` **dominates** the whole cluster's
  "check is a function of the history" claim. If it fails, any individual
  finding property can be green in one argument order and red in the other.
- `merge-collision-correctness` and `merge-textual-conflict-flagged` are the two
  "failed" detectors; they share the base-text comparison semantics. Neither
  covers add/add, which is exactly the gap `merge-verdict-symmetry` exposes.
- `merge-relint-gates-verdict` is the catch-all: it can fail a merge the other
  passes deemed clean, but it can only see the union tree, whose selection is
  the symmetry bug.
- `merge-modify-delete-flagged` shares `union_tree` with symmetry; a generator
  that emits delete/edit pairs exercises both.
- `merge-needs-review-reachable` is the generator guard for
  `merge-blast-radius-intersection` and the `rename_replay` path; implement it
  alongside them, not after.

## 4. Compile emission purity

*Properties:* `compile-artifact-determinism`, `compile-totality`,
`compile-id-preservation`, `compile-no-semantic-drift`,
`compile-artifact-stem-collision-refusal`.

Shared mechanism: pure emitters over a `Spec`, with internal gates for totality,
id preservation, and TOML drift.

- `compile-artifact-determinism` is the **only non-gate** property in the
  cluster: the other three restate checks `compile_spec` already performs before
  returning `Ok`. A green run of those three is weak evidence; a red
  determinism run is strong.
- `compile-totality`, `compile-id-preservation`, and
  `compile-no-semantic-drift` share one generator (linted-and-covered specs with
  all families) and can be one Hegel test group.
- `compile-artifact-stem-collision-refusal` is independent: it operates on
  paths, not spec content, and is the one compile property with a filesystem
  identity dimension (`canonical_artifact_path`).

## Cross-cluster notes

- **Rename and merge share the reference graph.** `merge`'s blast radius is the
  acset closure; `rename`'s verify gate uses `graph::build`. A defect in the
  instance/graph adapter would show up as a merge false-clean and a rename
  false-accept. No property here targets the adapter directly; `acset_parity`
  covers it (out of scope).
- **Generator sharing.** One IR→Markdown corpus generator serves clusters 1, 2,
  and 4. Merge needs three such corpora plus path-level mutations. The generator
  is the dominant adoption cost (see `recommendation.md`).
- **Near-tautology cluster.** `rename-writer-parity`,
  `writer-emit-identity-byte-exact`, `writer-apply-faithful-roundtrip`,
  `compile-totality`, `compile-id-preservation`, and
  `compile-no-semantic-drift` are all gate-enforced or structural. They are
  catalogued for completeness and as regression alarms, but they should not
  drive the adoption decision or consume generator budget.

## Unclustered / excluded

- `compile-naturality` (`compile(rename(I)) == rename(compile(I))`) is excluded:
  no artifact-rename operation exists. It would sit between clusters 1 and 4 if
  one is defined.
- `reachable` check mode is unused: no Antithesis workload is in scope, so no
  property asserts a workload path. Generator non-vacuity is covered by the two
  `sometimes` properties.
---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/specodelic
commit: c76e2b0c
updated: 2026-11-18
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Supplies the tests-only scope, the exact-pin/static-engine guidance, and the cost baseline this recommendation refines for one crate.
  - path: /var/home/sasha/para/areas/dev/gh/charly/specodelic/.wai/resources/research/2026-11-18-antithesis-skills-method-inventory.md
    why: Supplies the oracle-validity and seeded-mutation gate used as the adoption condition, and the warning against full ensemble/harness machinery here.
  - path: https://hegel.dev/
    why: Candidate executor; its harness, `#[hegel::test]`, state-machine support, and model-based generation determine the implementation cost.
---

# Recommendation — Hegel for `specodelic`'s own laws

## Verdict

**YES — add Hegel property tests in `tests/` only, for the 17 entries listed
below (15 properties plus two generator guards), conditional on building an
IR→Markdown corpus generator that passes a round-trip self-check and a
seeded-mutation check.**

If that condition is not met, the answer is not "no Hegel forever"; it is the
explicit fallback: **NO to the full set, YES to a 1.5–2 day fixture-driven
subset of three properties.** Do not replace or port the existing proptest
suite, and do not touch the generated artifacts or `spk verify`.

## Implement with Hegel (YES)

Tier 1 — the two predicted reds plus the merge verdict laws (highest signal):

| Property | Why |
|---|---|
| `merge-verdict-symmetry` | Predicted red; exposes the add/add hole and the order-dependent union tree. |
| `rename-kind-preserved` | Predicted red; keyword-colliding row ids corrupt other rows' `kind` cells. |
| `writer-untouched-bytes-preserved` | The substantive writer promise; prose/CRLF/padding survival. |
| `merge-collision-correctness` | Independently-minted ids are the primary merge hazard. |
| `merge-blast-radius-intersection` | The `needs_review` semantics that distinguish the tool from git. |
| `merge-modify-delete-flagged` | Both argument orders; shares the union-tree harness with symmetry. |

Tier 2 — algebra, failure labels, and determinism:

| Property | Why |
|---|---|
| `rename-composition-associative` | The associativity half of the rename law; catches offset drift. |
| `rename-collision-refused` | Precondition for composition; cheap and exact. |
| `writer-edit-composition` | Same equation at the writer altitude; shrinking-friendly. |
| `writer-span-coherence` | The real content behind writer identity. |
| `writer-span-failure-labeled` | The failure-side contract; parser drift must refuse, not panic. |
| `compile-artifact-determinism` | The only non-gate compile property; protects digest/staleness gates. |
| `merge-textual-conflict-flagged` | Completes the failed-verdict partition with collision. |
| `merge-relint-gates-verdict` | The catch-all that can fail a merge the heuristics deemed clean. |
| `compile-artifact-stem-collision-refusal` | Silent artifact overwrite is unrecoverable; pure path logic, cheap. |

Generator guards (implement alongside Tier 1/2, as `sometimes` legs):

| Property | Why |
|---|---|
| `rename-success-reachable` | Fails if the generator never produces an accepted rename. |
| `merge-needs-review-reachable` | Fails if the generator never reaches the middle verdict. |

That is 17 entries; the two guards are test-level non-vacuity assertions
rather than separate product properties, so the implementation can be 15 Hegel
tests plus two `sometimes` checks.

## Do not use Hegel for (NO)

| Property | Reason |
|---|---|
| `rename-writer-parity` | **Invalidated**: `run` delegates to `run_via_writer`, so it is a self-comparison. |
| `writer-emit-identity-byte-exact` | Near-tautological: `emit` returns the source string. Existing proptest covers it. |
| `writer-apply-faithful-roundtrip` | Gate-enforced by `verify_roundtrip`; keep the existing proptest. |
| `compile-totality` | Type-enforced and gate-checked; a generated red test adds no signal. |
| `compile-id-preservation` | Gate-enforced inside `compile_spec`. |
| `compile-no-semantic-drift` | Gate-enforced TOML reparse/re-emit. |
| `rename-identity-noop` | One-line early return; a fixed regression test is enough. |
| `rename-write-set-realizes-target` | Largely restates the verify gate; keep as a fixed apply-and-reparse test. |

Also NO: any change to `specodelic/*_props.rs`, `spk verify`, or the regular
`proptest` dependency; any Hegel use in a shipped code path.

## The condition: corpus generator with oracle validity

The writer and rename properties cannot be driven by hand-built `Spec` values:
spans are produced by the parser, and `apply` refuses (or corrupts) stale
spans. The generator must therefore:

1. build a typed model of a corpus (intents, constraints, states, transitions,
   properties, links, checklists);
2. render it to valid Markdown;
3. parse it and assert the resulting `Spec` matches the intended model
   (the **round-trip self-check**);
4. feed the text and the parsed `Spec` to the property.

The adoption condition is two-part:

- **Generator self-check:** over a large sample, every generated corpus parses
  and its parsed ids/links/kinds/cells equal the model. A generator that
  rejection-samples unparseable output is acceptable only if the rejection rate
  is measured and shrinking still produces structured cases.
- **Seeded-mutation check (oracle validity):** before trusting any green run,
  inject one realistic defect per red property and confirm the property fires:
  - in `collect_cell_repls`, drop the column restriction (or seed a
    keyword-colliding id) → `rename-kind-preserved` must fail;
  - in `union_tree`, keep the current add/add behavior but remove the base
    lookup guard (or seed the add/add triple) → `merge-verdict-symmetry` must
    fail;
  - a no-op mutation (e.g. rename a local variable) must leave both green.

Without this check, a green Hegel run is a green run of a generator, not
evidence about the crate.

## Cost estimate

[ASSUMPTION] Estimates are engineer-days, unmeasured; no Hegel test was built
for this crate in this pass.

| Work item | Estimate |
|---|---|
| Typed corpus model + Markdown renderer + parser round-trip self-check | 2–3 days |
| Table/state/link/checklist coverage, syntax variants (padding, backticks, CRLF, quoted ids), shrinking-friendly composites | 1–2 days |
| Merge triple generator (base/current/incoming) with add/edit/delete/path-mutation operations | 1 day |
| 15 Hegel tests + two `sometimes` legs + shared oracles | 1.5–2.5 days |
| Hegel dev-dependency integration (`static-engine`, exact pin, CI run) | 0.5 day |
| **Total** | **5–8 engineer-days** (overlap expected; call it one focused week) |

Cost drivers, in order:

1. **ACSet/IR construction.** The `Spec` is a parse product with byte spans.
   Hand-constructing it bypasses the very span logic under test, so the
   generator must render text and parse it. This is the single largest cost and
   the reason a "small generator" is not small.
2. **Table/state/link coverage.** Four table families plus state bullets,
   transitions, and structured links; the writer's exact-raw-match law means
   padded vs exact links are distinct cases. Cross-file links and checklist
   manifests add multi-file corpora.
3. **Parser round-trip requirement.** Every generated case must parse and match
   the model; invalid shapes must be rejected without collapsing the
   distribution. String-level generation shrinks poorly; a typed model plus a
   renderer is the maintainable path.
4. **Merge needs three trees.** A single corpus generator is not enough: merge
   properties need a base and two mutated copies, including path-level
   add/add/delete/modify operations that are not part of the spec IR.
5. **Oracle validity.** The seeded-mutation harness is small but mandatory; it
   is what separates a property suite from a generator smoke test.

**Fallback if the generator is not funded:** 1.5–2 days for fixture-driven
Hegel tests of `merge-verdict-symmetry`, `rename-kind-preserved`, and
`writer-untouched-bytes-preserved`, using hand-written corpora that include the
two counterexamples. This does not amortize across the catalog, but it pins the
two predicted reds and the writer's core promise.

## Sequencing

1. Decide the two product questions: restrict the local-id rewrite to id-bearing
   columns (or exclude `kind`), and treat add/add as a conflict. These decide
   whether the reds are fixed first or landed as red tests.
2. Build the generator + self-check.
3. Land Tier 1, run the seeded-mutation check, then triage reds as fixes or
   explicit red tests before any CI gate.
4. Add Tier 2 and the two `sometimes` legs.
5. Leave the existing proptests, generated artifacts, and `verify` alone.

## Explicit non-goals

- No Antithesis container topology (`deployment-topology.md`).
- No rewrite of the existing proptest suite.
- No change to `proptest = "1"` as a regular dependency.
- No Hegel in the generated artifacts or the `spk verify` scratch crate.
- No new production API just to test; every catalogued property has a current
  seam.
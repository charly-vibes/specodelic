---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/specodelic
commit: c76e2b0c
updated: 2026-11-18
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: The suite evaluation whose specodelic verdict ("tests only; generated artifacts and verify stay on proptest") this synthesis confirms and narrows per property.
  - path: /var/home/sasha/para/areas/dev/gh/charly/specodelic/.wai/resources/research/2026-11-18-antithesis-skills-method-inventory.md
    why: Evaluation-separation and oracle-validity lessons; its "discovery finds, evaluation finds problems" split is applied below in single-agent mode.
  - path: https://hegel.dev/
    why: Engine under evaluation; its shrinking/model-based generation limits drive the implementability and cost findings.
---

# Evaluation synthesis

## Method

Single-agent evaluation of the 25-property catalog, run as four explicit lens
passes (Antithesis fit, coverage balance, implementability, wildcard) over
`property-catalog.md`, `sut-analysis.md`, `deployment-topology.md`,
`existing-assertions.md`, and the per-property evidence files. No ensemble was
spawned: this is a single-crate, documentation-only pass, and the method
inventory explicitly warns against full ensemble runs for single-crate work.

No property was executed. Every red/green is source-derived; the report says so
at each occurrence.

## Findings

### Gaps

| # | Finding | Scope | Evidence | Action |
|---|---|---|---|---|
| G1 | **Merge has no add/add conflict kind and no symmetry property.** Two branches adding the same path with different text are silently resolved to whichever branch is passed as `a`, and relint can then differ by argument order. | Catalog-wide (merge) | `union_tree` (`src/merge.rs:310-380`), `textual_conflict_findings` base lookup (`:223-247`), verdict precedence (`:170-182`). Counterexample in `properties/merge-verdict-symmetry.md`. | Added `merge-verdict-symmetry` as P0/RED. This is the highest-value property in the catalog. |
| G2 | **Row-kind corruption is uncovered and outside the existing generator's distribution.** `collect_cell_repls` rewrites every column whose value equals the local id, including `kind`. A row id equal to a kind keyword corrupts other rows' kinds, and the roundtrip oracle's row-count check can still pass. | Property-specific (rename/writer) | `writer.rs:378-457`, `verify_roundtrip` cell counting (`:576-591`), existing generator ids `r{n}`/`renamed{n}` (`tests/acset_writer_apply.rs:409-478`). | Added `rename-kind-preserved` as P0/RED. |
| G3 | **Compile determinism is asserted nowhere except as internal gates.** The existing generated properties check totality/id preservation/TOML drift, all of which `compile_spec` checks itself. No repeated-call byte-equality property exists. | Property-specific (compile) | `compile_spec` (`src/compile.rs:1980-2050`), no repeated-call test in `tests/`. | Added `compile-artifact-determinism`; classified the three gate properties as near-tautological P2. |
| G4 | **No generator non-vacuity guards.** Rename and merge suites can be green while never reaching an accepted rename or a `needs_review` verdict. | Catalog-wide | Generated artifacts use constant `Just("…")` strategies (`specodelic/merge_props.rs:28-76`); existing proptests have narrow fixed distributions. | Added `rename-success-reachable` and `merge-needs-review-reachable` as `sometimes` properties. |
| G5 | **`rename_naturality`'s compile-commutation case has no implementation seam.** `compile(rename(I)) == rename(compile(I))` is stated in the rename spec, but there is no artifact-rename operation; `rename` parses specs, and `.toml`/`.tla`/`*_props.rs` are not specs. | Catalog-wide (documented exclusion) | `src/rename.rs` (no artifact path), `src/compile.rs` (no artifact-rename function). | Excluded from the catalog with the reason; listed under "Invalidated / excluded". Testing it requires defining the operation first. |
| G6 | **The merge union tree is unobservable.** `MergeReport` exposes verdict and findings only, so callers and properties cannot inspect the tree relint judged. | Property-specific (merge) | `MergeReport` (`src/merge.rs:38-50`), `relint_findings` (`:382-420`). | Recorded as an open question; it limits the strength of `merge-relint-gates-verdict` and any symmetry check on content. |

### Biases

| # | Finding | Scope | Evidence | Consequence |
|---|---|---|---|---|
| B1 | **Near-tautology inflates the portfolio.** Six catalogued properties are gate-enforced or structural: `rename-writer-parity` (invalidated), `writer-emit-identity-byte-exact`, `writer-apply-faithful-roundtrip`, `compile-totality`, `compile-id-preservation`, `compile-no-semantic-drift`. Counting 25 properties overstates the evidence a green run would provide. | Catalog-wide | Their evidence files mark the gate explicitly; `writer::emit` returns `source.to_string()` (`writer.rs:56-60`). | Adoption should count the 15 properties the recommendation implements (plus two generator guards), not all 25. The catalog retains the gate entries as regression alarms. |
| B2 | **The existing suite is biased toward fixed fixtures and keyword-free ids.** `acset_writer_apply` always uses `r{pick}` → `renamed{pick}` and `kind = invariant`; the generated artifacts use constant strategies. This is a plausible reason G2 is not covered. | Catalog-wide | `tests/acset_writer_apply.rs:409-478`, `specodelic/*_props.rs`. | Any "all green today" claim is a claim about those distributions. The two predicted reds sit just outside them. |
| B3 | **The Hegel report's three-cluster framing is coarse.** "Rename algebra, writer identity, merge verdicts" names the right areas but hides that writer identity is the tautological half and merge has an order-dependence hole. | Catalog-wide | Hegel report §3, this catalog. | The recommendation replaces the cluster names with a property-level yes/no list. |
| B4 | **No concurrency/crash properties — correctly.** Unlike the suite-wide catalog, this SUT is pure in-process; adding such properties would be fault injection with nothing to inject. | Catalog-wide | `sut-analysis.md` state/concurrency section; `deployment-topology.md`. | Recorded as a deliberate absence, not a discovery miss. This is the main place where this scratchbook should diverge from the suite-level one. |

### Refinements

Applied directly to the catalog and evidence files:

| # | Finding | Action |
|---|---|---|
| R1 | `rename-writer-parity` compares `run` to `run_via_writer`, but `run` delegates to `run_via_writer` at HEAD. | Marked **INVALIDATED — vacuous self-comparison**; kept only as a delegation guard. Snapshot parity remains the real oracle. |
| R2 | Writer identity is byte-for-byte tautological. | Marked near-tautological in the catalog and evidence; reclassified to P2; the substantive content moved to span coherence / untouched bytes. |
| R3 | Composition is not true for corpora with dangling links that alias the intermediate id. | Scoped `rename-composition-associative` to link-closed corpora and documented the counterexample. |
| R4 | Merge messages contain branch labels, so byte comparison is wrong for symmetry. | Symmetry invariant compares verdict + finding-kind multiset, not messages. |
| R5 | `write_artifacts_into` passes through `rustfmt`, so cross-machine byte determinism is not a property of `compile_spec`. | Determinism property scoped to `compile_spec`; the rustfmt/environment question recorded as an open question. |
| R6 | `apply`'s identity short-circuit skips `verify_spans`. | Documented in `writer-span-failure-labeled`; harmless (no splice) but it means the no-op path is not a span gate. |
| R7 | Predicted reds are unexecuted. | Every entry says "source-derived, not executed"; the recommendation requires a seeded-mutation/oracle check before trusting green, and a fix-first vs red-test triage for the two reds. |

## Hegel crates-worth-it verdict

**Worth it for this crate: YES — for tests only, and conditional on the corpus
generator.** The suite report's specodelic verdict stands ("tests only;
generated artifacts and `verify` stay on proptest"), but the cluster-level
recommendation understates the cost and overstates the writer half:

- The writer identity property that motivated "writer identity" is
  near-tautological and already covered by a proptest. Hegel's marginal value
  there is shrink quality on span failures, not new coverage.
- The two genuine defects found by inspection — `rename-kind-preserved` and
  `merge-verdict-symmetry` — are deterministic and reachable with a small
  generated corpus. They are also **cheap to pin as fixed regression tests**,
  which is the honest comparison: Hegel is worth it only if the generator is
  reused across enough properties to amortize its cost.
- `specodelic`'s existing proptests are narrow, not absent; the gap is input
  distribution, not the engine. Hegel's model-based/state-machine features help
  rename composition and merge triples; its shrinker helps span failures. Both
  are real but secondary.

**Do not** use Hegel for the generated artifacts, `spk verify`, or the six
gate/near-tautological properties. **Do not** replace the existing suite; add a
`tests/` file (one per cluster is acceptable) and leave the generated codegen
untouched.

## Recommendation

The standalone, explicit recommendation is in
[`../recommendation.md`](../recommendation.md). In summary:

- **YES** to Hegel in `tests/` for the non-gate properties listed there,
  starting with the two predicted reds.
- **NO** to Hegel for the six gate/near-tautological properties, the invalidated
  parity property, and anything touching generated artifacts or `verify`.
- **Condition** on an IR→Markdown corpus generator that passes a parser
  round-trip self-check and a seeded-mutation check. Without it, do the
  1.5–2 day fixture-driven subset (`merge-verdict-symmetry`,
  `rename-kind-preserved`, `writer-untouched-bytes-preserved`).
- **Triage the two reds before CI**: fix first (restrict the cell rewrite;
  flag add/add) or land them as explicitly red tests. Do not put either in a
  required gate without that decision.

## Passes (checked, no action)

- All 25 slugs have evidence files and vice versa; relationships reference only
  catalog slugs.
- Every property names a real public or crate-visible seam; no property needs a
  new `pub` API.
- The two `sometimes` properties are the only non-universal checks; this is the
  right use (generator non-vacuity), and no invariant is weakened into a
  `sometimes`.
- Compile's `BTreeMap`/`Vec` discipline was checked by inspection; no
  `HashMap`/clock/random in the in-scope modules.
- No Antithesis SDK exists in the crate, so every property is new
  instrumentation; no evidence file duplicates an existing assertion.

## Uncertainties

- Predicted reds are not executed. The code paths are concrete, but a run is the
  missing confirmation; the generator must produce the keyword-colliding id and
  the add/add triple for the reds to surface.
- The cost estimates in `recommendation.md` are [ASSUMPTION]-grade, not
  measured. No Hegel test was built for this crate.
- The add/add product intent is not stated in `merge.md`; the recommendation
  assumes git-like conflict semantics (flag, do not pick a side), which is a
  judgment call for the owner.
- `rustfmt`-version-dependent bytes from `write_artifacts_into` are outside the
  determinism claim; if the product wants hermetic artifact bytes, that is a
  separate decision.
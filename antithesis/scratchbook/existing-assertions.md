---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/specodelic
commit: c76e2b0c
updated: 2026-11-18
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Defines the suite-wide assertion baseline this scan extends (no Antithesis SDK in any crate).
  - path: /var/home/sasha/para/areas/dev/gh/charly/specodelic/.wai/resources/research/2026-11-18-antithesis-skills-method-inventory.md
    why: Method inventory's "existing assertions" step and its warning that proptest codegen is product machinery, not test instrumentation.
  - path: https://hegel.dev/
    why: Target harness whose assertions these properties would become; no such assertions exist yet.
---

# Existing assertions

## Antithesis SDK

**None found.** A search over `*.rs` and `*.toml` for `antithesis`,
`assert_always`, `assert_sometimes`, `assert_reachable`, `assert_unreachable`,
and the non-macro equivalents returns no hits in this crate.

Consequence: every property in `property-catalog.md` is **new instrumentation**
at a library seam. Nothing needs to be reconciled with existing Antithesis
properties, and no evidence file suggests an assertion that already exists.

## Handwritten property tests (proptest)

`proptest = "1"` is a **regular dependency** (`Cargo.toml:33`) because the
compiler emits `proptest!` blocks and `spk verify` executes them. It is not only
a dev-dependency.

| File | Blocks | What it pins | Distribution limits relevant to this pass |
|---|---|---|---|
| `tests/acset_writer_identity.rs` | 1 (`identity_emission_exact`, 64 cases) | `emit(parse(x)) == x` for generated spec-shaped text | Directly tests the near-tautological half of the writer; spans are verified as a side effect. |
| `tests/acset_writer_apply.rs` | 1 (`roundtrip_faithful`, 64 cases) | `apply` row rename reparses faithfully | Local ids are always `r{pick}` → `renamed{pick}`; `kind` columns are always literal `invariant`. The keyword-collision case is outside the distribution. |
| `tests/rename_writer_parity.rs` | 1 (`rename_writer_parity_exact`, 64 cases) | `run` vs `run_via_writer` byte parity | At HEAD `run` **delegates to** `run_via_writer` (`src/rename.rs:65-73`), so this is a self-comparison. See `properties/rename-writer-parity.md`. |
| `tests/acset_parity.rs`, `tests/acset_query.rs`, `tests/fragment_helpers.rs`, `tests/law_case_bodies.rs` | several | acset/graph parity, closure, fragments, law cases | Adjacent coverage; not the laws catalogued here. |

Checked-in regression seed: `tests/rename_writer_parity.proptest-regressions`.

Other fixed-example coverage: `tests/acset_writer_spans.rs` (span presence,
CRLF, non-serialization), `tests/acset_writer_parity.rs` (rename write-set byte
snapshots under `tests/snapshots/rename_baseline/`), and `tests/merge_flow.rs`
(subprocess tests for collision, clean merge, rename replay, modify/delete in
both directions). These are examples, not generated coverage.

## Generated product artifacts (out of scope)

`specodelic/*_props.rs` contains the compiler's output, including
`merge_props.rs`. Its strategies are constant `Just("…")` values — e.g.
`two_branches_each_independently_defining_new_id` — so a green run is not
evidence that the underlying scenario was generated or explored. This is
documented here for context only; the generated artifacts and `spk verify`
remain on proptest and are not part of this catalog's adoption case.

## Implication for evidence files

Each property's evidence file carries an "Existing instrumentation" line:
- **missing** — no test checks this law;
- **partial** — a fixed example or a narrower proptest block touches it;
- **gate** — the implementation itself checks the predicate internally, so the
  property is near-tautological for successful results.
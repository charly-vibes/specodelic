# Rule of 5 review — specodelic-lf3 compile property migration

Work reviewed: the diff of `openspec/specs/compile/spec.md` (property cells),
`src/lib.rs` (fixtures module), `justfile` (verify-specs recipe + ci stage),
`tests/compile_fixtures.rs` (new), and the regenerated
`specodelic/compile_props.rs` + `specodelic/compile.check.json`.

## Stage 1: DRAFT

Assessment: the shape is right — every one of the 35 property rows keeps its
original prose claim and gains exactly one executable `**rust:**` fragment; the
input surface is a single fixture registry in `src/lib.rs`; the CI gate is
scoped to one corpus file.

Shape quality: GOOD

## Stage 2: CORRECTNESS

[CORR-001] HIGH — `openspec/specs/compile/spec.md`, row `roundtrip_stable`
Description: the fragment's expect message claimed the tool ran "its own TOML
re-parse and re-emit gate" — no such gate exists (no `idempot`, `re-emit` or
`reparse` seam in `src/`), and the fragment compiled the same parsed spec
twice, which is determinism, not the claim's round trip.
Evidence: `grep -rn "idempot\|re-emit\|reemit\|deterministic" src/` returns no
compiler-side hit; `src/compile.rs` exposes no TOML re-parse entry point
(`constraints_to_toml` is one-way).
Recommendation: make the message honest and run the cycle the tool actually
exposes — re-parse the spec text, compile again, compare the three artifacts
byte for byte — and say in the cell that the tool exposes no re-parse of a
compiled artifact, so that half of the claim is checked at the spec level.
Status: FIXED.

[CORR-002] MEDIUM — row `id_preservation_holds`
Description: the fragment seeded `artifact_ids` with `spec.intent.id` before
the containment check, so the intent-id half of "ids_in(file) ⊆
ids_in(compile(file))" could never fail.
Evidence: the original fragment began `let mut artifact_ids: Vec<String> =
vec![spec.intent.id.clone()];` and then asserted every id was in that vector.
Recommendation: derive the ids from the artifacts (the props' `// id:`
comments, the TOML's `id =` lines, the TLA's text) and check the intent id
against the TOML's `source` field.
Status: FIXED.

[CORR-003] MEDIUM — row `output_function_covers_emitting_states_only`
Description: the domain check scanned the whole text after `Output ==` with
`contains`, which cannot distinguish an entry from an incidental mention of a
state id.
Evidence: the original fragment asserted `out.contains(state.id.as_str())` for
every non-emitting state over an unbounded tail.
Recommendation: extract the function's keys exactly (split on `" :> "`,
unquote) and compare the key list to the emitting states; check each value
against the IR's `emits_values`.
Status: FIXED.

[CORR-004] LOW — row `toml_roundtrips`
Description: the fragment asserted the emitted TOML contains the row's fields
but not that the row appears exactly once, so a duplicated entry would pass.
Recommendation: also require exactly one `[[constraints]]` entry.
Status: FIXED (cheap).

[CORR-005] LOW — row `kernel_expr_extracts`
Description: the tri-state membership test was trivially true for every
current variant of `ThreeValued` — it could not fail.
Evidence: `matches!(value, Verified) || matches!(value, Counterexample) ||
matches!(value, Unknown)` over a three-variant enum.
Recommendation: pin the fixture's actual verdict — probe-confirmed `Verified`
for `|State| == 2` over two states.
Status: FIXED (cheap).

Correctness quality: GOOD (after the fixes)

## Stage 3: CLARITY

[CLAR-001] LOW — row `precondition_violation_rejected`
Description: the fragment gives the fixture a synthetic file path
(`spec.path = Some(...)`) because the precondition gate matches lint findings
by file label, and the lint labels a pathless spec `<id>` — the reason for the
synthetic identity was invisible in the cell.
Impact: a reader could read the path assignment as a fudge.
Recommendation: say it in the cell's prose.
Status: FIXED (cheap).

[CLAR-002] LOW — `justfile`, recipe `verify-specs`
Description: an empty verify output produced a JSON traceback instead of a
diagnosis.
Recommendation: print an explicit message and exit 1.
Status: FIXED (cheap).

Clarity quality: GOOD

## Stage 4: EDGE CASES

[EDGE-001] LOW — `tests/compile_fixtures.rs`
Description: `generator_names` restates the compiler's private helper, so the
test can drift from the compiler.
Impact: a compiler change to the generator-name grammar would not be caught
by the correspondence test.
Scenario: the compiler's generator-name grammar widens or narrows.
Recommendation: tracked — the compiler's helper is private; the restatement
carries a comment saying so.
Status: TRACKED (not cheap — the alternative is widening the compiler's API).

[EDGE-002] LOW — `tests/compile_fixtures.rs`
Description: the correspondence test pins the corpus at 35 property rows.
Impact: adding a row fails the test until the registry gains a fixture.
Scenario: the next corpus migration adds a row.
Recommendation: tracked — the tripwire is deliberate: it forces the registry
to be updated with the corpus.
Status: TRACKED (deliberate).

Edge case coverage: GOOD

## Stage 5: EXCELLENCE

No further findings.

Excellence assessment:
- Structure: GOOD
- Correctness: GOOD
- Clarity: GOOD
- Edge Cases: GOOD
- Overall: GOOD

Production Ready: YES

## Post-fix evidence

- `cargo test` — 47 test binaries, 0 failures
- the regenerated artifact's blocks — 35 passed, 0 failed
- `just verify-specs` — exit 0
- `cargo fmt --check` — clean
- `cargo clippy -- -D warnings` — clean
- `spk lint openspec/specs` — 0 issues
- `ah check` — 0 findings

# Tasks: add-verify-runner-seams

## 1. P1 — the tool-dependency seam (RED → GREEN)

- [x] 1.1 RED: `tests/verify_runner_seams.rs` — a property row whose
      `**rust:**` predicate fragment calls `specodelic::lint::lint_corpus`;
      the properties gate is `Uncompilable` ("unresolved crate
      `specodelic`") before the seam.
- [x] 1.2 GREEN: `src/verify.rs` — the scratch manifest gains
      `[dependencies] specodelic = { path = "<crate root>" }` exactly when
      the staged artifact references `specodelic::`; the tool root is
      `env!("CARGO_MANIFEST_DIR")` with the `SPECODELIC_TOOL_PATH`
      override; a needed-but-absent tool root is a labeled runner error.
- [x] 1.3 The scratch lockfile is seeded from the repository's
      `Cargo.lock` when the tool dependency is needed (offline-friendly
      resolution).
- [x] 1.4 GREEN: the same test now reports `Pass` — the fragment calls
      the real linter and the real rule id is observed.

## 2. P2 — the fixture API (RED → GREEN)

- [x] 2.1 RED: `tests/verify_runner_seams.rs` — the fixture registry
      resolves a fixture name to a concrete spec text that the real
      parser accepts; the emitted strategy materializes through the
      registry instead of `Just("<name>")`.
- [x] 2.2 GREEN: `src/lib.rs` — the public, documented `fixtures`
      module (`registry`, `lookup`, `materialize`, `names`, `is_known`)
      with the shipped fixtures (`property_row_with_one_property`,
      `spec_file_with_undeclared_transition_target`).
- [x] 2.3 GREEN: `src/compile.rs` — a generator whose name is a fixture
      name yields `specodelic::fixtures::materialize("<name>")`; every
      other generator keeps today's `Just("<name>".into())` emission
      byte-for-byte.

## 3. P3 — the properties-only scope verdict (RED → GREEN)

- [x] 3.1 RED: `tests/verify_runner_seams.rs` + `src/verify.rs` unit
      tests — a failed block reports `properties_failed`; a passing
      properties gate reports a non-`verified` status; a missing or stale
      model run never reports `properties_failed`.
- [x] 3.2 GREEN: `src/verify.rs` — `properties_only_verdict` (and the
      `VerifyScope` enum), sharing the properties branch with the full
      verdict so the two cannot drift; the full verdict's precedence and
      `both_gates_required` semantics are unchanged.
- [x] 3.3 `justfile` — the `verify-properties` recipe gates on the
      envelope's per-file `properties.state` (`"failed"` ⟺ a block
      failed); it never fails on a skipped or stale model stage.
- [x] 3.4 The scoped verdict lives in the crate-root module
      `specodelic::scope_verdict` (re-exported from `src/verify.rs`) —
      the `pretender.toml` `file_lines_max = 2300` ratchet forbids
      growing `src/verify.rs` past the cap, and `pretender.toml` is
      outside the ticket's allowed file set (design D3).

## 4. The contract change (dual-format)

- [x] 4.1 `openspec/changes/add-verify-runner-seams/specs/verify/` — the
      delta (the properties-only scope requirement + its scenarios) and
      the layer sidecar (the full post-change layer: the new Constraint
      and its deriving Property).
- [x] 4.2 `openspec/changes/add-verify-runner-seams/specs/compile/` — the
      delta (the mirrored scenario of the migrated row) and the layer
      sidecar (the full post-change layer with the migrated row).
- [x] 4.3 `.espectacular/{verify,compile}/*.toml` — contract TOMLs for
      the new scenarios, bound to the tests that prove them.
- [x] 4.4 `just archive-change id=add-verify-runner-seams` — the corpus
      edit lands through the archive companion (never hand-edited).
- [x] 4.5 The demonstration: exactly one compile-spec property row
      migrated to a real tool-calling `**rust:**` fragment, proven by
      `corpus_property_row_executes_against_the_real_compiler` — the
      shipped corpus row (RED before the archive, GREEN after) runs
      against the real compiler, not against a hand-written fixture.

## 5. Gates

- [x] 5.1 `cargo fmt` before every commit.
- [x] 5.2 Targeted tests green (the new suite + the verify/compile unit
      tests).
- [x] 5.3 `spk lint openspec/specs` clean after the archive.
- [x] 5.4 `ah check` clean (no new findings).
- [x] 5.5 `just ci` green.
- [x] 5.6 `pretender check` green.

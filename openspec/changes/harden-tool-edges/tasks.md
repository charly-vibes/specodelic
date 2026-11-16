# Tasks: harden-tool-edges

## 1. Capability deltas

- [ ] 1.1 `specs/compile/spec.md`: ADDED requirement "Duplicate artifact
      stems refused before any write" (label, both paths, hint, exit 2,
      pre-lint/pre-write, unique stems unchanged, repeated path is one
      file) and "Scope refusal keeps the corpus notes".
- [ ] 1.2 `specs/verify/spec.md`: ADDED requirement "Artifact locations
      share one routing decision" (props source, check report, `.tla`
      modules, scope-digest hashes; per-file duplicate-stem reads; flat
      layout for unique stems).
- [ ] 1.3 `specs/audit/spec.md`: ADDED requirements "Merged evidence row
      keys are unique" (folded row yields the first free `~<n>`, oracle
      row keeps its id, both rows survive) and "Disambiguation is
      deterministic".

## 2. Dual-format layer sidecars

- [ ] 2.1 `specs/compile/layer.md`: full layer snapshot with
      `artifact_stem_unique` and `scope_refusal_envelope`, the
      `artifact_stem_unique` conjunct on the `begin` transition, and four
      deriving properties.
- [ ] 2.2 `specs/verify/layer.md`: full layer snapshot with
      `routed_artifact_resolution`, `no_cross_file_evidence`, and
      `unique_stem_layout_stable`, the `routed` state and `route`
      transition, and four deriving properties.
- [ ] 2.3 `specs/audit/layer.md`: full layer snapshot with
      `row_keys_unique` and `disambiguation_deterministic`, the `merged`
      state and `merge` transition, and three deriving properties.

## 3. Evidence pins (implemented behavior — no code change in this change)

- [ ] 3.1 compile scenarios trace to
      `tests/cli/compile.rs::duplicate_artifact_stem_is_a_labeled_scope_refusal_with_no_writes`
      (:157), `::repeated_path_argument_is_not_a_stem_collision` (:235),
      `::refusal_keeps_hostile_input_notes_in_the_envelope` (:285), and
      `::distinct_artifact_stems_compile_every_file_with_exit_0` (:345).
- [ ] 3.2 verify scenarios trace to
      `tests/verify_runner_seams.rs::verify_stage_artifact_paths_route_duplicate_stems_per_file`
      (:375) and `::verify_stage_artifact_paths_stay_flat_for_unique_stems`
      (:459); the properties arm is the shared `routed_artifact_dir` call
      at src/verify.rs:615.
- [ ] 3.3 audit scenarios trace to
      `oracle_row_id_colliding_with_a_bind_row_key_never_duplicates_scenario_ids`
      (src/audit.rs:2172) and the `fold_rows` seam it drives (:449).
- [ ] 3.4 Every observable named in a requirement (label string, exit
      codes, path forms, suffix scheme) is read from source, not inferred:
      src/commands/corpus.rs:630-686, src/verify.rs:339-375, :438, :615,
      src/audit.rs:425-476.

## 4. Validation and archive

- [ ] 4.1 `openspec validate harden-tool-edges --strict` passes.
- [ ] 4.2 Each sidecar lints clean: `spk lint` over
      `openspec/changes/harden-tool-edges/specs/<cap>`, plus
      `just lint-specs` for the corpus.
- [ ] 4.3 `just archive-change id=harden-tool-edges` deploys the deltas and
      re-attaches each sidecar fail-closed; the deployed
      `compile`/`verify`/`audit` specs carry the new requirements and
      layer rows.

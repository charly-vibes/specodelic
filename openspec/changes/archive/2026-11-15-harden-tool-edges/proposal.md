# Change: Specify the implemented compile, verify, and audit edge behavior

## Why

Three edge behaviors have landed in code and are pinned by tests, but they
exist in no capability spec:

- `spk compile` refuses a corpus whose distinct files produce one artifact
  stem — `artifact_stem_collision` (src/commands/corpus.rs:630, refused at
  :673, label at :649) — tested by
  `tests/cli/compile.rs::duplicate_artifact_stem_is_a_labeled_scope_refusal_with_no_writes`
  (:157), `::repeated_path_argument_is_not_a_stem_collision` (:235),
  `::refusal_keeps_hostile_input_notes_in_the_envelope` (:285), and
  `::distinct_artifact_stems_compile_every_file_with_exit_0` (:345).
- `spk verify` resolves every artifact location through
  `citation_corpus::artifact_dir` — `routed_artifact_dir`
  (src/verify.rs:339), used by the model gate (:373), the scope digest's
  per-file hashes (:438), and the properties gate (:615) — tested by
  `tests/verify_runner_seams.rs::verify_stage_artifact_paths_route_duplicate_stems_per_file`
  (:375) and `::verify_stage_artifact_paths_stay_flat_for_unique_stems`
  (:459).
- `spk audit` disambiguates merged row keys — `fold_rows`
  (src/audit.rs:449) called from `run` (:818) — tested by
  `oracle_row_id_colliding_with_a_bind_row_key_never_duplicates_scenario_ids`
  (src/audit.rs:2172).

None of `openspec/specs/compile/spec.md`, `verify/spec.md`, or
`audit/spec.md` states any of this. Each behavior is exactly the kind a
plausible refactor ("simplify the path join", "recompute the row maps",
"warn instead of refuse") would remove while every existing requirement
still passes — the spec would stay green on behavior the tool no longer
has.

## What Changes

- **compile** — two ADDED requirements: duplicate artifact stems are
  refused with `duplicate_artifact_stem`, both colliding paths, a
  remediation hint and exit 2 before lint/compile/write, with unique
  stems and repeated paths unaffected; the refusal re-attaches the
  corpus build's parse/hostile-input notes.
- **verify** — one ADDED requirement: the properties source, check
  report, `.tla` modules and scope-digest hashes all resolve through the
  one shared `artifact_dir` routing decision, duplicate-stem corpora read
  per file with no cross-reads, and corpus-unique stems keep the flat
  layout byte-identically.
- **audit** — two ADDED requirements: the merged report never carries two
  rows with one `scenario_id` (the folded `bind:`/`pack:` row yields with
  the first free `~<n>` suffix while the oracle row keeps its corpus id
  verbatim, both rows surviving), and that disambiguation is
  deterministic for a given corpus.
- One `layer.md` sidecar per capability: the full dual-format layer
  snapshot (existing tables plus the new constraint/model/property rows),
  ready for the archive companion to deploy verbatim.
- No behavior changes: this is a specification-only change. No code,
  exit-code, or output change is proposed.

## Impact

- Affected specs: `compile`, `verify`, `audit` — five new requirements
  under `## ADDED Requirements` (compile 2, verify 1, audit 2), no
  existing requirement modified.
- Affected code: none. The guarded behavior is already implemented at
  the sites cited above; the tests cited above are the acceptance pins.
- Artifacts: `openspec/changes/harden-tool-edges/specs/{compile,verify,audit}/{spec.md,layer.md}`.
- Consumers: the archive companion (`just archive-change`) deploys each
  sidecar's tables onto the deployed capability spec; the deployed layer
  gains the new rows fail-closed at archive time.

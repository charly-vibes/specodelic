# Command Reference

**Exit codes** (uniform across verbs): `0` = success (lint with zero
findings counts); `1` = the stage produced findings or a tool-level
failure; `2` = invocation error — nothing was processed (path not found,
no spec files matched, unreadable input). clap argument-parse failures
also exit 2. The JSON envelope's `envelope_kind` is `"error"` whenever
the exit code is nonzero-by-invocation.

All commands emit through the genesis envelope: JSON for pipes,
human-readable for TTYs (`--human` / `--format json` to override).
Failures carry a remediation hint.

## `spk lint <files|dirs>`

The gate. Runs every rule in the catalog over spec files (recursively;
files without frontmatter are skipped with a warning). Each finding
carries `rule_id` (`linter.<name>`) and `rule_semantics`.

- Spec: [linter rules](specs/linter-frontmatter.md) (one file per rule family)

Ingestion is hostile-input hardened (specodelic-suz): only regular files
under a 2 MiB cap are ever read — a FIFO, device file, or oversized file
named `*.md` is labeled, named, and skipped (never blocking or
unbounded-memory); when nothing else was linted, the labeled notes ride
the failure envelope.

## `spk graph <files|dirs>`

Derives the reference graph from `[[wiki-links]]`: nodes, edges, fan-in/
fan-out per row, dangling references. The rename/merge/refactor
advisors build on it.

- Spec: [graph](specs/graph.md)

## `spk new <id>`

Scaffolds a spec file from the template — per-layer HTML-comment
guidance (valid kinds, what a guard may cite, law-case requirements),
rendered from the same constants the linter enforces.

## `spk compile <files> --out-dir <dir>`

Compiles a lint-clean spec into three artifacts: `<stem>.toml`
(Constraints), `<stem>_props.rs` (proptest scaffolding), `<stem>.tla`
(TLA+ module). Total or labeled failure — never a silent partial result.

- Spec: [compile](specs/compile.md)

## `spk model-check <files>`

Runs the compiled model through the embedded stateright backend within
a stated bound (`--max-depth`, `--max-states`, `--timeout-secs`).
The native backend interprets guards as prose and executes no invariant
predicates, so it reports `exploration_only` (space exhausted within the
bound) or `timed_out` — never `no_counterexample`, which is reserved for
a backend that actually executed invariant predicates; run reports
persist as `<stem>.check.json` with the consumed module's SHA-256.

- Spec: [model_check](specs/model_check.md)

## `spk verify <files>`

Executes the emitted proptest scaffolding and rejects stale clean
results via the `.check.json` hash.

Both gates must hold — `verified` is the conjunction, never a single
green stage:

- **Properties gate** — the `*_props.rs` artifact is staleness-checked
  against the current spec by block-metadata fingerprint (ids, cases,
  generators, predicates — hand-translated predicate bodies keep the
  fingerprint, spec edits change it), then really executed: the artifact
  is staged into a scratch cargo crate and run via `cargo test`, so an
  un-translated `todo_predicate!` fails honestly (`properties_failed`),
  never skipped. Failing blocks report proptest's shrunk minimal input.
- **Model gate** — the `<stem>.check.json` run report must be current
  (its `artifact_sha256` matches the on-disk `<stem>.tla`; a missing
  module fails closed as stale) and its outcome must be
  `no_counterexample` — which the native backend never reports
  (`exploration_only` is explicitly not clean).

`.data.status` is `verified` exactly under the conjunction, else the
first blocking stage (`missing_properties_artifact`,
`stale_properties_artifact`, `properties_failed`,
`properties_uncompilable`, `runner_unavailable`, `missing_model_run`,
`stale_model_run`, `model_not_clean`). Verify never re-compiles and
never re-runs the model checker.

- Spec: [verify](specs/verify.md)

## `spk doctor`

Classifies the workspace (`self_hosting` vs `consumer`), checks the
local corpus's `Revision N` against the binary's embedded
`format_revision` — warns, never fails, on lag.

## `spk explain [topic]`

The embedded format guide: `format`, `ears`, `kinds`, `references`,
`lifecycle`, `lint-rules`. No repository access required.

## `spk init`, `spk feedback`

`spk init` writes/refreshes the `SPECODELIC` managed block in the
repo's `AGENTS.md` (rule catalog + format revision). `spk feedback`
opens a prefilled feedback channel for the tool.

## `spk hooks install`, `spk hooks uninstall`

`spk hooks install` wires the dual-format gate (`spk lint openspec`)
into the repo's pre-commit chain as a marker-guarded managed block in
`lefthook.yml` — purely additive (beads' `core.hooksPath` → lefthook
chain keeps flowing), idempotent, indentation inferred from the
stage's own children. Install runs the gate once and reports the
outcome (`data.outcome`, `data.gate_dry_run`); a failing gate is a
warning with the failure summary and an escape hint (`spk hooks
uninstall`), never a commit trap. Requires a lefthook config (never
created from scratch) and an `openspec/` tree; husky and prek repos
get a labeled refusal with a manual-wiring hint. `spk hooks
uninstall` strips only the managed block.

## `spk rename <old_id> <new_id> [files|dirs]`

The atomic rename (specs/rename.md): updates the defining row (an
Intent frontmatter id or a table row's qualified id) and every
referencing `[[link]]` — including child refs like `[[old_id.child]]` —
as one transaction. A file whose Intent id is renamed also gets its
filename renamed per the `-` ⇔ `.` naming law. Everything is computed
and verified in memory (re-parse + zero dangling) before any byte is
written, so a failed rename leaves the repo byte-identical: collisions,
unknown ids, and verify-gate rejections are labeled failures with
remediation hints.

- Spec: [rename](specs/rename.md)

## Pipeline stubs

`spk merge`, `spk refactor`, `spk orchestrate` are fully specced
([merge](specs/merge.md), [refactor](specs/refactor.md),
[orchestrate](specs/orchestrate.md)) and currently exit non-zero with
hints until implemented.

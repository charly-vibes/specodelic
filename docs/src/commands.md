# Command Reference

All commands emit through the genesis envelope: JSON for pipes,
human-readable for TTYs (`--human` / `--format json` to override).
Failures carry a remediation hint.

## `spk lint <files|dirs>`

The gate. Runs every rule in the catalog over spec files (recursively;
files without frontmatter are skipped with a warning). Each finding
carries `rule_id` (`linter.<name>`) and `rule_semantics`.

- Spec: [linter rules](specs/linter-frontmatter.md) (one file per rule family)

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
Reports `no_counterexample` or `timed_out`; run reports persist as
`<stem>.check.json` with the consumed module's SHA-256.

- Spec: [model_check](specs/model_check.md)

## `spk verify <files>`

Executes the emitted proptest scaffolding and rejects stale clean
results via the `.check.json` hash.

- Spec: [verify](specs/verify.md) *(in implementation — specodelic-1pv)*

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

## Pipeline stubs

`spk rename`, `spk merge`, `spk refactor`, `spk orchestrate` are fully
specced ([rename](specs/rename.md), [merge](specs/merge.md),
[refactor](specs/refactor.md), [orchestrate](specs/orchestrate.md)) and
currently exit non-zero with hints until implemented.

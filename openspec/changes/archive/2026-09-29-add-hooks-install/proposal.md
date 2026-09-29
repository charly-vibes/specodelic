# Change: `spk hooks install` — wire the dual-format gate as a lefthook managed block

## Why

The repo's pre-commit path is owned by beads (`core.hooksPath` →
`.beads/hooks`), whose shims chain to lefthook; `lefthook.yml` is the
sanctioned extension point (see this repo's `lefthook.yml` header and the
AGENTS.md sibling-tool hard blockers). genesis-vibes 0.8 shipped
`genesis::git_hooks` (marker-guarded install, ownership detection,
`lefthook::ensure_wired`), unblocking specodelic-cxr. But there is no
`spk` command that wires the specodelic dual-format gate into a repo's
hook chain — dual-format drift is currently caught only when CI runs,
after the commit has landed.

## What Changes

- New CLI surface: `spk hooks install` and `spk hooks uninstall` —
  inject/remove a marker-guarded managed block into the `pre-commit:`
  stage of the repo's lefthook config, wiring the gate command
  `spk lint openspec` (the dual-format linter, self-contained — no
  external openspec CLI or python needed at hook time).
- Install-time gate dry-run: `install` runs the gate once and reports
  the outcome over the envelope — a repo whose openspec tree would
  fail immediately gets a warning with the failure summary and an
  `spk hooks uninstall` escape hint, never a trap.
- **Two-case anchor strategy** (grounded empirically): lefthook errors on
  duplicate `commands:` keys (`yaml: unmarshal errors: mapping key
  "commands" already defined`), so when the stage already has a
  `commands:` mapping the entry is inserted *inside* it; when it does
  not, genesis `lefthook::ensure_wired` injects the full wrapper.
- Hard-constraint compliance: never claims `core.hooksPath`, never
  writes `.git/hooks/*` or `.beads/hooks/*` hook files, never replaces
  or reorders existing commands — wiring is purely additive and
  marker-guarded, so the beads → lefthook chain keeps flowing.
- New capability spec `hooks` (dual format) under `openspec/specs/`.
- Follow-up filed upstream: genesis should grow a command-level anchor
  (`ensure_command_wired`) so consumers don't re-implement the
  inside-mapping insertion (see tasks).

## Impact

- Affected specs: new capability `hooks`
- Affected code: new `src/hooks.rs`, CLI dispatch in `src/main.rs`,
  CHANGELOG/README/USAGE command tables; `lefthook.yml` of any repo the
  user runs `spk hooks install` in (dogfood: this repo's own
  `lefthook.yml`)
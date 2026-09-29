## 1. Design / grounding (done pre-approval)

- [x] 1.1 Verify genesis 0.8 `git_hooks` primitives cover the constraint set (resolve_hooks_dir, Owner::Bd sigils, framework(), marker-guarded install/uninstall, lefthook::ensure_wired/is_wired)
- [x] 1.2 Empirical test: lefthook 1.13.6 rejects duplicate `commands:` keys → two-case anchor strategy (design Decision 1)
- [x] 1.3 Scaffold proposal/design/delta; `openspec validate add-hooks-install --strict` + `spk lint openspec` + section-sync green

## 2. Hooks module (TDD)

- [x] 2.1 Red: `src/hooks.rs` unit tests on fixture configs — anchor
  rule defined first (design Decision 1): children indent inferred from
  the stage's first child key (2 spaces for an empty stage); entry
  inserted inside existing `commands:` mapping at indent+2 (existing
  entries unchanged, no duplicate key); full-wrapper path delegates to
  `lefthook::ensure_wired` at children indent; unanchorable
  `commands:` key / unascertainable children indent refused without
  modification; uninstall strips only the marked block; unwired
  uninstall is a no-op. Fixture matrix: 2-space config, 4-space
  config, no trailing newline, CRLF
- [x] 2.2 Green: implement install/uninstall around `genesis::git_hooks` (`repo_root`, `framework` gate per Decision 3, `BlockDef::with_markers` with `# ` comment-prefixed markers, `is_wired` verification); honest error labels + remediation hints on every refusal path
- [x] 2.3 Refactor: collapse duplication between the two wiring cases; docs comments cite the empirical duplicate-key finding and the AGENTS.md hard blockers
  - GREEN-LESSON: genesis `lefthook::ensure_wired` is NOT used for injection — red tests proved it glues the END marker onto the next existing line (its own tests pin `END  parallel: true`), which with comment-prefixed markers turns that line into a YAML comment and silently deletes the following key. Local injection for both cases; deviation documented in design.md wrapper_at + CHANGELOG #43; upstream consolidation is specodelic-x56

## 3. CLI wiring (TDD)

- [x] 3.1 Red: integration tests — `spk hooks install` in a fixture repo (with pre-existing lefthook config) reports wired outcome over the envelope and config stays valid YAML; missing config → error envelope + hint; `spk hooks uninstall` on unwired fixture → success no-op
- [x] 3.2 Green: `Hooks { action }` arm in `src/main.rs` dispatch through the genesis `Output::emit` envelope (value-enum `HooksAction`); completions pick it up automatically
- [x] 3.3 Fail-early pre-check: no `openspec/` directory → labeled error + hint (Decision 2)
- [x] 3.4 Red: install-time gate dry-run — outcome reported in the install envelope; failing gate → warning with failure summary + `spk hooks uninstall` escape hint, install still succeeds (design Decision 2, Rule-of-5 EDGE-001); green: implement, dogfood the warning against a not-yet-dual-format fixture tree
  - polish: envelope JSON output is single-line — `summarize_gate_output` recognizes the envelope and summarizes semantically (`clean (0 lint issues)` / `N lint issue(s): rules…`), truncating only non-envelope output

## 4. Docs + changelog

- [x] 4.1 README command table + Status section; docs/src/commands.md entry (the repo's command reference — USAGE.md has no command section, task adjusted)
- [x] 4.2 CHANGELOG entry (#43)

## 5. Gates + dogfood

- [x] 5.1 `just ci` green (fmt, clippy -D warnings, tests, release build, openspec strict, lint-deltas, section-sync, guard-siblings) — 82 unit + 46 integration
- [x] 5.2 Dogfood: `spk hooks install` in this repo → `lefthook.yml` gained the marker block inside the existing `pre-commit.commands` mapping (gate passes — corpus is dual-format, dry-run `clean (0 lint issues)`); `lefthook run pre-commit` parses and runs both sibling-blockers and specodelic-gates; uninstall → byte-identical restore → re-wired. PENDING USER DECISION: keep the wiring committed
- [x] 5.3 File genesis follow-up ticket: upstream `lefthook::ensure_command_wired()` (command-level anchor) so consumers stop re-implementing the inside-mapping insertion — specodelic-x56

## 6. Ship

- [ ] 6.1 Update `bd show specodelic-cxr` notes; close ticket on push
- [ ] 6.2 Commit + push (session completion protocol)
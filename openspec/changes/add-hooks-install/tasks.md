## 1. Design / grounding (done pre-approval)

- [x] 1.1 Verify genesis 0.8 `git_hooks` primitives cover the constraint set (resolve_hooks_dir, Owner::Bd sigils, framework(), marker-guarded install/uninstall, lefthook::ensure_wired/is_wired)
- [x] 1.2 Empirical test: lefthook 1.13.6 rejects duplicate `commands:` keys → two-case anchor strategy (design Decision 1)
- [x] 1.3 Scaffold proposal/design/delta; `openspec validate add-hooks-install --strict` + `spk lint openspec` + section-sync green

## 2. Hooks module (TDD)

- [ ] 2.1 Red: `src/hooks.rs` unit tests on fixture configs — anchor
  rule defined first (design Decision 1): children indent inferred from
  the stage's first child key (2 spaces for an empty stage); entry
  inserted inside existing `commands:` mapping at indent+2 (existing
  entries unchanged, no duplicate key); full-wrapper path delegates to
  `lefthook::ensure_wired` at children indent; unanchorable
  `commands:` key / unascertainable children indent refused without
  modification; uninstall strips only the marked block; unwired
  uninstall is a no-op. Fixture matrix: 2-space config, 4-space
  config, no trailing newline, CRLF
- [ ] 2.2 Green: implement install/uninstall around `genesis::git_hooks` (`repo_root`, `framework` gate per Decision 3, `BlockDef::with_markers` with `# ` comment-prefixed markers, `is_wired` verification); honest error labels + remediation hints on every refusal path
- [ ] 2.3 Refactor: collapse duplication between the two wiring cases; docs comments cite the empirical duplicate-key finding and the AGENTS.md hard blockers

## 3. CLI wiring (TDD)

- [ ] 3.1 Red: integration tests — `spk hooks install` in a fixture repo (with pre-existing lefthook config) reports wired outcome over the envelope and config stays valid YAML; missing config → error envelope + hint; `spk hooks uninstall` on unwired fixture → success no-op
- [ ] 3.2 Green: `Hooks { Install, Uninstall }` arm in `src/main.rs` dispatch through the genesis `Output::emit` envelope; completions pick it up automatically
- [ ] 3.3 Fail-early pre-check: no `openspec/` directory → labeled error + hint (Decision 2)
- [ ] 3.4 Red: install-time gate dry-run — outcome reported in the install envelope; failing gate → warning with failure summary + `spk hooks uninstall` escape hint, install still succeeds (design Decision 2, Rule-of-5 EDGE-001); green: implement, dogfood the warning against a not-yet-dual-format fixture tree

## 4. Docs + changelog

- [ ] 4.1 README command table + STATUS §3 row; USAGE.md command reference entry
- [ ] 4.2 CHANGELOG entry (next free number)

## 5. Gates + dogfood

- [ ] 5.1 `just ci` green (fmt, clippy -D warnings, tests, release build, openspec strict, lint-deltas, section-sync, guard-siblings)
- [ ] 5.2 Dogfood: run `spk hooks install` in this repo → `lefthook.yml` gains the marker block inside the existing `pre-commit.commands` mapping (gate passes — corpus is dual-format); `lefthook run pre-commit` parses and runs both sibling-blockers and specodelic-gates; then decide with the user whether to keep the wiring committed
- [ ] 5.3 File genesis follow-up ticket: upstream `lefthook::ensure_command_wired()` (command-level anchor) so consumers stop re-implementing the inside-mapping insertion

## 6. Ship

- [ ] 6.1 Update `bd show specodelic-cxr` notes; close ticket on push
- [ ] 6.2 Commit + push (session completion protocol)
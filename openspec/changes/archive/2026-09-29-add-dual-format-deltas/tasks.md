## 1. Policy & docs
- [x] 1.1 Rewrite the openspec↔specodelic paragraph in `openspec/project.md` (Purpose → "Relation to OpenSpec") to state the dual-format protocol instead of "don't conflate"
- [x] 1.2 Document the archive recipe (`--skip-specs` + verbatim copy) and the `id: spec` naming acceptance in the same section

## 2. Tooling
- [x] 2.1 Add `just openspec-validate` recipe: `openspec validate --strict` over every active change, non-interactive
- [x] 2.2 Add `just lint-deltas` recipe: `spk lint` over every dual-format delta file (file-by-file until `specodelic-6pi` recursion lands; recurse after) — note: 6pi landed first, so the recipe lints the whole `openspec/` tree recursively; only frontmatter'd files are checked
- [x] 2.3 Add `just archive-change id=<id>` recipe: `openspec archive <id> --skip-specs --yes` + copy each `specs/<cap>/spec.md` from the archive dir to `openspec/specs/<cap>/spec.md`
- [x] 2.4 Add section-sync check: `ADDED Requirements` section text == `Requirements` section text for every dual-format delta (small script invoked by `just ci`; fails with file + divergent requirement name) — `scripts/check_section_sync.py`, recipe `just sync-sections`
- [x] 2.5 Wire `openspec-validate`, `lint-deltas`, and the section-sync check into `just ci`

## 3. Pilot migration
- [x] 3.1 Convert `add-compile-functor`'s delta (`specs/compile/spec.md`) to dual format: add frontmatter (`id: spec`), Constraints/Model/Properties tables mirroring its engineering requirements; rename `## ADDED Requirements` block unchanged; add `## Purpose` + duplicate `## Requirements` section — NOTE: the change was archived before this task ran, so the pilot migrated the archived capability spec `openspec/specs/compile/spec.md` to dual format instead (frontmatter + tables + Purpose; Requirements verbatim) — the meaningful equivalent
- [x] 3.2 Verify the migrated delta passes `openspec validate add-compile-functor --strict` and `spk lint` (0 issues) — verified via `openspec validate compile --type spec --strict` + `spk lint`: both green; the other three capability specs (doctor, embedded-guide, lint-findings) were migrated the same way

## 4. Validation (this change dogfoods itself)
- [x] 4.1 `openspec validate add-dual-format-deltas --strict` passes (this delta is dual-format)
- [x] 4.2 `spk lint` on this delta: 0 issues
- [x] 4.3 Section-sync check passes on this delta (ADDED ≡ Requirements)
- [x] 4.4 `just ci` green with the new gates in place

## 5. Follow-up filing
- [x] 5.1 File beads issue (or mp1 row) for a potential `spk lint` rule that structurally recognizes dual-format files (frontmatter + `## ADDED Requirements` coexistence), so the pattern is enforced by the tool, not convention
# Tasks: mdBook docs site

## 1. Book skeleton
- [x] 1.1 `docs/book.toml` (single book manifest at `docs/` — src `src`, site-url `/specodelic/`, git-repository-url + edit-url-template; deliberately NOT the espectacular two-book.toml layout)
- [x] 1.2 `docs/src/SUMMARY.md`: Introduction, Getting Started (installation & quick start), Command Reference, Specs part A (18 domain corpus files under `specs/`), Specs part B (dual-format capability specs under `openspec/specs/`)
- [x] 1.3 `docs/src/index.md` (what specodelic is, the four layers, pipeline, pointer to `spk explain` for agents), `docs/src/installation.md`, `docs/src/commands.md`
- [x] 1.4 Static `llms.txt` at repo root (espectacular pattern: what it is, core concepts, command surface)

## 2. Build recipe
- [x] 2.1 `just docs-build`: copy `specs/` + `openspec/specs/` into `docs/src/`, `mdbook build docs`, copy `llms.txt` into `docs/book/`

## 3. Deployment workflow
- [x] 3.1 `.github/workflows/docs.yml`: push to main + workflow_dispatch; checkout → install mdBook (peaceiris/actions-mdbook@v2) → copy spec trees → `mdbook build docs` → cp `llms.txt docs/book/` → upload-pages-artifact → deploy-pages@v4; concurrency group `pages`; permissions contents:read, pages:write, id-token:write — **artifact-based deployment, no gh-pages branch** (user decision 2026-09-29)
- [x] 3.2 GitHub Pages — verified already enabled with `build_type: workflow` (`gh api repos/charly-vibes/specodelic/pages`); no manual step needed

## 4. Validation
- [x] 4.1 `just docs-build` succeeds locally; `docs/book/index.html` exists; all 22 spec pages render (18 corpus + 4 capability), zero mdbook warnings
- [x] 4.2 `just ci` still green (docs changes add no new gates; existing gates unaffected)
- [ ] 4.3 After merge: site live at https://charly-vibes.github.io/specodelic/ with llms.txt at root (verify on first workflow run)

## 5. Follow-up filing
- [ ] 5.1 File beads issue: CI check that every `specs/*.md` and `openspec/specs/*/spec.md` appears in `docs/src/SUMMARY.md` (dual-format section-sync pattern)

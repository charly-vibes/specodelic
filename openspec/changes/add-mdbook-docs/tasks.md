# Tasks: mdBook docs site

## 1. Book skeleton
- [ ] 1.1 `book.toml` at repo root (espectacular pattern: title specodelic, site-url `/specodelic/`, git-repository-url + edit-url-template, build-dir `docs/_book`)
- [ ] 1.2 `docs/src/SUMMARY.md`: Introduction, Getting Started (installation & quick start), Command Reference (lint, graph, new, compile, model-check, doctor, explain, init, feedback), Specs part A (18 domain corpus files under `specs/`), Specs part B (dual-format capability specs under `openspec/specs/`)
- [ ] 1.3 `docs/src/index.md` (what specodelic is, the four layers, pointer to `spk explain` for agents) and `docs/src/commands.md`
- [ ] 1.4 Static `llms.txt` at repo root (espectacular pattern: what it is, core concepts, command surface)

## 2. Build recipe
- [ ] 2.1 `just docs-build`: copy `specs/` + `openspec/specs/` into `docs/src/`, `mdbook build docs`, copy `llms.txt` into `docs/book/`

## 3. Deployment workflow
- [ ] 3.1 `.github/workflows/docs.yml`: push to main + workflow_dispatch; checkout → install mdBook (peaceiris/actions-mdbook@v2) → copy spec trees → `mdbook build docs` → cp `llms.txt docs/book/` → upload-pages-artifact → deploy-pages@v4; concurrency group `pages`; permissions contents:read, pages:write, id-token:write
- [ ] 3.2 Enable GitHub Pages (deploy from workflow) in repo settings — manual step, note in tasks

## 4. Validation
- [ ] 4.1 `just docs-build` succeeds locally; `docs/book/index.html` exists; spec pages resolve
- [ ] 4.2 `just ci` still green (docs changes add no new gates; existing gates unaffected)
- [ ] 4.3 After merge: site live at https://charly-vibes.github.io/specodelic/ with llms.txt at root

## 5. Follow-up filing
- [ ] 5.1 File beads issue: CI check that every `specs/*.md` and `openspec/specs/*/spec.md` appears in `docs/src/SUMMARY.md` (dual-format section-sync pattern)

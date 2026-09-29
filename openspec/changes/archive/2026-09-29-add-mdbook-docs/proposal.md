# Change: mdBook docs site on GitHub Pages

## Why

specodelic is the only charly-vibes tool without a public docs site.
Siblings (espectacular, pretender, vampiro) ship an mdBook on GitHub
Pages from a `docs.yml` workflow, and espectacular's book already
renders its capability specs from `openspec/specs/`. With the
dual-format protocol landed (add-dual-format-deltas), the spec corpus
and the engineering specs speak one grammar — the docs site can render
both from one source without a second authoring surface.

## What Changes

- Add an mdBook site under `docs/` (book.toml at repo root following
  the espectacular layout): introduction/quickstart, a command
  reference, and one page per spec file — the 18-file domain corpus
  under `specs/` and every dual-format capability spec under
  `openspec/specs/`.
- Add `docs.yml` (genesis/espectacular pattern): on push to `main`,
  copy the spec trees into the book source, `mdbook build`, copy
  `llms.txt` to the site root, deploy via `actions/deploy-pages@v4`
  at site-url `/specodelic/`.
- Add a static `llms.txt` (espectacular pattern) summarizing the tool
  for agents, and a `just docs-build` recipe for local builds.
- Add the `docs.yml` workflow deploy step behind the same gates the
  family uses (no new CI gate in this change; `just ci` is unchanged).

## Impact

- Affected specs: `docs-site`
- Affected code: new `docs/` tree, `book.toml`, `.github/workflows/docs.yml`,
  `justfile` (docs-build recipe), `llms.txt`

---
id: docs.site
kind: intent
statement: "THE docs system SHALL render the spec corpus and the command surface as an mdBook site deployed to GitHub Pages on every push to main."
---

# docs-site Specification

## Purpose
Give specodelic the family-standard public docs: an mdBook that renders
the introduction, command reference, and the spec corpus itself (domain
specs plus dual-format capability specs), served at
`charly-vibes.github.io/specodelic` with an `llms.txt` for agents.

## Constraints

| id              | kind      | expr                                                                                                                                          | traces_to |
|-----------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| process_lifecycle | invariant | `the capability advances through its declared lifecycle states under the repo's change process — each stage transition fires only when its stage gate holds` | [[docs.site]] |
| site_present    | invariant | `the repo carries an mdBook under docs/ with book.toml at the root, an index, and a command reference`                                          | [[docs.site]]  |
| specs_in_book   | invariant | `SUMMARY.md links every domain spec file under specs/ and every dual-format capability spec under openspec/specs/; specs are copied at build time, never vendored` | [[docs.site]]  |
| pages_deployed  | invariant | `pushing to main builds the book and deploys it to GitHub Pages at /specodelic/ via docs.yml (upload-pages-artifact → deploy-pages)`             | [[docs.site]]  |
| llms_txt_served | invariant | `the site serves an llms.txt tool summary at its root, copied into the built book`                                                               | [[docs.site]]  |

## Model

### States
- `authored`
- `built`
- `deployed`

### Transitions

| id            | from     | to       | guard                    |
|---------------|----------|----------|--------------------------|
| docs_build    | authored | built    | [[docs.site.site_present]]  |
| docs_deploy   | built    | deployed | [[docs.site.pages_deployed]]  |

## Properties

| id          | kind | derives_from            | generator             | predicate                                        |
|-------------|------|-------------------------|-----------------------|--------------------------------------------------|
| process_lifecycle_checked | unit | [[docs.site.process_lifecycle]] | `lifecycle_model_present()` | `check(file) == passed` |
| p_site      | unit | [[docs.site.site_present]]   | `fresh_checkout()`    | `docs-build() == ok ∧ index_html_exists`          |
| p_specs     | unit | [[docs.site.specs_in_book]]  | `arbitrary_spec_file()` | `SUMMARY.md links f ⇒ book page for f resolves`  |
| p_deploy    | unit | [[docs.site.pages_deployed]] | `push_to_main()`      | `pages_url == /specodelic/ ∧ book deployed`       |
| p_llms      | unit | [[docs.site.llms_txt_served]]| `site_fetch()`        | `GET /llms.txt == 200 ∧ body non-empty`           |

## ADDED Requirements

### Requirement: mdBook docs site
The repo SHALL carry an mdBook site under `docs/` — `book.toml` at the
repo root, `SUMMARY.md`, an introduction, and a command reference —
buildable locally via `just docs-build`, which copies the spec trees
into the book source, runs `mdbook build docs`, and places `llms.txt`
in the built book.

#### Scenario: Local build succeeds
- **WHEN** `just docs-build` runs in a fresh checkout with mdBook installed
- **THEN** the build exits 0 and `docs/book/index.html` exists

### Requirement: Spec corpus rendered
The book SHALL render one page per domain spec file under `specs/` and
one page per dual-format capability spec under `openspec/specs/`, with
`SUMMARY.md` linking every such page; the spec trees are copied at
build time and never vendored into `docs/src/`.

#### Scenario: Every corpus page reachable
- **WHEN** the book is built
- **THEN** every `specs/*.md` file and every `openspec/specs/*/spec.md`
  file is linked from `SUMMARY.md` and renders as a book page

### Requirement: GitHub Pages deployment
Pushing to `main` SHALL build the book and deploy it to GitHub Pages at
`/specodelic/` through the `docs.yml` workflow — mdBook installed via
`peaceiris/actions-mdbook@v2`, artifact uploaded via
`actions/upload-pages-artifact@v3`, deployed via `actions/deploy-pages@v4`.

#### Scenario: Push deploys the site
- **WHEN** a commit is pushed to `main`
- **THEN** the workflow builds the book and deploys it, and the site is
  served at `https://charly-vibes.github.io/specodelic/`

### Requirement: Agent summary served
The site SHALL serve an `llms.txt` summarizing the tool — what it is,
core concepts, command surface — at the site root, copied into the
built book by both the local recipe and the workflow.

#### Scenario: Agent fetches the summary
- **WHEN** `GET /llms.txt` is issued against the deployed site
- **THEN** the response is 200 with a non-empty body

## Requirements

### Requirement: mdBook docs site
The repo SHALL carry an mdBook site under `docs/` — `book.toml` at the
repo root, `SUMMARY.md`, an introduction, and a command reference —
buildable locally via `just docs-build`, which copies the spec trees
into the book source, runs `mdbook build docs`, and places `llms.txt`
in the built book.

#### Scenario: Local build succeeds
- **WHEN** `just docs-build` runs in a fresh checkout with mdBook installed
- **THEN** the build exits 0 and `docs/book/index.html` exists

### Requirement: Spec corpus rendered
The book SHALL render one page per domain spec file under `specs/` and
one page per dual-format capability spec under `openspec/specs/`, with
`SUMMARY.md` linking every such page; the spec trees are copied at
build time and never vendored into `docs/src/`.

#### Scenario: Every corpus page reachable
- **WHEN** the book is built
- **THEN** every `specs/*.md` file and every `openspec/specs/*/spec.md`
  file is linked from `SUMMARY.md` and renders as a book page

### Requirement: GitHub Pages deployment
Pushing to `main` SHALL build the book and deploy it to GitHub Pages at
`/specodelic/` through the `docs.yml` workflow — mdBook installed via
`peaceiris/actions-mdbook@v2`, artifact uploaded via
`actions/upload-pages-artifact@v3`, deployed via `actions/deploy-pages@v4`.

#### Scenario: Push deploys the site
- **WHEN** a commit is pushed to `main`
- **THEN** the workflow builds the book and deploys it, and the site is
  served at `https://charly-vibes.github.io/specodelic/`

### Requirement: Agent summary served
The site SHALL serve an `llms.txt` summarizing the tool — what it is,
core concepts, command surface — at the site root, copied into the
built book by both the local recipe and the workflow.

#### Scenario: Agent fetches the summary
- **WHEN** `GET /llms.txt` is issued against the deployed site
- **THEN** the response is 200 with a non-empty body

# Design — mdBook docs site

## Context

The family pattern is established: mdBook in `docs/`, `SUMMARY.md`
linking index/commands/specs, a `docs.yml` deploying `docs/book` via
`actions/deploy-pages@v4` on push to main (espectacular, pretender,
vampiro). specodelic's differentiator: the spec corpus is the product —
18 dual-format-ready domain specs plus 4 capability specs that are now
lintable specodelic files. The site's core value is rendering the
corpus itself.

## Decisions

- **Copy specs at build time, do not vendor them into `docs/src/`.**
  The workflow copies `specs/` and `openspec/specs/` into the book
  source before building (espectacular's `cp -r openspec/specs
  docs/src/specs`). One source of truth; the repo tree stays canonical.
- **`just docs-build` mirrors the workflow steps locally** so a docs
  failure is reproducible without pushing: copy → `mdbook build` →
  llms.txt into the book dir. CI keeps its own copy step (the workflow
  cannot depend on local state).
- **Static `llms.txt`, not generated.** A hand-maintained summary
  (espectacular pattern) is honest about curation; a generated one
  would need a new `spk` surface and a drift guard — deferred until
  there is evidence it drifts.
- **No summary/TOC generation tooling.** SUMMARY.md is hand-written
  and lists every spec file explicitly; a CI check that every spec file
  appears in SUMMARY.md is a candidate follow-up (like the dual-format
  section-sync check), not part of this change.

## Risks / Trade-offs

- Hand-written SUMMARY.md can silently miss new spec files → accepted
  for v1; the follow-up check is cheap once the page list stabilizes.
- mdbook version drift between local and CI → pinned via
  `peaceiris/actions-mdbook@v2` with `mdbook-version: latest` in CI and
  whatever the local install provides; byte-stability is not required
  for docs.

## Open Questions

- Should the domain corpus pages be grouped by layer (frontmatter /
  constraints / model / properties) or listed flat? → authoring-time
  choice for the SUMMARY.md, revisitable without spec changes.
- Should `spk explain` topics become book pages? → defer; the embedded
  guide and the book serve different audiences (agent-at-runtime vs
  human-browsing).

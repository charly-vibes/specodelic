# Change: Shared fragment definitions per file

## Why

The catitas kata studies (O15, bundle v2) showed fragments cannot share
helpers: any helper model must be copied into every row, which grew spec
size ~3× per revision and embedded the model table in properties twice
(44 KB of copies in the bowling study). Deduplicated definitions are the
single highest-leverage size fix the evidence names.

## What Changes

- New optional per-file `## Fragments` section: named shared definitions
  (`def <name>:` fenced rust bodies) that fragment cells may reference by
  name; emission inlines each referenced definition at each use site.
- Hygiene applies to helper bodies exactly as to fragment bodies (the
  substring-based token ban — with its known imprecision documented).
- Name collision with a generated symbol fails labeled (prefix rule), never
  silently shadows.
- Byte-preservation: files without a `## Fragments` section compile
  byte-identically (pure widening).
- The format addition is recorded in the `compile.layer` sidecar extension
  (fail-closed at archive time — the deployed compile layer is large, so the
  sidecar is authored/extended at implementation, not stubbed in the delta).
- Provenance: catitas item A5; conversations `claude/56d084d5…` (bundle v2/v3),
  `mistral-work/923c311c…`. Review finding EDGE-001 honored: rust-only,
  no py/ts scope creep — those remain parked
  (decisions/2026-10-20-park-py-ts-fragment-emission.md).

## Impact

- Affected specs: `compile` (ADDED requirement; no existing requirement
  modified).
- Affected code: `src/compile.rs` (extraction + emission), lint advisory
  (unused-helper warning on the warnings channel), docs.

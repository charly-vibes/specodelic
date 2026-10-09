# Status

**experimental** — early-stage; surface may be renamed or sunset. Version 0.7.0.

## Implemented

| Command | Status | Notes |
|---|---|---|
| `spk` (specodelic) | beta | lints, compiles, verifies, refactors markdown specs (alias `spk`) |
| spec format | beta | frontmatter, Constraints, state Model, Properties tables |
| verification claims | beta | every opted-in invariant claim (Rust, kernel, citation) must verify before a file verifies; prose-only invariants stay unchecked; claim reports are versioned and scope-bound — JSON, human, and persisted views name the same blockers |

## In progress

- **Self-hosting round** — the `specs/` corpus is specodelic's own
  specification, written in the format it defines, so the tool lints and
  verifies its own documentation the way it would a user's spec (the
  self-hosting-compiler analogy). The round keeps that corpus lint-clean
  as the format grows, making the tool's own spec the first consumer of
  every new rule.
- **Docs conformance round** — the project's documentation is held to a
  conformance standard like the specs' own: the mdBook site now builds
  from a single root `book.toml`, so one build renders the whole book —
  including the generated spec and openspec pages — and drift between
  the prose and the corpus it mirrors surfaces as a build or gate
  failure instead of rotting silently.

## Mapped to specs

- The format itself: every file in `specs/` is a spec in the format; `openspec` proposals for changes.

**Verification is not application testing** — the canonical statement of
what specodelic's verification does and does not assure lives in the repo
[README](../../README.md); this page keeps the capability table only and
does not restate the assurance semantics.

## Dogfooding

- **espectacular** installs specodelic from git in its CI (unpinned — to reconcile in DDL-1a0)
- specodelic's own CI runs `just ci` and validates its specs

# Status

**experimental** — early-stage; surface may be renamed or sunset. Version 0.5.2.

## Implemented

| Command | Status | Notes |
|---|---|---|
| `spk` (specodelic) | beta | lints, compiles, verifies, refactors markdown specs (alias `spk`) |
| spec format | beta | frontmatter, Constraints, state Model, Properties tables |
| verification claims | beta | every opted-in invariant claim (Rust, kernel, citation) must verify before a file verifies; prose-only invariants stay unchecked; claim reports are versioned and scope-bound — JSON, human, and persisted views name the same blockers |

## In progress

- Self-hosting round: `specs/` described in its own format.
- Docs conformance round (DDL-u8x epic) — book migrated to root `book.toml`.

## Mapped to specs

- The format itself: every file in `specs/` is a spec in the format; `openspec` proposals for changes.

**Verification is not application testing:** verification is bounded model
checking of opted-in invariant claims plus compiled property execution — not
a substitute for the application's own test suite.

## Dogfooding

- **espectacular** installs specodelic from git in its CI (unpinned — to reconcile in DDL-1a0)
- specodelic's own CI runs `just ci` and validates its specs

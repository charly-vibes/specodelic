# Status

**experimental** — early-stage; surface may be renamed or sunset. Version 0.1.0.

## Implemented

| Command | Status | Notes |
|---|---|---|
| `spk` (specodelic) | beta | lints, compiles, verifies, refactors markdown specs (alias `spk`) |
| spec format | beta | frontmatter, Constraints, state Model, Properties tables |

## In progress

- Self-hosting round: `specs/` described in its own format.
- Docs conformance round (DDL-u8x epic) — book migrated to root `book.toml`.

## Mapped to specs

- The format itself: every file in `specs/` is a spec in the format; `openspec` proposals for changes.

## Dogfooding

- **espectacular** installs specodelic from git in its CI (unpinned — to reconcile in DDL-1a0)
- specodelic's own CI runs `just ci` and validates its specs

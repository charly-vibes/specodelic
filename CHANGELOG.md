# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> The detailed, ticket-referenced release history lives in
> [`openspec/CHANGELOG.md`](openspec/CHANGELOG.md) — this root file tracks
> user-facing release summaries.

## [0.14.0] — 2026-11-15

### Added

- **Oracle expectations for conform** (`specodelic-f4sx`, `specodelic-18mt`) — scenario records accept an optional `expect` field; forbidden verdicts matching the declaration are agreement-tier and exit 0, unmet expectations stay gating with an `expectation_unmet` reason prefix
- **Vacuity advisories** (`specodelic-h5ks`) — two advisory lint rules: `linter.fragment_binds_nothing` and `linter.vacuous_invariant`
- **Hardened tool edges** — archive-companion and CLI input validation hardening (Rule-of-5 reliability round)
- README front-door pass

## [0.13.0] — 2026-11-15

### Changed

- Format revision 22 — evidence audit, typed-reference enforcement, revision upgrades (see `openspec/CHANGELOG.md` § #135)

## [0.12.0] — and earlier

See [`openspec/CHANGELOG.md`](openspec/CHANGELOG.md) for the full history
(v0.12.0 plain deltas + layer.md sidecars; v0.11.0 archive-companion
end-to-end ownership; v0.10.0 single-tree corpus release; and earlier).

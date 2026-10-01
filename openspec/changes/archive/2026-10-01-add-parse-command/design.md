# Design: add-parse-command

## Context

`spec::Spec` already derives `Serialize` and carries the complete structured
layer: `intent`, `constraints: Vec<Row>`, `states`, `transitions`, `properties:
Vec<Row>` (cells as `BTreeMap<String, String>`), `links: Vec<Link>`, plus the
dual-format drift bodies. The CLI needs a thin exposure, not new modeling.

Requested by espectacular (`derive-contracts-from-specodelic`, design D2) as
their upstream ask; replaces their task "0.4 file the upstream request" with
implementation.

## Decisions

### D1 — Thin wrapper over `spec::parse`; no new IR
`spk parse` parses with the same code path as `spk lint`/`spk compile` and
serializes the resulting `Spec` directly under `data`. No transformation, no
filtering: every structured field is emitted (consumers decide what they need).

### D2 — Envelope conformance
Output follows the shared `--json` envelope discipline: `ok`, `envelope_version`,
`cli_version`, `envelope_kind`, `data`, `warnings`, `hints`, `meta`. `hints`
suggests `spk lint <file>` — parse never implies validity.

### D3 — Parse is syntax-only; lint stays authoritative
A file that parses but fails `spk lint` still parses successfully. No lint
status in the envelope. Rationale: single-purpose envelopes; the refusing
consumer (`ah sync`) chains `spk lint` first, so embedding status would create
two sources of truth about validity. Verified acceptable by the consuming
design (D2 there: "two chained subprocess invocations").

### D4 — Error contract
Unparseable input → labeled error envelope (`ok: false`, error kind + message
+ remediation hint per the error-contract capability), non-zero exit. File-not-
found is a labeled error, not a panic.

### D5 — One file per invocation
No globbing, no bulk mode. Consumers loop; keeps the envelope small and the
error surface simple. Revisit if a consumer needs corpus export.

## Alternatives considered

- **Extend `spk compile --json` with the IR**: rejected — compile is about
  artifact generation; coupling parse output to it bloats the envelope and
  conflates syntax with compilation.
- **Embed lint status** (`lint_ok` field): rejected for now — see D3; can be
  added additively later without breaking consumers.
- **Have spk emit consumer-specific contracts** (espectacular TOMLs):
  rejected — couples spk to ah's schema; the typed IR is the neutral boundary.

## Migration

No migration: additive subcommand. espectacular pins the release containing
it and deletes its provisional markdown-table parsing plans (their tasks 2.1
already target the IR, not the tables).
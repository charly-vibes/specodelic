# Change: Add parse — structured Spec IR export (`spk parse <file>`)

## Why

The four-layer grammar is fully parsed in-process — `spec::Spec` is a typed,
`Serialize`-derived IR (intent, constraints, states, transitions, properties,
links) — but the only consumers who can touch it are in this repo. External
tools that need the structured layer today must either reimplement the
markdown-table parser (pipe escaping, wiki-links, whitespace rules — a silent
drift vector against format revisions) or scrape `spk compile --json`, which
embeds generated artifact *text* and exposes no Properties rows as structure.

The concrete driver is espectacular's `derive-contracts-from-specodelic`
design (D2): `ah sync` must read Properties rows it never re-parses, refuse to
derive from lint-dirty files, and consume a structured IR via the fleet's
versioned-binary boundary. A parse subcommand is the minimal exposure: the
lib work is done; only the CLI surface is missing.

## What Changes

- **Tool — `spk parse <file>`:** new subcommand emitting the parsed `Spec` IR
  as a `--json` envelope (`ok` / `data` / `warnings` / `hints`), conforming to
  the shared envelope and error contracts. Parse is syntax-only: it succeeds
  on files that fail `spk lint` and embeds no lint status — consumers chain
  `spk lint` themselves, keeping each envelope single-purpose.
- **Errors — labeled and hinted:** unparseable input produces a labeled error
  envelope with a remediation hint and non-zero exit (error-contract
  discipline).
- **Docs:** `USAGE.md` entry and a `guide` topic mention; no format changes.

## Capabilities

### Added requirements
- `parse` — Structured Parse Export

## Non-Goals

- Not embedding lint status in the parse envelope (espectacular's `ah sync`
  chains `spk lint` then `spk parse`; revisit if a consumer needs one-call).
- Not changing `spk compile` output or the format itself (still Revision 12).
- Not adding bulk/multi-file export; one file per invocation.

## Impact

- `src/main.rs` (subcommand registration), new thin `src/parse.rs` wrapping
  `spec::parse` + serde output. No parser changes, no schema changes, no
  format revision bump. Consumers: espectacular `ah sync` (upstream ask, their
  task group 1), any future fleet tooling.

## Risks and dependencies

- Envelope evolution pre-1.0: consumers must tolerate unknown fields and
  `data: null` (espectacular's bridge already does; their derivation tests
  assert it).
- Release coordination: espectacular pins the spk version that ships this.
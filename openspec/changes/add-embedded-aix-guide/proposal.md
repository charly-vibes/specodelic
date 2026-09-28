# Change: Embedded format guide (`spk explain`), self-describing lint findings, doctor dual-mode

## Why

The format's agent-experience thesis — agents write specs against a
lint/compile feedback loop — only holds if the agent can learn the
format from the tool itself. Today all format knowledge (the five EARS
patterns, the closed `kind` sets, the Reference Typing table, the
lifecycle) lives exclusively in this repo's `specs/*.md` corpus. An
agent in a **consumer repo** running the released `spk` binary has no
access to it: `rg include_str src/` is empty, `spk lint --help` teaches
nothing about *what* the invariants are, `spk new` scaffolds a skeletal
template, and `spk doctor` demands `specs/specodelic.md` on disk —
reporting "missing" on a perfectly valid consumer workspace. The tool
can check specs but cannot explain them.

## What Changes

- Add `spk explain [TOPIC]` — a distilled, machine-facing format guide
  embedded in the binary via `include_str!` and emitted through the
  normal JSON envelope. Topics: `format`, `ears`, `kinds`, `references`,
  `lifecycle`, `lint-rules`. Bare `spk explain` lists topics. Unknown
  topic fails with a remediation hint listing valid topics
- Add the embedded primer source `src/guide.md` — curated, deduplicated
  machine-facing content (closed sets, tables, patterns), not a copy of
  the human revision-archaeology corpus. Closed value sets are defined
  **once** as Rust constants consumed by both the parser/linter and the
  rendered guide, so drift is a compile/test failure, not a doc bug
- **`spk --version --json` reports the embedded format revision**;
  `explain` output carries the same revision so an agent can detect
  binary/corpus drift
- **Self-describing lint findings**: every finding carries its rule id
  (`linter.ears_syntax`) and a one-line semantics string, so the error
  itself is the documentation; `spk explain lint-rules` catalogs every
  rule id the binary can emit
- **`spk new` template teaches the format**: per-layer comment guidance
  (valid constraint kinds, what a guard may cite, law-case requirements)
  so the scaffold explains itself while being filled in
- **`spk doctor` dual-mode**: distinguishes a *self-hosting corpus*
  workspace (has `specs/specodelic.md`) from a *consumer workspace*
  (specs dir without the core spec); consumer mode no longer fails on
  the missing core spec and instead reports the embedded guide revision,
  warning when a local corpus declares a newer format revision than the
  binary embeds

Not in scope: implementing the still-specced pipeline commands
(`rename`, `merge`, `refactor`, `model-check`, `verify`, `orchestrate`).
Their stubs keep exiting non-zero with hints.

## Impact

- Affected specs: `embedded-guide`, `lint-findings`, `doctor` (new
  OpenSpec capabilities; the domain spec of record for the format itself
  remains `specs/specodelic.md` — this change embeds a *rendering* of
  it, it does not reinterpret it)
- Affected code: new `src/guide.rs` (+ embedded `src/guide.md`), thin
  wiring in `src/main.rs` (new subcommand, version payload, doctor
  checks), `src/lint.rs` (rule id + semantics on findings), `tests/cli.rs`
- Beads: consolidates the AIX-knowledge half of `specodelic-mp1`
  ("decide open spec questions before pipeline v1"); doctor consumer-mode
  overlaps nothing currently tracked — file `specodelic-*` on approval

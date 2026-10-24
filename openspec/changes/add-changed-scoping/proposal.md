# Change: Changed-path scoping for orchestrate's artifact-local stages

> **PARKED (2026-10-25, meter-first — simplification review):** the 642s
> datum behind this proposal is espectacular's `ah check` gate, NOT
> `spk orchestrate` — no measured specodelic slowness exists. Gate:
> the orchestrate-metering decision ticket. If metering shows wall-clock
> pain on this corpus, un-park as written (specodelic-523w remains the
> prerequisite).

## Why

The mistral Beck-critique study metered the verification-queue cost
(642s median pushes with `ah check --run-tests` in the sibling corpus —
espectacular#42) and named differential verification as the next
optimization frontier. `spk orchestrate` runs every stage over the whole
corpus, but compile/model-check/verify are artifact-local stages whose
inputs are single files — only lint must stay corpus-wide.

## What Changes

- `spk orchestrate --changed <paths...>`: restricts the artifact-local
  stages (compile, model-check, verify) to the changed paths; lint always
  runs corpus-wide (total_refs is corpus-wide — specodelic-0mn1 shows
  per-file lint can never be clean in self-hosting corpora), so scoping
  lint is explicitly out of scope and documented as such.
- Scope-guard discipline: the command's own scope guard prevents
  later-same-stem shadowing — **blocked by specodelic-523w** (compile's
  corpus runner lacks a scope guard); that bug is the prerequisite.
- Summaries unchanged in shape: the ladder-rung surfacing ticket
  (catitas B2) is the presentation layer that consumes this.
- Provenance: catitas item B3; conversations `mistral-work/a84fccc2…`
  (Beck-critique metering), `chatgpt/6acbbcb0…` (gate table context).
  Review finding CLAR-001 resolved: scoping covers artifact-local stages
  only.

## Impact

- Affected specs: `orchestrate` (ADDED requirement; existing orchestrate
  requirements unmodified).
- Dependencies: **blocked by specodelic-523w** (compile corpus-runner scope
  guard — the later-same-stem shadowing bug this command's scoped path would
  otherwise inherit); consumes Wave 2 B2 (specodelic-4m3y) as the summary
  presentation layer once that lands.
- Affected code: `src/orchestrate.rs` (path filter), `src/compile.rs`
  scope-guard prerequisite (523w), docs (decision-guide row).

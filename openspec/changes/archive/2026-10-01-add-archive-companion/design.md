# Design: archive-companion

## Context

The 2026-09-28 spike (openspec/changes/archive/2026-09-28-spike-dual-format)
verified that default `openspec archive` regenerates deployed capability
specs from parsed deltas and destroys the dual-format layer, and that
`archive --skip-specs` + verbatim cp preserves it. The just recipe has
run for real (docs-site, hooks, model-check, spec-integration, …) and
was hardened twice (idempotent, sort|tail-1 archive-dir resolution).

## Decisions

### D1 — companion flow over post-hoc restore

The command wraps the archive step (the recipe's exact sequence) rather
than offering `spk restore <cap>` that re-attaches a layer onto an
already-stripped deployed spec. Rationale: the companion path is
byte-exact by construction (the archived delta IS the deployed spec's
desired content); restore-from-stripped requires merging surviving prose,
recovering table rows from git history, and re-deriving the
ADDED↔Requirements mirror — fragile, unverifiable derivation. If a repo
is already stripped, the migration recipe (`spk explain dual-format`)
is the documented manual path. Consequence: the companion is the only
sanctioned archive path for dual-format repos; the capability-format CI
check catches any archive that bypassed it.

### D2 — `openspec` stays an external dependency

spk never re-implements openspec's archive logic (parsing deltas,
applying them, renaming directories). The command shells out to
`openspec archive <id> --skip-specs --yes`. A missing or failing
`openspec` binary is a labeled error with a remediation hint, never a
silent fallback. Rationale: re-implementing the merge logic would fork
openspec's semantics; the recipe already depends on the binary.

### D3 — fail-closed dual-format guard

Before copying a delta to a deployed spec, the module verifies the
specodelic layer: first line is `---` (frontmatter) and the body
contains a `## Constraints` header. A delta failing either check is
refused (no partial restore — the whole command fails, listing every
refused delta), with the remediation hint naming the migration recipe.
Rationale: the failure mode this capability prevents is *deploying a
stripped spec*; copying a plain delta would recreate exactly that
failure with the tool's blessing. Refusal beats repair.

### D4 — verbatim copy, no normalization

The archived delta is copied byte-identical to
`openspec/specs/<cap>/spec.md`. No re-formatting, no table
re-alignment, no prose edits. The section-sync CI check governs the
ADDED↔Requirements mirror; the linter governs everything else.
Rationale: byte-stability is a corpus invariant (deterministic_rerun);
"helpful" normalization would break archive-and-rebuild reproducibility.

### D5 — idempotent re-run

If `openspec/changes/<id>` does not exist but a matching
`openspec/changes/archive/*-<id>/` does, the `openspec` invocation is
skipped and the copy step re-runs. Rationale: the recipe's
`openspec list | grep` guard made re-runs safe; the tool preserves that
(always safe to re-run after a crashed or partially completed first run).

### D6 — newest archive dir wins

When multiple archive directories match `*-<id>` (a change id re-used
across archival cycles), the lexicographically greatest directory name
wins (the recipe's `sort | tail -1`). Envelope data reports the
directory used, so ambiguity is observable, never silent.

### D7 — dry-run resolves, never mutates

`--dry-run` performs resolution (change location, archive dir, delta
list, dual-format verification) and emits the same envelope with
`restored` replaced by the planned targets and a `dry_run: true` flag —
without invoking `openspec` and without writing. Rationale: the command
writes deployed specs; a no-mutation preview is the cheapest honesty
check, mirroring the hooks gate dry-run pattern.

### D8 — zero deltas is a success, not an error

A change with no `specs/*/spec.md` deltas (docs-only or CI-only change)
archives fine and restores nothing: envelope `restored: []`, exit 0.
Rationale: the recipe's glob-expansion failure on an empty set is a
shell wart, not a semantic error; "nothing to restore" is the honest
outcome. The envelope states it so nothing is implied.

## Risks / Trade-offs

- Depends on the `openspec` CLI at run time (D2) — accepted; CI and the
  recipe already do. Tests inject a seam rather than shell out.
- `--skip-specs` means openspec does not apply deltas itself; the
  verbatim copy replaces the deployed spec wholesale. This is correct
  for self-contained dual-format deltas (the corpus convention) and
  wrong for partial deltas — the proposal's Why cites the spike's
  self-containment requirement; a delta that omits pre-existing
  requirements would silently narrow the deployed spec. Accepted
  (convention-governed), surfaced in the spec's `delta_self_contained`
  constraint.

## Migration Plan

1. Land the module + CLI arm (TDD).
2. Switch `just archive-change` to delegate to the subcommand
   (interface unchanged: `just archive-change id=<id>`).
3. No corpus migration: deployed specs are already dual-format.

# Decision: the openspec-validate pin is retired — plain 1.x invocation

- **Date**: 2026-10-19
- **Status**: Accepted
- **Tickets**: beads specodelic-s8fg (task 6) · GH-20
- **Supersedes**: `2026-10-19-openspec-validate-pin.md`
- **Scope**: repo tooling (`scripts/spec_validate.py`, `justfile`,
  `.github/workflows/ci.yml`) — no change to `openspec/specs/` or
  `openspec/changes/` (read-only rule untouched)

## Context

The 0.19.0 pin existed for exactly one reason: the dual-format delta
shape (byte-identical `## Requirements` mirror + Specodelic
Constraints/Model/Properties tables) was *required* by `spk lint` and
rejected by `openspec 1.x --strict`, so the validate gate could not run
the sweep's 1.x validator without failing the corpus for format reasons
(see the superseded decision for the full anatomy).

That reason is gone:

- **Revision 19** (change `add-delta-layer-sidecar`, tasks 1–2) retired
  the dual-format delta class: changes carry plain deltas plus a
  `layer.md` sidecar (`id: <cap>.layer`), and the delta-pairing lint
  rules (`linter.dual_format_valid`, `linter.requirement_drift`'s
  delta-pairing arm) are removed.
- The corpus was migrated (task 5): all three remaining dual deltas are
  now plain delta + sidecar, and `openspec validate --all --strict`
  under `@fission-ai/openspec@1.14.2` passes **52/52 with the gate exit
  0** — verified through the collapsed script itself (task 5.3, re-run
  at task 6.1).

The fixture-proved residual risk — 1.14.2 hard-errors a zero-scenario
requirement — is a *validation-time content concern* that 1.x now
enforces on every plain delta directly (design.md Decision 4): the
pin's job minus the pin.

## Decision

`scripts/spec_validate.py` collapses to a **single plain invocation** of
`openspec validate --all --strict --no-interactive`:

- No version sniffing (`openspec --version` detection, caret matching),
  no `npx` fallback, no `PINNED_VERSION` runtime constant. The expected
  validator version is documented in the script docstring and the
  failure remediation hint.
- Exit code is the validator's exit code (CI safe, unchanged).
- The version-guard machinery and its fallback tests are deleted; the
  gate's tests assert the plain single invocation and the *absence* of
  the fallback (the fallback behavior no longer exists).

**Pinned validator: `@fission-ai/openspec@1.14.2`** — the version the
corpus validated green against (task 5's fixture gate uses it; record
the same pin in both places below).

Install contract — `openspec` on PATH must BE the pinned 1.x:

- **CI**: `.github/workflows/ci.yml` installs
  `@fission-ai/openspec@1.14.2` globally (bumped from 0.19.0).
- **Dev machines**: contributors must run
  `npm install -g @fission-ai/openspec@1.14.2` once. This is the
  documented home for that requirement (no new doc pages). Without it,
  a dev machine whose global `openspec` drifted to another version runs
  the gate against a different validator — the gate is content-honest
  again (its verdict tracks the corpus), but version drift is now the
  operator's responsibility, not the script's.

## Rejected alternatives

- **Keep a version guard that falls back to pinned 1.x via npx**: the
  guard existed to insulate the gate from ambient npm state when the
  ambient validator was format-incompatible with the corpus. With
  validator and corpus converged on 1.x, the guard adds sniffing
  complexity and a second code path for a drift that is now
  remediable by the documented install command.
- **Run the validator via npx unconditionally**: every gate invocation
  pays npx cache-resolution latency and couples the gate to node's
  package cache state; the global install is already a CI step.

## Consequences

- The org-wide openspec sweep (GH-20) is unblocked: dev machines and CI
  converge on the same pinned 1.x validator, and the corpus validates
  strict-clean under it.
- Version drift on a dev machine now surfaces as a validator verdict
  against a different openspec version, with the remediation hint
  naming the install command — visible, not silent.
- The 1.14.x shape fixture (`tests/fixtures/os1x-shape/`, task 5.2)
  remains the regression guard for the three post-Revision-19 shapes.
- If openspec ships another validator-affecting format change, that is
  a new format decision, not a gate-internal pin: the gate stays plain.

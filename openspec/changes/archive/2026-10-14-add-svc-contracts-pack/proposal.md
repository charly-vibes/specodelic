# Proposal: add-svc-contracts-pack

> **Status: ADOPTED (OQ-P1 decided 2026-10-14, maintainer decision
> `specodelic-kjzh`): ship the pack at `draft` lifecycle — single-requester
> demand, conservative reading; promote when a second independent consumer
> exists. The `evidence/` directory ships in this change directory.**
>
> **What "evidence" means here.** Evidence IDs (`E1`…`E12`) refer to
> `evidence/run-evidence.sh` and its logs (committed under this change's
> `evidence/`).
> They were run with the `spk` **0.8.0** release binary (the `v0.8.0` tag) against
> the source tree at commit `2939a0b2abff5f7d6d9f9959464431684a728491` (taken
> from the source zip's archive comment; confirmed to exist upstream). No file
> under `src/`, `packs/`, `specs/` or `openspec/specs/` differs between that tag
> and that commit. The behavioral findings were also re-run with the **0.10.0**
> binary against an upstream snapshot (`49e1a2e…`, 2026-10-13) and all
> reproduced (see the bundle README, "Currency check").
>
> **Paths.** Rebased to the current tree (2026-10-14): the former `specs/`
> corpus now lives at `openspec/specs/<dir>/spec.md` (specodelic-jf1), the
> changelog at `openspec/CHANGELOG.md`, and the format primer prose at
> `openspec/USAGE.md`.

## Why

The originating question: how can specodelic describe services whose
contracts live in *different repositories*, and can one feature change touch
several of them?

Two facts, both verified, shape this pack:

1. **The base format already carries the cross-file edges.** A consumer row's
   `satisfies` points at a provider's `extension_point`; its `observes` points
   at a provider's `effect`; both resolve corpus-wide across directories
   (E3a/E3b: two directories linted together are clean in either order; E2: the
   consumer alone fails with exactly two dangling references — its imports).
   `USAGE.md` §2.6 already documents this open/closed pattern. **This pack
   therefore adds no edge.**
2. **What the base lacks is vocabulary**, and the lack is visible: a `svc.*`
   kind in a Properties `kind` cell is rejected as `property_kind_closed` +
   `orphan_vocabulary` when no pack declares it (E10c). The base has no typed
   place to tie a published contract to (a) its owning repository and (b) the
   authoritative external artifact (OpenAPI, AsyncAPI, protobuf, JSON Schema
   file), and no law kind that forces a contract's owner to state what happens
   to existing consumers when something is *added* and when something is
   *removed*.

## What Changes

- **New capability `svc-contracts-pack`** (delta under `specs/`; dual-format).
- **New in-repo pack artifact (draft included in the bundle):**
  `packs/svc-contracts.md`, id `svc.contracts`, `kind: profile`, lifecycle
  `draft`. It declares, in the six manifest tables:
  - Section `## Interfaces` — rows `| name | kind | repo | binding | evolution |`
    (`kind` ∈ `request|event|schema`; `evolution` ∈ `additive|versioned|frozen`;
    `repo` and `binding` opaque).
  - Kind `svc.compat` — a compatibility law over a published interface.
  - Reference field `published_as` — a contract row → an `## Interfaces` row of
    the same file (outbound leaf, intra-file; the `capped_by` / `produced_by`
    precedent).
  - Checkers `svc.interface_closed`, `svc.compat_cases` — declarations only.
  - Floor `svc.compat` → case labels `added`, `removed`.
  - Requires `base` → `specodelic.md Revision 18`; no other pack dependency.
- **No change to `src/`.** Nothing in this change touches Rust code, the base
  format, `specs/`, or any existing capability.

## Capabilities

### New Capabilities
- `svc-contracts-pack`: the declared profile file, the namespaced `Interfaces`
  section and `svc.compat` kind, the `published_as` outbound leaf, the
  compatibility floor, and the guarantee that the pack adds no cross-file edge
  and no consumer listing.

### Modified Capabilities
- None.

## Out of scope (and why)

| Not in this change | Reason |
|---|---|
| Any `## Rollout` / step-ordering section for a multi-repo feature | Ephemeral workflow state; belongs in OpenSpec `tasks.md` / the issue tracker, not the durable corpus. See design D9. |
| A `consumers` / `consumed_by` field on a contract | `USAGE.md` §2.6: a publisher must not enumerate its conformers. The direction is derived by the graph. See design D5. |
| Technology-specific kinds (`svc.http`, `svc.grpc`) | Binds the spec to a transport; the opaque `binding` already bridges to the authoritative file. |
| Normalizing the `repo` identifier | Opaque by design; normalization is an unresolved question (OQ-P2). |
| Executing the pack's checkers | The mechanism names pack checkers and never runs them (`src/packs.rs` header; E10d, E10e). Executing them is mechanism work for a separate change. |
| Making cross-repo lint order-independent | That is `add-workspace-scope` (E10f/E10g show the order dependence the pack inherits until then). |

## Alignment

- **`append_only_packs` / `base_closed_sets_frozen`:** `## Kinds` rows are
  pack-qualified (`svc.compat`); no base set grows or narrows. `pack_shape` over
  the draft pack reports zero findings (E10a).
- **No pack, no change:** `lint specs` and `lint openspec` over the repo's own
  corpora are byte-identical with and without the pack present (E11).
- **`single_root_reachable`:** `published_as` is an outbound leaf, like
  `capped_by`; no carve-out machinery.
- **`AGENTS.md` / `openspec/specs/AGENTS.md` rule #3a ("existing mechanism, new scope"):**
  see design D3 for the overlap check against `kernel.binding` and
  `data.lineage`.
- **Self-hosting:** the capability delta is dual-format and lints clean (0
  findings) with `spk lint`.

## Governance and demand — what this proposal does NOT claim

The ancestor pack proposals cite a "≥2 independent domains" adoption bar. This
proposal **does not claim to meet it.** The only demand evidence is the single
originating request; independent demand is **UNKNOWN**. Whether a pack with one
requester should ship at all is a maintainer decision (OQ-P1). The conservative
reading is to land the pack at `draft` lifecycle only — which is how the draft
artifact is written — and promote it when a second independent consumer exists.

## Risks and honest limits

1. **Thin by design.** Given E10d/E10e/`packs.rs`, nothing in this pack is
   *executed* today: a dangling `published_as` name and a `svc.compat` row
   without the floor's labels both lint clean. What the pack buys today is
   (a) acceptance of the `svc.compat` kind instead of a hard
   `property_kind_closed` failure (E10c vs E10b) and (b) the activation
   advisory that names the pack and its draft status (E10b).
2. **Order-dependent discovery across repos** (E10f vs E10g) until
   `add-workspace-scope` lands.
3. **Closed value sets in cells are unchecked** (`kind`, `evolution`): the
   mechanism never checks row-kind cells; the closure is the pack's prose.
4. **Token residual:** `Interfaces` has zero exact-case hits in `openspec/specs/` and
   `openspec/specs/` but one case-insensitive hit (a lowercase word in
   `openspec/CHANGELOG.md`, a non-spec file) — E12.

## Impact

- **New files:** `openspec/changes/add-svc-contracts-pack/**`; at landing,
  `packs/svc-contracts.md` and its committed compile artifacts under
  `specodelic/` if the repo's precedent for pack files applies (the existing
  `packs/quant-finance.md` has `specodelic/quant-finance.*` — whether that is
  required for every pack is **UNKNOWN**).
- **Touched at landing (per the existing quant precedent):**
  `docs/src/SUMMARY.md` (capability entry), `openspec/CHANGELOG.md`.
- **Code:** none. **Format revision:** none (no base change).
- **Relationship to `add-workspace-scope`:** independent in both directions.
  The pack works single-repo today; the CLI change works with base-format
  `satisfies`/`observes` alone and never reads this pack.

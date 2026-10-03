# Design: add-data-lineage-pack

## Context

`add-domain-pack-mechanism` (archived 2026-10-03) landed the pack
mechanism: `kind: profile` pack files with six per-facet manifest tables,
corpus-scan discovery, advisory-first opt-in, `uses` outbound leaf,
lifecycle + skew advisories. Standard packs are thin instances — the
mechanism is where the checking lives; the pack only declares vocabulary.

R3 (external-review synthesis) converged on a domain-neutral shape: a
typed `## Data` table (name, shape, dtype, units, range in mistral's
proposal; `dataset|artifact|environment` kinds in the science pack),
`produced_by`/`consumed_by`/`produces` lineage edges, and — the one point
both camps agreed on — a *typed external binding* (grok-bioimage: keep
data models in external standards, bridge them; mistral: same, phrased as
outbound-leaf discipline).

## Goals / Non-Goals

- **Goal:** the first standard pack, proving the mechanism end-to-end
  before the domain-specific pilot (bioimage).
- **Goal:** zero core growth — no base closed set changes, no `src/`
  edits, no checker code.
- **Non-goal:** dtype enums, shape conventions, URI schemes, or any
  external-schema vocabulary in closed sets (grok's counter-position,
  honored verbatim).
- **Non-goal:** cross-file lineage resolution in v1 — the closure checker
  is intra-file; a corpus-wide lineage view belongs to `spk graph`
  follow-on work if demanded.

## Decisions

### D1 — Thin vocabulary (one section, three kinds, three fields, one checker, one floor)

| Facet    | Declaration |
|----------|-------------|
| Sections | `## Data` with row shape `\| name \| kind \| dtype \| units \| binding \|` |
| Kinds    | `data.dataset`, `data.artifact`, `data.environment` — the R3 closed trio |
| References | `produced_by`, `consumed_by` — each resolves to a `## Data` row of the declaring file. Bare `produces` is deliberately NOT declared: the word appears in corpus prose (`compile.md`, `graph.md`, `orchestrate.md`), and the mechanism's vocabulary match is whole-word over file text — declaring it would falsely activate the pack on every lint. Append-only lets a later release add it once activation can disambiguate |
| Checkers | `data.lineage.closure` — every lineage field names a declared `## Data` row; honest-empty otherwise |
| Floors   | `data.artifact` requires a `**provenance:**` case label (the `**name:**` machinery with a different required label set, per mistral's R4 reuse phrasing) |
| Requires | `base` pinned at `specodelic.md Revision 14` (the mechanism revision) |

Kept minimal deliberately: the pilot (bioimage) is the stress test for
richer row shapes; `append_only_packs` lets later releases add columns and
kinds without breaking this one.

### D2 — Lineage fields resolve intra-file, outbound-leaf

Resolving to a `## Data` row of the declaring file keeps the checker
file-local (advisory-first, no corpus-wide pass) and keeps the fields
outbound leaves: they join no reachability path and no acyclic edge set,
so `single_root_reachable` needs no carve-out (the `uses`/`observes`
precedent). A dangling lineage reference is a labeled finding naming the
row id and both remediations (error-contract) — never a generic dangling
message (specodelic-2q8 precedent).

### D3 — Binding column stays opaque

The `binding` column carries a typed pointer (schema id, URI, standard
reference) that the format never parses. This is the grok counter-position
made structural: external standards (OME/NGFF, Parquet, BioImage.IO)
remain authoritative; the pack's only opinion is that the pointer exists.
Consequence: lint behavior is byte-identical regardless of binding target.

### D4 — Pack file lives at `packs/data-lineage.md`

Discovery scans every `.md` under the git toplevel for `kind: profile`
frontmatter — a dedicated `packs/` directory at the repo root is
convention, not requirement, and keeps standard packs visible next to
`specs/` and `openspec/`. File-naming law: stem `data-lineage` ⇔ id
`data.lineage`.

### D5 — Lifecycle: starts `draft`, publishes after gates

The artifact lands in `draft` (advisory naming the draft status); the
publish transition guard — `pack_shape` zero findings over the declared
vocabulary — is checked in the dogfood task, then the state flips to
`published` in the same change, before archive.

### D6 — Vocabulary hygiene beats symmetry (the `produces` cut)

The R3 synthesis named three lineage edges (`produces`, `produced_by`,
`consumed_by`) and D1 originally declared all three. Dogfooding caught
the trap: `produces` is an ordinary English word the corpus itself uses
in prose and constraint exprs (`compile.md` "produces all three
artifacts", `graph.md`, `orchestrate.md`), and the mechanism's
activation signal is whole-word containment over file text — declaring
`produces` would activate the pack for six corpus files with zero
vocabulary intent. Since pack declarations are append-only, shipping
`produced_by`/`consumed_by` now and adding `produces` later (only when
a workspace-anchored signal exists) costs nothing; shipping all three
and trying to un-declare later is forbidden by `append_only_packs`.
Same shape as the v1 orphan-detection call (typed signal over prose
heuristic — honest-empty beats over-reporting).

### D7 — The delta file's own activation is honest noise

The capability delta (`specs/data-lineage-pack/spec.md` in this change)
is a dual-format file whose mirrored requirement text names the pack's
kinds and fields, so linting the `openspec` tree vocabulary-activates
the pack for the `spec` delta file — one advisory on the warnings
channel, exit 0, findings empty. Not fixed by rewording the
requirements (their text is the contract); the advisory disappears when
the delta archives to `changes/archive/` (outside the linted tree per
the summary-completeness exclusion) and the capability spec under
`openspec/specs/` carries the same text deliberately — the format's own
documentation of a pack legitimately mentions the pack.

## Rejected alternatives

- **Richer row shape (`| name | shape | dtype | units | range |`)** —
  mistral's full proposal; deferred to the pilot with real fixtures.
  Append-only allows adding columns later.
- **Corpus-wide lineage closure** — requires a cross-file pass the
  mechanism does not model for pack checkers; `spk graph` is the natural
  home if demand appears.
- **Absorbing `## Data` into core** — R3 was tiered "Core candidate" for
  the *binding mechanism* only; as a standard pack it stays opt-in and
  files without it lint byte-identically.

## Review outcome

Ro5 review at proposal time: verdict READY WITH_NOTES — findings folded
back into this delta before implementation (D2's labeled-finding clause,
D3's opacity consequence, D5's draft-first lifecycle).
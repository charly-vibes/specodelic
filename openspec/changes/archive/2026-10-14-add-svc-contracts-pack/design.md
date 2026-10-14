# Design: add-svc-contracts-pack

Evidence IDs (`E…`) refer to the bundle's `evidence/` directory. "Verified"
means observed with the `spk` 0.8.0 binary or read in the source at commit
`2939a0b…` (from the zip archive comment). "Proposal" means a design choice made
here that the maintainers may overrule. Anything not verified is marked
**UNKNOWN**.

## Context

A pack is a `kind: profile` spec file declaring six two-column tables
(Sections, Kinds, References, Checkers, Floors, Requires). `pack_shape`
requires every table to be present and non-empty, every row to have exactly two
columns, and every kind to be pack-qualified (contain a dot, not a base kind)
(`src/packs.rs`, `pack_shape` findings; verified by reading). Packs are
discovered by scanning the corpus under the git toplevel of the last linted
path; there is no config file and no registry. Pack **checkers are declarations
only**: the mechanism names them and never runs them (`src/packs.rs` header;
verified by E10d, E10e). A pack activates *advisory-first* when a file uses its
vocabulary — kinds, section headings, reference-field names — matched as exact,
case-sensitive, word-boundary strings over the raw file text.

## Goals / Non-goals

**Goals.** Give a multi-repo corpus (a) a typed registry row tying a published
contract to its repo and to the authoritative external artifact, (b) a law kind
whose floor forces the added/removed cases, (c) acceptance of the namespaced
kind instead of a hard failure — with zero change to the base and zero new edge.

**Non-goals.** Executing checkers; cross-repo discovery order (see
`add-workspace-scope`); any rollout/ordering vocabulary; any consumer listing;
repo-identifier normalization.

## Decisions

### D1 — A thin pack: registry + law kind + one outbound leaf

*Proposal.* Three declarations carry the whole value: the `## Interfaces`
section, the `svc.compat` kind with its floor, and `published_as`. Everything
else the multi-repo question needs already exists in the base (E3a/E3b).

### D2 — Vocabulary-hygiene audit (verified, E12)

Tokens are matched with exact case and word boundaries, so a bare English word
that appears capitalised in corpus prose would falsely activate the pack. The
quant-finance precedent audits every token against the corpus. Result for
this corpus (`ci` = case-insensitive files, `exact` = exact-case files;
each as `openspec/specs`) — **re-run at landing, 2026-10-14, against the
post-jf1 tree** (`evolution`: 2 ci / 1 exact in `linter-schema_shape/spec.md`
prose — column name, not scanned):

| Token | ci | exact | Verdict |
|---|---|---|---|
| `Interfaces` (section) | 0/0 | 0/0 | Safe at landing (the former lowercase changelog hit lived in the pre-jf1 `specs/CHANGELOG.md`, now `openspec/CHANGELOG.md` — outside the corpus scan either way). |
| `svc.compat` (kind) | 0/0 | 0/0 | Safe. |
| `published_as` (field) | 0/0 | 0/0 | Safe. |
| `svc.contracts` (pack id) | 0/0 | 0/0 | Safe. |
| `Rollout`, `Peers`, `Topology`, `Services` | 0/0 | 0/0 | Also safe — considered and not used (D9). |
| `Contracts` | 5/1 | 0/0 | Exact-case safe; the case-insensitive hits are lowercase prose. Not used. |
| `Repos` | 1/0 | 0/0 | Exact-case safe. Not used. |
| `evolution` | 2/1 | 1/1 | **Has exact-case hits in `openspec/specs/`.** Not scanned: it is a *column* name, and columns are not vocabulary (the scan covers kinds, sections and reference-field names). Used only as a column header; if a maintainer prefers a zero-hit name, rename the column (OQ-P7). |

Limits of this audit: it protects *this* repository's corpus (the dogfood
lint). A downstream user whose prose capitalises "Interfaces" would see an
advisory activation, never a failure (advisory-first). That residual is
accepted, as it was for `Limits`/`Fixtures` in the quant pack.

### D3 — Rule #3a check ("existing mechanism, new scope")

| Existing mechanism | Why it does not already cover this |
|---|---|
| `extension_point` + `satisfies`; `effect` + `observes` | These *are* the cross-repo edges — and are reused unchanged (D4). They carry no repo identity, no external-artifact pointer, no evolution mode. |
| `kernel.binding` (opaque column carrying an external checker's claim) | Different direction: it records that an external checker claims a *constraint*, not where a *contract's authoritative definition* lives. The pack reuses the *idiom* (an opaque, never-parsed string), not the column. |
| `data.lineage` pack (`## Data` rows with `binding`, `produced_by`/`consumed_by`) | Its kinds describe measurements, datasets, artifacts and environments produced by pipelines. A request/response or event interface is none of those. **OQ-P3:** a maintainer may prefer modelling an event payload schema as a `data.artifact` and requiring `data.lineage`, avoiding a new section. |

### D4 — `published_as` is an intra-file outbound leaf; cross-file stays base

*Proposal, following precedent.* `capped_by` (quant) and `produced_by`
(data.lineage) resolve to a registry row *of the declaring file*. `published_as`
does the same: a contract row (`extension_point` or `effect`) names the
`## Interfaces` row that registers it. Cross-file resolution is deliberately
**not** attempted by the pack — the mechanism keeps pack-section resolution
file-local, and the base already resolves `satisfies`/`observes` corpus-wide.

**Verified limit (E10d).** In 0.8.0 a dangling `published_as` name produces no
finding, and a search of `src/` finds no code that resolves pack reference
fields (the names `capped_by`, `produced_by`, `measured_by` do not appear).
The quant-finance design document reports a labeled finding for a dangling
`capped_by` (its probe (a)); that is **not reproduced here** — whether it came
from a different binary or commit is **UNKNOWN**. The pack therefore claims only
what is declared: `svc.interface_closed` is a named declaration.

### D5 — No consumer listing (consumer-driven direction)

`openspec/USAGE.md` §2.6 tells authors not to enumerate a contract's conformers in
the publisher's file. The pack honours it: no `consumers`/`consumed_by` field.
The provider→consumer direction is derived (`graph`, and the proposed
`workspace` report in `add-workspace-scope`). *Non-normative note:* this is also
the stance of consumer-driven contract testing (Pact), where the consumer's
expectations define the contract; consulted in earlier research, not
re-verified for this bundle.

### D6 — Floor cases `added`, `removed`

*Proposal; rationale is mine, not sourced.* Whether existing consumers keep
working after a change depends on two things: whether something was added
(compatible with every consumer) and whether something was removed (breaking for
any consumer that used it). A modification is a removal plus an addition. Two
labels keep the floor minimal. **OQ-P4:** maintainers may want `retyped` as a
third label. The floor is declaration-only (E10e).

### D7 — Requires pin

`Requires`: `base` → `specodelic.md Revision 18` (the corpus revision at this
commit). No pack-on-pack dependency. Pack-vs-corpus revision skew is handled by
the existing mechanism.

### D8 — Draft lifecycle encoding

*Proposal, with a verified mechanism fact.* `lifecycle_of` reads a Model whose
States list has exactly one entry as that entry's name (`draft`); three states
read as `published`/`deprecated` by position (`src/packs.rs`, verified by
reading). A Model needs a Transitions table (removing it fails `model_present`
and reachability), so the draft pack carries one self-loop transition
(`declare: draft→draft`). **UNKNOWN / OQ-P5:** whether maintainers prefer the
self-loop or shipping the full three-state machine at landing.

### D9 — Considered and rejected

| Rejected | Why |
|---|---|
| `## Rollout` rows (`step, repo, phase, after`) for expand/contract ordering of a multi-repo feature | Workflow state is ephemeral and mutable; the durable corpus should state *what must hold* (contracts, compat laws), not *who merges first*. OpenSpec `tasks.md` and the tracker already hold ordering. Revisit only with maintainer demand. |
| `consumed_by` / consumers list | Violates USAGE §2.6 (D5). |
| Per-transport kinds (`svc.http`, `svc.grpc`, `svc.kafka`) | Couples the spec to a technology; `binding` already bridges to the authoritative artifact. |
| `version` column on `Interfaces` rows | Versions live in the external artifact; duplicating them creates a drift source. |
| A `repo` format (URL, org/name) enforced by the pack | Normalization of repo identity is unresolved (OQ-P2). Mechanism never checks row cells anyway. |
| Adding `satisfies`-like pack edges across repos | Base edges already resolve corpus-wide (E3). |

## Risks / Trade-offs

See `proposal.md` "Risks and honest limits". In short: the pack is declarative
and thin; its executed effect today is kind acceptance and the advisory
activation notice; closure of cell values is unchecked; and cross-repo use is
order-dependent until `add-workspace-scope` lands (E10f/E10g).

## Migration Plan

Additive. Corpora using none of the vocabulary are byte-identical before and
after (E11). Adoption is by corpus scan (the pack file exists under the git
toplevel) or the explicit `uses` edge; there is no migration.

## Open Questions

- **OQ-P1** Ship a pack with one known requester, or hold the proposal until a
  second independent consumer exists? (Demand beyond the requester: UNKNOWN.)
- **OQ-P2** What is the canonical `repo` identifier (`org/name`, URL, other),
  and who normalizes it? (A Cargo issue about git-URL spelling differences,
  consulted earlier, shows why identity matters; not re-verified.)
- **OQ-P3** `data.artifact` + `data.lineage` instead of a new `Interfaces`
  section? (D3)
- **OQ-P4** Third floor label `retyped`? (D6)
- **OQ-P5** Draft pack lifecycle: single-state self-loop, or full three-state
  machine at landing? (D8)
- **OQ-P6** Are committed `specodelic/<pack>.*` compile artifacts required for
  every pack file? (Precedent exists for quant; requirement UNKNOWN.)
- **OQ-P7** Rename the `evolution` column to a name with zero hits in the
  corpus, even though columns are not scanned as vocabulary? (D2)

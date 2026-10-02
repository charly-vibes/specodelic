> Snapshot of `openspec/research/2026-10-02-external-review-synthesis/synthesis.md` (canonical). Re-import after edits; do not edit this copy.

# External review synthesis — requirements, rough edges, and the abstract extension mechanism

**Date:** 2026-10-02 · **Inputs:** 26 external deliverables (7 grok packages, 5 mistral docs, 7 qwen chats, 7 zai files) in `~/Downloads/cv/` from four vendors evaluating or proposing against specodelic v0.4.0 (commit `c0f95c6`): **grok** (Grok 4.5), **mistral** (Vibe / GLM), **qwen** (partial repo access — weakest grounding), **zai** (x-preview-l; some stale repo facts, e.g. claims `compile`/`model-check` unimplemented — they shipped).

Domains probed: quant finance, FP&A, bioimage analysis, scientific software generally, and modular/composition (wiring diagrams).

---

## 1. Convergent requirements (what ≥2 vendors independently demanded)

Ranked by how many independent proposals converge on the same demand.

### R1 — Quantities, units, dimensions (mistral ×2, grok, qwen, zai)
Every domain pack independently invented a quantity layer. Shapes seen:
- `## Quantities` table (`id | kind | unit | domain`) with UCUM-ish units (mistral science pack)
- `quantity`/`unit`/`limit` fields on constraint rows + workspace `units.md` unit table as an extension point (mistral quant)
- `tolerance` constraint/property kind carrying `{metric, abs, rel, unit}` (grok quant)
- frontmatter `physical_units` + `## Mathematics` section (qwen)

The recurring verdict sentence: *"the strongest finance/science constraints are exactly the ones prose_untouched makes uncheckable."*

### R2 — Machine-checkable numeric predicates in guards (all four)
Guards are propositional references only; real-valued predicates cannot gate a transition. Mistral-bioimage scoped the smallest credible grammar: five predicates (`dtype_is`, `shape_eq`, `same_shape_as`, `within`, `units_convertible`) resolving against a `## Data` table, compiled into `invariants_checked`. Everyone wants *some* closed predicate grammar, nobody proposes an open DSL.

### R3 — Data / lineage layer (mistral ×2, qwen, grok-bioimage, zai)
- `## Data` typed table (name, shape, dtype, units, range) — mistral bioimage + quant
- `produced_by`/`consumed_by`/`produces` lineage edges; `## Data` with `dataset|artifact|environment` kinds (mistral science pack)
- `## Data Lineage` ETL section (qwen)
- Explicit counter-position: grok-bioimage says *don't* absorb data models — keep them in external standards (OME/NGFF, BioImage.IO) and bridge. Both positions agree the *binding* must be typed.

### R4 — Statistical / tolerance property kinds (mistral ×2, grok, zai)
`kind` set grows: `tolerance` (grok), `statistic` with `stat_test ∈ {stationarity, mean_eq, ci_within, distribution}` (mistral quant), empirical bounds with scaling generators (grok FP&A). Crucially, mistral-phrased the reuse: *same machine-findable `**name:**` case-label enumeration mechanism, a different required label set* — tolerance laws owe `**bound:**` + `**against:**`, not identity/associativity.

### R5 — Composition / wiring (grok, mistral, zai, qwen)
- Open-system Model mode: `kind: open_system` + `## Ports`/`## Boxes`/`## Wires` tables; boxes = `[[wiki-link]]` refs to other files (grok wiring)
- `## Ports` table + `;`/`⊗` composition + `identity_l`/`identity_r`/`tensor_unit` laws (mistral monoidal wiring)
- Colimits/pushouts for merge, monoidal parallel composition, 2-categorical enrichment (zai ×2, grok CT)
Convergent categorical reading: box = morphism in an SMC, port = object, wire = generating morphism, nesting = operadic substitution.

### R6 — External artifact binding (mistral, grok, qwen)
`validated_against` (mistral bioimage) / `--external-validator` (qwen): Property → benchmark manifest (dataset URI + checksum + metric + threshold). Universally typed as an **outbound leaf** so `single_root_reachable` needs no carve-out. `verify` fails closed on checksum mismatch; reports cite the checksum.

### R7 — Time (mistral, zai, qwen)
Temporal guard clauses (`within <duration> of <event-ref>`), `### Continuous` variables subsection for hybrid models, fiscal periods/calendars (FP&A).

### R8 — The extension mechanism itself (zai, grok-bioimage)
zai explicitly proposes "community-driven domain packs"; grok-bioimage ships a *guidance-only* change and defers schema hooks until the pattern is dogfooded. This is the meta-requirement this synthesis feeds.

---

## 2. Rough edges found in the format

1. **EARS can't state quantitative requirements.** "SHALL converge to 1e-6 within 500 iterations" matches none of the five patterns (mistral). Proposed append-only sixth pattern: `THE <system> SHALL <behavior> WITHIN <ε> [<unit>]`.
2. **`kind` overload** (CLAR-002, grok CT eval) — 𝒦 objects vs. the row sub-kind column. Worse if domain packs add kinds.
3. **Global closed kind sets don't scale to packs.** Two vendors proposed a `tolerance` kind — on different layers (grok-quant: Constraint.kind; mistral-science: Property kind set) — and a third (mistral-quant) proposed `statistic`. Under a single closed set, independent domain packs collide. Needs namespacing or profile scoping.
4. **`prose_untouched` ↔ domain semantics tension** — the founding rule is exactly what makes the dominant content of scientific/quant specs uncheckable. All four vendors hit this wall first.
5. **Mealy-only-Moore gap; no dual 𝒦ᵒᵖ, no 2-cells** (grok CT): emits is state→output only; duality/adjunction extensions (coverage dual, blast-radius as Kan extension, Mealy dual) are conservative additive moves the theory already supports.
6. **`law_requires_cases` floor is not domain-universal** (mistral bioimage, flagged as its one minor breaking change): identity/associativity are meaningless for scientific-transform laws; floor should be kind/domain-dependent, enumeration *form* unchanged.
7. **Honest-empty convention is the safety rail** — `invariants_checked: []` honesty is praised by every vendor; every extension proposal preserves it (`tolerance_checked: []`, `evaluated: []`, `statistical_invariants_checked: []`, fail-open passes).
8. **Report-grounding variance**: qwen could not fully fetch the repo (its proposals invent non-existent CLI flags like `spk formalize`, `checked_against_core` frontmatter); zai evaluated a stale snapshot (marks shipped commands unimplemented, cites v0.1.0/v0.2.0). grok/mistral read live v0.4.0 and their format-conformance is high — mistral even nailed the dual-format archive hazard and outbound-leaf conventions.

---

## 3. The general pattern: how every vendor extends the format

Across ~10 independent change proposals, the *same six moves* recur. The format already implies an extension mechanism; nobody invented a new one:

| Move | Instances |
|---|---|
| **Optional new section** (sub-layer) | `## Quantities`, `## Data`, `## Ports`, `### Continuous`, `## Fixtures` |
| **Append-only kind growth** under a new Revision heading | `tolerance`, `statistic`, `pipeline`, `interface`, `composition_law` constraint kinds; open_system Model mode |
| **New outbound-leaf reference fields** | `validated_against`, `consumes`/`produces`, `produced_by`/`consumed_by`, `against`, `ref`→Quantity |
| **New checker spec** owning the new invariants, appended to Checker Ownership (parallel branch, non-gating on files without the new sections) | `linter-quantities.md`, `linter-data_shape.md`, `linter-quant_expr.md`, `linter-handoff.md`, `linter-numeric_shape.md` |
| **Required-case-label floor per kind** (reuse `**name:**` enumeration machinery) | tolerance → `**bound:**`+`**against:**`; statistic → `**alpha:**`+`**window:**`; laws stay identity/associativity |
| **Advisory-first, honest-empty enforcement** | new passes report empty checked-sets when the workspace provides no fixtures/units/datasets; `spk doctor` reports enabled extensions |

And the universally respected invariants: `append_only_variants`, `prose_untouched`, typed-`[[wiki-link]]` foreign keys, DAG-ness, `single_root_reachable` (satisfied by outbound leaves), byte-stable compile artifacts, self-hosting (the extension is itself a spec in the corpus).

**Conclusion of the pattern observation:** the format's extension mechanism is *already categorical and already uniform* — extensions are (a) new objects in a closed set grown under a Revision, (b) new morphisms typed as outbound leaves, (c) new checker-limit points in the ownership DAG, (d) new required-label floors over the same enumeration machinery. What is *missing* is making this mechanism **first-class**: today an "extension" is only a convention followed by hand; nothing in the format names it.

---

## 4. What a first-class extension mechanism needs (draft shape)

Requirements distilled from R8 + the rough edges that block it:

1. **Declared, not conventional.** An extension (call it *domain pack* / *profile*) is a spec file that publishes: the optional sections it introduces, the kind-set growth it claims, the reference fields it adds, the checkers that own them, and the format_revision it extends from.
2. **Namespaced variant sets.** Kind growth scoped to the pack (e.g. `quant.tolerance`) so independent packs don't collide in the global closed set — resolves rough edge 3 and softens CLAR-002.
3. **Capability negotiation.** `spk doctor` / lint report which packs a workspace enables; checkers gate only on files that opt in (the "parallel branch, never gating on files without the new sections" rule every vendor restated).
4. **Profile = new intent kind.** Several packs (FP&A, quant, bioimage) each proposed a top-level capability intent (`fpa.support`, `quant.extensions`, `bioimage-domain`). A pack is structurally exactly what a published `extension_point` already is — consumers `satisfies` it. The mechanism may already exist at the row level; the open question is whether *sections and kind-sets* can ride the same edge.
5. **Floor registry.** Required-case-label sets registered per kind by the pack, reusing the `**name:**` machinery — turns rough edge 6 from a breaking change into data.
6. **Additive-only enforcement stays.** A pack can never narrow an existing set or typing — `append_only_variants` applies to packs themselves.

**Open questions to grill before any proposal:**
- Is an extension a *new kind of spec file* (a seventh kind? no — likely a `kind: profile` intent) or a conventional capability spec + a machine-readable manifest section?
- Do optional sections need to be registered in a closed set too (`section_kind_closed`), or does anything-goes-with-registered-checker hold?
- How do two packs claiming overlapping vocabulary (e.g. two `tolerance` kinds) get reconciled — namespace law, or first-past-the-post?
- Does the Grothendieck namespacing story (theory.md) extend to cover pack-qualified kinds, or is that overreach?
- MoSCoW: which of R1–R7 is actually in demand vs. speculative? (All four vendors were prompted toward a domain — demand signal is the convergence itself, not any single domain.)

---

## 5. Source map

| Vendor | File(s) | Contribution |
|---|---|---|
| grok | `specodelic-design-evaluation-2026-10-02/` | CT consistency audit; duality/adjunction extension menu; residual tensions (CLAR-002) |
| grok | `specodelic-change-proposal/` (wiring) | open_system Model, Ports/Boxes/Wires, SMC reading, phased compile targets |
| grok | `specodelic-quant-proposal/`, `specodelic-fpa-evaluation-package/` | tolerance kind, hybrid `### Continuous`, quant law cases; FP&A gaps (dimensions, perf bounds) |
| grok | `add-bioimage-domain-support/` | hybrid-ownership counter-position (bridge, don't absorb); guidance-first, no-revision change |
| mistral | `specodelic-science-pack-…` | `## Quantities` (UCUM), `tolerance` property kind w/ `bound:`/`against:` floors, `## Data` lineage, sixth EARS pattern, outbound-leaf discipline |
| mistral | `specodelic-quant-finance-…` | quantity/unit/limit fields, unit tables as extension point, temporal guards, `statistic` kind, `## Fixtures`, honest-empty eval |
| mistral | `add-bioimage-data-semantics-…` | 5-predicate grammar, `validated_against`, `kind: pipeline` + handoff unification (rank ≤ 6), kind-dependent law floor |
| mistral | `add-monoidal-wiring-layer-…` | `## Ports`, `;`/`⊗`, composition laws, trace deferred |
| qwen | 7 chat transcripts | progressive formalization (`formalization_mode: lite`), external validators, dimensional schema + lineage sections (weak grounding — verify anything before reuse) |
| zai | 01, 06 (CT) | inconsistency table (prose/machine duality, missing verify adjunction), self-hosting fixed-point/coalgebra observation, 𝒦ᵒᵖ duality |
| zai | 02, 03, 05, 07 (domains) | quant/bioimage/FP&A/scientific gap matrices, domain-pack roadmap, graph filtering/abstraction/aggregation |
| zai | 04 (modular) | pipeline/routing expressiveness verdict, orchestrate-maturity caveat |
---

## 6. Tier stratification — core vs. abstract extension vs. domain pack (2026-10-02, post-review)

The vendor proposals are domain-first: each pack re-derives quantities, data,
and tolerance machinery from scratch. The convergence across 4/4 independent
domains is the signal that these are **general mechanisms wearing domain
clothes** — only the vocabularies differ.

| Demand | Domain-neutral mechanism | Tier | Pack-side vocabulary |
|---|---|---|---|
| R1 quantities/units | typed-value rows + unit-compatibility checking | **Core candidate** | UCUM vs ISO4217 vs microscopy units |
| R2 numeric guards | predicate-grammar *mechanism* (registered closed predicate sets → compiled invariants) | **Abstract extension** | the predicates: `dtype_is`/`shape_eq` vs `≤` on quantities vs latency bounds |
| R3 data/lineage | `## Data` row shape + `produced_by`/`consumed_by`/`produces` + outbound-leaf external binding | **Core candidate** | schemas, dtype enums, OME-Zarr/parquet URIs, benchmark manifests |
| R4 tolerance/statistic | one "empirical" property kind + per-kind floor registry (the `**name:**` machinery is already generic) | **Abstract extension** | `stat_test` names, metrics, thresholds, reference datasets |
| R5 composition/wiring | Ports/Wires row shapes + open-system Model mode | **Abstract extension** (semantically deep; land as extension first) | box/port types |
| R6 time | temporal guard-clause grammar + continuous vars | **Abstract extension** | fiscal calendars, settlement windows, market hours |
| R7 EARS pattern | sixth quantitative pattern (`WITHIN <ε> [<unit>]`) — tiny, closed | **Core candidate** | — |
| R8 pack mechanism | the extension mechanism itself | **Core** | — |

**Effect on domain packs:** they become thin instances — bioimage = core data
layer + dtype enum + axis semantics + OME binding; quant = core quantities +
currency vocabulary + regulatory invariants. The mistral-vs-mistral `## Data`
collision (criterion 14) and most of the namespacing problem dissolve at the
root because vocabulary, not mechanism, is what gets namespaced.

**Proposed promotion rule (grill item D7):** demanded by ≥3 independent
domains AND vocabulary-independent semantics AND a small closed row shape
⇒ core; else abstract extension; domain-only vocabularies ⇒ pack.

**Counterweight to respect:** grok-bioimage's guidance-first stance and the
format's closed-minimal-core virtue — promotion to core must itself travel
the openspec proposal gates, not ride a vendor pile-on.

---

## 7. Strict thin core — categorical analysis (2026-10-02, user position)

**User position (D7):** adding to CORE is held very strict; thin core is good;
the format should be very extendable. The §6 "core candidate" labels for
R1/R3/R7 are demoted accordingly.

**Categorical framing (grounded in specs/theory.md):**

- Core = the schema category 𝒦 (5 kinds) + generating morphisms (Reference
  Typing) + laws + append-only discipline. An instance is a functor
  I : 𝒦 → Set; linting checks I is well-formed.
- **Strict core rule (categorical form):** core grows only when a mechanism
  is required for *any* instance to be a well-formed functor at all — i.e.,
  it belongs to 𝒦's structure (objects, morphisms, laws, the definition of
  an instance). What a domain *says* is extra structure on I and must never
  touch 𝒦.
- **Packs are Grothendieck one level up.** theory.md already namespaces
  cross-file ids as a Grothendieck construction (∫ over feature files).
  Pack-qualified kinds (`quant.tolerance`) are the same construction with
  pack ids as the index: the base 𝒦 never changes; membership in closed
  sets becomes *fiber-relative*. Consequence: the core kind set literally
  stops growing — E2 (kind collisions) dissolves completely, and D3
  (namespacing scope) is answered: everything vocabulary-like is a fiber.
- **Revisions remain for instance-definition changes only.** E.g. R7's sixth
  EARS pattern parses frontmatter — that is instance-level machinery, so it
  is a genuine (tiny) format Revision, NOT a pack item; giving statement
  grammar to packs would weaken "machines parse only frontmatter and fixed
  tables".
- **Standard packs, not core.** R1 quantities, R3 data, R4 empirical-kind +
  floor registry, R5 wiring, R6 time ship as blessed standard packs
  (in-repo, dogfooded, versioned, but mechanically ordinary packs). A
  domain pack = standard packs + its vocabulary fiber + domain invariants.

**Generality test across further domains (reasoned, not fetched — treat as
speculative):** robotics (units SI, sensor lineage, state machines), ML
pipelines (dataset lineage, statistical metrics, tolerances), embedded
firmware (timing, units, state machines), IaC (wiring/composition, time
windows), clinical trials (data lineage, statistical tests, tolerances).
Every domain wants R1–R6's *mechanisms*; none shares another's *vocabulary*.
Mechanisms ⇒ standard packs; vocabularies ⇒ domain fibers.

**Revised tier table (strict):**
| Tier | Contents |
|---|---|
| Core (revision-gated, rare) | existing 𝒦 + R8 extension mechanism + instance-definition fixes (e.g. sixth EARS pattern) |
| Standard packs (blessed, dogfooded) | quantities (R1), data/lineage (R3), empirical property kind + floor registry (R4), composition/wiring (R5), time (R6), numeric predicate grammar (R2) |
| Domain packs (thin) | vocabulary fibers: unit sets, dtype enums, stat_tests, benchmark manifests, axis semantics + domain invariants/checkers |

## 8. Grill resolutions (2026-10-02)

- **D2 (section registry) — RESOLVED by the fiber model:** sections are
  registered per-pack in the pack manifest; two packs claiming the same
  section *mechanism* is legitimate (both may extend the data layer);
  collision only exists at the *vocabulary* level (e.g. two dtype enums),
  which is resolved by pack-qualified naming. Registry law = declaration,
  not first-come ownership.
- **D3 (namespacing scope) — RESOLVED:** everything vocabulary-like
  (kinds, sections' row vocabularies, predicate names, floors) is
  pack-fiber-relative via the Grothendieck construction one level up
  (theory.md precedent). Base closed sets stop growing except by true
  format Revision (instance-definition changes only).
- **D4 (Grothendieck for packs) — RESOLVED:** yes, and it is not overreach —
  it is the existing construction reused with pack ids as index; no new
  mathematics.
- **D5 (row-level edge) — RESOLVED:** moot as the mechanism; the
  extension_point/satisfies edge remains what consumers use to conform to
  a pack's contracts, but it cannot carry section/kind/floor registration
  (matrix column 03 already demonstrates the gaps).
- **D7 (core promotion) — RESOLVED (strict):** core grows only for
  instance-definition changes (𝒦 structure, laws, statement grammar); R1–R6
  mechanisms ship as standard packs; domain content is vocabulary fibers.
  Standard-pack governance (new): a pack is "standard" when it ships
  in-repo, is dogfooded by the corpus, versions with the format, and passed
  the ≥3-independent-domain demand rule + Ro5 at proposal time. Recorded
  spec-first in a future specs/ file (e.g. specs/packs.md).

Still open: **D1** (pack artifact shape), **D6** (pilot pack).

## 9. D1 v2 — after Rule-of-5 review (2026-10-02)

Review verdict: READY WITH_NOTES (converged Stage 5; 0 CRITICAL; 5 HIGH all
resolved or verified — TypeSafe: CORR-002 0.88, CORR-003 0.82, EDGE-001 0.92
≥ gate; DRAFT-001/CORR-001 below gate, re-confirmed mechanically).

D1 v2 supersedes the single-table sketch:

- **Pack artifact:** four-layer spec file, `kind: profile` (append-only
  INTENT_KINDS growth landed as an explicit Revision delta), EARS statement
  pattern: "WHEN a workspace enables this pack, THE format SHALL provide …".
- **Manifest = six per-facet tables** (closed row shapes, per Checker
  Ownership precedent): `## Sections`, `## Kinds`, `## References`,
  `## Checkers`, `## Floors`, `## Requires` (standard-pack deps + base
  format_revision + pack's own Revision headings).
- **Discovery = corpus scan:** every `kind: profile` file in the workspace
  is a candidate pack; no config file. Opt-in = vocabulary-use-triggered
  checking (advisory-first), upgradeable to a declared `uses` edge;
  vocabulary used with pack absent = labeled finding naming the pack
  (error-contract).
- **Lifecycle:** pack Model = draft→published→deprecated; consumers pin via
  `Requires`; revision skew = labeled advisory, never silent.
- **Terminology block required:** pack / standard pack / profile / fiber
  defined once (CLAR-002 overload managed by naming, not avoided).
- Open for the change proposal: name of the opt-in edge (`uses` vs
  overloading `satisfies`); whether `## References` rows may add fields to
  existing row shapes or only new fields.

## 10. Decision of record (2026-10-02)

**D6 — RESOLVED:** pilot pack = bioimage-data (most mechanically complete
vendor design; exercises data/lineage + numeric predicates + empirical-kind
standard packs at once). Quant lands second.

**`wai matrix decide` — APPROACH SELECTED: 02-profile-intent.** Decision
record: `.wai/projects/domain-specific-extensions/designs/matrix/decision.md`;
design doc scaffolded at
`.wai/projects/domain-specific-extensions/designs/2026-10-02-profile-intent.md`.
Full rationale recorded verbatim there (thin core D7, fiber model, D1 v2
manifest shape, rejections of 01/03/04, D6 pilot choice).

**Next step (fresh session):** openspec change proposal
`add-domain-pack-mechanism` (R8 core: kind: profile Revision delta, the six
per-facet manifest tables, discovery/opt-in/lifecycle) + the three standard
packs the pilot needs (data/lineage, numeric predicates, empirical kind),
then `add-bioimage-pack` as the thin domain instance. Spec-first:
`specs/packs.md` before checker code, per repo convention.

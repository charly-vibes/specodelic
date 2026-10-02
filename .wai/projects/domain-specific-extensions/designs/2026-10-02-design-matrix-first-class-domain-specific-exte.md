> **SUPERSEDED** by the wai decision matrix (`designs/matrix/`) — kept as the findings/requirements backing tables (sections A/B). Do not update; edit the matrix or the canonical synthesis instead.

# Design matrix — first-class domain-specific extensions for the specodelic format

**Project:** domain-specific-extensions · **Phase:** research → design · **Date:** 2026-10-02
**Canonical source:** `openspec/research/2026-10-02-external-review-synthesis/synthesis.md` — this matrix is the decision record; that file is the findings source of truth.
**Inputs:** 26 external deliverables, 4 vendors (grok, mistral, qwen, zai) — full source map in research/2026-10-02-external-review-synthesis-requirements-rough.md

---

## A. Requirements matrix

Legend — **Demand**: count of independent vendor proposals converging. **Surface**: what of the format it touches (S=section, K=kind set, R=reference typing, C=checker, P=pipeline/compile). **Fit**: does the format's existing six-move extension pattern cover it as-is (✅), partially (◑), or not at all (✗). **Tier**: enforcement stance every vendor converged on. **Priority**: MoSCoW for a first-class extension mechanism, not for any single domain pack.

| # | Requirement (distilled) | Demand | Surface | Fit | Tier | Priority | Risk if ignored |
|---|---|---|---|---|---|---|---|
| R1 | Quantities/units/dimensions vocabulary | 4/4 vendors | S, K, C | ◑ (section+kind moves exist; no unit registry pattern) | gating lint, honest-empty eval | Should | Each domain pack reinvents units differently → incompatible corpora |
| R2 | Machine-checkable numeric predicates in guards | 4/4 | K, C, P | ◑ (guard typing exists; no closed-predicate-grammar pattern) | compiled into `invariants_checked`, honest-empty | Must | `prose_untouched` keeps the strongest domain constraints permanently uncheckable |
| R3 | Data/lineage layer + external dataset binding | 4/4 (incl. grok's "bridge, don't absorb" counter-position agreeing binding must be typed) | S, R, C | ◑ (outbound-leaf pattern proven by `satisfies`/`observes`; no data-row shape) | lint advisory, verify fail-closed on checksum | Should | datasets/ground truth smuggled into prose; reproducibility claims unverifiable |
| R4 | Statistical/tolerance property kinds w/ per-kind required-case floors | 3/4 | K, C | ✅ (law-floor machinery generalizes; floors are just different label sets) | law checker reuses `**name:**` machinery | Should | kind collisions between packs (2 vendors claimed `tolerance` (different layers) + `statistic` in a third) |
| R5 | Composition/wiring (open systems, ports, monoidal structure) | 4/4 | S, K, C, P | ◑ (Moore `emits` only; no port/wire row shapes; theory supports it) | lint (ports typed, no dangling wires), compile phased | Could | format stays traceability-only; composition described in prose |
| R6 | Time / hybrid continuous-discrete models | 3/4 | S, K, C | ✗ (no temporal clause grammar, no continuous vars) | model-check bound honesty | Could | settlement windows/staleness unrepresentable |
| R7 | EARS can't state quantitative requirements | 1 (mistral, but blocks R1/R2 authoring) | K (intent statement patterns) | ◑ (EARS set is closed; sixth pattern is append-only) | lint | Should | authors fight the grammar on their most important sentences |
| R8 | The extension mechanism itself is first-class (packs/profiles) | 2 explicit (zai, grok-bioimage); implicit in all | **all** | ✗ — mechanism is followed by convention only | capability negotiation via doctor; parallel-branch checkers | **Must** | every pack collides in global closed sets; no versioning/namespacing |

**Priority note:** R8 is the only *Must* — it is the meta-requirement that makes R1–R7 additive, namespaced, and non-colliding. R1–R7 become **domain packs riding R8**, not format revisions.

## B. Rough-edges matrix

| # | Rough edge (source) | Blocks/Shapes | Resolution direction |
|---|---|---|---|
| E1 | `kind` overload: 𝒦 objects vs row sub-kind column (grok, CLAR-002) | R8, R4 | namespaced kinds (`quant.tolerance`) shrink the collision surface; possibly a rename traveling `rename_naturality` |
| E2 | Global closed kind sets don't scale to packs (2 vendors claimed `tolerance`, different layers) | R8, R4 | pack-scoped variant sets under `append_only_variants` |
| E3 | `prose_untouched` ↔ domain semantics tension (all four) | R2, R1, R3 | closed predicate grammars + typed data rows — check *structure of the claim*, never prose |
| E4 | EARS gap for quantitative statements (mistral) | R1, R2 authoring | sixth append-only pattern `WITHIN <ε> [<unit>]` |
| E5 | `law_requires_cases` floor not domain-universal (mistral bioimage; its only breaking change) | R4 | floor registry per kind — data, not code; enumeration form unchanged |
| E6 | Mealy/Moore gap, no 𝒦ᵒᵖ, no 2-cells (grok CT) | R5 | conservative additive duals; theory.md already carries the vocabulary |
| E7 | Honest-empty convention is the safety rail (all four praise `invariants_checked: []`) | all | every new pass fails open: `tolerance_checked: []`, `evaluated: []`, `statistical_invariants_checked: []` |
| E8 | Grounding variance in the review corpus (qwen invented flags; zai stale snapshot) | trust of this synthesis | grok/mistral read live v0.4.0 and are format-conformant — weight accordingly; re-verify any qwen claim |

## C. Mechanism gap analysis — the six moves vs. what's not first-class

Every vendor's proposal reduces to six recurring moves. Five already exist as format law; the gap is that they are *conventions*, not declared objects.

| Move | Exists today? | First-class form needed |
|---|---|---|
| 1. Optional new section (sub-layer) | as a table convention | pack declares the sections it introduces; possibly `section_kind_closed` |
| 2. Append-only kind growth under a Revision | `append_only_variants` law | growth scoped/namespaced per pack, not format-global |
| 3. New outbound-leaf reference fields | Reference Typing table | pack-declared typing rows; `single_root_reachable` untouched |
| 4. Checker spec in ownership DAG (parallel branch, non-gating unless opted in) | Checker Ownership table | pack registers its checkers; doctor reports enabled set |
| 5. Required-case-label floor over `**name:**` machinery | law checker | floor registry per kind, pack-provided |
| 6. Advisory-first, honest-empty enforcement | report honesty conventions | stays mandatory for every pack pass |

**Decision rows (open questions to grill):**
- D1: Is a pack a `kind: profile` intent spec + machine-readable manifest section, or a new artifact kind? *(every vendor's pack was a four-layer intent file — strong signal for the former)*
- D2: Do optional sections need registration in a closed set, or does "anything goes if a registered checker owns it" hold?
- D3: Overlapping vocabulary across packs — namespace law, first-past-the-post, or workspace-level reconciliation?
- D4: Does theory.md's Grothendieck namespacing extend to pack-qualified kinds, or is that overreach for v1?
- D5: Can the existing row-level `extension_point`/`satisfies` edge carry pack capability declaration (the mechanism may already exist at row granularity)?
- D6: Which of R1–R7 ships as the pilot pack? (bioimage-data and quant both have the most complete vendor designs to crib from)

## D. Invariants every vendor preserved (non-negotiable for any mechanism design)

`append_only_variants` · `prose_untouched` · typed `[[wiki-link]]` foreign keys · DAG-ness · `single_root_reachable` (outbound leaves, no carve-outs) · byte-stable compile artifacts · self-hosting (the extension mechanism must itself be expressible as a spec in the corpus) · honest-empty reporting · advisory-before-gating adoption path.

---
tags: [design]
tracks:
- .wai/projects/domain-specific-extensions/designs/matrix
---

# Design: profile-intent

Decision: 02-profile-intent
Date: 2026-10-02T19:40:18Z
Matrix: .wai/projects/domain-specific-extensions/designs/matrix/

## Rationale

Profile intent + manifest section wins. Evidence: only approach green on pack-namespacing, capability-negotiation, invariant-preservation, core-minimality and discoverability while keeping migration cost zero for the existing corpus. Decided under the strict thin-core position (D7): core grows once for the mechanism itself (kind: profile as an explicit Revision delta), everything else rides packs as Grothendieck fibers one level up (theory.md precedent) — closed sets become fiber-relative and stop growing. D1 v2 (Ro5-reviewed, TypeSafe-verified on the HIGH findings): pack = four-layer kind: profile spec + six per-facet manifest tables (Sections/Kinds/References/Checkers/Floors/Requires) + corpus-scan discovery (no config file) + advisory-first vocabulary-triggered opt-in + draft→published→deprecated lifecycle. 04-external-manifest rejected: violates the self-hosting ethic every vendor preserved. 03-row-level-extension-point rejected: the edge cannot carry section/kind/floor registration (its column shows red on floors, time, EARS, invariants). 01-status-quo rejected: global closed sets collide (2 vendors claimed tolerance on different layers; two mistral proposals collided on ## Data). D6: pilot pack = bioimage-data (most mechanically complete vendor design: spec-first checkers, outbound-leaf validated_against proof, rank-≤6 handoff unification; exercises data/lineage + numeric predicates + empirical-kind standard packs at once); quant second. Status quo's zero-migration advantage is preserved by design: packs are optional, files without packs lint identically.

## Trade-offs

(describe the trade-offs accepted)

## Decision-time snapshot — winning column

### 01-quantities-units

Pack manifest declares its Quantities section and namespaced kind growth
(quant.tolerance); the workspace units table stays an extension point the
manifest cites. grok-fpa and mistral-quant designs drop in as packs without
touching the global kind set.

### 02-numeric-guards

The manifest registers the pack's closed predicate grammar + its checker
(mistral's 5-predicate grammar becomes manifest data + one ownership-table
row). Guards citing pack constraints join invariants_checked per the
existing honesty convention; prose-only guards stay uninterpreted.

### 03-data-lineage-binding

Manifest declares the ## Data section and types validated_against as an
outbound leaf — mistral D4 proved no single_root_reachable carve-out is
needed. Checksum fail-closed verify stays a pack-owned checker + verify rule.

### 04-per-kind-floors

The floor registry is manifest data: pack states required case labels per
kind; the law checker's **name:** machinery is reused unchanged. E5 turns
from mistral's breaking change into a manifest row.

### 05-composition-wiring

Ports/Boxes/Wires become pack-declared optional sections; open_system Model
mode is pack-scoped kind growth under a Revision heading (grok's
compatibility statement already claims append-only + zero migration).
Compile targets phase in per the pack, not the format.

### 06-time-hybrid

Temporal clauses and Continuous subsections are pack-declared; two packs
with incompatible temporal grammars coexist namespaced instead of forking
the format grammar.

### 07-quantitative-ears

Open sub-question: statement patterns are parsed in frontmatter, so a pack-
declared sixth pattern needs the format's EARS validator to consult enabled
packs — a small, bounded extension point in one validator.

### 08-pack-namespacing

Core strength: kind growth is namespaced in the manifest (quant.tolerance),
resolving E1/E2. theory.md's Grothendieck namespacing story extends
naturally to pack-qualified ids (open question D4: formalize or defer).

### 09-capability-negotiation

doctor/lint read manifests and report the enabled set; checkers gate only
files that opt in; honest-empty per pack (tolerance_checked: []). This is
the negotiation every vendor described, made machine-readable.

### 10-invariant-preservation

Enforceable: a checker validates manifests against the base format_revision
(a pack may only add, never narrow — append_only_variants applies to pack
manifests themselves). The invariant becomes lintable, not conventional.

### 11-migration-cost

Zero corpus cost (packs optional, files without packs lint identically).
Tooling cost: manifest parsing, namespacing in lint/graph, doctor
negotiation — one bounded mechanism instead of N ad hoc wiring.

### 12-discoverability

Self-hosting: the pack IS a four-layer spec in the corpus — lintable,
graphable (satisfies/observes edges), explainable (spk explain topics render
from manifests). zai's community-pack vision lands here.

### 13-pack-revision-compat

The manifest carries a base format_revision (approach description), so
doctor/lint can warn when the corpus revision moves past a pack's base —
the knowledge-currency warning pattern already shipped in spk doctor
(Revision N vs embedded FORMAT_REVISION) generalizes to per-pack bases.

### 14-section-collision

The manifest registers section names, so a checker can detect the collision
at enable time. Whether section names are pack-namespaced (like kinds) or
registry-claimed first-come is open sub-question D2 — either way the
collision becomes detectable instead of silent.

### 15-core-minimality

Core grows once — the pack mechanism itself — then stays thin: kind/section
membership becomes fiber-relative (pack-qualified ids are the existing
Grothendieck construction one level up, per theory.md), and R1-R6 ship as
standard packs rather than core layers. The one instance-level exception
(EARS pattern) stays a genuine, tiny Revision.


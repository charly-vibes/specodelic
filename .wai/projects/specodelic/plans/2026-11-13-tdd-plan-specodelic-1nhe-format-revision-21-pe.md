---
tags: [pipeline-run:tdd-ro5-2026-11-13-specodelic-1nhe-law-kind-per-case-fragment-bodies, pipeline-step:plan]
---

TDD plan — specodelic-1nhe (format Revision 21: per-case fragment bodies for law-kind Property rows)

DESIRED BEHAVIOR
1. Grammar: a law-kind Property row's predicate cell enumerates cases as '**name:**' labels (unchanged, Revision 13). A per-case executable body is introduced by a CASE KEY occurrence '**name::**' immediately followed by the fragment marker in fragment position (a backtick directly before the marker): '**identity::** `**rust:** <expr>`'. Bodies form a map keyed by the enumerated labels; one body per case.
2. Labeled extraction failures (compile stage 'fragment_extraction', message names the rule and the offending case): a fragment in fragment position not associated with a case key; two bodies for one case; a body whose key names a case the predicate does not enumerate; a case key on a non-law row.
3. Emission: properties_to_proptest emits one block per case (cardinality unchanged); the block of a case that has a body carries that body verbatim; a case without a body keeps the honest todo_predicate! placeholder. verify's both_gates_required is untouched — never a passing stub.
4. One shared parser (src/spec.rs) reads the cases for BOTH lint's law_cases rule and compile's block expansion; the '**rust:**' marker is never counted as a case label; a law row with bodies but no labels still fails law_cases; an empty label set falls back to the identity/associativity floor.
5. Back-compat: cells that compiled before compile byte-identically (rust_back_compat); the 'at most one executable-fragment marker per cell' wording is amended for law rows in the same Revision.

OUT OF SCOPE
- Migrating the corpus law rows (specodelic.md rename_naturality, kinds.md kind_shape_naturality) — lf3's follow-up.
- Widening verify-specs beyond [compile, packs]; checker re-implementation in fragments; weakening any invariant.

TEST CASES (new tests/law_case_bodies.rs; RED first)
1. per_case_extraction: two case keys -> two bodies keyed by their labels (shared parser).
2. per_case_emission: block count == case count; the keyed case's block carries its body; the other block keeps todo_predicate!.
3. placeholder_fallback: a case with no body keeps the placeholder.
4. labeled_failures: unassociated fragment / duplicate body / unknown case label / case key on a non-law row — each a compile error naming the rule and the case.
5. label_marker_interaction: '**rust:**' is never a case label; law row with bodies but no labels fails lint law_cases; empty label set falls back to identity+associativity.
6. back_compat: a pre-Revision-21 cell compiles byte-identically.

NARROW TEST COMMAND: cargo test --test law_case_bodies (plus cargo test --lib for the unit-level parser).
FULL VERIFICATION: just ci; ah check --run-tests --all-tests; just lint-specs; just verify-specs; openspec validate --all --strict.

FILES LIKELY TOUCHED: openspec/specs/specodelic/spec.md (Revision 21 heading + row rewording), openspec/specs/compile/spec.md (per-case rows + amended one-marker wording), openspec/specs/linter-coverage/spec.md (law_cases wording only if needed), src/spec.rs (shared law-case parser), src/compile.rs (validate + emission), src/lint/observability.rs (law_cases reads the shared parser), src/migrate.rs (LATEST_REVISION 21 + entry), src/guide.rs + src/guide.md (FORMAT_REVISION / primer note), openspec/CHANGELOG.md, openspec/STATUS.md, AGENTS.md (managed block), tests/**, regenerated specodelic/*_props.rs.

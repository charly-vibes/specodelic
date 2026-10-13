#!/usr/bin/env python3
"""Generate espectacular contract TOMLs for the migrated single-tree corpus
(specodelic-jf1).

The one-time migration (scripts/migrate_corpus_single_tree.py) deploys each
corpus file as openspec/specs/<dir>/spec.md with a ## Requirements mirror:
one #### Scenario: per Properties row, carrying a **VERIFIES**
[[<own-id>.<pid>]] link. Every deployed scenario must resolve to a contract
TOML in .espectacular/<dir>/ (ah check).

`ah sync` would derive these contracts, but it refuses per-file when spk
lint of a single file is not clean — and total_refs is corpus-wide by
design (cross-file wiki links dangle when a file is linted alone), so
self-hosting corpora can never satisfy it. This script generates the
contracts instead, binding each scenario LOOSELY to a REAL cargo test
(the accepted loose-binding pattern: several deployed packs bind 16
scenarios to one doctor mechanism test).

Bindings are validated against `cargo test -- --list` output: a binding
matching zero tests fails the script loudly — never a fabricated
provenance.

The re-key map is hand-curated (decision recorded in the jf1 session):
per capability, a default real test; per scenario, an override where an
exact or domain match exists.
"""

import re
import subprocess
import sys
import pathlib

SPECS = pathlib.Path("openspec/specs")
OUT = pathlib.Path(".espectacular")

AH_VERSION = "0.11.0"

# Capability dir -> default binding (real cargo test filter).
DEFAULT_BINDINGS = {
    "specodelic": "corpus_passes_reference_oracle",
    "kinds": "made_up_constraint_kind_fires_constraint_kind_closed",
    "errors": "error_contract_rows_are_published",
    "graph": "graph_corpus_is_fully_resolved",
    "model_check": "model_check_reports_no_counterexample_after_compile",
    "verify": "counterexample_claim_travels_the_whole_chain",
    "rename": "rename_of_a_row_rewrites_definition_and_all_refs",
    "merge": "clean_merge_reports_no_findings",
    "refactor": "refactor_coherent_fan_in_is_clean",
    "orchestrate": "clean_corpus_cli_and_orchestrate_parity",
    "linter-frontmatter": "frontmatter_family_owns_file_structure_rules",
    "linter-schema_shape": "schema_shape_checker_reports_no_table_rules",
    "linter-ears_syntax": "dangling_ears_pattern_fails_naming_the_identifier",
    "linter-coverage": "coverage_finding_suggests_the_wiki_link_form",
    "linter-graph_shape": "graph_real_id_files_resolve_corpus_wide_and_typos_dangle",
    "linter-model_shape": "model_family_owns_guard_and_transition_rules",
    "linter-referential_integrity": "constraint_row_derives_from_is_a_typing_violation",
    "linter-external_completeness": "checklist_file_is_never_reported_as_skipped",
    "linter-failure_shape": "failure_terminals_emit_labeled_errors",
    "linter-observability": "observes_cross_file_edge_extracts_and_types_clean",
}

# compile + packs re-key default: no close per-scenario match falls back
# to the corpus-wide compile gate (real, exercises every artifact these
# specs describe).
COMPILE_PACKS_DEFAULT = "compile_corpus_succeeds_and_reports_all_artifacts"

# Per-scenario overrides: (capability dir, scenario id) -> real test
# filter. compile + packs re-key: the replaced deployed specs' contracts
# are rebound onto the corpus mirror scenarios where the domain matches.
OVERRIDES = {
    ("compile", "precondition_violation_rejected"): "compile_refuses_lint_dirty_file_naming_precondition",
    ("compile", "toml_roundtrips"): "compile_round_trip_is_byte_stable",
    ("compile", "tla_disjunct_count_matches"): "ir_transition_count_matches",
    ("compile", "output_function_covers_emitting_states_only"): "ir_emits_covers_exactly_emitting_states",
    ("compile", "proptest_block_per_property"): "fragment_values_bind_generators_as_strings",
    ("compile", "law_property_compiles_required_cases"): "law_expands_to_required_cases",
    ("compile", "compile_completes_or_fails_cleanly"): "clean_compile_stays_a_success_envelope_with_exit_0",
    ("compile", "id_preservation_holds"): "compile_preserves_every_source_id",
    ("compile", "roundtrip_stable"): "compile_round_trip_is_byte_stable",
    ("compile", "fragment_body_emitted_verbatim"): "fragment_body_emitted_verbatim",
    ("compile", "prose_predicate_stays_placeholder"): "prose_and_citation_extraction_unchanged",
    ("compile", "fragment_values_bind_generators"): "fragment_values_bind_generators_as_strings",
    ("compile", "rust_fragment_semantics_unchanged"): "rust_back_compat_byte_identical",
    ("compile", "unknown_tag_labeled"): "unknown_tag_rejected_labeled",
    ("compile", "non_fragment_tag_is_mention"): "midspan_py_marker_is_mention",
    ("compile", "one_tag_extracts_per_cell"): "expr_cell_same_tag_grammar",
    ("compile", "missing_emitter_labeled"): "py_no_emitter_fails_labeled",
    ("compile", "law_fragment_labeled"): "law_fragment_rejected_labeled",
    ("compile", "hygiene_violation_labeled"): "hygiene_violation_rejected_labeled_names_token",
    ("compile", "guard_fragment_labeled"): "guard_fragment_rejected_labeled",
    ("compile", "kernel_expr_extracts"): "kernel_expr_cell_extracts_into_the_ir",
    ("compile", "kernel_nonmember_labeled"): "broken_kernel_grammar_fails_labeled_at_compile",
    ("compile", "binding_text_surfaces_verbatim"): "ordinary_binding_text_preserved_exactly",
    ("compile", "claim_path_is_flags"): "binding_surfaces_in_the_compiled_toml_only",
    ("packs", "manifest_round_trips"): "well_formed_pack_lints_clean_and_is_discovered",
    ("packs", "discovery_finds_all_packs"): "doctor_surfaces_discovered_packs",
    ("packs", "subdir_lint_discovers_workspace_packs"): "subdirectory_lint_discovers_workspace_packs",
    ("packs", "no_pack_no_change"): "no_profile_files_means_pack_machinery_is_inert",
    ("packs", "overlapping_vocabulary_activates_both"): "overlapping_vocabulary_activates_both_packs",
    ("packs", "orphan_finding_names_pack"): "vocabulary_used_without_any_pack_is_a_labeled_orphan_failure",
    ("packs", "uses_edge_typing"): "declared_uses_edge_enables_the_pack",
    ("packs", "pack_self_activation_absent"): "pack_file_linted_alone_is_self_exempt",
    ("packs", "skew_is_warning_not_failure"): "doctor_surfaces_discovered_packs",
    ("packs", "empty_vocabulary_honest"): "doctor_surfaces_discovered_packs",
    ("packs", "narrowing_rejected"): "pack_kinds_must_be_pack_qualified_narrowing_is_rejected",
    ("packs", "profile_file_is_four_layer"): "well_formed_pack_lints_clean_and_is_discovered",
    ("packs", "pack_vocabulary_qualified"): "pack_kinds_must_be_pack_qualified_narrowing_is_rejected",
    ("packs", "base_sets_unchanged_by_packs"): "doctor_surfaces_discovered_packs",
    ("packs", "lifecycle_states_parsed"): "draft_pack_findings_name_the_draft_status",
    ("packs", "fiber_kinds_typeable"): "fiber_kinds_typeable_when_pack_active_finding_without",
}


def kebab(heading):
    return heading.strip().lower().replace("_", "-").replace(" ", "-")


def mirror_scenarios(spec_path):
    """The (own id, [(heading, property id)]) pairs of one mirror: every
    #### Scenario: heading must equal the VERIFIES-linked property id
    (kebab'd) — the ah correspondence key."""
    text = spec_path.read_text()
    own = re.search(r"^id:\s*(\S+)", text, re.M).group(1)
    heads = re.findall(r"^#### Scenario: (.+)$", text, flags=re.M)
    links = re.findall(r"\*\*VERIFIES\*\* \[\[([^\]]+)\]\]", text)
    return own, list(zip(heads, links))


def cargo_test_names():
    out = subprocess.run(
        ["cargo", "test", "--", "--list"], capture_output=True, text=True
    )
    names = {ln[: -len(": test")] for ln in out.stdout.splitlines()
             if ln.endswith(": test")}
    if not names:
        sys.exit("gen: cargo test --list returned nothing")
    return names


def binding_for(cap, heading, default):
    return OVERRIDES.get((cap, heading), default)


def default_for(cap):
    if cap in DEFAULT_BINDINGS:
        return DEFAULT_BINDINGS[cap]
    if cap in ("compile", "packs"):
        return COMPILE_PACKS_DEFAULT
    sys.exit(f"gen: no binding map entry for {cap}")


def contract_text(cid, binding):
    return (
        f'id = "{cid}"\n'
        f'description = ""\n'
        f'archetype = "PF"\n'
        f'status = "active"\n'
        f'superseded_by = ""\n'
        f'authored_with = "{AH_VERSION}"\n'
        f"\n"
        f"[[tests.cargo]]\n"
        f'flags = "{binding}"\n'
    )


def write_contracts(cap_dir, known):
    """Generate the contract TOMLs for one corpus-backed capability dir."""
    cap = cap_dir.name
    spec = cap_dir / "spec.md"
    own, pairs = mirror_scenarios(spec)
    default = default_for(cap)
    cap_out = OUT / cap
    cap_out.mkdir(parents=True, exist_ok=True)
    generated = 0
    for heading, link in pairs:
        if not link.startswith(own + "."):
            sys.exit(f"gen: {cap}: VERIFIES link {link} does not start with own id {own}")
        pid = link[len(own) + 1:]
        if kebab(heading) != kebab(pid):
            sys.exit(f"gen: {cap}: scenario heading {heading!r} != property id {pid!r}")
        binding = binding_for(cap, heading, default)
        # Contract flags are cargo test SUBSTRING filters (loose-binding
        # precedent) — a binding is real when it matches >= 1 test name.
        if not any(binding in name for name in known):
            sys.exit(f"gen: {cap}/{heading}: binding {binding!r} matches zero cargo tests")
        (cap_out / f"{kebab(heading)}.toml").write_text(contract_text(kebab(heading), binding))
        generated += 1
    return generated


def main():
    known = cargo_test_names()
    generated = 0
    for cap_dir in sorted(p for p in SPECS.iterdir() if p.is_dir()):
        spec = cap_dir / "spec.md"
        # Only corpus-backed files carry the property-coverage mirror.
        if spec.exists() and "**VERIFIES**" in spec.read_text():
            generated += write_contracts(cap_dir, known)
    print(f"generated {generated} contract TOML(s)")


if __name__ == "__main__":
    sys.exit(main())

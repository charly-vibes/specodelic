---
tags: [pipeline-run:tdd-ro5-2026-11-24-qbox-artifact-gate-lints-the-judged-file-in-corpus, pipeline-step:refactor]
---

REFACTOR (2026-11-24, specodelic-qbox): the GREEN shape was tightened before the suite was re-run — (1) clippy's too_many_arguments rejected the 8-argument cmd_check_in_corpus and emit_judged, so the corpus context and its warnings are bundled into a new check::CorpusGate struct (corpus + warnings, with CorpusGate::isolated() for the labeled fallback) and emit_judged now takes the Judgement (report/exit/contradicted) instead of three loose scalars — both are back to 7 arguments; (2) the isolation label is emitted once (CorpusGate::isolated() carries it; cmd_check_in_corpus only adds it when a caller built a gate without one), so conform and check cannot drift into two wordings; (3) the labels themselves live in the library (conform::ISOLATED_GATE_WARNING, conform::partial_corpus_warning) — the binary never words a gate fact; (4) clippy --all-targets (stricter than the repo's 
running 460 tests
test acset::doc::tests::a_changed_header_is_an_error ... ok
test acset::doc::tests::an_uninterpretable_cell_is_an_error ... ok
test acset::doc::tests::later_sentence_object_mentions_are_not_claims ... ok
test acset::doc::tests::the_section_grammar_reads_today_s_table ... ok
test acset::doc::tests::doc_missing_one_row_of_the_schema ... ok
test acset::doc::tests::schema_missing_one_row_of_the_typing_table ... ok
test acset::instance::tests::model_anchor_link_is_a_node_not_a_panic ... ok
test acset::instance::tests::dangling_is_a_value ... ok
test acset::instance::tests::attribute_cells_carried_untouched ... ok
test acset::instance::tests::forbidden_edge_not_stored ... ok
test acset::schema::tests::emits_refinement_enforced ... ok
test acset::schema::tests::duplicate_morphism_rejected ... ok
test acset::schema::tests::canonical_schema_is_valid ... ok
test acset::instance::tests::no_link_dropped ... ok
test acset::schema::tests::endo_cycle_detected ... ok
test acset::schema::tests::sixth_object_rejected ... ok
test acset::instance::tests::rebuild_is_byte_stable ... ok
test acset::schema::tests::typing_read_from_schema ... ok
test acset::schema::tests::typing_reasons_match_pinned_prose ... ok
test acset::schema::tests::unflagged_endo_cycle_tolerated ... ok
test archive_companion::tests::resolve_archive_dir_none_is_labeled_error ... ok
test acset::instance::tests::duplicate_id_first_wins_parity ... ok
test archive_companion::tests::restore_empty_delta_set_is_success ... ok
test archive_companion::tests::resolve_archive_dir_newest_wins ... ok
test archive_companion::tests::restore_refuses_sidecar_missing_a_table ... ok
test archive_companion::tests::restore_refuses_sidecarless_delta_before_any_write ... ok
test archive_companion::tests::restore_post_attach_shape_assertion_refuses_delta_headers ... ok
test archive_companion::tests::run_dry_run_refuses_sidecarless_delta ... ok
test archive_companion::tests::run_dry_run_never_invokes_or_writes ... ok
test archive_companion::tests::run_open_spec_failure_is_labeled ... ok
test archive_companion::tests::run_refuses_sidecarless_active_change_before_invocation ... ok
test archive_companion::tests::run_open_spec_missing_is_labeled ... ok
test archive_companion::tests::restore_recognizes_already_deployed_sidecarless_archive ... ok
test archive_companion::tests::run_refuses_unlintable_sidecar_before_invocation ... ok
test archive_companion::tests::run_dry_run_over_active_change_reports_plan ... ok
test archive_companion::tests::run_unknown_change_is_labeled_error ... ok
test archive_companion::tests::run_reverts_archive_move_on_attach_violation ... ok
test audit::binds_nothing_tests::declared_model_ids_covers_every_model_element_never_the_intent ... ok
test audit::binds_nothing_tests::binds_nothing_round_trips_and_rides_rank_1 ... ok
test audit::binds_nothing_tests::non_contradiction_rows_keep_the_currency_overlay ... ok
test audit::binds_nothing_tests::resolved_contradiction_is_current_and_passes ... ok
test audit::binds_nothing_tests::sustained_contradiction_stays_contradicted_and_fails_the_run ... ok
test audit::binds_nothing_tests::unseen_contradiction_stays_contradicted_not_new ... ok
test audit::binds_nothing_tests::honesty_mix_exits_zero_with_counts_in_both_views ... ok
test audit::binds_nothing_tests::citation_match_is_whole_token_never_substring ... ok
test audit::binds_nothing_tests::fragment_over_an_empty_model_binds_nothing ... ok
test audit::binds_nothing_tests::sparse_phase_model_is_never_flagged ... ok
test audit::diff_tests::digest_mismatch_is_a_labeled_refusal ... ok
test audit::binds_nothing_tests::vacuous_evidence_units_flag_binds_nothing ... ok
test audit::diff_tests::empty_prior_report_makes_every_row_new ... ok
test audit::diff_tests::worsened_class_is_stale_and_a_fresh_contradiction_stays_contradicted ... ok
test audit::diff_tests::overlay_class_ranks_worsening_only ... ok
test audit::diff_tests::same_or_improved_class_is_current ... ok
test audit::diff_tests::row_absent_from_prior_is_new ... ok
test audit::diff_tests::schema_version_mismatch_is_a_labeled_refusal ... ok
test audit::pack_row_tests::all_unchecked_pack_rows_exit_zero_with_counts ... ok
test audit::pack_row_tests::empty_checked_set_is_unchecked_never_a_pass ... ok
test audit::emission_tests::emission_is_both_layered_across_row_kinds ... ok
test audit::emission_tests::count_maps_agree_between_the_compute_sites ... ok
test audit::emission_tests::row_order_is_key_sorted_across_kinds_in_both_paths ... ok
test audit::pack_row_tests::violation_rows_carry_the_real_verdict ... ok
test audit::emission_tests::pack_row_key_collision_disambiguates_the_folded_row ... ok
test audit::emission_tests::oracle_row_id_colliding_with_a_bind_row_key_never_duplicates_scenario_ids ... ok
test audit::pack_row_tests::pack_rows_fold_alongside_oracle_rows ... ok
test audit::pack_row_tests::pack_violation_rows_fail_the_run ... ok
test audit::pack_row_tests::pack_rows_count_toward_report_totals ... ok
test checklist::tests::backticked_status_cells_are_accepted ... ok
test checklist::tests::deeper_header_inside_mapping_does_not_reset_parsing ... ok
test blocks::tests::block_content_lists_every_rule_and_revision ... ok
test checklist::tests::detects_checklist_paths ... ok
test checklist::tests::duplicate_item_id_is_a_defect ... ok
test checklist::tests::item_missing_its_description_is_a_defect ... ok
test blocks::discovery_tests::register_creates_manifest_with_specodelic_entry ... ok
test checklist::tests::bom_is_tolerated ... ok
test checklist::tests::item_missing_its_id_is_a_defect ... ok
test checklist::tests::mapped_ids_accept_bare_and_wikilinked_comma_lists ... ok
test checklist::tests::mapping_row_for_undeclared_item_is_a_defect ... ok
test checklist::tests::missing_sections_are_defects ... ok
test checklist::tests::nested_item_is_a_defect ... ok
test checklist::tests::empty_manifest_declaring_no_items_is_a_defect ... ok
test checklist::tests::parses_well_formed_manifest ... ok
test checklist::tests::mapping_row_with_wrong_cell_count_is_a_defect ... ok
test checklist::tests::rewrite_mapped_ids_cell_preserves_nonmatches_and_padding ... ok
test blocks::discovery_tests::register_is_idempotent_and_merges_with_siblings ... ok
test archive_companion::tests::restore_post_attach_lint_failure_reverts_write ... ok
test blocks::tests::inject_is_idempotent_and_refreshes ... ok
test archive_companion::tests::restore_new_capability_deploys_from_sidecar_tables ... ok
test archive_companion::tests::restore_replaces_archiver_placeholder_with_delta_purpose ... ok
test checklist::tests::rewrite_mapped_ids_cell_renames_bare_wrapped_and_children ... ok
test archive_companion::tests::restore_rekeys_sidecar_id_and_refs ... ok
test checklist::tests::partial_section_manifests_report_their_missing_half ... ok
test archive_companion::tests::restore_newest_archive_wins ... ok
test archive_companion::tests::restore_keeps_real_deployed_purpose_over_delta_purpose ... ok
test archive_companion::tests::run_already_archived_skips_openspec ... ok
test compile::tests::compile_refuses_stateless_spec_with_labeled_failure ... ok
test checklist::tests::rewrite_text_updates_only_mapped_ids_cells ... ok
test checklist::tests::wrong_mapping_columns_are_a_defect ... ok
test archive_companion::tests::restore_assembles_dual_format_from_sidecar_and_applied_spec ... ok
test compile::tests::guard_citations_extracted_from_invariant_rows_prose_excluded ... ok
test compile::tests::ir_emits_covers_exactly_emitting_states ... ok
test compile::tests::ir_transition_count_matches ... ok
test compile::tests::sanitize_ident_leading_digit_and_unicode_are_valid_rust_idents ... ok
test compile::tests::sanitize_ident_never_yields_a_rust_keyword_or_empty ... ok
test compile::tests::sanitize_ident_caps_extremely_long_ids_and_stays_unique_deterministic ... ok
test compile::tests::tla_artifact_is_byte_stable ... ok
test compile::tests::tla_disjunct_count_matches_transitions_plus_stuttering ... ok
test compile::tests::tla_no_emits_means_empty_output ... ok
test compile::tests::tla_module_header_is_single_line_tla_plus_form ... ok
test compile::tests::tla_module_name_sanitized_and_guard_verbatim ... ok
test compile::tests::tla_output_covers_emitting_states_only ... ok
test compile::tests::toml_ids_preserved_verbatim ... ok
test compile::tests::toml_roundtrips_field_for_field ... ok
test doctor::tests::beads_check_passes_when_initialized ... ok
test doctor::tests::block_check_warns_when_missing_and_names_the_fix ... ok
test doctor::tests::block_check_passes_when_current ... ok
test doctor::tests::block_check_warns_when_stale ... ok
test doctor::tests::beads_check_warns_when_missing ... ok
test doctor::tests::discovery_advises_when_no_corpus ... ok
test doctor::tests::consumer_report_has_zero_failures_even_when_not_healthy ... ok
test doctor::tests::block_check_is_auto_fixable_and_fix_injects ... ok
test doctor::tests::discovery_passes_when_specs_dir_exists ... ok
test doctor::tests::mode_is_consumer_otherwise ... ok
test doctor::tests::discovery_passes_on_openspec_tree_and_counts_files ... ok
test doctor::tests::mode_is_self_hosting_when_core_spec_present ... ok
test doctor::tests::mode_is_self_hosting_with_legacy_two_tree_corpus ... ok
test compile::tests::law_without_labels_falls_back_to_floor ... ok
test doctor::tests::payload_renders_findings_for_broken_workspaces ... ok
test doctor::tests::runner_fix_dispatch_repairs_the_block ... ok
test doctor::tests::payload_matches_the_hand_rolled_shape ... ok
test compile::tests::parameterless_generator_rows_emit_plain_test_fns ... ok
test ears::tests::event_driven ... ok
test compile::tests::parameterless_fragment_row_asserts_outside_proptest ... ok
test ears::tests::folded_frontmatter_lines_match ... ok
test doctor::tests::runner_report_never_fails_on_any_workspace_shape ... ok
test ears::tests::has_shall_works ... ok
test ears::tests::optional_feature ... ok
test ears::tests::state_driven ... ok
test compile::tests::law_expands_to_required_cases ... ok
test ears::tests::ubiquitous ... ok
test ears::tests::unwanted_behavior ... ok
test graph::tests::anchors_and_bare_local_rows ... ok
test graph::tests::dotted_file_id_resolves_last_dot_first ... ok
test graph::tests::dotted_row_ids_are_unaddressable ... ok
test ears::tests::no_shall_fails ... ok
test graph::tests::edge_projection_unknown_ids_carry_empty_kind ... ok
test graph::tests::edge_projection_normalizes_sorts_and_preserves_multiplicity ... ok
test graph::tests::member_path_resolves_in_first_dot_arm ... ok
test doctor::tests::runner_report_is_healthy_on_a_set_up_self_hosting_repo ... ok
test graph::tests::mermaid_escape_neutralizes_label_breakers ... ok
test graph::tests::render_wiring_dot_pins_sample_shape ... ok
test graph::tests::render_wiring_labels_empty_views ... ok
test graph::tests::render_wiring_mermaid_pins_exact_bytes ... ok
test graph::tests::wiring_projection_collapses_drops_and_aggregates ... ok
test graph::tests::wiring_projection_unattributable_endpoint_falls_back_to_itself ... ok
test graph::tests::typed_reference_columns_derive_from_schema ... ok
test guide::tests::conform_contract_topics_are_appended_after_conform ... ok
test guide::tests::claim_executable_topic_documents_the_permitted_rule ... ok
test guide::tests::dual_format_topic_documents_self_intent_trace_and_drift_rule ... ok
test guide::tests::graph_views_topic_is_appended_last ... ok
test guide::tests::format_topic_documents_escaped_pipes ... ok
test guide::tests::closed_sets_render_from_constants ... ok
test guide::tests::format_topic_points_at_the_file_qualified_ref_law ... ok
test guide::tests::guard_typing_admits_state_citation ... ok
test compile::tests::generators_get_spec_gen_helpers ... ok
test guide::tests::lifecycle_topic_states_the_claim_gate_and_assurance_distinction ... ok
test guide::tests::pack_mechanism_sets_grown_per_revision_14 ... ok
test compile::tests::unit_property_yields_at_least_one_block ... ok
test guide::tests::law_cases_machine_form_is_ratified_in_the_corpus ... ok
test guide::tests::format_revision_matches_corpus ... ok
test guide::tests::graph_views_topic_documents_taxonomy_flags_and_recipes ... ok
test compile::tests::colliding_fn_names_are_disambiguated ... ok
test guide::tests::oracle_records_topic_documents_the_jsonl_record_schema ... ok
test guide::tests::primer_prose_does_not_duplicate_closed_sets ... ok
test guide::tests::revision_comparison_is_numeric ... ok
test guide::tests::pack_mechanism_ratified_in_the_corpus ... ok
test guide::tests::predicate_fragments_ratified_in_the_corpus ... ok
test guide::tests::references_topic_documents_file_qualified_ref_syntax ... ok
test guide::tests::unknown_topic_renders_none ... ok
test guide::tests::schema_export_accepts_constructed_schemas ... ok
test guide::tests::value_set_payload_carries_kinds_row_shapes_and_revision ... ok
test guide::tests::schema_export_projects_canonical_schema ... ok
test guide::tests::verdict_taxonomy_topic_documents_the_closed_verdicts ... ok
test guide::tests::schema_export_is_deterministic ... ok
test hooks::tests::golden_install_uninstall_round_trip_is_byte_identical_2_space ... ok
test hooks::tests::install_appends_stage_section_when_missing ... ok
test hooks::tests::golden_install_uninstall_round_trip_is_byte_identical_4_space ... ok
test hooks::tests::install_handles_config_without_trailing_newline ... ok
test hooks::tests::install_delegates_to_genesis_ensure_command_wired ... ok
test hooks::tests::install_handles_crlf_config ... ok
test hooks::tests::install_inserts_inside_existing_commands_mapping ... ok
test hooks::tests::install_matches_children_indent_of_the_stage ... ok
test hooks::tests::install_is_idempotent ... ok
test hooks::tests::install_refuses_mis_indented_commands_key ... ok
test hooks::tests::install_refuses_prek_framework ... ok
test hooks::tests::install_refuses_without_modification_when_stage_key_unanchorable ... ok
test hooks::tests::uninstall_on_missing_config_is_a_no_op ... ok
test hooks::tests::install_uses_wrapper_when_stage_has_no_commands ... ok
test human::tests::archive_companion_human_summary ... ok
test hooks::tests::uninstall_leaves_appended_stage_as_documented_residue ... ok
test hooks::tests::uninstall_on_unwired_repo_is_a_no_op ... ok
test human::tests::debug_markers_never_appear ... ok
test human::tests::graph_text_names_dangling_refs ... ok
test hooks::tests::uninstall_strips_only_the_marked_block ... ok
test hooks::tests::uninstall_refuses_unbalanced_markers ... ok
test human::tests::lint_text_carries_rule_ids_and_counts ... ok
test guide::tests::topic_bodies_render_for_all_topics ... ok
test human::tests::payload_formatters_render_from_envelope_values ... ok
test lint::checker_tests::frontmatter_family_owns_file_structure_rules ... ok
test lint::checker_tests::ears_family_owns_statement_and_id_shape_rules ... ok
test compile::tests::compile_preserves_every_source_id ... ok
test archive_companion::tests::restore_leaves_the_deployed_text_unchanged_when_the_delta_has_no_purpose ... ok
test lint::checker_tests::model_family_owns_guard_and_transition_rules ... ok
test lint::checker_tests::referential_family_owns_unique_id_and_total_refs ... ok
test lint::checker_tests::graph_family_owns_acyclic_and_reachability ... ok
test lint::graph::declared_ids_pins::conn_edges_resolve_only_declared_state_endpoints ... ok
test lint::graph::declared_ids_pins::rows_projects_constraints_properties_states_never_transitions ... ok
test lint::tests::anchors_resolve_but_name_no_row ... ok
test lint::row_shape::tests::the_allowed_fields_follow_the_schema_as_data ... ok
test lint::revision::tests::a_silent_growth_is_found_and_a_declared_one_is_not ... ok
test lint::tests::bare_local_row_and_bare_file_id ... ok
test lint::revision::tests::a_removed_member_is_found ... ok
test archive_companion::tests::run_rerun_converges_to_identical_deployed_bytes ... ok
test hooks::tests::install_errors_on_missing_config_and_never_creates_one ... ok
test lint::row_shape::tests::an_extra_constraint_column_is_rejected ... ok
test lint::row_shape::tests::an_extra_state_field_is_rejected ... ok
test guide::tests::python_schema_view_renders_constructed_export_delta ... ok
test lint::checker_tests::made_up_property_kind_fires_property_kind_closed ... ok
test lint::checker_tests::schema_shape_checker_reports_no_table_rules ... ok
test lint::checker_tests::missing_kind_cell_fires_the_kind_closed_rule ... ok
test lint::checker_tests::made_up_constraint_kind_fires_constraint_kind_closed ... ok
test hooks::tests::install_refuses_husky_framework ... ok
test lint::tests::content_advisories::declared_forbidden_state_in_phase_model_stays_silent ... ok
test hooks::tests::install_never_touches_hook_files_or_hooks_path ... ok
test lint::checker_tests::families_composed_equal_the_flat_pass ... ok
test lint::tests::content_advisories::fragment_interior_is_not_a_placeholder ... ok
test lint::tests::content_advisories::fragment_citing_no_model_element_warns_binds_nothing ... ok
test lint::tests::content_advisories::invariant_citing_undeclared_forbidden_state_warns ... ok
test lint::tests::content_advisories::content_placeholder_and_tautology_warn_never_fail ... ok
test lint::tests::content_advisories::law_predicate_fragment_is_outside_the_vacuity_surface ... ok
test lint::tests::content_advisories::scaffold_id_prefix_without_scaffold_signature_is_silent ... ok
test lint::tests::content_advisories::stub_statement_alone_makes_scaffold_ids_warn ... ok
test lint::tests::dotted_file_id_row_member_resolves_in_last_dot_arm ... ok
test lint::tests::dotted_file_id_row_resolves ... ok
test lint::tests::dotted_row_ids_are_unaddressable ... ok
test lint::tests::content_advisories::statement_is_never_tautology_judged ... ok
test lint::tests::coverage_finding_suggests_the_wiki_link_form ... ok
test lint::tests::covered_target_must_be_a_row_not_a_file_or_anchor ... ok
test lint::tests::dangling_observes_is_total_refs_beat_not_observability ... ok
test lint::tests::exact_file_id_beats_split_arms ... ok
test lint::tests::delta_without_requirements_section_is_no_longer_a_finding ... ok
test lint::tests::content_advisories::path_qualified_api_call_exempts_binds_nothing ... ok
test lint::tests::dual_format_file_with_both_halves_is_clean ... ok
test lint::tests::duplicate_claim_rejected ... ok
test lint::tests::every_state_in_a_transition_passes ... ok
test lint::tests::dangling_mapped_id_rejected ... ok
test lint::tests::member_path_resolves_in_first_dot_arm ... ok
test lint::tests::metasyntactic_targets_are_skipped ... ok
test lint::tests::law_property_deriving_from_property_is_not_an_orphan ... ok
test lint::tests::fully_mapped_checklist_passes ... ok
test lint::tests::law_row_missing_one_floor_case_names_only_it ... ok
test lint::tests::content_advisories::declared_state_literal_counts_as_a_citation ... ok
test lint::tests::law_row_with_prose_only_cases_is_rejected ... ok
test lint::tests::law_row_with_floor_and_extra_case_is_clean ... ok
test lint::tests::content_advisories::tautology_judges_invariant_exprs_not_effect_exprs ... ok
test lint::tests::cross_file_only_rows_warn_advisory_own_file_chain_passes ... ok
test lint::tests::mirror_rules_retired_by_construction ... ok
test lint::tests::misspelled_floor_label_is_not_the_floor ... ok
test lint::tests::modified_delta_in_sync_lints_clean ... ok
test lint::tests::modified_delta_with_drifted_mirror_is_no_longer_a_finding ... ok
test lint::tests::no_checklist_is_not_applicable ... ok
test lint::tests::mutual_traces_cycle_fails_acyclic ... ok
test lint::tests::no_orphan_property_hint_teaches_wiki_link_syntax ... ok
test lint::tests::orphan_cluster_fails_single_root_reachable ... ok
test lint::tests::malformed_checklist_rejected ... ok
test lint::tests::orphan_property_fires_when_derives_from_resolves_to_property ... ok
test lint::tests::content_advisories::citation_proxy_ignores_literals_comments_and_short_ids ... ok
test lint::tests::plain_spec_without_added_section_is_exempt ... ok
test lint::tests::ref_kind::derives_from_state_from_unit_property_fires ... ok
test lint::tests::ref_kind::emits_citing_invariant_constraint_fires ... ok
test lint::tests::content_advisories::forbidden_state_scan_skips_comments_and_requires_the_state_binder ... ok
test lint::tests::ref_kind::expr_cell_links_are_not_typed ... ok
test lint::tests::ref_kind::finding_carries_semantics_and_remediation_hint ... ok
test lint::tests::ref_kind::guard_citing_advisory_constraint_fires ... ok
test lint::tests::ref_kind::guard_citing_state_passes ... ok
test lint::tests::observed_effect_is_silent_and_self_observation_does_not_count ... ok
test lint::tests::ref_kind::law_derives_from_law_property_passes ... ok
test lint::tests::revision18::modified_delta_with_non_spec_id_fires_dual_format_valid ... ok
test lint::tests::ref_kind::member_arm_is_typed_as_its_row ... ok
test lint::tests::revision18::real_id_dual_format_files_resolve_cross_file ... ok
test lint::tests::ref_kind::dangling_target_stays_total_refs_beat ... ok
test lint::tests::revision18::spec_md_derives_expected_id_from_parent_dir ... ok
test lint::tests::revision18::spec_md_underscore_dir_is_literal ... ok
test lint::tests::ref_kind::observes_non_effect_fires ... ok
test lint::tests::revision18::same_id_files_do_not_collide_in_reference_resolution ... ok
test lint::tests::rule_table_keys_are_unique ... ok
test lint::tests::revision18::spec_md_with_legacy_id_spec_fires_id_matches_file ... ok
test lint::tests::ref_kind::satisfies_non_extension_point_fires ... ok
test lint::tests::ref_kind::ref_kind_clean_baseline_lints_clean ... ok
test lint::tests::revision18::real_id_files_resolve_corpus_wide_and_typos_still_dangle ... ok
test lint::tests::ref_kind::supersedes_across_kinds_fires ... ok
test lint::tests::ref_kind::supersedes_same_kind_passes ... ok
test lint::tests::ref_kind::traces_to_constraint_fires ... ok
test lint::tests::revision18::dual_format_file_with_wrong_id_fires_id_matches_file ... ok
test lint::tests::revision18::spec_md_without_parent_dir_keeps_stem_derivation ... ok
test lint::tests::transition_to_undeclared_state_fails_every_transition_valid ... ok
test lint::tests::self_referencing_row_fails_no_self_ref ... ok
test lint::tests::total_refs_hint_absent_for_genuinely_unknown_targets ... ok
test merge::blast_radius_parity::dependents_forward_reach_joins_the_radius ... ok
test merge::textual_conflict_dedup::duplicate_path_in_tree_flags_textual_conflict_once ... ok
test migrate::rekey_tests::rekey_dry_run_semantics_via_outcome ... ok
test migrate::rekey_tests::rekey_idempotent_second_run_is_noop ... ok
test migrate::rekey_tests::rekey_preserves_prose_mentions ... ok
test migrate::rekey_tests::rekey_refuses_bare_spec_md ... ok
test migrate::rekey_tests::rekey_refuses_plain_delta ... ok
test migrate::rekey_tests::rekey_rewrites_id_and_refs ... ok
test migrate::revision_tests::file_revision_detects_the_pre_18_shape ... ok
test migrate::revision_tests::only_the_id_law_carries_a_step ... ok
test migrate::revision_tests::plan_applies_the_registered_steps_in_order ... ok
test migrate::revision_tests::plan_leaves_an_unchanged_file_byte_identical ... ok
test migrate::revision_tests::plan_refuses_unknown_and_downgrade_requests ... ok
test migrate::revision_tests::plan_skips_a_file_that_is_not_a_corpus_spec ... ok
test migrate::revision_tests::registry_covers_the_known_revision_range ... ok
test migrate::revision_tests::registry_entries_carry_a_note ... ok
test migrate::revision_tests::registry_latest_matches_the_embedded_format_revision ... ok
test migrate::tests::crlf_delta_gets_crlf_sidecar ... ok
test migrate::tests::degenerate_one_line_delta_is_refused_labeled ... ok
test migrate::tests::frontmatter_carrying_file_is_refused ... ok
test migrate::tests::no_delta_section_is_refused ... ok
test migrate::tests::no_delta_section_message_names_both_accepted_sections ... ok
test migrate::tests::plain_spec_is_refused ... ok
test lint::tests::unit_rows_never_trigger_law_cases ... ok
test migrate::tests::sidecar_id_ignores_the_delta_filename ... ok
test migrate::tests::wraps_plain_delta_into_a_sidecar_leaving_the_delta_alone ... ok
test model_check::tests::artifact_manifest_parses_into_shape ... ok
test model_check::tests::artifact_output_entry_without_spec_emits_is_rejected ... ok
test model_check::tests::backend_version_matches_the_cargo_dep ... ok
test model_check::tests::bound_defaults_to_max_depth_100 ... ok
test model_check::tests::consistency_checks_the_manifest_against_the_ir ... ok
test model_check::tests::cyclic_automaton_terminates_and_explores_each_state_once ... ok
test model_check::tests::deleted_disjunct_code_with_comment_left_behind_is_rejected ... ok
test model_check::tests::depth_budget_exhaustion_yields_timed_out ... ok
test model_check::tests::depth_cap_confirmation_never_runs_alongside_other_caps ... ok
test model_check::tests::depth_cap_equal_to_the_diameter_still_confirms_exploration ... ok
test model_check::tests::edited_code_edge_without_recompile_is_rejected ... ok
test model_check::tests::edited_model_without_recompile_is_a_stale_artifact_error ... ok
test model_check::tests::edited_transitions_without_recompile_are_rejected_too ... ok
test model_check::tests::empty_model_is_a_labeled_error ... ok
test model_check::tests::exhaustive_run_with_zero_invariants_is_never_reported_clean ... ok
test model_check::tests::exploration_is_exhaustive_within_the_bound ... ok
test model_check::tests::exploration_only_outcome_serializes_snake_case ... ok
test model_check::tests::exploration_run_names_backend_and_restates_bound ... ok
test model_check::tests::exploration_starts_at_the_first_listed_state ... ok
test model_check::tests::facts_map_to_outcomes ... ok
test model_check::tests::facts_parse_from_the_scratch_line ... ok
test model_check::tests::genuinely_truncated_depth_budget_still_yields_timed_out ... ok
test lint::tests::total_refs_hint_points_at_the_file_qualified_form ... ok
test model_check::tests::output_matching_the_ir_passes ... ok
test model_check::tests::output_missing_when_the_spec_emits_is_rejected ... ok
test model_check::tests::output_value_drift_without_recompile_is_rejected ... ok
test model_check::tests::prose_invariants_are_reported_unchecked_not_fabricated ... ok
test model_check::tests::report_serializes_outcome_snake_case_and_omits_absent_violation ... ok
test model_check::tests::scratch_source_compiles_fragments_verbatim ... ok
test model_check::tests::sha256_is_hex_and_deterministic ... ok
test model_check::tests::stale_report_detected_by_artifact_hash_change ... ok
test model_check::tests::state_budget_exhaustion_yields_timed_out ... ok
test model_check::tests::state_budget_wide_enough_for_the_space_yields_exploration_only ... ok
test model_check::tests::time_budget_exhaustion_yields_timed_out ... ok
test model_check::tests::tlc_completed_run_parses_as_completed_with_states ... ok
test lint::tests::unmapped_item_rejected ... ok
test model_check::tests::tlc_depth_cutoff_parses_as_not_completed ... ok
test lint::tests::ref_kind::uses_non_profile_file_fires ... ok
test lint::tests::unknown_status_leaves_item_unaccounted ... ok
test model_check::tests::non_emitted_module_is_rejected_not_silently_run ... ok
test model_check::tests::tlc_invariant_violation_is_detected ... ok
test model_check::tests::tlc_max_states_budget_is_unsupported_and_labeled ... ok
test model_check::tests::tlc_completed_run_reports_backend_tlc_and_exploration_only ... ok
test model_check::tests::tlc_run_with_missing_binary_is_a_labeled_error_never_a_verdict ... ok
test model_check::tests::tlc_run_with_missing_jar_is_a_labeled_error_never_a_verdict ... ok
test model_check::tests::tlc_version_line_parses ... ok
test model_check::tests::tlc_depth_cutoff_reports_timed_out ... ok
test model_check::tests::transition_referencing_an_unknown_state_is_a_labeled_error ... ok
test model_check::tests::tlc_engine_invariant_violation_is_a_labeled_error_not_a_counterexample ... ok
test model_check::tests::tlc_exit_zero_without_a_known_verdict_fails_closed ... ok
test model_check::tests::tlc_nonzero_exit_without_completion_is_a_labeled_error ... ok
test model_check::tests::tlc_wall_clock_budget_exhaustion_kills_the_run_as_timed_out ... ok
test lint::tests::ref_kind::uses_profile_file_passes ... ok
test lint::tests::unobserved_effect_warns_but_never_fails ... ok
test lint::tests::unrationalized_waiver_rejected ... ok
test orchestrate::tests::parse_error_holds_lint ... ok
test migrate::tests::scaffold_lints_clean_through_the_real_linter ... ok
test orchestrate::tests::upstream_failure_skips_dependents ... ok
test orchestrate::tests::coverage_failure_holds_compile ... ok
test parse::tests::not_found_names_the_path_and_hint ... ok
test rename::tests::checklist_without_matching_ids_is_not_written ... ok
test rename::tests::crlf_files_keep_their_terminators ... ok
test rename::tests::identity_rename_writes_nothing ... ok
test rename::tests::invalid_new_ids_are_rejected_before_any_lookup ... ok
test rename::tests::mapped_ids_follow_intent_rename ... ok
test rename::tests::rename_gate_rejects_dangling_mapped_id ... ok
test orchestrate::tests::independent_branches_report_regardless ... ok
test update_notice::tests::no_notice_when_current ... ok
test rename::tests::rename_reaches_into_checklist_mapped_ids ... ok
test update_notice::tests::stale_cache_triggers_backoff_write_not_error ... ok
test update_notice::tests::update_notice_rides_warnings_when_newer_version_exists ... ok
test update_notice::tests::transport_failure_is_silent_never_an_error ... ok
test orchestrate::tests::dependency_skip_stays_within_its_branch ... ok
test verify::tests::both_gates_clean_passes ... ok
test verify::tests::cached_result_rejected_by_staleness ... ok
test verify::tests::every_verdict_message_carries_a_remediation_hint ... ok
test verify::tests::blocks_from_source_reads_compile_emission ... ok
test verify::tests::law_cases_all_run_partial_pass_is_failure ... ok
test verify::tests::missing_artifacts_fail_closed ... ok
test verify::tests::blocks_from_source_reads_law_case_blocks ... ok
test verify::tests::model_gate_missing_report ... ok
test verify::tests::model_gate_clean_and_not_clean_outcomes ... ok
test verify::tests::metadata_comparison_ignores_translated_bodies ... ok
test verify::tests::model_gate_stale_when_artifact_changed ... ok
test verify::tests::model_gate_stale_when_module_gone ... ok
test verify::tests::properties_blocker_precedes_model_blocker ... ok
test verify::tests::model_gate_unreadable_report_fails_closed ... ok
test lint::tests::unresolved_derives_from_target_stays_total_refs_beat ... ok
test verify::tests::properties_gate_no_rows_passes_without_runner ... ok
test verify::tests::properties_gate_missing_artifact ... ok
test verify::tests::properties_gate_failed_block_is_not_a_skip ... ok
test verify::tests::prune_defers_target_reset_while_model_check_builds_are_in_flight ... ok
test verify::tests::prune_defers_target_reset_while_peer_builds_are_in_flight ... ok
test verify::tests::prune_ignores_entries_that_are_not_scratch_crate_dirs ... ok
test verify::tests::properties_gate_runs_translated_artifact ... ok
test verify::tests::properties_gate_stale_when_artifact_predates_spec_edit ... ok
test verify::tests::prune_keeps_fresh_crate_dirs_target_and_lock ... ok
test verify::tests::prune_keeps_target_under_size_cap ... ok
test verify::tests::prune_removes_stale_orphan_crate_dirs ... ok
test verify::tests::report_claim_fields_derives_records_and_sets ... ok
test verify::tests::rerun_matches_prior_outcome ... ok
test verify::tests::prune_resets_target_when_over_size_cap ... ok
test verify::tests::properties_gate_runner_errors_fail_labeled ... ok
test lint::tests::unused_state_fails_every_state_used ... ok
test orchestrate::tests::every_stage_reported_and_native_model_check_is_never_clean ... ok
test verify::tests::run_bounded_completes_within_budget ... ok
test verify::tests::scratch_base_default_is_temp_specodelic_verify ... ok
test orchestrate::tests::external_completeness_never_gates ... ok
test verify::tests::scratch_base_env_override_wins_over_default ... ok
test verify::tests::single_gate_insufficient_model_only ... ok
test verify::tests::single_gate_insufficient_properties_only ... ok
test verify::tests::timed_out_verdict_is_distinct_from_properties_failed ... ok
test verify::tests::timeout_maps_to_labeled_gate_state_not_test_failure ... ok
test lint::tests::well_formed_file_passes_graph_and_model_shape ... ok
test verify::tests::run_bounded_unbounded_when_no_budget ... ok
test lint::tests::content_advisories::tautology_forms_are_narrow ... ok
test orchestrate::tests::note_prose_never_flips_the_parse_gate ... ok
test verify::tests::run_bounded_reaps_the_killed_child ... ok
test orchestrate::tests::failure_shape_checker_rides_the_lint_stage ... ok
test verify::tests::run_bounded_kills_hanging_process_and_labels_timeout ... ok
test orchestrate::tests::rerun_is_byte_identical ... ok
test verify::tests::run_bounded_kill_takes_down_the_runner_tree_not_just_cargo ... ok
test orchestrate::tests::verify_stage_passes_only_on_the_conjunction ... ok
test lint::tests::content_advisories::placeholder_words_warn_table_driven ... ok
test verify::tests::run_bounded_drains_output_beyond_the_pipe_buffer ... ok
test lint::tests::catalog_covers_every_rule_the_linter_can_emit ... ok
test lint::tests::every_finding_carries_rule_id_and_semantics ... ok

test result: ok. 460 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.06s


running 3 tests
test tests::broken_pipe_is_not_a_report_write_failure ... ok
test tests::other_write_errors_fail_the_run ... ok
test tests::corpus_walk_terminates_on_a_symlink_cycle ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test tests::broken_pipe_is_not_a_report_write_failure ... ok
test tests::other_write_errors_fail_the_run ... ok
test tests::corpus_walk_terminates_on_a_symlink_cycle ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 18 tests
test member_path_link_projects_canonical_edge ... ok
test dangling_member_path_stays_dangling_on_labeled_path ... ok
test dangling_shapes_match_oracle ... ok
test dangling_fixture_covers_named_shapes ... ok
test small_clean_corpus_is_clean ... ok
test zero_file_and_single_intent_corpora_are_clean ... ok
test parity_property_edges_from_specs_matches_graph_build ... ok
test typing_violations_match_oracle ... ok
test typing_violations_fixture_covers_named_shapes ... ok
test typing_allowed_fixture_covers_named_edges ... ok
test typing_allowed_edges_match_oracle ... ok
test adapter_graph_equivalent_over_fixture_corpora ... ok
test schema_missing_one_row_of_the_typing_table ... ok
test real_doc_typing_table_matches_canonical_schema ... ok
test adapter_graph_equivalent_under_shuffle ... ok
test adapter_graph_equivalent_over_the_real_corpus ... ok
test lint_without_the_format_doc_is_a_no_op ... ok
test lint_reports_drift_when_the_doc_loses_a_row ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s


running 9 tests
test unknown_seed_rejected ... ok
test fan_in_matches_preimage ... ok
test dangling_not_followed_in_traversal ... ok
test closure_on_cycle_terminates ... ok
test supersedes_scope_exact ... ok
test results_sorted_by_id ... ok
test closure_laws ... ok
test blast_radius_parity ... ok
test walk_parity_fan_and_cycles ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.27s


running 15 tests
test apply_failure_label_asserted_on_unspanned_bullet ... ok
test apply_failure_label_asserted_on_unspanned_intent_id ... ok
test edit_missing_from_file_is_a_no_op ... ok
test identity_edit_is_a_no_op ... ok
test intent_rename_moves_file ... ok
test padding_not_realigned ... ok
test row_rename_rewrites_cells_and_links_through_spans ... ok
test prose_survives_edit ... ok
test bare_derives_from_cell_follows_local_rename ... ok
test writer_does_no_io ... ok
test crlf_survives_edit ... ok
test edits_compose ... ok
test roundtrip_failure_label_asserted ... ok
test text_realizes_instance ... ok
test roundtrip_faithful ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 6 tests
test span_failure_label_asserted_on_disagreement ... ok
test span_failure_label_asserted_on_out_of_bounds ... ok
test identity_emission_exact_under_crlf ... ok
test identity_emission_over_fixture_directories ... ok
test identity_emission_exact ... ok
test identity_emission_over_the_real_corpus ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


running 2 tests
test baseline_crlf_row_family ... ok
test baseline_write_set_snapshots ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test every_id_cell_and_link_and_bullet_carries_a_span ... ok
test spans_survive_crlf ... ok
test spans_are_not_serialized ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
test recipe_wires_the_normalizer ... ok
test id_prefix_form_is_normalized ... ok
test bare_form_is_preserved ... ok
test empty_id_fails_with_remediation_hint ... ok
test recipe_fails_on_empty_id_with_hint ... ok
test recipe_forwards_bare_id_to_companion ... ok
test recipe_accepts_bare_positional_form ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 2 tests
test before_sustained_contradiction_exits_1_and_stays_contradicted ... ok
test before_unseen_contradiction_exits_1_and_stays_contradicted_not_new ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s


running 1 test
test binds_nothing_flags_on_the_warnings_channel_exit_zero ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s


running 14 tests
test exit_code_one_iff_contradicted_or_stale_under_declared_expectation ... ok
test audit_classifies_records_current_unchecked_contradicted_carrying_the_conform_verdict ... ok
test before_record_absent_from_prior_classifies_new ... ok
test before_digest_mismatch_fails_labeled ... ok
test before_parse_refuses_duplicate_prior_rows ... ok
test before_schema_version_mismatch_refuses ... ok
test before_worsened_class_classifies_stale_and_same_class_stays_current ... ok
test run_refuses_zero_record_corpus_labeled ... ok
test audit_report_carries_evidence_scope_verbatim_and_corpus_scope_digest ... ok
test audit_cli_envelope_and_exit_1_on_contradicted ... ok
test audit_cli_exit_2_on_zero_records_and_gate_refusal ... ok
test audit_cli_digest_mismatch_refuses_labeled ... ok
test run_refuses_gate_failure_labeled ... ok
test audit_cli_stale_fails_only_when_prior_declared_expectation ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 9 tests
test no_evidence_reports_rung_1_stated_as_such ... ok
test rung_is_the_highest_evidence_backed_rung_across_mixes ... ok
test human_view_surfaces_the_rung ... ok
test model_checked_only_reports_rung_3_never_4_or_5 ... ok
test prior_rung_never_drives_the_current_rung ... ok
test stale_check_report_does_not_lift_current_one_does ... ok
test vacuous_rows_never_lift_the_rung ... ok
test audit_cli_without_check_evidence_reports_rung_2_never_higher ... ok
test audit_cli_model_checked_only_reports_rung_3_in_both_views ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 12 tests
test a_claim_carries_the_strongest_class_among_its_rows ... ok
test a_near_miss_name_places_no_claim ... ok
test a_property_row_places_the_claim_it_derives_from ... ok
test the_human_view_carries_the_coverage_counts_on_the_verdict_line ... ok
test the_section_carries_a_machine_readable_scope ... ok
test a_model_row_places_the_claim_it_names ... ok
test an_unchecked_claim_does_not_fail_the_run ... ok
test the_section_uses_only_audits_closed_class_set ... ok
test claims_are_named_by_their_file_qualified_id ... ok
test a_row_that_places_no_claim_lands_in_the_residual_bucket ... ok
test the_section_is_byte_identical_across_runs ... ok
test the_section_rides_the_check_envelope_beside_an_unchanged_report ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
test contract_documented_claim_of_record_is_drift_visible_and_graded ... ok
test contract_unchecked_run_exits_zero_with_counts_in_both_views ... ok
test contract_gate_refusal_names_the_gate_and_the_remedy ... ok
test contract_could_not_judge_is_exit_two_in_both_views ... FAILED
test contract_contradiction_fails_the_run_in_both_views ... ok
test contract_stale_or_unreadable_model_report_is_refused_never_rewritten ... ok
test contract_declared_expectation_staleness_gates ... ok

failures:

---- contract_could_not_judge_is_exit_two_in_both_views stdout ----

thread 'contract_could_not_judge_is_exit_two_in_both_views' (868925) panicked at tests/check_contract_exit.rs:374:9:
it never reads as a pass: {"ok":false,"envelope_version":"0.1","cli_version":"0.14.0","envelope_kind":"error","data":null,"warnings":[{"rule_name":"","message":"verify_model: no readable model-check report (no model-check run report at /tmp/.tmpwWimKf/out/demo.refuse.check.json: No such file or directory (os error 2)) — run: spk model-check <file>"},{"rule_name":"","message":"no corpus root discovered — the gate linted the file in ISOLATION; cross-file references may dangle (pass the file inside its openspec tree, or run: spk lint <corpus-root>)"}],"hints":[{"command":"spk model-check <file> (check never compiles or rewrites the report)","description":"run: spk model-check <file> (check never compiles or rewrites the report)"}],"meta":{"duration_ms":0,"tx":null,"request_id":null}}

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    contract_could_not_judge_is_exit_two_in_both_views

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.98s lint recipe, which does not cover test targets) surfaced two leftovers that are now fixed: an unused binding in tests/archive_change_recipe.rs (the header lookup, rewritten as a scan with an explicit emptiness assertion) and two  calls with no placeholders in tests/cli/check.rs. Post-refactor evidence: cargo clippy --all-targets -- -D warnings -> clean; cargo test --test cli -> 354 passed / 0 failed / 1 ignored; cargo test --test archive_change_recipe -> 7 passed / 0 failed; cargo fmt applied.

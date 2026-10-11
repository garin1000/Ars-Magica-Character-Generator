//! The root of this crate's single integration-test binary.
//!
//! Every `tests/*.rs` file is a module of this one crate rather than a binary
//! of its own, so an engine edit relinks one test binary instead of one per
//! file, and the tests of all files run in parallel. `Cargo.toml` sets
//! `autotests = false` and names this file as the `all` target, so a new test
//! file must be declared here; `test_targets.rs` fails until it is.
//!
//! Run one file's tests with its module path as the filter:
//! `cargo test --workspace --test all de_vf_names::`.

// Shared helpers, used through `crate::`.
mod citation_support;

mod a1_max_age;
mod ability_category_line;
mod ability_grant_summaries;
mod ability_text_facts;
mod aging_table_row;
mod b1_category_and_ability_prohibitions;
mod b2_social_status_floor;
mod b3_predicate_exclusions;
mod b4_parameter_gated_effects;
mod b5_ability_roll_mod;
mod book_templates;
mod bve_s1_uninspirational_effective;
mod bve_s4_granted_ability_bonus_target;
mod c5a_multi_ref_parameter;
mod c5c_corrupted_and_enchanting;
mod citation_relocation;
mod composite_spell_five_dimensions;
mod core_type_conformance;
mod coverage_export_sheet;
mod coverage_ruleset_error;
mod coverage_spell_caps;
mod cv4_ability_parameter_migration;
mod cv5_bound_link_matching;
mod cv5_link_fold_and_unlink;
mod cv6_ability_parameter_options;
mod cv7_link_target_resolved;
mod d42_realms;
mod d48_instance_pools;
mod d79_potent_magic;
mod d80_optional_parameter;
mod d81_devil_child;
mod d81_exclusions;
mod d81_export_incompatible_arts;
mod d81_incompatible_arts;
mod d81_merinita_warping;
mod d87_tread_the_earth_rename;
mod data_integrity;
mod de_quote_pairs;
mod de_summary_terms;
mod de_vf_names;
mod enchanting_ability_label;
mod equipment_table_row;
mod export_golden;
mod f2_conditional_modifiers;
mod focus_marker_needs_focus;
mod grog_story_flaw_is_a_warning;
mod i2_banked_xp_names;
mod i3_bonus_sources;
mod i4_spell_code;
mod i4_spell_code_export;
mod i5b_fixed_grant_text;
mod l1a_language_catalogues;
mod l1b_language_move_notice;
mod l1b_language_save_migration;
mod l2_language_requirements;
mod l2_typed_language_in_session;
mod l3_catalogue_notice_gating;
mod life_sections_fold;
mod life_timeline;
mod n1_spell_level_caps_marked_figures;
mod n3b_saga_year_validation;
mod n4a_native_language_catalogue;
mod n4b_native_language_save;
mod n4b_native_language_value;
mod overflow_checks_profile;
mod parameter_catalogues;
mod parameter_catalogues_literals;
mod parameter_catalogues_ruleset;
mod r2_text_param_compare;
mod r3_potent_magic_spell_cap;
mod r3_potent_magic_spell_cap_fields;
mod r4_prereq_score_basis;
mod r6_mythic_status_virtue;
mod rc_review_c_fixes;
mod requisite_casting_total;
mod requisite_level_cap;
mod robust_overflow;
mod roundtrip_proptest;
mod row55_grant_realm;
mod row56_lab_years_migration;
mod rulebook_citations;
mod rules_i18n_ascii_hyphen;
mod rules_md_citations;
mod rules_source_provenance;
mod saga_year;
mod save_invariant;
mod selection_param_value;
mod source_citations;
mod source_relocation;
mod spell_cap_effective_intelligence;
mod spell_description_verbatim;
mod spell_mastery_limits;
mod spell_stat_line;
mod t2_art_parameterized_names;
mod test_targets;
mod uncomputed_clauses;
mod vf_descriptor_line;
mod vf_magnitude_brackets;
mod vf_summary_facts;
mod w3_xp_total_supply;
mod x10bc_banked_xp_and_within_focus;
mod x1_authorization_family;
mod x2_reclassification;
mod x2t_true_friend;
mod x3_trained_gate;
mod x4_incompatibilities;
mod x5_prerequisites;
mod x5b_ability_minimums;
mod x6a_parameter_engine;
mod x6b_parameter_data;
mod x6c_academic_concentration_domain;
mod x6c_label_parameters;
mod x7a_lab_rows;
mod x7a_no_hardcoded_vf_ids;
mod x7bd_wrong_numbers;
mod x7be_row42;
mod x7c_computed_entries;
mod x8a_german_text;
mod x8c_export_vf_text;
mod x8d_german_names;
mod x9b_bard_rename;
mod x9c_descriptor_sweep;

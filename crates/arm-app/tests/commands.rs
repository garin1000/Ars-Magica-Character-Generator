//! Integration tests for the webview-free command logic. These exercise the
//! real loader, validator, and save/load against the repository's shipped rules
//! data — no Tauri runtime or webview required.

use std::fs;
use std::path::PathBuf;

use arm_app::error::AppError;
use arm_app::ruleset_io::{
    AgingApplication, AgingProjection, AgingReversion, ChildhoodApplication, RULESET_ID,
    RULESET_VERSION, apply_childhood_package_loaded, effective_scores_loaded, ensure_extension,
    export_markdown_to_path, load_entity_from_path, load_ruleset_from_dir, missing_core_files,
    pick_rules_dir, save_entity_to_path, validate_loaded,
};
use arm_rules::{
    ArtScore, CreationPhase, Entity, Id, Ruleset, RulesetSources, Selection, ValidationMode,
};
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rules_dir() -> PathBuf {
    repo_root().join("rules")
}

fn sample_entity() -> Entity {
    let json = fs::read_to_string(repo_root().join("examples/companion_sample.json")).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn load_ruleset_yields_companion_profile_and_all_items() {
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    // `Ruleset.id` is now an `Id` newtype; compare its string form.
    assert_eq!(localized.ruleset.id.as_str(), RULESET_ID);
    assert_eq!(localized.ruleset.version, RULESET_VERSION);
    // Catalogue size is data, not code: prove a known item loaded, never an exact
    // V/F total (which would break when any item is added to the JSON).
    assert!(
        localized
            .ruleset
            .item(&Id::new("virtue.the_gift"))
            .is_some()
    );
    assert!(localized.ruleset.profile(&Id::new("companion")).is_some());
}

#[test]
fn load_ruleset_yields_houses_with_localized_names() {
    // The shipped houses.json + its i18n must load through the real production
    // path (from_core_json in the engine tests does not exercise houses). Prove a
    // known House loaded and its display name is merged, never an exact count.
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    assert!(
        localized
            .ruleset
            .house(&Id::new("house.bjornaer"))
            .is_some()
    );
    assert_eq!(
        localized.display_name(&Id::new("house.bjornaer")),
        Some("Bjornaer")
    );
    // The House-granted Virtues and the Mystery abilities they seed also loaded.
    assert!(
        localized
            .ruleset
            .item(&Id::new("virtue.heartbeast"))
            .is_some()
    );
    assert!(
        localized
            .ruleset
            .ability(&Id::new("ability.heartbeast"))
            .is_some()
    );
}

/// The shipped `core/childhoods.json` + its i18n must reach the app through the
/// real production loader: a package the engine can price is useless if the
/// binary never reads the file. Proves a known package loaded with the slots a UI
/// has to ask for, and that its name is localized in both languages — never a
/// package count, which is data.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2388 (Traveling
/// Childhood), and its German name at
/// `Ars Magica Definitive Edition Basisregeln.md:2388`.
#[test]
fn load_ruleset_yields_childhood_packages_with_localized_names() {
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let traveling = localized
        .ruleset
        .childhood(&Id::new("childhood.traveling"))
        .expect("Traveling Childhood loaded");

    // "Area A Lore 1, Area B Lore 1 … Living Language 1": two instances of one
    // parameterized Ability plus the spread's second language, each a slot the
    // player fills in.
    let slots: Vec<(&str, &str)> = traveling
        .slots()
        .map(|(slot, ability)| (slot, ability.as_str()))
        .collect();
    assert_eq!(
        slots,
        vec![
            ("area_a", "ability.area_lore"),
            ("area_b", "ability.area_lore"),
            ("language", "ability.living_language"),
        ]
    );
    assert_eq!(
        localized.display_name(&Id::new("childhood.traveling")),
        Some("Traveling Childhood")
    );

    let german = load_ruleset_from_dir(&rules_dir(), "de").unwrap();
    assert_eq!(
        german.display_name(&Id::new("childhood.traveling")),
        Some("Reisende Kindheit")
    );
}

#[test]
fn load_ruleset_yields_spells_with_localized_names() {
    // The shipped spells.json + its i18n must load through the real production
    // path. Prove a known spell loaded, its Technique/Form resolve, and its
    // display name is merged (localized) — never an exact count.
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let spell = localized
        .ruleset
        .spell(&Id::new("spell.pilum_of_fire"))
        .expect("Pilum of Fire loaded");
    assert_eq!(spell.technique, Id::new("art.creo"));
    assert_eq!(spell.form, Id::new("art.ignem"));
    assert_eq!(spell.level, Some(20));
    // A known non-ritual reads ritual = false (Pilum of Fire is Formulaic).
    assert!(!spell.ritual, "Pilum of Fire is not a ritual");
    assert_eq!(
        localized.display_name(&Id::new("spell.pilum_of_fire")),
        Some("Pilum of Fire")
    );
    // A General spell loads with no fixed level, and a known ritual reads
    // ritual = true (Aegis of the Hearth is a Year/Boundary ritual).
    let aegis = localized
        .ruleset
        .spell(&Id::new("spell.aegis_of_the_hearth"))
        .expect("Aegis of the Hearth loaded");
    assert_eq!(aegis.level, None);
    assert!(aegis.ritual, "Aegis of the Hearth is a ritual");
    // The magus profile carries the 120-level spell budget.
    assert_eq!(
        localized
            .ruleset
            .profile(&Id::new("magus"))
            .map(|p| p.spell_levels),
        Some(120)
    );
}

#[test]
fn load_ruleset_localizes_spell_names_in_german() {
    let localized = load_ruleset_from_dir(&rules_dir(), "de").unwrap();
    assert_eq!(
        localized.display_name(&Id::new("spell.pilum_of_fire")),
        Some("Pilum aus Feuer")
    );
}

#[test]
fn german_spell_descriptions_are_never_empty_via_english_fallback() {
    // German spell tooltips render the description; where a German description is
    // not yet translated, the loader fills it from English so the tooltip is never
    // empty. The name stays German (a present field is never overwritten by the
    // fallback). This exercises the loader's non-English fallback path end to end.
    let de = load_ruleset_from_dir(&rules_dir(), "de").unwrap();
    let pilum = Id::new("spell.pilum_of_fire");

    let de_desc = de
        .description(&pilum)
        .expect("German spell description must be present (via fallback if untranslated)");
    assert!(!de_desc.is_empty());
    assert_eq!(
        de.display_name(&pilum),
        Some("Pilum aus Feuer"),
        "the German name is not overwritten by the English fallback"
    );
}

#[test]
fn load_ruleset_yields_abilities_and_characteristics() {
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    // Catalogue size is data, not code: prove the catalogue loaded via a known
    // ability, never an exact count (the full catalogue is a data-only add).
    assert!(
        localized
            .ruleset
            .ability(&Id::new("ability.awareness"))
            .is_some()
    );
    assert!(localized.ruleset.characteristic_rules().is_some());
    // Ability display names are merged into the localized text alongside V/F.
    assert_eq!(
        localized.display_name(&Id::new("ability.awareness")),
        Some("Awareness")
    );
}

/// The shipped `core/aging.json` must reach the engine the way the app loads it —
/// through `load_ruleset_from_dir`, not through a test that reads the file bytes
/// itself. Structural assertions only: the two tables are catalogue *data*, so
/// their row counts belong in `data_integrity.rs`, never here.
#[test]
fn load_ruleset_yields_the_shipped_aging_tables() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let aging = ruleset
        .aging()
        .expect("the shipped ruleset ships aging rules");
    // "Characters begin aging in the Winter after they turn 35"
    // (Ars Magica - Definitive Edition (Core Rules).md:16565).
    assert_eq!(aging.start_age, 35);
    assert!(
        !aging.living_conditions.is_empty(),
        "the Living Conditions table reached the engine"
    );
    assert!(
        !aging.outcomes.is_empty(),
        "the Aging Roll table reached the engine"
    );
}

/// Every shipped character type ends its guided rail with the Aging phase, and
/// the position is the point of the test: it guards against a later reorder that
/// would look harmless and quietly compute the aging total from an unfinished
/// character.
///
/// Aging is last because the rulebook puts it there. The creation summary
/// (Ars Magica - Definitive Edition (Core Rules).md:2205-2222) runs steps 1..11
/// and never mentions aging at all; aging enters only in the next section,
/// "Starting Character Age", whose `:2232` places the rolls "before the game
/// begins" — the last thing done to a built character, not one of the steps that
/// build it. Mechanically the total needs the finished character too: it reads
/// the final age, the final Characteristics (aging points reduce them, `:16579`)
/// and the Longevity Ritual bonus, so any earlier slot would total up a
/// half-built character. The wizard appends its implicit `review` step after the
/// declared list, so declaring `aging` last makes the rail end
/// `… -> aging -> review`.
#[test]
fn every_shipped_profile_declares_the_aging_phase_last() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    assert!(ruleset.profile_count() > 0, "the ruleset ships profiles");
    for profile in ruleset.profiles() {
        assert_eq!(
            profile.creation_phases.last(),
            Some(&CreationPhase::Aging),
            "profile '{}' must declare the aging phase last",
            profile.id
        );
    }
}

#[test]
fn load_ruleset_localized_names_differ_between_languages() {
    let en = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let de = load_ruleset_from_dir(&rules_dir(), "de").unwrap();

    let id = Id::new("flaw.poor_student");
    let en_name = en.display_name(&id).expect("en name present");
    let de_name = de.display_name(&id).expect("de name present");
    assert_ne!(en_name, de_name, "translations should differ");

    // Ability names are localized too (Awareness -> Aufmerksamkeit).
    let aware = Id::new("ability.awareness");
    assert_ne!(
        en.display_name(&aware).unwrap(),
        de.display_name(&aware).unwrap()
    );
}

#[test]
fn sample_entity_with_characteristics_and_abilities_validates() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let entity = sample_entity();
    // The shipped sample now carries characteristics, ability scores, and a bank,
    // and is kept at the current schema version so a save/load round trip on it is
    // an identity (see `save_then_load_round_trips_with_byte_stable_canonical_json`).
    assert_eq!(entity.schema_version, 16);
    assert!(!entity.characteristics.is_empty());
    assert!(!entity.ability_scores.is_empty());
    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(result.is_valid(), "unexpected issues: {:?}", result.issues);
}

/// The payload the wizard reads is the one this command returns, and it must carry
/// the completeness report against the **shipped** profiles — the phase lists no
/// test fixture can stand in for. A brand-new magus has touched nothing, so every
/// declared step is outstanding.
#[test]
fn validating_a_fresh_character_reports_its_untouched_phases() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let entity = arm_rules::Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(ruleset.id.clone(), ruleset.version.clone()),
    );

    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    let profile = ruleset.profile(&Id::new("magus")).unwrap();
    // Every declared step of a brand-new character is untouched: the read-only
    // `type` step that used to be the one exemption is gone (review #1).
    let expected: Vec<arm_rules::CreationPhase> = profile.creation_phases.clone();
    assert_eq!(result.completeness.incomplete_phases, expected);
}

/// The completeness report crosses the Tauri boundary on the validation payload and
/// `ui/src/lib/types.ts` mirrors it **by hand** — a sibling of
/// [`every_aging_field_is_mirrored_in_the_frontend_types`], for the same reason: a
/// renamed Rust field would leave the mirror compiling and the rail silently
/// reading `undefined`, i.e. nothing ever marked incomplete.
#[test]
fn the_completeness_report_is_mirrored_in_the_frontend_types() {
    let result = arm_rules::ValidationResult {
        issues: vec![],
        completeness: arm_rules::CompletenessReport {
            incomplete_phases: vec![arm_rules::CreationPhase::Abilities],
        },
    };

    let mut keys = std::collections::BTreeSet::new();
    mirrored_keys(&serde_json::to_value(&result).unwrap(), &mut keys);
    assert!(
        keys.contains("completeness") && keys.contains("incomplete_phases"),
        "expected the validation payload to carry the completeness field names, got {keys:?}"
    );

    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    for key in keys.iter().map(String::as_str) {
        assert!(
            types.contains(&format!("{key}:")) || types.contains(&format!("{key}?:")),
            "ui/src/lib/types.ts declares no '{key}' property"
        );
    }
}

#[test]
fn load_ruleset_missing_language_is_io_error() {
    let err = load_ruleset_from_dir(&rules_dir(), "xx").unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

#[test]
fn pick_rules_dir_returns_first_candidate_that_holds_rules() {
    // A portable Linux build's `BaseDirectory::Resource` points at a system path
    // that does not exist; the picker must skip it and fall back to the real
    // directory next to the executable.
    let empty = tempfile::tempdir().unwrap();
    let picked = pick_rules_dir(&[empty.path().to_path_buf(), rules_dir()]);
    assert_eq!(picked, Some(rules_dir()));
}

#[test]
fn pick_rules_dir_is_none_when_no_candidate_holds_rules() {
    let empty = tempfile::tempdir().unwrap();
    assert_eq!(pick_rules_dir(&[empty.path().to_path_buf()]), None);
}

/// V9: a directory carrying only `core/character_types.json` used to pass
/// `pick_rules_dir`'s check (a single-file presence test), get accepted as
/// "the" rules directory, and only then fail deep inside
/// `load_ruleset_from_dir` with a raw "file not found" for whichever of the
/// other dozen files was missing — a confusing error that named neither the
/// directory nor what was actually wrong with it. The picker must now reject
/// a stale/partial directory outright and skip to the next candidate.
#[test]
fn pick_rules_dir_rejects_a_directory_carrying_only_one_required_file() {
    let stale = tempfile::tempdir().unwrap();
    fs::create_dir_all(stale.path().join("core")).unwrap();
    fs::write(stale.path().join("core/character_types.json"), "[]").unwrap();

    let picked = pick_rules_dir(&[stale.path().to_path_buf(), rules_dir()]);
    assert_eq!(
        picked,
        Some(rules_dir()),
        "a partial directory must be skipped in favor of a complete one"
    );
    assert_eq!(
        pick_rules_dir(&[stale.path().to_path_buf()]),
        None,
        "a partial directory alone must not be picked"
    );
}

/// [`missing_core_files`] is what lets a caller build a message naming
/// exactly what is missing, rather than a bare "not found" (V9).
#[test]
fn missing_core_files_names_every_absent_file() {
    let stale = tempfile::tempdir().unwrap();
    fs::create_dir_all(stale.path().join("core")).unwrap();
    fs::write(stale.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(stale.path().join("core/virtues_flaws.json"), "[]").unwrap();

    let missing = missing_core_files(stale.path());
    assert!(!missing.contains(&"core/character_types.json"));
    assert!(!missing.contains(&"core/virtues_flaws.json"));
    // Every other required core file is genuinely absent from this fixture.
    assert!(missing.contains(&"core/abilities.json"));
    assert!(missing.contains(&"core/arts.json"));
    assert!(missing.contains(&"core/houses.json"));
    assert!(missing.contains(&"core/mythic_companion_types.json"));
    assert!(missing.contains(&"core/spells.json"));
    assert!(missing.contains(&"core/spell_mastery_abilities.json"));
    assert!(missing.contains(&"core/equipment.json"));
    assert!(missing.contains(&"core/characteristics.json"));
    assert!(missing.contains(&"core/life_stages.json"));
    assert!(missing.contains(&"core/childhoods.json"));
    assert!(missing.contains(&"core/aging.json"));
    assert_eq!(missing.len(), 11);
}

#[test]
fn missing_core_files_is_empty_for_the_real_shipped_rules_directory() {
    assert_eq!(missing_core_files(&rules_dir()), Vec::<&str>::new());
}

#[test]
fn missing_core_files_is_the_full_list_for_a_directory_that_does_not_exist() {
    let missing = missing_core_files(&PathBuf::from("/does/not/exist/at/all"));
    assert_eq!(missing.len(), 13);
}

#[test]
fn load_ruleset_malformed_rules_is_ruleset_error() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("core")).unwrap();
    fs::create_dir_all(tmp.path().join("i18n/en")).unwrap();
    fs::write(tmp.path().join("core/virtues_flaws.json"), "not valid json").unwrap();
    fs::write(tmp.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(tmp.path().join("core/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spells.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spell_mastery_abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/characteristics.json"), "").unwrap();
    fs::write(tmp.path().join("core/life_stages.json"), "").unwrap();
    fs::write(tmp.path().join("core/childhoods.json"), "").unwrap();
    fs::write(tmp.path().join("core/aging.json"), "").unwrap();
    fs::write(tmp.path().join("i18n/en/virtues_flaws.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/spells.json"), "{}").unwrap();
    fs::write(
        tmp.path().join("i18n/en/spell_mastery_abilities.json"),
        "{}",
    )
    .unwrap();
    fs::write(tmp.path().join("i18n/en/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/childhoods.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/aging.json"), "{}").unwrap();

    let err = load_ruleset_from_dir(tmp.path(), "en").unwrap_err();
    let AppError::Ruleset {
        ruleset_kind,
        errors,
    } = &err
    else {
        panic!("expected ruleset error, got {err:?}");
    };
    assert_eq!(ruleset_kind, "parse", "malformed JSON is a parse failure");
    assert_eq!(errors.len(), 1, "parse failure carries one message");

    // The serialized payload the frontend receives must carry the engine kind
    // and the per-violation list, not a newline-joined English blob.
    let json: serde_json::Value = serde_json::to_value(&err).unwrap();
    assert_eq!(json["kind"], "ruleset");
    assert_eq!(json["ruleset_kind"], "parse");
    assert!(json["errors"].is_array(), "errors serialized as a list");
}

#[test]
fn integrity_failure_preserves_individual_messages() {
    // Two distinct unknown-prereq references must survive as two list entries,
    // with the engine's "integrity" kind preserved (not flattened to English).
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("core")).unwrap();
    fs::create_dir_all(tmp.path().join("i18n/en")).unwrap();
    fs::write(
        tmp.path().join("core/virtues_flaws.json"),
        // Carries a personality-category item so the only integrity failures are
        // the two unresolved prerequisites (not the engine-required-category check).
        r#"[
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.x"}},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.y"}},
          {"id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"]}
        ]"#,
    )
    .unwrap();
    fs::write(tmp.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(tmp.path().join("core/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spells.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spell_mastery_abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/characteristics.json"), "").unwrap();
    fs::write(tmp.path().join("core/life_stages.json"), "").unwrap();
    fs::write(tmp.path().join("core/childhoods.json"), "").unwrap();
    fs::write(tmp.path().join("core/aging.json"), "").unwrap();
    fs::write(tmp.path().join("i18n/en/virtues_flaws.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/spells.json"), "{}").unwrap();
    fs::write(
        tmp.path().join("i18n/en/spell_mastery_abilities.json"),
        "{}",
    )
    .unwrap();
    fs::write(tmp.path().join("i18n/en/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/childhoods.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/aging.json"), "{}").unwrap();

    let err = load_ruleset_from_dir(tmp.path(), "en").unwrap_err();
    let AppError::Ruleset {
        ruleset_kind,
        errors,
    } = &err
    else {
        panic!("expected ruleset error, got {err:?}");
    };
    assert_eq!(ruleset_kind, "integrity");
    assert_eq!(
        errors.len(),
        2,
        "two distinct integrity violations preserved"
    );
}

#[test]
fn sample_companion_is_valid_in_enforced_mode() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let result = validate_loaded(&sample_entity(), &ruleset, ValidationMode::Enforced);
    assert!(result.is_valid(), "unexpected issues: {:?}", result.issues);
}

/// A companion picking a forbidden hermetic virtue produces errors. We reuse
/// this entity to prove the three validation modes behave differently.
fn companion_with_forbidden_item() -> Entity {
    let mut entity = sample_entity();
    entity
        .selections
        .push(arm_rules::Selection::new(Id::new("virtue.gentle_gift")));
    entity
}

#[test]
fn forbidden_item_errors_in_enforced_but_downgrades_in_advisory_and_clears_in_silent() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let entity = companion_with_forbidden_item();

    let enforced = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(!enforced.is_valid(), "should have errors in enforced mode");

    let advisory = validate_loaded(&entity, &ruleset, ValidationMode::Advisory);
    assert!(advisory.is_valid(), "advisory has no errors");
    assert!(
        advisory.warnings().next().is_some(),
        "advisory keeps warnings"
    );

    let silent = validate_loaded(&entity, &ruleset, ValidationMode::Silent);
    assert!(silent.issues.is_empty(), "silent clears all issues");
}

#[test]
fn over_budget_virtues_is_reported() {
    // A deliberately tiny type profile (1 virtue point) forces an overrun the
    // shipped companion profile can't express with the current catalogue.
    let core = rules_dir().join("core");
    let items = fs::read_to_string(core.join("virtues_flaws.json")).unwrap();
    let abilities = fs::read_to_string(core.join("abilities.json")).unwrap();
    let arts = fs::read_to_string(core.join("arts.json")).unwrap();
    let characteristics = fs::read_to_string(core.join("characteristics.json")).unwrap();
    let houses = fs::read_to_string(core.join("houses.json")).unwrap();
    let tiny_type = r#"[{
        "id": "tiny",
        "budget": { "virtue_points": 1, "flaw_points": 10 },
        "permitted_categories": ["general"],
        "creation_phases": ["virtues_flaws"]
    }]"#;
    // The catalogue's ability_score_grant effects reference abilities, and the
    // four Outer-Mystery Virtues carry a `House` prerequisite, so the ruleset
    // must carry both registries for referential integrity to pass.
    let ruleset = Ruleset::from_sources(RulesetSources {
        id: RULESET_ID,
        version: RULESET_VERSION,
        point_items: &items,
        type_profiles: tiny_type,
        abilities: Some(&abilities),
        arts: Some(&arts),
        houses: Some(&houses),
        characteristics: Some(characteristics.as_str()),
        ..RulesetSources::default()
    })
    .unwrap();

    let entity: Entity = serde_json::from_str(&format!(
        r#"{{
            "schema_version": 1,
            "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
            "entity_kind": "character",
            "type_id": "tiny",
            "selections": [{{ "ref": "virtue.keen_vision" }}, {{ "ref": "virtue.large" }}]
        }}"#
    ))
    .unwrap();

    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "over_budget_virtues"),
        "expected over_budget_virtues, got {:?}",
        result.issues
    );
}

#[test]
fn save_then_load_round_trips_with_byte_stable_canonical_json() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.json");
    let entity = sample_entity();

    save_entity_to_path(&entity, &path).unwrap();
    let first = fs::read_to_string(&path).unwrap();

    let reloaded = load_entity_from_path(&path).unwrap();
    assert_eq!(reloaded, entity, "round trip must preserve the entity");

    // Re-saving the reloaded entity yields byte-identical output.
    save_entity_to_path(&reloaded, &path).unwrap();
    let second = fs::read_to_string(&path).unwrap();
    assert_eq!(first, second, "canonical output must be byte-stable");
}

/// A pre-schema-14 save carrying the flat `talisman_attunements` list migrates
/// through the **real** load path the app uses, not just the engine helper: the
/// attunements arrive under `Entity.talisman`, the version is bumped to current, and
/// re-saving writes only the new shape. This is the engine-boundary half of the
/// migration proof (the UI-boundary half is `ui/e2e/specs/talisman.e2e.js`).
#[test]
fn legacy_talisman_save_migrates_through_the_real_load_path() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("legacy-magus.json");
    fs::write(
        &path,
        format!(
            r#"{{
              "schema_version": 13,
              "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
              "entity_kind": "character",
              "type_id": "magus",
              "selections": [{{ "ref": "virtue.the_gift" }}],
              "talisman_attunements": [
                {{ "description": "Projecting bolts and missiles", "bonus": 3 }}
              ]
            }}"#
        ),
    )
    .unwrap();

    let migrated = load_entity_from_path(&path).unwrap();
    assert_eq!(
        migrated.schema_version, 16,
        "the field move bumps the schema"
    );
    let talisman = migrated
        .talisman
        .as_ref()
        .expect("legacy attunements become a talisman");
    assert_eq!(talisman.attunements.len(), 1);
    assert_eq!(talisman.attunements[0].bonus, 3);
    // Nothing is invented for the parts the old shape never stored.
    assert_eq!(talisman.description, "");
    assert!(talisman.effects.is_empty());

    // Saving the migrated entity writes the new shape only.
    save_entity_to_path(&migrated, &path).unwrap();
    let written = fs::read_to_string(&path).unwrap();
    assert!(!written.contains("talisman_attunements"), "got: {written}");
    assert!(written.contains("\"talisman\""), "got: {written}");
    assert!(written.contains("\"schema_version\": 16"), "got: {written}");
}

#[test]
fn save_stamps_current_schema_version() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.json");
    let mut entity = sample_entity();
    entity.schema_version = 3; // a stale in-memory version must be corrected on write

    save_entity_to_path(&entity, &path).unwrap();
    let written = fs::read_to_string(&path).unwrap();
    assert!(
        written.contains("\"schema_version\": 16"),
        "save must stamp the current schema version, got: {written}"
    );
}

#[test]
fn sample_save_loads_with_defaulted_aging_warping_annotations() {
    // A shipped example save carries none of the new annotation fields; loading it
    // must fill them with their empty/None defaults (additive backward compat).
    let path = repo_root().join("examples/companion_sample.json");
    let entity = load_entity_from_path(&path).unwrap();
    assert_eq!(entity.apparent_age, None);
    assert!(entity.warping_effect.is_empty());
    assert!(entity.decrepitude_effect.is_empty());
    assert!(entity.aging_log.is_empty());
}

#[test]
fn arts_round_trip_and_puissant_art_reports_bonus() {
    // The shipped ruleset carries the Art registry and Puissant Art.
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    assert!(ruleset.art(&Id::new("art.ignem")).is_some());
    assert!(ruleset.art(&Id::new("art.creo")).is_some());

    // A character with two Arts and Puissant (Ignem). Arts draw from the shared
    // `xp_pool` alongside abilities (the sample already sets a pool).
    let mut entity = sample_entity();
    entity.art_scores = vec![
        ArtScore {
            art: Id::new("art.creo"),
            score: 3, // 6 xp
        },
        ArtScore {
            art: Id::new("art.ignem"),
            score: 5, // 15 xp
        },
    ];
    entity.selections.push(Selection::with_params(
        Id::new("virtue.puissant_art"),
        BTreeMap::from([("art".into(), Id::new("art.ignem"))]),
    ));

    // Puissant (Ignem) surfaces as a +3 bonus on Ignem only.
    let effective = effective_scores_loaded(&entity, &ruleset);
    let ignem = effective
        .art_bonuses
        .iter()
        .find(|b| b.art == Id::new("art.ignem"))
        .expect("Ignem bonus present");
    assert_eq!(ignem.bonus, 3);
    assert!(
        !effective
            .art_bonuses
            .iter()
            .any(|b| b.art == Id::new("art.creo")),
        "Creo is unboosted and omitted"
    );

    // The Arts survive a canonical save/load round trip; the save stamps the
    // current schema version.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("magus.json");
    save_entity_to_path(&entity, &path).unwrap();
    let reloaded = load_entity_from_path(&path).unwrap();
    assert_eq!(reloaded.schema_version, 16);
    assert_eq!(reloaded.art_scores, entity.art_scores);
}

#[test]
fn effective_scores_surface_aged_characteristic_and_drop_count() {
    // Core Rules.md:16613 worked example: Communication +2 with 3 aging points
    // drops once → effective +1. The summary must report the aged effective value
    // and the drop count, and omit unchanged Characteristics.
    use arm_rules::Characteristic;
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.characteristics.insert(Characteristic::Com, 2);
    entity.aging_points.insert(Characteristic::Com, 3);

    let effective = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(
        effective.characteristic_effective.get(&Characteristic::Com),
        Some(&1)
    );
    assert_eq!(
        effective
            .characteristic_aging_drops
            .get(&Characteristic::Com),
        Some(&1)
    );
    // A Characteristic with no aging drop is omitted from the drop map.
    assert!(
        !effective
            .characteristic_aging_drops
            .contains_key(&Characteristic::Str)
    );
}

/// "a character over the age of 35 must make aging rolls … before the game begins"
/// (Core Rules.md:2232), and aging starts "the Winter after they turn 35"
/// (`:16565`) — so a character of 40 owes one roll a year from 36 through 40, five
/// in all. The whole read-out is a pure function of the character and the rules:
/// the die is the player's and never reaches the entity, which is why the
/// die-independent half rides on the always-recomputed payload.
#[test]
fn effective_scores_report_the_aging_rolls_a_character_of_forty_owes() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();

    let aging = effective_scores_loaded(&entity, &ruleset)
        .aging
        .expect("the shipped ruleset carries aging rules");

    assert_eq!(aging.begins_after_age, 35, "the book's own number (:16565)");
    assert_eq!(
        aging.first_roll_age, 36,
        "the Winter after 35 falls in year 36"
    );
    assert_eq!(aging.rolls_owed, 5, "36, 37, 38, 39 and 40");
    assert_eq!(aging.rolls_recorded, 0, "an empty log has settled none");
    assert_eq!(aging.schedule.len(), 5);
    assert_eq!(aging.schedule.first().map(|year| year.age), Some(36));
    assert_eq!(aging.schedule.last().map(|year| year.age), Some(40));
    assert!(
        aging.schedule.iter().all(|year| !year.recorded),
        "nothing is recorded yet: {:?}",
        aging.schedule
    );

    // "age/10 (round up)" (`:16567`) at the ACTUAL age (`:16577`).
    assert_eq!(aging.age_modifier, 4);
    // No ritual, so no bonus and no `:16575` clamp standing over this character.
    assert_eq!(aging.longevity_modifier, 0);
    assert!(!aging.longevity_clamp_active);
    // The whole non-die half, so the UI adds only the number the player typed — and
    // it is exactly the sum of the terms the read-out names, the Virtue/Flaw one
    // included. Naming three of four is what made the displayed formula contradict
    // itself (guided-creation-review-2026-08 #22).
    assert_eq!(
        aging.fixed_total,
        aging.age_modifier - aging.living_conditions_modifier - aging.longevity_modifier
            + aging.trait_modifier
    );
}

/// A character of 40 rolling a 10: `10 + ⌈40/10⌉ = 14`, which the shipped table
/// answers with "1 Aging Point in Qik" (Core Rules.md:16603). One less on the die
/// lands on 13 — "Gain sufficient Aging Points … to reach the next level in
/// Decrepitude, and Crisis" (`:16602`) — so the preview must say a Crisis follows.
///
/// The die is the player's, typed in and never stored, which is why this is a
/// command rather than a field of the character.
#[test]
fn aging_preview_totals_the_typed_die_and_names_the_outcome() {
    use arm_rules::{AgingPointTarget, Characteristic};
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.living_conditions.clear();
    entity.longevity_ritual = None;

    let AgingProjection::Previewed { total, outcome, .. } =
        arm_app::ruleset_io::aging_preview_loaded(
            &entity,
            &ruleset,
            40,
            10,
            &BTreeMap::new(),
            None,
        )
    else {
        panic!("the shipped ruleset carries aging rules");
    };
    assert_eq!(total.die, 10);
    assert_eq!(total.age_modifier, 4);
    assert_eq!(total.total, 14);
    assert!(!total.capped_by_longevity, "no ritual, so no :16575 clamp");
    assert_eq!(outcome.total, 14);
    assert!(
        outcome.apparent_age_increases,
        "14 is well over the 3 of :16600"
    );
    assert_eq!(
        outcome
            .awards
            .iter()
            .map(|award| award.target.clone())
            .collect::<Vec<_>>(),
        vec![AgingPointTarget::Named(Characteristic::Qik)],
        "row 14 names Quickness and nothing else"
    );
    assert!(!outcome.crisis);

    let AgingProjection::Previewed { total, outcome, .. } =
        arm_app::ruleset_io::aging_preview_loaded(&entity, &ruleset, 40, 9, &BTreeMap::new(), None)
    else {
        panic!("the shipped ruleset carries aging rules");
    };
    assert_eq!(total.total, 13);
    assert!(outcome.crisis, "13 is the first Crisis row (:16602)");
}

/// A mistyped die has to be recoverable — a magus of 60 owes 25 rolls (`:2232`) —
/// so applying a year and reverting it must leave the character it started from,
/// byte for byte, not merely something equivalent.
#[test]
fn an_applied_aging_year_reverts_to_the_character_it_started_from() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.normalize();
    let before = serde_json::to_string(&entity).unwrap();

    // Row 14 names its own Characteristic, so the player places nothing.
    let AgingApplication::Applied {
        entity: applied,
        total,
        outcome,
        ..
    } = arm_app::ruleset_io::aging_apply_loaded(&entity, &ruleset, 40, 10, &BTreeMap::new(), None)
    else {
        panic!("a year the character owes and has not rolled applies");
    };
    assert_eq!(
        total.total, 14,
        "the applied roll comes back with the entity"
    );
    assert!(!outcome.crisis);
    assert_ne!(
        serde_json::to_string(&*applied).unwrap(),
        before,
        "the year was written"
    );
    assert_eq!(applied.aging_log.len(), 1);

    let AgingReversion::Reverted { entity: reverted } =
        arm_app::ruleset_io::aging_revert_loaded(&applied, &ruleset, 40)
    else {
        panic!("the year just applied is recorded, so it reverts");
    };
    assert_eq!(serde_json::to_string(&*reverted).unwrap(), before);

    // A year no entry records is a refusal the player is told about, never a
    // silent no-op — and it crosses the boundary as a localizable finding.
    let AgingReversion::Rejected { issues } =
        arm_app::ruleset_io::aging_revert_loaded(&entity, &ruleset, 40)
    else {
        panic!("nothing is recorded for that year");
    };
    assert_eq!(
        issues
            .iter()
            .map(|issue| issue.code.as_str())
            .collect::<Vec<_>>(),
        vec![arm_rules::ValidationIssue::CODE_AGING_YEAR_NOT_RECORDED]
    );
}

/// The calculator has to show the Crisis BEFORE Apply, and it has to show the one
/// Apply will write — which is not the one a bare `crisis_preview` of the
/// character standing in front of you answers.
///
/// > **Crisis:** Increase the character's Decrepitude first, and then roll on the
/// > Crisis Table. (`:16619`)
///
/// The five Aging Points row 13 awards ARE that increase, so the CRISIS TOTAL adds
/// the Decrepitude the year itself raised. Read off the character before the year
/// is applied, the same die answers 14 and the same player would then watch the
/// log record 15. So the preview resolves the year in memory and throws the
/// character away: the preview and the apply send one identical request, and the
/// reading cannot disagree with what lands.
#[test]
fn aging_preview_reads_the_crisis_off_the_year_it_would_apply() {
    use arm_rules::Characteristic;
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.living_conditions.clear();
    entity.longevity_ritual = None;

    let distribution = BTreeMap::from([(Characteristic::Sta, 5)]);
    let AgingProjection::Previewed {
        total,
        outcome,
        crisis,
    } = arm_app::ruleset_io::aging_preview_loaded(
        &entity,
        &ruleset,
        40,
        9,
        &distribution,
        Some(10),
    )
    else {
        panic!("the shipped ruleset carries aging rules");
    };
    assert_eq!(total.total, 13);
    assert!(outcome.crisis, "13 is the first Crisis row (:16602)");
    let previewed = crisis.expect("a Crisis with a die rolled reads whole");
    assert_eq!(
        previewed.total.decrepitude_score, 1,
        "the increase :16619 puts first"
    );
    assert_eq!(previewed.total.total, 15);
    assert_eq!(previewed.row, Id::new("crisis.minor_illness"));

    // THE POINT: what the player is shown is what the year writes.
    let AgingApplication::Applied {
        crisis: applied, ..
    } = arm_app::ruleset_io::aging_apply_loaded(&entity, &ruleset, 40, 9, &distribution, Some(10))
    else {
        panic!("the same request applies");
    };
    assert_eq!(applied.as_deref(), Some(&*previewed));

    // And the reading the un-applied character would give is the wrong one, which
    // is what makes resolving the year first load-bearing rather than tidy.
    assert_eq!(
        arm_rules::crisis_preview(&entity, &ruleset, 40, 10)
            .expect("the shipped ruleset carries a Crisis Table")
            .total
            .total,
        14,
        "one Decrepitude short, because this year's points have not landed"
    );

    // Before the points are placed there is no honest Crisis to read, but the
    // total and the outcome still stand: the player is told a Crisis follows and
    // what to place, which is the order :16619 asks for.
    let AgingProjection::Previewed {
        total,
        outcome,
        crisis,
    } = arm_app::ruleset_io::aging_preview_loaded(
        &entity,
        &ruleset,
        40,
        9,
        &BTreeMap::new(),
        Some(10),
    )
    else {
        panic!("an unplaced distribution is not a reason to withhold the total");
    };
    assert_eq!(total.total, 13);
    assert!(outcome.crisis);
    assert_eq!(crisis, None);
}

/// The Crisis the player rolled has to reach the character, and the two things
/// only the applied year can say have to reach the player.
///
/// A companion of 40 rolling a 9 totals `9 + ⌈40/10⌉ = 13`, which is "Gain
/// sufficient Aging Points (in any Characteristics) to reach the next level in
/// Decrepitude, and Crisis" (`:16602`) — five points from nothing. The Crisis is
/// then read off the character those five points already made (`:16619`), so a
/// Simple Die of 10 totals `10 + 4 + 1 = 15`: the minor illness of `:16628`,
/// survivable on a Stamina stress roll against an Ease Factor of 3 or a CrCo20.
///
/// Until the die crossed this edge every Crisis the shipped app recorded was owed
/// and unrolled, whatever the player had thrown.
#[test]
fn an_applied_crisis_year_comes_back_with_the_crisis_and_the_ritual_it_spent() {
    use arm_rules::{AgingNote, Characteristic, CrisisOutcome, CrisisSeverity, LongevityRitual};
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.living_conditions.clear();
    // A ritual with no bonus leaves the AGING TOTAL alone, so the year still lands
    // on 13 — and it is still a ritual the Crisis spends (`:16573`).
    entity.longevity_ritual = Some(LongevityRitual {
        source: arm_rules::LongevitySource::External,
        bonus: Some(0),
        focus: String::new(),
    });

    let distribution = BTreeMap::from([(Characteristic::Sta, 5)]);
    let AgingApplication::Applied {
        total,
        outcome,
        crisis,
        notes,
        ..
    } = arm_app::ruleset_io::aging_apply_loaded(&entity, &ruleset, 40, 9, &distribution, Some(10))
    else {
        panic!("a year the character owes and has not rolled applies");
    };
    assert_eq!(total.total, 13);
    assert!(outcome.crisis, "13 is the first Crisis row (:16602)");

    let crisis = crisis.expect("a Crisis with a die rolled comes back resolved");
    assert_eq!(crisis.total.die, 10, "the player's Simple Die (:16621)");
    assert_eq!(crisis.total.age_modifier, 4);
    assert_eq!(
        crisis.total.decrepitude_score, 1,
        "the five points this very year awarded, counted first (:16619)"
    );
    assert_eq!(crisis.total.total, 15);
    assert_eq!(crisis.row, Id::new("crisis.minor_illness"));
    assert_eq!(
        crisis.outcome,
        CrisisOutcome::Illness {
            severity: CrisisSeverity::Minor,
            ease_factor: Some(3),
            ritual_level: 20,
        }
    );
    let survival = crisis
        .survival
        .expect("an illness is survivable, so it has a read-out");
    assert_eq!(survival.ease_factor, Some(3));
    assert_eq!(survival.ritual_level, 20);
    assert_eq!(survival.allowances.len(), 1, "one doctor only (:16634)");

    // "its power is spent, and the focal ritual must be performed again"
    // (`:16573`) — reported, because the entity keeps the stored choice.
    assert_eq!(notes, vec![AgingNote::LongevityRitualSpent]);

    // A year with no Crisis die is still written, with the Crisis owed and
    // unrolled — the aging roll happened whether or not the second die was thrown.
    let AgingApplication::Applied { crisis: none, .. } =
        arm_app::ruleset_io::aging_apply_loaded(&entity, &ruleset, 40, 9, &distribution, None)
    else {
        panic!("an unrolled Crisis is a legitimate state, not a refusal");
    };
    assert_eq!(none, None);
}

#[test]
fn effective_scores_surface_house_grants_read_only() {
    // The V/F view renders House grants read-only, so effective scores must carry
    // the derived grant Selections without the UI re-deriving them. A Bjornaer
    // magus is granted Heartbeast (a fixed grant), free of the point budget.
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.house = Some(Id::new("house.bjornaer"));

    let effective = effective_scores_loaded(&entity, &ruleset);
    assert!(
        effective
            .granted_selections
            .iter()
            .any(|s| s.item_ref == Id::new("virtue.heartbeast")),
        "Bjornaer's granted Heartbeast should appear in granted_selections, got {:?}",
        effective.granted_selections
    );

    // A character with no House is granted nothing.
    let mut houseless = sample_entity();
    houseless.house = None;
    assert!(
        effective_scores_loaded(&houseless, &ruleset)
            .granted_selections
            .is_empty(),
        "a character with no House has no granted selections"
    );
}

#[test]
fn effective_scores_surface_virtue_flaw_balance() {
    // The balance bar must read the engine's own spent Virtue/Flaw points
    // (`validation::compute_balance` — the same function `export.rs`'s Markdown
    // export and the over-budget/unbalanced-Virtues validation issues already
    // read) rather than re-deriving them a third time in TypeScript. Audit
    // finding G1 (round 4): the same defect class already fixed once for
    // `characteristic_points_used` (VA1/GF1/GD4).
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut companion = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    let bare = effective_scores_loaded(&companion, &ruleset);
    assert_eq!(
        (bare.virtue_points, bare.flaw_points),
        (0, 0),
        "no selections means no spent points"
    );

    // Self-Confident is a shipped Minor Virtue (1 point); Infamous is a shipped
    // Minor Flaw (1 point) — rules/core/virtues_flaws.json.
    companion
        .selections
        .push(Selection::new(Id::new("virtue.self_confident")));
    companion
        .selections
        .push(Selection::new(Id::new("flaw.infamous")));
    let effective = effective_scores_loaded(&companion, &ruleset);
    assert_eq!(effective.virtue_points, 1, "one minor virtue = 1 point");
    assert_eq!(effective.flaw_points, 1, "one minor flaw = 1 point");
}

#[test]
fn load_entity_from_missing_path_is_io_error() {
    let err = load_entity_from_path(&repo_root().join("does/not/exist.json")).unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

#[test]
fn effective_scores_surface_confidence_and_supernatural_slots() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut companion = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    // Companion default: Confidence 1/3, no free Supernatural slot (unGifted).
    let base = effective_scores_loaded(&companion, &ruleset);
    assert_eq!((base.confidence_score, base.confidence_points), (1, 3));
    assert_eq!(base.supernatural_free_total, 0);

    // Self-Confident raises Confidence to 2/5 (the shipped V/F).
    companion
        .selections
        .push(Selection::new(Id::new("virtue.self_confident")));
    let confident = effective_scores_loaded(&companion, &ruleset);
    assert_eq!(
        (confident.confidence_score, confident.confidence_points),
        (2, 5)
    );

    // Taking The Gift opens exactly one free Supernatural-Ability slot.
    companion
        .selections
        .push(Selection::new(Id::new("virtue.the_gift")));
    assert_eq!(
        effective_scores_loaded(&companion, &ruleset).supernatural_free_total,
        1
    );

    // Infamous surfaces a Local reputation slot naming the Flaw that opened it.
    companion
        .selections
        .push(Selection::new(Id::new("flaw.infamous")));
    let grants = effective_scores_loaded(&companion, &ruleset).reputation_grants;
    assert_eq!(grants.len(), 1, "one grant per granting V/F: {grants:?}");
    assert_eq!(grants[0].kind, Some(arm_rules::ReputationType::Local));
    assert_eq!(grants[0].score, 4);
    assert_eq!(grants[0].source, Id::new("flaw.infamous"));

    // Famous leaves the type to the player. That is ONE slot the player types
    // themselves, not one slot per Reputation type — `validate_reputations`
    // allows exactly one wildcard Reputation, so flattening it into four
    // offered four ways to overspend a single legal slot.
    companion
        .selections
        .push(Selection::new(Id::new("virtue.famous")));
    let grants = effective_scores_loaded(&companion, &ruleset).reputation_grants;
    assert_eq!(grants.len(), 2, "one grant per granting V/F: {grants:?}");
    let wildcard = grants
        .iter()
        .find(|g| g.source == Id::new("virtue.famous"))
        .expect("Famous surfaces a grant");
    assert_eq!(wildcard.kind, None, "Famous fixes no Reputation type");
    assert_eq!(wildcard.score, 4);

    // A magus gets no free Supernatural slot (his free ability is Hermetic magic).
    let magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    assert_eq!(
        effective_scores_loaded(&magus, &ruleset).supernatural_free_total,
        0
    );
}

/// The Abilities view has to show a magus which Hermetic minimums it still owes, so
/// the checklist crosses the boundary with the effective scores rather than being
/// re-derived in JS from the rules file. Against the **real shipped ruleset**: the
/// three minimums of Core Rules.md:2437 plus the four recommendations of
/// `:2451-2461`, seven rows.
///
/// The same payload carries `xp_general_pool` — the pool the solve funds from, which
/// `xp_general_used` (a max-flow value) cannot be divided by.
#[test]
fn effective_scores_surface_the_magus_minimum_ability_checklist() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    let checklist = effective_scores_loaded(&magus, &ruleset).magus_minimum_abilities;
    assert_eq!(
        checklist.len(),
        7,
        "3 required + 4 recommended: {checklist:?}"
    );
    assert!(
        checklist.iter().all(|row| !row.met),
        "a fresh magus owes all of them: {checklist:?}"
    );

    // Buying Magic Theory 1 flips exactly one row. Magic Theory, not Parma Magica:
    // Parma 1 is demanded by BOTH lists (`:2437` and `:2459`), so buying it correctly
    // flips two, while the recommended Magic Theory threshold is 3 — which makes this
    // the only clean single-flip probe.
    magus.ability_scores = vec![arm_rules::AbilityScore {
        ability: Id::new("ability.magic_theory"),
        score: 1,
        specialty: None,
        parameter: None,
    }];
    let checklist = effective_scores_loaded(&magus, &ruleset).magus_minimum_abilities;
    assert_eq!(checklist.iter().filter(|row| row.met).count(), 1);
    let met = checklist
        .iter()
        .find(|row| row.met)
        .expect("one row is met now");
    assert_eq!(met.ability, Id::new("ability.magic_theory"));
    assert_eq!(met.min_score, 1);
    assert_eq!(met.score, 1);

    // The magus's general pool is its apprenticeship experience once it is built
    // through its life stages, and the typed pool otherwise.
    magus.xp_pool = 240;
    let scores = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(scores.xp_general_pool, 240);
    assert_eq!(scores.xp_general_bonus, 0);

    // Skilled Parens raises that pool by 60 — "an additional 60 experience points …
    // during apprenticeship" (Core Rules.md:4966). The bar shows the typed 240 as the
    // editable total, so the bonus has to reach it as a figure of its own; without it
    // the pool and the field it is entered in differ with nothing to explain the gap.
    magus.selections = vec![arm_rules::Selection::new(Id::new("virtue.skilled_parens"))];
    let scores = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(scores.xp_general_pool, 300);
    assert_eq!(scores.xp_general_bonus, 60);
    magus.selections.clear();

    // A companion is not admitted to the Order, so it gets no checklist at all —
    // exactly like the magus-only spell-level caps.
    let companion = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    assert!(
        effective_scores_loaded(&companion, &ruleset)
            .magus_minimum_abilities
            .is_empty()
    );
}

/// The XP bar must be able to show an OVERSPENT pool as a negative "Available".
/// `xp_general_used` cannot express that: it is a max-flow value capped by the pool
/// itself, so `pool - general_used` never goes below zero. The overspend lives in
/// `total_demand - max_flow` (the same figure `not_enough_xp` reports as its
/// shortfall), so `max_flow` has to reach the frontend.
#[test]
fn effective_scores_surface_max_flow_so_the_ui_can_show_an_overspent_pool() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    // A 10-xp pool buying an Ability score of 2 (15 xp on the advancement table):
    // a 5-xp overspend.
    entity.xp_pool = 10;
    entity.ability_scores = vec![arm_rules::AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 2,
        specialty: None,
        parameter: None,
    }];

    let effective = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(effective.xp_total_demand, 15, "score 2 costs 15 xp");
    // The general pool is drained but cannot cover the demand...
    assert_eq!(effective.xp_general_used, 10);
    // ...so the fundable total stops at the pool, and the 5-xp shortfall is exactly
    // total_demand - max_flow — the negative the bar renders as "Available: -5".
    assert_eq!(effective.xp_max_flow, 10);
    assert_eq!(effective.xp_total_demand - effective.xp_max_flow, 5);

    // A pool that covers the spend reports no shortfall.
    entity.xp_pool = 50;
    let legal = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(legal.xp_total_demand, legal.xp_max_flow);
}

/// The Characteristic point cost is surfaced by the engine rather than recomputed
/// in the frontend. `ui/src/lib/derive.ts` carried its own copy of the point-buy
/// table (audit findings VA1/GF1/GD4, raised by three separate reviewers), which
/// could drift from `CharacteristicRules::total_cost` silently. The payload now
/// carries the authoritative figure so the UI has nothing to recompute.
///
/// Uses the rulebook's own worked example so the assertion is anchored to the
/// source, not to whatever the code happens to return:
/// Ars Magica - Definitive Edition (Core Rules).md:2358 — Int +3 (6), Per +1 (1),
/// Pre -3 (-6), Com -1 (-1), Sta 0 (0), Qik +2 (3), Str +2 (3), Dex +1 (1) => 7.
#[test]
fn effective_scores_surface_the_characteristic_points_used() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    use arm_rules::Characteristic;
    entity.characteristics = std::collections::BTreeMap::from([
        (Characteristic::Int, 3),
        (Characteristic::Per, 1),
        (Characteristic::Pre, -3),
        (Characteristic::Com, -1),
        (Characteristic::Sta, 0),
        (Characteristic::Qik, 2),
        (Characteristic::Str, 2),
        (Characteristic::Dex, 1),
    ]);

    let effective = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(
        effective.characteristic_points_used, 7,
        "the rulebook's worked example nets gains against spends"
    );
}

/// The V/F spell-levels contribution is surfaced as its OWN payload figure, not
/// only folded into the effective budget, so the spell-levels bar can show the
/// editable base beside a labelled bonus — the way the XP bar lists extra pools
/// beside the general one.
#[test]
fn effective_scores_surface_the_spell_levels_bonus_separately() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    // No spell-levels V/F: the base profile budget with nothing added.
    let plain = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(plain.spell_levels_profile_base, 120);
    assert_eq!(plain.spell_levels_budget, 120);
    assert_eq!(plain.spell_levels_bonus, 0);

    // Skilled Parens (+30) reports the bonus on its own, and the budget still
    // carries the total, so base + bonus == budget.
    magus
        .selections
        .push(arm_rules::Selection::new(Id::new("virtue.skilled_parens")));
    let boosted = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(boosted.spell_levels_bonus, 30);
    assert_eq!(boosted.spell_levels_budget, 150);

    // The bonus is independent of the per-character base override: overriding the
    // base to 80 keeps the +30 reportable and the budget at 110.
    magus.spell_levels_override = Some(80);
    let overridden = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(overridden.spell_levels_bonus, 30);
    assert_eq!(overridden.spell_levels_budget, 110);
}

/// The levels of spells a magus took out of its post-Gauntlet points reach the
/// frontend as a figure of their own, beside the profile base and the V/F bonus, so
/// the spell-levels bar can label all three parts of the budget rather than folding
/// them into an unexplained total.
///
/// They are additive, not a second budget: the profile's 120 are apprenticeship's
/// (`:2435`), while these are the player's chosen slice of the fungible "30 points
/// per year" (`:2471`).
#[test]
fn effective_scores_surface_the_post_gauntlet_spell_levels_separately() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    // A magus of 60 gauntleted at 25: 35 years at 30 points each, 300 of them taken
    // as levels of spells.
    magus.age = Some(60);
    // The funding mode is stored since schema 16, so a plan needs it to be live.
    magus.ability_funding = arm_rules::AbilityFunding::LifeStages;
    magus.life_stages = Some(arm_rules::LifeStagePlan {
        gauntlet_age: Some(25),
        post_gauntlet_spell_levels: 300,
        ..arm_rules::LifeStagePlan::default()
    });
    let experienced = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(experienced.spell_levels_life_stage, 300);
    assert_eq!(experienced.spell_levels_profile_base, 120);
    assert_eq!(experienced.spell_levels_bonus, 0);
    // base + bonus + life stage == budget.
    assert_eq!(experienced.spell_levels_budget, 420);

    // A magus standing at its Gauntlet has lived no year past it, so it takes no
    // levels out of them and its budget is the profile's 120 — the pre-6b5 number.
    magus.age = Some(25);
    magus.life_stages = Some(arm_rules::LifeStagePlan::default());
    let fresh = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(fresh.spell_levels_life_stage, 0);
    assert_eq!(fresh.spell_levels_budget, 120);
}

/// A companion built through its life stages, speaking `native_language` — the
/// character a Sample Childhood package is applied to (childhood exists only in
/// life-stage mode, and the package's native-language entry takes the plan's
/// language).
fn life_stage_companion(native_language: &str) -> Entity {
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    entity.age = Some(25);
    // The funding mode is stored since schema 16, so a plan needs it to be live.
    entity.ability_funding = arm_rules::AbilityFunding::LifeStages;
    entity.life_stages = Some(arm_rules::LifeStagePlan {
        native_language: Some(native_language.to_string()),
        ..arm_rules::LifeStagePlan::default()
    });
    entity
}

/// The entity's Ability rows as `(ability, parameter, score)`, in the canonical
/// order the applied entity comes back normalized into.
fn ability_rows(entity: &Entity) -> Vec<(&str, Option<&str>, u8)> {
    entity
        .ability_scores
        .iter()
        .map(|row| (row.ability.as_str(), row.parameter.as_deref(), row.score))
        .collect()
}

/// Applying a shipped package through the command path writes its entries as
/// ordinary bought Ability rows, the native-language one under the language the
/// plan names — "Athletic Childhood: Athletics 2, Brawl 2, Native Language 5,
/// Swim 2" (Ars Magica - Definitive Edition (Core Rules).md:2384).
#[test]
fn apply_childhood_package_writes_the_package_rows() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;

    let outcome = apply_childhood_package_loaded(
        &life_stage_companion("German"),
        &Id::new("childhood.athletic"),
        &BTreeMap::new(),
        &ruleset,
    );

    let ChildhoodApplication::Applied { entity } = outcome else {
        panic!("Athletic Childhood asks the player for nothing, so it applies: {outcome:?}");
    };
    assert_eq!(
        ability_rows(&entity),
        vec![
            ("ability.athletics", None, 2),
            ("ability.brawl", None, 2),
            ("ability.living_language", Some("German"), 5),
            ("ability.swim", None, 2),
        ]
    );
    assert_eq!(
        entity
            .life_stages
            .and_then(|plan| plan.childhood_package)
            .as_ref(),
        Some(&Id::new("childhood.athletic")),
        "the package taken is recorded on the plan"
    );
}

/// A slot the player never answered is reported as an ordinary
/// `ValidationIssue`, carrying the Ability, its parameter key, and the slot the
/// UI highlights — Traveling asks for two Area Lores plus a second language
/// (Core Rules.md:2388), and only two of the three arrive here.
#[test]
fn apply_childhood_package_rejects_an_unfilled_slot() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut slot_values = BTreeMap::new();
    slot_values.insert("area_a".to_string(), "Rhine".to_string());
    slot_values.insert("language".to_string(), "Italian".to_string());

    let outcome = apply_childhood_package_loaded(
        &life_stage_companion("German"),
        &Id::new("childhood.traveling"),
        &slot_values,
        &ruleset,
    );

    let ChildhoodApplication::Rejected { issues } = outcome else {
        panic!("area_b was never answered, so nothing may be written: {outcome:?}");
    };
    assert_eq!(
        issues.len(),
        1,
        "one unanswered slot, one issue: {issues:?}"
    );
    let issue = &issues[0];
    assert_eq!(issue.code, "childhood_slot_unfilled");
    assert_eq!(
        issue.args.get("ability").map(String::as_str),
        Some("ability.area_lore")
    );
    assert_eq!(issue.args.get("key").map(String::as_str), Some("area"));
    assert_eq!(issue.args.get("slot").map(String::as_str), Some("area_b"));
}

/// The outcome crosses IPC as a tagged union, because the frontend switches on
/// `status` — and rejections travel as issue codes, never as English prose.
#[test]
fn apply_childhood_package_serializes_as_a_tagged_union() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;

    let applied = serde_json::to_value(apply_childhood_package_loaded(
        &life_stage_companion("German"),
        &Id::new("childhood.athletic"),
        &BTreeMap::new(),
        &ruleset,
    ))
    .unwrap();
    assert_eq!(applied["status"], "applied");
    assert!(
        applied["entity"].is_object(),
        "an applied outcome carries the entity: {applied}"
    );

    // A package id the ruleset does not ship is a rejection, not a silent no-op.
    let rejected = serde_json::to_value(apply_childhood_package_loaded(
        &life_stage_companion("German"),
        &Id::new("childhood.nonesuch"),
        &BTreeMap::new(),
        &ruleset,
    ))
    .unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["issues"][0]["code"], "childhood_package_unknown");
}

/// Extracts every fixed issue code from the engine's validation source so the
/// Fluent coverage check tracks the codes the engine actually emits. The engine
/// declares the closed set as `pub const CODE_*: &'static str = "...";` and
/// emits them via `ValidationIssue::CODE_*`, so the code values live in those
/// const definitions.
fn validation_codes() -> Vec<String> {
    // The engine's `validation` module was split into a directory
    // (`validation/mod.rs` + cohesive submodules); scan every `.rs` file in it so
    // codes declared in any submodule are still tracked.
    let dir = repo_root().join("crates/arm-rules/src/validation");
    let mut codes = Vec::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let full = fs::read_to_string(&path).unwrap();
        // Ignore each file's `#[cfg(test)]` module, whose fixtures use fake codes.
        let src = full.split("mod tests").next().unwrap();
        // Each fixed code is the string literal in a `const CODE_* : &'static str =
        // "code";` definition.
        for fragment in src.split("const CODE_").skip(1) {
            let Some(open) = fragment.find('"') else {
                continue;
            };
            let rest = &fragment[open + 1..];
            if let Some(end) = rest.find('"') {
                codes.push(rest[..end].to_string());
            }
        }
    }
    codes.sort();
    codes.dedup();
    assert!(!codes.is_empty(), "expected to find validation codes");
    codes
}

/// The per-category flaw caps emit codes derived at runtime from the category
/// slug (`too_many_[major_]<category>_flaws`), so they are not source literals
/// the scraper above can see. Recompute them from the shipped profiles so the
/// Fluent coverage check still tracks every code the engine can actually emit.
fn dynamic_flaw_cap_codes() -> Vec<String> {
    let json = fs::read_to_string(repo_root().join("rules/core/character_types.json")).unwrap();
    let profiles: serde_json::Value = serde_json::from_str(&json).unwrap();
    let mut codes = Vec::new();
    for profile in profiles.as_array().unwrap() {
        let caps = profile["budget"]["flaw_category_caps"].as_array();
        for cap in caps.into_iter().flatten() {
            let category = cap["category"].as_str().unwrap();
            let major_only = cap["major_only"].as_bool().unwrap_or(false);
            codes.push(if major_only {
                format!("too_many_major_{category}_flaws")
            } else {
                format!("too_many_{category}_flaws")
            });
        }
    }
    codes.sort();
    codes.dedup();
    assert!(
        !codes.is_empty(),
        "expected shipped flaw-category cap codes"
    );
    codes
}

/// The per-category *virtue* caps emit codes derived at runtime from the
/// category slug (`too_many_[major_]<category>_virtues`) — the virtue analog of
/// [`dynamic_flaw_cap_codes`]. The magus `≤1 Major Hermetic Virtue` cap lives
/// here, so recompute it from the shipped profiles for the Fluent coverage check.
fn dynamic_virtue_cap_codes() -> Vec<String> {
    let json = fs::read_to_string(repo_root().join("rules/core/character_types.json")).unwrap();
    let profiles: serde_json::Value = serde_json::from_str(&json).unwrap();
    let mut codes = Vec::new();
    for profile in profiles.as_array().unwrap() {
        let caps = profile["budget"]["virtue_category_caps"].as_array();
        for cap in caps.into_iter().flatten() {
            let category = cap["category"].as_str().unwrap();
            let major_only = cap["major_only"].as_bool().unwrap_or(false);
            codes.push(if major_only {
                format!("too_many_major_{category}_virtues")
            } else {
                format!("too_many_{category}_virtues")
            });
        }
    }
    codes.sort();
    codes.dedup();
    assert!(
        !codes.is_empty(),
        "expected shipped virtue-category cap codes"
    );
    codes
}

#[test]
fn devil_child_resolves_infernal_might_and_power_budget_end_to_end() {
    // Building a Devil Child mythic companion against the shipped rules must yield
    // a nonzero effective Infernal Might and power-levels budget: the type requires
    // Demonic Blood (Infernal Might 5 + 30 power levels, RoP:Infernal:4120-4122) and
    // grants a choice of Demonic Might (+2) or Demonic Powers (+20 levels).
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("mythic_companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    entity.mythic_type = Some(Id::new("mythic_type.devil_child"));
    // Demonic Blood is a *required* Virtue (user-selected), not a type grant.
    entity
        .selections
        .push(Selection::new(Id::new("virtue.demonic_blood")));
    // Pick Demonic Might for the type's free-Minor choice (+2 Infernal Might).
    entity.mythic_choices.insert(
        "devil_child_free_minor".into(),
        Selection::new(Id::new("virtue.demonic_might")),
    );

    let scores = effective_scores_loaded(&entity, &ruleset);
    let might = scores
        .might
        .expect("a Devil Child has an effective Might score");
    assert_eq!(might.realm, arm_rules::Realm::Infernal);
    assert_eq!(might.score, 7); // 5 (Demonic Blood) + 2 (Demonic Might grant)
    assert_eq!(scores.power_levels_budget, 30); // Demonic Blood's 30 levels
}

/// The real UI strings for a language, keyed by Fluent message name — the map the
/// frontend hands to the Markdown export. Only argument-free single-line messages
/// are usable: the engine links no Fluent formatter, so a message interpolating
/// `{ $arg }` could never be resolved there (see `arm_rules::export`).
fn locale_labels(lang: &str) -> BTreeMap<String, String> {
    let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
    ftl.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split_once(" = "))
        .filter(|(_, value)| !value.contains('{'))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// Exporting writes a Markdown document whose chrome is the **real** English UI
/// wording, not a synthetic key map: the headings, column headers, and the
/// untitled-character title all come from `locales/en/main.ftl`. This is the
/// production path the export command takes; the engine's own golden test
/// deliberately feeds a `key -> key` map instead.
#[test]
fn exporting_writes_markdown_with_the_real_english_labels() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("companion.md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let labels = locale_labels("en");

    export_markdown_to_path(&sample_entity(), Some(&localized), &labels, &path).unwrap();
    let doc = fs::read_to_string(&path).unwrap();

    // The sample has no name, so the title is the localized untitled marker, and
    // the subtitle names the character type.
    assert!(doc.starts_with("# Untitled character\n"), "got: {doc}");
    assert!(doc.contains("*Companion*"), "got: {doc}");
    // Sections use the shipped English headings.
    assert!(doc.contains("## Characteristics"), "got: {doc}");
    assert!(doc.contains("## Virtues & Flaws"), "got: {doc}");
    assert!(doc.contains("## Abilities"), "got: {doc}");
    // Values: a bought Characteristic, and Awareness 2 raised to an effective 4 by
    // the sample's Puissant Awareness.
    assert!(doc.contains("| Intelligence | +2 |"), "got: {doc}");
    // The Virtue row carries its type (the item's category) and its magnitude, both
    // localized — `category-general` is one of the families the frontend composes.
    assert!(
        doc.contains("| Puissant Awareness | General | Minor |"),
        "got: {doc}"
    );
    assert!(
        doc.contains("| Awareness | searching | 2 | 4 |"),
        "got: {doc}"
    );
    assert!(doc.contains("- **XP pool**: "), "got: {doc}");
    // No chrome key leaks through as its own label — that is the fallback the
    // formatter uses for a key the caller failed to supply.
    assert!(!doc.contains("export-col-"), "got: {doc}");
    assert!(!doc.contains("identity-name"), "got: {doc}");
}

/// A destination inside a directory that does not exist is a filesystem failure,
/// surfaced as `AppError::Io` (the frontend maps the variant to its own message).
#[test]
fn exporting_into_a_missing_directory_is_io_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("no-such-dir").join("companion.md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    let err = export_markdown_to_path(
        &sample_entity(),
        Some(&localized),
        &locale_labels("en"),
        &path,
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

/// A destination the user typed without an extension gets `.md`, the same way a
/// save without one gets `.armc` — this is the enforcement the export command
/// applies before writing.
#[test]
fn exporting_a_path_without_an_extension_writes_a_dot_md_file() {
    let tmp = tempfile::tempdir().unwrap();
    let target = ensure_extension(tmp.path().join("companion"), "md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    export_markdown_to_path(
        &sample_entity(),
        Some(&localized),
        &locale_labels("en"),
        &target,
    )
    .unwrap();

    assert_eq!(target, tmp.path().join("companion.md"));
    assert!(target.is_file(), "the .md file must exist");
}

/// Exporting before a ruleset is loaded cannot resolve a single display name, so
/// it fails with the same `NotLoaded` the other ruleset-dependent commands use.
/// The absent ruleset is an argument here rather than a lock read inside the
/// command, which is what makes the case reachable without a Tauri runtime.
#[test]
fn exporting_without_a_loaded_ruleset_is_not_loaded_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("companion.md");

    let err =
        export_markdown_to_path(&sample_entity(), None, &locale_labels("en"), &path).unwrap_err();
    assert!(matches!(err, AppError::NotLoaded), "got {err:?}");
    assert!(
        !path.exists(),
        "a failed export must not leave a file behind"
    );
}

/// A label map missing even one chrome key fails the whole export rather than
/// letting the raw key reach the document (`arm_rules::export::ExportError`,
/// CLAUDE.md: "never render a raw ID or enum value as a user-facing label").
/// `AppError::Export` carries the engine's own list of everything unresolved, and
/// nothing is written.
#[test]
fn exporting_with_an_incomplete_label_map_is_export_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("companion.md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    let err = export_markdown_to_path(&sample_entity(), Some(&localized), &BTreeMap::new(), &path)
        .unwrap_err();
    let AppError::Export { missing } = err else {
        panic!("expected AppError::Export, got {err:?}");
    };
    assert!(!missing.is_empty(), "expected at least one missing key");
    assert!(
        !path.exists(),
        "a failed export must not leave a file behind"
    );
}

/// The command that tells the frontend which labels to send must hand back the
/// engine's own list, never a hand-maintained copy of it.
#[test]
fn export_label_keys_command_returns_the_engine_list() {
    assert_eq!(
        arm_app::commands::export_label_keys(),
        arm_rules::export::LABEL_KEYS
            .iter()
            .map(|key| key.to_string())
            .collect::<Vec<String>>()
    );
}

/// Every document-chrome key the exporter can ask for must exist in every locale,
/// or the exported sheet would print the raw key. Mirrors
/// `every_validation_code_has_a_fluent_key_in_each_locale`.
#[test]
fn every_export_label_key_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for key in arm_rules::export::LABEL_KEYS {
            assert!(
                ftl.contains(&format!("{key} =")),
                "locale '{lang}' is missing key '{key}'"
            );
        }
    }
}

/// The wizard's step rail labels each creation phase through `phase-<slug>`, so a
/// phase with that key missing would render as its raw slug — the one thing a label
/// may never do. `CreationPhase::ALL` is the source of the set, so adding a phase
/// fails this test until both locales carry the key.
///
/// The `wizard-guidance-<slug>` half of this check went with the guidance paragraph
/// itself (manual-testing-findings #21): the wizard no longer explains a step, so
/// demanding copy per phase would demand a string nothing renders.
#[test]
fn every_creation_phase_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for phase in arm_rules::CreationPhase::ALL {
            let key = format!("phase-{phase}");
            assert!(
                ftl.contains(&format!("{key} =")),
                "locale '{lang}' is missing key '{key}'"
            );
        }
    }
}

/// A life-stage XP pool is labelled by which block it is, not by the abilities it
/// may fund (its list is the whole childhood spread, and both childhood blocks
/// share it), so every block needs its own key or the bar would print the slug.
#[test]
fn every_life_stage_xp_pool_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for block in arm_rules::LifeStageBlock::ALL {
            assert!(
                ftl.contains(&format!("xp-pool-{block} =")),
                "locale '{lang}' is missing key 'xp-pool-{block}'"
            );
        }
    }
}

/// The frontend filters each wizard step's findings on the issue's phase, so its
/// `CreationPhase` union has to hold every variant the engine can send. A missing
/// arm is not a type error on the JS side — it is a step that silently shows
/// nothing — so the Rust enum is the source and this test pins the mirror.
#[test]
fn every_creation_phase_is_mirrored_in_the_frontend_union() {
    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    for phase in arm_rules::CreationPhase::ALL {
        assert!(
            types.contains(&format!("'{phase}'")),
            "ui/src/lib/types.ts is missing the CreationPhase member '{phase}'"
        );
    }
}

/// The surfaced-modifier read-out labels every aging modifier through
/// `derived-detail-<slug>`, so a kind no locale names would reach the panel as its
/// own raw slug — the one thing a label may never do. `AgingEffect::ALL` is the
/// source of the set, so adding a kind fails this test until both locales carry it.
#[test]
fn every_aging_effect_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for kind in arm_rules::AgingEffect::ALL {
            assert!(
                ftl.contains(&format!("derived-detail-{kind} =")),
                "locale '{lang}' is missing key 'derived-detail-{kind}'"
            );
        }
    }
}

/// The crisis panel names an illness's severity through `crisis-severity-<slug>`,
/// and the log entry that records one does the same. `CrisisSeverity` is a Rust
/// taxonomy, so a rank no locale names would reach the screen as its own raw slug —
/// the one thing a label may never do. `CrisisSeverity::ALL` is the source of the
/// set, so adding a rank fails this test until both locales carry it.
#[test]
fn every_crisis_severity_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for severity in arm_rules::CrisisSeverity::ALL {
            assert!(
                ftl.contains(&format!("crisis-severity-{severity} =")),
                "locale '{lang}' is missing key 'crisis-severity-{severity}'"
            );
        }
    }
}

/// The frontend's `CrisisSeverity` union types the rank a log entry records. A
/// missing member is caught by nothing the engine runs, so the Rust enum is the
/// source and this test pins the mirror.
#[test]
fn every_crisis_severity_is_mirrored_in_the_frontend_union() {
    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    for severity in arm_rules::CrisisSeverity::ALL {
        assert!(
            types.contains(&format!("'{severity}'")),
            "ui/src/lib/types.ts is missing the CrisisSeverity member '{severity}'"
        );
    }
}

/// The frontend's `AgingEffect` union types every `aging_mod` effect it reads off a
/// loaded item. A missing member is not caught by anything the engine runs, so the
/// Rust enum is the source and this test pins the mirror.
#[test]
fn every_aging_effect_is_mirrored_in_the_frontend_union() {
    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    for kind in arm_rules::AgingEffect::ALL {
        assert!(
            types.contains(&format!("'{kind}'")),
            "ui/src/lib/types.ts is missing the AgingEffect member '{kind}'"
        );
    }
}

/// `ui/src/lib/state.svelte.ts` re-declares `SCHEMA_VERSION` by hand — the frontend
/// stamps it onto every entity it builds from scratch — and TypeScript cannot notice
/// when the Rust constant moves. A stale mirror is silent: the app keeps running and
/// writes saves labelled with a version the engine no longer speaks, so the Rust
/// constant is the source and this test pins the mirror.
#[test]
fn the_frontend_mirrors_the_engine_schema_version() {
    let state = fs::read_to_string(repo_root().join("ui/src/lib/state.svelte.ts")).unwrap();
    let declaration = format!(
        "export const SCHEMA_VERSION = {};",
        arm_rules::SCHEMA_VERSION
    );
    assert!(
        state.contains(&declaration),
        "ui/src/lib/state.svelte.ts must declare `{declaration}`"
    );
}

/// Provenance is deliberately not mirrored to the frontend — `PointItem` and
/// `House` drop their `source` too — so a `SourceRef` is never a drift risk and its
/// key (and the `file`/`lines` inside it) is skipped by [`mirrored_keys`].
const PROVENANCE_KEY: &str = "source";

/// Every field name `value` carries, recursively, as the frontend sees them:
/// nested objects and array elements contribute their keys as well, so mirroring
/// `LifeStageRules` covers the `ChildhoodRules`/`LaterLifeRules` inside it and
/// mirroring a `ChildhoodPackage` covers its `ChildhoodEntry` rows.
///
/// [`PROVENANCE_KEY`] is skipped whole, subtree included.
fn mirrored_keys(value: &serde_json::Value, into: &mut std::collections::BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                if key == PROVENANCE_KEY {
                    continue;
                }
                into.insert(key.clone());
                mirrored_keys(nested, into);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                mirrored_keys(item, into);
            }
        }
        _ => {}
    }
}

/// The life-stage payloads cross the Tauri boundary as JSON, and
/// `ui/src/lib/types.ts` mirrors them **by hand**. TypeScript cannot notice when a
/// Rust field is renamed — the mirror keeps compiling against a key the engine no
/// longer sends, and the life-stage surface silently reads `undefined` — so the
/// serialized shape is the source and this test pins the mirror.
///
/// Every optional field is populated on purpose: `skip_serializing_if` would
/// otherwise drop `slot`, `native`, `childhood_package` and `native_language` from
/// the serialization and hide them from the check.
///
/// **Scope: these six types plus the two `Ruleset`/`Entity` member names.** It is
/// deliberately NOT an assertion over `Ruleset`'s whole key set — `types.ts` omits
/// `age_ability_caps`, `categories_requiring_virtue` and `scholarly_language`, so
/// widening it that far could only fail. Mirroring those is separate work, not a
/// reason to loosen or "tighten" this test.
#[test]
fn every_life_stage_field_is_mirrored_in_the_frontend_types() {
    let plan = arm_rules::LifeStagePlan {
        native_language: Some("German".to_string()),
        childhood_package: Some(Id::new("childhood.athletic")),
        // The three post-Gauntlet choices, populated for the same reason every other
        // optional field here is: `skip_serializing_if` would otherwise drop them
        // from the serialization and hide them from the check.
        gauntlet_age: Some(25),
        post_gauntlet_lab_seasons: 10,
        post_gauntlet_spell_levels: 300,
    };
    let budget = arm_rules::LifeStageBudget {
        childhood_native_xp: 75,
        childhood_spread_xp: 45,
        later_life_years: 20,
        later_life_rate: 15,
        later_life_xp: 300,
        apprenticeship_years: 15,
        apprenticeship_xp: 240,
        // A magus of 60 gauntleted at 25, ten of its lab seasons charged and 300 of
        // the remaining points taken as spell levels. Unlike the plan's choices
        // above, these five are unconditional fields of the derived budget: the
        // engine sends them on every payload, so `types.ts` mirrors them here.
        gauntlet_age: 25,
        post_gauntlet_years: 35,
        post_gauntlet_points: 950,
        post_gauntlet_spell_levels: 300,
        post_gauntlet_xp: 650,
    };
    let rules = arm_rules::LifeStageRules {
        apprenticeship: Some(arm_rules::ApprenticeshipRules {
            // Populated on purpose, like every other optional field here: the
            // Gauntlet-age field's placeholder is what blank means, and it reads the
            // baseline off this key.
            default_gauntlet_age: Some(25),
            minimum_abilities: vec![arm_rules::AbilityRequirement {
                ability: Id::new("ability.parma_magica"),
                exemplar: None,
                min_score: 1,
                parameter: None,
            }],
            recommended_abilities: vec![arm_rules::AbilityRequirement {
                ability: Id::new("ability.dead_language"),
                // Populated on purpose, like every other optional field here.
                exemplar: Some("latin".to_string()),
                min_score: 4,
                parameter: Some("Latin".to_string()),
            }],
            recommended_xp: 90,
            xp: 240,
            years: 15,
        }),
        childhood: arm_rules::ChildhoodRules {
            years: 5,
            native_language_ability: Id::new("ability.living_language"),
            native_language_xp: 75,
            spread_xp: 45,
            spread_abilities: [Id::new("ability.swim")].into_iter().collect(),
        },
        later_life: arm_rules::LaterLifeRules { xp_per_year: 15 },
        // Populated on purpose, like every other optional field here: the block is
        // what a UI showing the post-Gauntlet rate reads it off, so its three field
        // names have to be mirrored.
        post_apprenticeship: Some(arm_rules::PostApprenticeshipRules {
            lab_season_cost: 10,
            max_charged_lab_seasons_per_year: 3,
            points_per_year: 30,
        }),
    };
    // One row of the magus checklist, which reaches the frontend on
    // `EffectiveScores.magus_minimum_abilities` — the payload the Abilities view
    // renders. Its `met`/`requirement` are the two fields nothing else carries, so
    // without this row a rename of either would leave `types.ts` compiling and the
    // checklist silently reading `undefined`.
    let minimum = arm_rules::MagusMinimumAbility {
        ability: Id::new("ability.dead_language"),
        // Populated on purpose, like every other optional field here.
        exemplar: Some("latin".to_string()),
        parameter: Some("Latin".to_string()),
        min_score: 1,
        score: 0,
        met: false,
        requirement: arm_rules::AbilityRequirementKind::Required,
    };
    let package = arm_rules::ChildhoodPackage {
        id: Id::new("childhood.traveling"),
        entries: vec![arm_rules::ChildhoodEntry {
            ability: Id::new("ability.area_lore"),
            score: 1,
            slot: Some("area_a".to_string()),
            native: true,
        }],
        source: Some(arm_rules::SourceRef::new(
            "Ars Magica - Definitive Edition (Core Rules).md",
            arm_rules::LineRange::new(2388, 2388),
        )),
    };

    let mut keys = std::collections::BTreeSet::new();
    for payload in [
        serde_json::to_value(&plan).unwrap(),
        serde_json::to_value(budget).unwrap(),
        serde_json::to_value(&rules).unwrap(),
        serde_json::to_value(&package).unwrap(),
        serde_json::to_value(&minimum).unwrap(),
    ] {
        mirrored_keys(&payload, &mut keys);
    }
    // A floor, so a collector that silently gathered nothing cannot look green.
    assert!(
        keys.len() >= 30,
        "expected the life-stage payloads to carry at least 30 field names, got {keys:?}"
    );

    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    // The two member names the frontend reaches the whole surface through: the
    // ruleset's `life_stages`/`childhoods` catalogues and the entity's own plan.
    for key in keys
        .iter()
        .map(String::as_str)
        .chain(["life_stages", "childhoods"])
    {
        assert!(
            types.contains(&format!("{key}:")) || types.contains(&format!("{key}?:")),
            "ui/src/lib/types.ts declares no '{key}' property"
        );
    }
}

/// The aging payloads cross the Tauri boundary as JSON and `ui/src/lib/types.ts`
/// mirrors them **by hand**, exactly like the life-stage ones — so this is a
/// SIBLING of [`every_life_stage_field_is_mirrored_in_the_frontend_types`], not a
/// widening of it. That test's scope note fixes it at six named types plus two
/// member names, and folding the aging surface into it would dilute its floor
/// rather than add a check.
///
/// Every optional field is populated on purpose: `skip_serializing_if` would
/// otherwise drop `year`, `age`, `die`, `total`, `living_conditions`,
/// `apparent_age_increased`, `crisis` and the four fields a resolved Crisis
/// records (`crisis_die`, `crisis_total`, `crisis_row`, `crisis_severity`) from
/// the serialization and hide them from the check.
///
/// **`AgingLogEntry::points` is deliberately left empty.** It is a
/// `BTreeMap<Characteristic, u8>`, so its serialized *keys* are Characteristic
/// slugs (`sta`) rather than field names, and [`mirrored_keys`] cannot tell the
/// two apart — a populated map would demand a `sta:` property of `types.ts`. The
/// key itself is covered by the chained member names below.
#[test]
fn every_aging_field_is_mirrored_in_the_frontend_types() {
    let readout = arm_app::ruleset_io::AgingReadout {
        first_roll_age: 36,
        begins_after_age: 35,
        // Two years, the first dated: a `None` calendar year would be skipped and
        // hide the `year` key, so one of each proves both shapes serialize.
        schedule: vec![
            arm_app::ruleset_io::AgingScheduleYear {
                age: 36,
                year: Some(1216),
                recorded: true,
            },
            arm_app::ruleset_io::AgingScheduleYear {
                age: 37,
                year: None,
                recorded: false,
            },
        ],
        rolls_owed: 2,
        rolls_recorded: 1,
        age_modifier: 4,
        living_conditions_modifier: -3,
        longevity_modifier: 5,
        // Faerie Blood's -1 (`:3801`): the term the book's three-line formula does
        // not name, which the read-out must still surface for its own arithmetic to
        // add up (guided-creation-review-2026-08 #22).
        trait_modifier: -1,
        // Populated on purpose: `true` is the standing `:16575` predicate, and the
        // field is the one thing that tells it apart from the per-roll cap flag.
        longevity_clamp_active: true,
        // 4 - (-3) - 5 + (-1): the terms above, as the engine sums them.
        fixed_total: 1,
    };
    let entry = arm_rules::AgingLogEntry {
        year: Some(1220),
        age: Some(40),
        effect: "Grey at the temples.".to_string(),
        die: Some(9),
        total: Some(13),
        living_conditions: [Id::new("living_condition.work_in_a_mine")]
            .into_iter()
            .collect(),
        points: BTreeMap::new(),
        apparent_age_increased: true,
        crisis: true,
        crisis_die: Some(7),
        crisis_total: Some(12),
        crisis_row: Some(Id::new("crisis.minor_illness")),
        crisis_severity: Some(arm_rules::CrisisSeverity::Minor),
    };

    let mut keys = std::collections::BTreeSet::new();
    for payload in [
        serde_json::to_value(&readout).unwrap(),
        serde_json::to_value(&entry).unwrap(),
    ] {
        mirrored_keys(&payload, &mut keys);
    }
    // A floor, so a collector that silently gathered nothing cannot look green.
    assert!(
        keys.len() >= 15,
        "expected the aging payloads to carry at least 15 field names, got {keys:?}"
    );

    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    // The two member names the frontend reaches the rest of the surface through:
    // the effective-scores read-out and the character's own standing conditions.
    for key in keys
        .iter()
        .map(String::as_str)
        .chain(["aging", "living_conditions", "points"])
    {
        assert!(
            types.contains(&format!("{key}:")) || types.contains(&format!("{key}?:")),
            "ui/src/lib/types.ts declares no '{key}' property"
        );
    }
}

/// Every `{placeholder}` key the shipped catalogues declare, across the three
/// parameterized kinds (Virtues/Flaws, Abilities, spells). Each names both the
/// placeholder in an item's localized name and its `param-label-<key>` label — the
/// slot label the Markdown export prints wherever a parameterized name has no chosen
/// value (`export::Doc::parameterized_name`). The family is catalogue *data*, so it
/// is recomputed from the shipped rules rather than listed in `LABEL_KEYS`; the
/// frontend composes the same set for the label map it sends
/// (`composedExportLabelKeys` in `ui/src/lib/state.svelte.ts`).
fn shipped_parameter_keys() -> Vec<String> {
    let read = |relative: &str| -> serde_json::Value {
        let json = fs::read_to_string(repo_root().join(relative)).unwrap();
        serde_json::from_str(&json).unwrap()
    };
    let mut keys: Vec<String> = Vec::new();
    let collect = |items: Option<&Vec<serde_json::Value>>, keys: &mut Vec<String>| {
        for item in items.into_iter().flatten() {
            for parameter in item["parameters"].as_array().into_iter().flatten() {
                keys.push(parameter["key"].as_str().unwrap().to_string());
            }
        }
    };
    let items = read("rules/core/virtues_flaws.json");
    collect(items.as_array(), &mut keys);
    let spells = read("rules/core/spells.json");
    collect(spells["spells"].as_array(), &mut keys);
    let abilities = read("rules/core/abilities.json");
    for ability in abilities["abilities"].as_array().into_iter().flatten() {
        if let Some(key) = ability["parameter"].as_str() {
            keys.push(key.to_string());
        }
    }
    keys.sort();
    keys.dedup();
    // One known key per scanned catalogue, so a scan that silently collected nothing
    // cannot leave the coverage check looking green.
    for known in ["ability", "form", "language"] {
        assert!(
            keys.contains(&known.to_string()),
            "expected the shipped parameter key '{known}', got {keys:?}"
        );
    }
    keys
}

/// The slot label for a parameterized name is composed from the parameter key, so a
/// key the locales do not translate would reach the exported sheet (and the picker)
/// as its own Fluent key. Recomputed from the catalogue, so a newly parameterized
/// item fails here until both locales name its slot.
#[test]
fn every_shipped_parameter_key_has_a_param_label_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for key in shipped_parameter_keys() {
            assert!(
                ftl.contains(&format!("param-label-{key} =")),
                "locale '{lang}' is missing key 'param-label-{key}'"
            );
        }
    }
}

/// The CC-BY-SA rules text is bundled into every installer, so its attribution
/// has to travel with it. `rules/NOTICE.md` is that attribution, and the only
/// legal text that reaches a Linux user's disk — the deb/rpm/AppImage bundlers
/// never read `bundle.licenseFile`. Without this test the sole check on it is a
/// real bundle build, which only runs after a release tag is already pushed.
#[test]
fn attribution_notice_ships_with_the_bundled_rules_data() {
    let notice = repo_root().join("rules/NOTICE.md");
    assert!(
        notice.is_file(),
        "rules/NOTICE.md is missing; the bundled rules data would ship without \
         its CC-BY-SA attribution"
    );

    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("crates/arm-app/tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let resources = &config["bundle"]["resources"];
    assert!(
        resources
            .as_object()
            .expect("bundle.resources should be a map of source -> destination")
            .contains_key("../../rules/NOTICE.md"),
        "tauri.conf.json bundle.resources must map ../../rules/NOTICE.md so the \
         notice is installed alongside rules/core and rules/i18n"
    );
}

/// K1/VA4: `app.security.csp` must never regress to `null` (which disables
/// Tauri's CSP injection into the webview entirely). No injection sink exists
/// today (no `{@html}`/`innerHTML` anywhere in the frontend), but the CSP is
/// the defence-in-depth backstop for the day one is introduced by accident —
/// and in a Tauri webview that backstop matters more than in an ordinary
/// browser tab, because script running there sits behind the same origin the
/// `invoke()` IPC bridge trusts.
#[test]
fn csp_is_set_and_restrictive() {
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("crates/arm-app/tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("app.security.csp must be a restrictive policy string, not null");
    assert!(
        csp.contains("default-src 'self'"),
        "csp must default-deny to the app's own origin, got: {csp}"
    );
    assert!(
        !csp.contains("unsafe-eval"),
        "csp must not permit unsafe-eval, got: {csp}"
    );
}

/// Each rules directory states its own licensing, because the two are not the
/// same: `rules/core/` is MIT throughout, while `rules/i18n/` is MIT in form and
/// CC-BY-SA 4.0 in the rules text it carries. Both ride into the installers with
/// the directories they describe.
#[test]
fn each_rules_directory_carries_its_own_license_file() {
    for dir in ["core", "i18n"] {
        let license = rules_dir().join(dir).join("LICENSE");
        assert!(
            license.is_file(),
            "rules/{dir}/LICENSE is missing; the bundled data would not state \
             which license covers it"
        );
    }
}

/// GA1: a confirmed discard latches `CloseGuardState::confirmed = true` (see
/// `main.rs`'s `guard_blocks_quit`) so a re-issued close/quit belonging to the
/// SAME confirmed action passes through without a second dialog. That latch
/// must not survive into the NEXT dirty-state report: `update_close_guard`
/// fires on every dirty-state transition the frontend reports (whenever the
/// entity's snapshot-compared `dirty` flag changes — see
/// `AppStore.closeGuardPayload` / the `$effect` in `App.svelte`), and a stale
/// `confirmed == true` there would let a LATER close/quit skip the
/// discard-confirmation dialog for edits the user never actually confirmed
/// discarding. Dormant today (no window-reactivation path exists in
/// `main.rs` yet — closing the window force-closes it via `destroy()`), but a
/// real one-way latch with no reset otherwise, on exactly the surface
/// CLAUDE.md's "Unsaved-changes guard" section calls load-bearing on macOS,
/// where `RunEvent::ExitRequested` (Cmd+Q) is independent of the window-close
/// path and does not itself clear anything.
#[test]
fn a_fresh_dirty_state_report_clears_the_confirmed_discard_latch() {
    use arm_app::commands::{CloseGuardLabels, CloseGuardState};

    let mut guard = CloseGuardState {
        dirty: true,
        // As main.rs's dialog callback leaves it once the user confirms
        // discarding: `guard.confirmed = true;` right before `on_discard` runs.
        confirmed: true,
        ..CloseGuardState::default()
    };

    // The frontend reports a fresh dirty state — e.g. the character was edited
    // again after the window that showed the dialog was destroyed but the
    // process (macOS) lived on.
    guard.report_dirty_state(true, CloseGuardLabels::default());

    assert!(
        !guard.confirmed,
        "a fresh dirty-state report must clear the one-shot discard latch, or a \
         later close/quit could silently skip the confirmation dialog for edits \
         the user never confirmed discarding"
    );
}

/// K5/VA5: `ARM_E2E_FILE` must be inert unless the crate is built with the
/// `e2e-testing` Cargo feature. Without that gate this override compiled
/// unconditionally into the exact release binary end users install, letting
/// anything that can set an env var before launch silently redirect
/// Save/Open away from the native dialog with no user-facing confirmation.
/// This test runs under the plain `cargo test -p arm-app` gate (no features
/// enabled), which is exactly the build users receive.
#[cfg(not(feature = "e2e-testing"))]
#[test]
fn e2e_file_override_is_compiled_out_of_the_default_build() {
    // SAFETY: no other test in this binary reads or writes ARM_E2E_FILE, so
    // there is no cross-test race on this process-global.
    unsafe { std::env::set_var("ARM_E2E_FILE", "/nonexistent/should-not-be-honored") };
    let result = arm_app::commands::e2e_file_override();
    unsafe { std::env::remove_var("ARM_E2E_FILE") };
    assert!(
        result.is_none(),
        "ARM_E2E_FILE must not be honored unless the `e2e-testing` feature is \
         enabled at compile time; this test builds without it, matching the \
         binary shipped to users"
    );
}

/// Sibling of the above for the Markdown-export seam.
#[cfg(not(feature = "e2e-testing"))]
#[test]
fn e2e_export_file_override_is_compiled_out_of_the_default_build() {
    // SAFETY: no other test in this binary reads or writes ARM_E2E_EXPORT_FILE.
    unsafe {
        std::env::set_var(
            "ARM_E2E_EXPORT_FILE",
            "/nonexistent/should-not-be-honored.md",
        )
    };
    let result = arm_app::commands::e2e_export_file_override();
    unsafe { std::env::remove_var("ARM_E2E_EXPORT_FILE") };
    assert!(
        result.is_none(),
        "ARM_E2E_EXPORT_FILE must not be honored unless the `e2e-testing` \
         feature is enabled at compile time"
    );
}

/// Mirror of the two tests above, proving the gate actually opens rather than
/// just staying permanently shut: compiled WITH the `e2e-testing` feature
/// (exactly what `ui/e2e/wdio.conf.js` / `wdio.portable.conf.js` pass to
/// `cargo tauri build --no-bundle --features e2e-testing`), the overrides
/// must still work, or the e2e suite's 35 specs plus the portable-layout run
/// lose their save/load/export seam entirely.
#[cfg(feature = "e2e-testing")]
#[test]
fn e2e_file_override_is_honored_when_the_feature_is_enabled() {
    unsafe { std::env::set_var("ARM_E2E_FILE", "/nonexistent/honored-path.json") };
    let result = arm_app::commands::e2e_file_override();
    unsafe { std::env::remove_var("ARM_E2E_FILE") };
    assert_eq!(
        result,
        Some(PathBuf::from("/nonexistent/honored-path.json"))
    );
}

#[cfg(feature = "e2e-testing")]
#[test]
fn e2e_export_file_override_is_honored_when_the_feature_is_enabled() {
    unsafe { std::env::set_var("ARM_E2E_EXPORT_FILE", "/nonexistent/honored-path.md") };
    let result = arm_app::commands::e2e_export_file_override();
    unsafe { std::env::remove_var("ARM_E2E_EXPORT_FILE") };
    assert_eq!(result, Some(PathBuf::from("/nonexistent/honored-path.md")));
}

#[test]
fn every_validation_code_has_a_fluent_key_in_each_locale() {
    let mut codes = validation_codes();
    codes.extend(dynamic_flaw_cap_codes());
    codes.extend(dynamic_virtue_cap_codes());
    codes.sort();
    codes.dedup();
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for code in &codes {
            assert!(
                ftl.contains(&format!("issue-{code} =")),
                "locale '{lang}' is missing key 'issue-{code}'"
            );
        }
    }
}

/// Every checked-in `examples/*.json` fixture must actually load and validate
/// through the real production path — the same loader and validator the app
/// uses, against the same shipped `rules/` the app ships. Scans the directory
/// rather than naming files, so adding a future example needs no test change
/// (full-audit finding V20).
#[test]
fn every_example_save_parses_and_validates() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let examples_dir = repo_root().join("examples");
    let mut checked = 0;
    let mut entries: Vec<PathBuf> = fs::read_dir(&examples_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    entries.sort();

    for path in entries {
        let entity = load_entity_from_path(&path)
            .unwrap_or_else(|e| panic!("{} failed to load: {e}", path.display()));
        let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
        assert!(
            result.is_valid(),
            "{} failed to validate: {:?}",
            path.display(),
            result.issues
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "examples/ must contain at least one *.json fixture"
    );
}

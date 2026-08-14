//! Integration tests for the webview-free command logic. These exercise the
//! real loader, validator, and save/load against the repository's shipped rules
//! data — no Tauri runtime or webview required.

use std::fs;
use std::path::PathBuf;

use arm_app::error::AppError;
use arm_app::ruleset_io::{
    ChildhoodApplication, RULESET_ID, RULESET_VERSION, apply_childhood_package_loaded,
    effective_scores_loaded, ensure_extension, export_markdown_to_path, load_entity_from_path,
    load_ruleset_from_dir, pick_rules_dir, save_entity_to_path, validate_loaded,
};
use arm_rules::{ArtScore, Entity, Id, Ruleset, RulesetSources, Selection, ValidationMode};
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
    assert_eq!(entity.schema_version, 14);
    assert!(!entity.characteristics.is_empty());
    assert!(!entity.ability_scores.is_empty());
    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(result.is_valid(), "unexpected issues: {:?}", result.issues);
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
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.x"}},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.y"}},
          {"id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"]}
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
    let tiny_type = r#"[{
        "id": "tiny",
        "budget": { "virtue_points": 1, "flaw_points": 10 },
        "permitted_categories": ["general"],
        "creation_phases": ["virtues_flaws"]
    }]"#;
    // The catalogue's ability_score_grant effects reference abilities, so the
    // ruleset must carry the ability registry for referential integrity to pass.
    let ruleset = Ruleset::from_sources(RulesetSources {
        id: RULESET_ID,
        version: RULESET_VERSION,
        point_items: &items,
        type_profiles: tiny_type,
        abilities: Some(&abilities),
        arts: Some(&arts),
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
/// attunements arrive under `Entity.talisman`, the version is bumped to 14, and
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
        migrated.schema_version, 14,
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
    assert!(written.contains("\"schema_version\": 14"), "got: {written}");
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
        written.contains("\"schema_version\": 14"),
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
    assert_eq!(reloaded.schema_version, 14);
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

    // Infamous surfaces a Local reputation grant for the UI add-control.
    companion
        .selections
        .push(Selection::new(Id::new("flaw.infamous")));
    let grants = effective_scores_loaded(&companion, &ruleset).reputation_grants;
    assert!(
        grants
            .iter()
            .any(|g| g.kind == arm_rules::ReputationType::Local && g.score == 4),
        "Infamous grants a level-4 Local reputation"
    );

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
    assert_eq!(
        effective_scores_loaded(&magus, &ruleset).xp_general_pool,
        240
    );

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
/// phase with no key would render as its raw slug — the one thing a label may never
/// do. `CreationPhase::ALL` is the source of the set, so adding a phase fails this
/// test until both locales carry it.
#[test]
fn every_creation_phase_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for phase in arm_rules::CreationPhase::ALL {
            assert!(
                ftl.contains(&format!("phase-{phase} =")),
                "locale '{lang}' is missing key 'phase-{phase}'"
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
        // The post-Gauntlet choices are deliberately left unset, for the reason
        // `post_apprenticeship` is below: no view reads them yet, so populating them
        // would demand a `types.ts` mirror for a payload nothing consumes. They join
        // the check in the change that first sends them across the boundary.
        ..arm_rules::LifeStagePlan::default()
    };
    let budget = arm_rules::LifeStageBudget {
        childhood_native_xp: 75,
        childhood_spread_xp: 45,
        later_life_years: 20,
        later_life_rate: 15,
        later_life_xp: 300,
        apprenticeship_years: 15,
        apprenticeship_xp: 240,
    };
    let rules = arm_rules::LifeStageRules {
        apprenticeship: Some(arm_rules::ApprenticeshipRules {
            minimum_abilities: vec![arm_rules::AbilityRequirement {
                ability: Id::new("ability.parma_magica"),
                min_score: 1,
                parameter: None,
            }],
            recommended_abilities: vec![arm_rules::AbilityRequirement {
                ability: Id::new("ability.dead_language"),
                min_score: 4,
                // Populated on purpose, like every other optional field here.
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
        // The one optional field deliberately left unset: nothing in the frontend
        // reads the post-apprenticeship block yet, so populating it here would
        // demand a `types.ts` mirror for a payload no view consumes. It joins the
        // check in the same change that first sends it across the boundary.
        post_apprenticeship: None,
    };
    // One row of the magus checklist, which reaches the frontend on
    // `EffectiveScores.magus_minimum_abilities` — the payload the Abilities view
    // renders. Its `met`/`requirement` are the two fields nothing else carries, so
    // without this row a rename of either would leave `types.ts` compiling and the
    // checklist silently reading `undefined`.
    let minimum = arm_rules::MagusMinimumAbility {
        ability: Id::new("ability.dead_language"),
        // Populated on purpose, like every other optional field here.
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
    let mut collect = |items: Option<&Vec<serde_json::Value>>, keys: &mut Vec<String>| {
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

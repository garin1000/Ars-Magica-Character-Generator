//! L3 (try-out findings 13, 14): the two catalogue load notices are a one-time
//! MIGRATION report, not an every-open warning.
//!
//! `fold_catalogue_matching` turns typed text that names a catalogue entry
//! ("Latin", " Latein ") into the catalogue id on every load. That silent
//! normalisation stays. What changes is the reporting:
//!
//! - `migrated_catalogued_parameters` ("… were recognized from what you typed")
//!   and `unresolved_catalogued_parameters` ("… a value the rules catalogue does
//!   not recognize") are filled only when the file's RAW `schema_version` is
//!   below 18, the CV4 step (17 -> 18) that introduced catalogued values.
//! - A save at 18 or later was written by a build that offered the catalogue
//!   picker, so its free text ("Native language", "Gaelic") is a deliberate
//!   "Other…" choice. It never warns (Norbert, finding 14).
//!
//! "Raw" means the file's own claim, read before any other fold stamps the
//! current version over it — the same value L1b's move is gated on.
//!
//! Red-checkpoint protocol, phase 1: the notices are filled on every load today,
//! so the "no notice" tests fail on their own assertions. The pre-18 tests are
//! green pins: the notice must survive the gating.

use std::collections::BTreeMap;

use arm_rules::migration::{LoadedEntity, SCHEMA_VERSION, load_entity_migrating};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{AbilityParameterValue, Id};
use arm_rules::{DEFAULT_SAGA_YEAR, validate};

/// The CV4 schema version: the first save format with catalogued values. A
/// literal, not `SCHEMA_VERSION`, because it pins what a save written at 18
/// means, which must not drift with later bumps.
const CV4_VERSION: u32 = 18;
/// The last schema written before catalogued values existed.
const PRE_CV4_VERSION: u32 = 17;

const DEAD: &str = "ability.dead_language";
const LIVING: &str = "ability.living_language";

/// The shipped core ruleset, loaded exactly as `book_templates.rs` does.
fn full_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

/// The shipped catalogue names, both locales.
fn catalogue_names(ruleset: &Ruleset) -> BTreeMap<Id, Vec<String>> {
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

/// A minimal companion save at `schema_version` with one language row.
/// `ability_funding` and `saga_year` are present, so no other fold stamps the
/// version: the claimed version is the one the load sees.
fn save(schema_version: u32, ability: &str, parameter_json: &str) -> String {
    format!(
        r#"{{
          "schema_version": {schema_version},
          "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "ability_scores": [
            {{ "ability": "{ability}", "score": 1, "parameter": {parameter_json} }}
          ]
        }}"#
    )
}

fn load(json: &str) -> LoadedEntity {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    load_entity_migrating(json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .unwrap_or_else(|e| panic!("fixture save must load: {e}"))
}

fn catalogued(id: &str) -> Option<AbilityParameterValue> {
    Some(AbilityParameterValue::Catalogued { id: Id::new(id) })
}

// --- Pre-18 saves: the one-time migration notice ---------------------------------

/// Green pin. A pre-18 save (the book templates are 17) with typed "Latin" is
/// normalised AND reported as recognized — once, as part of the CV4 migration.
/// Version 0 is the untrusted-input floor and counts as pre-18 too.
#[test]
fn a_pre_18_save_reports_typed_latin_as_recognized() {
    for (version, parameter) in [
        (PRE_CV4_VERSION, r#""Latin""#),
        (PRE_CV4_VERSION, r#"{ "text": " Latein " }"#),
        (0, r#"{ "text": "latin" }"#),
    ] {
        let loaded = load(&save(version, DEAD, parameter));
        assert_eq!(
            loaded.entity.ability_scores[0].parameter,
            catalogued("language.latin"),
            "v{version} {parameter}: typed Latin is normalised to the catalogue id"
        );
        assert_eq!(
            loaded.migrated_catalogued_parameters.len(),
            1,
            "v{version} {parameter}: the CV4 migration reports the recognized value once, got: {:?}",
            loaded.migrated_catalogued_parameters
        );
        assert_eq!(
            loaded.migrated_catalogued_parameters[0].0,
            Id::new(DEAD),
            "v{version} {parameter}: reported under the Ability it sits on"
        );
        assert_eq!(
            loaded.migrated_catalogued_parameters[0].2,
            Id::new("language.latin"),
            "v{version} {parameter}: reported with the resolved catalogue id"
        );
    }
}

/// Green pin. A pre-18 save with a language the catalogue does not hold is
/// kept as typed and reported as unrecognized, once.
#[test]
fn a_pre_18_save_reports_unrecognized_gaelic() {
    let loaded = load(&save(PRE_CV4_VERSION, LIVING, r#""Gaelic""#));
    assert_eq!(
        loaded.entity.ability_scores[0].parameter,
        Some(AbilityParameterValue::text("Gaelic")),
        "an unrecognized value is kept exactly as typed"
    );
    assert_eq!(
        loaded.unresolved_catalogued_parameters,
        vec![(Id::new(LIVING), "Gaelic".to_string())],
        "the CV4 migration reports the unrecognized value once"
    );
}

/// Green pin. The gate reads the RAW version. A v16 save has no `saga_year`
/// key, so the saga-year fold stamps the current version long before the
/// catalogue fold runs; the notices must still fire, because the FILE was
/// written before catalogued values existed.
#[test]
fn the_gate_reads_the_raw_version_not_the_one_a_fold_stamped() {
    let json = r#"{
      "schema_version": 16,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "companion",
      "ability_funding": "pool",
      "ability_scores": [
        { "ability": "ability.dead_language", "score": 1, "parameter": "Latin" },
        { "ability": "ability.living_language", "score": 5, "parameter": "Gaelic" }
      ]
    }"#;
    let loaded = load(json);
    assert_eq!(
        loaded.entity.schema_version, SCHEMA_VERSION,
        "premise: the saga-year fold stamped the current version"
    );
    assert_eq!(
        loaded.migrated_catalogued_parameters.len(),
        1,
        "typed Latin in a v16 file is reported, got: {:?}",
        loaded.migrated_catalogued_parameters
    );
    assert_eq!(
        loaded.unresolved_catalogued_parameters,
        vec![(Id::new(LIVING), "Gaelic".to_string())],
        "Gaelic in a v16 file is reported"
    );
}

// --- 18+ saves: silent normalisation, no notice ---------------------------------

/// RED today. A save at 18 or later with typed "Latin" is still normalised to
/// the catalogue id on load, but silently: there was no migration to report.
#[test]
fn a_current_save_normalises_typed_latin_without_a_notice() {
    for version in [CV4_VERSION, SCHEMA_VERSION] {
        let loaded = load(&save(version, DEAD, r#"{ "text": "Latin" }"#));
        assert_eq!(
            loaded.entity.ability_scores[0].parameter,
            catalogued("language.latin"),
            "v{version}: typed Latin is still normalised on every load"
        );
        assert!(
            loaded.migrated_catalogued_parameters.is_empty(),
            "v{version}: normalising typed text is not a migration and must not be \
             announced, got: {:?}",
            loaded.migrated_catalogued_parameters
        );
    }
}

/// RED today (finding 14). Free text typed via "Other…" in a save at 18 or
/// later is a deliberate choice: kept as typed, never warned about.
#[test]
fn current_schema_free_text_never_warns() {
    for version in [CV4_VERSION, SCHEMA_VERSION] {
        for typed in ["Native language", "Gaelic", "English"] {
            let loaded = load(&save(
                version,
                LIVING,
                &format!(r#"{{ "text": "{typed}" }}"#),
            ));
            assert_eq!(
                loaded.entity.ability_scores[0].parameter,
                Some(AbilityParameterValue::text(typed)),
                "v{version}: {typed:?} is kept exactly as typed"
            );
            assert!(
                loaded.unresolved_catalogued_parameters.is_empty(),
                "v{version}: {typed:?} is legitimate free text and must not warn, got: {:?}",
                loaded.unresolved_catalogued_parameters
            );
        }
    }
}

/// RED today (finding 14's own scenario). The book's Hunter template (schema
/// 17, "Native language" typed under Living Language) warns once on open. Once
/// saved — which stamps the current version, as `save_entity_to_path` does —
/// it reopens without the notice, its free text intact.
#[test]
fn a_template_saved_at_the_current_schema_reopens_without_a_notice() {
    let first = load(include_str!("fixtures/book_templates/grog_hunter.json"));
    assert!(
        first
            .unresolved_catalogued_parameters
            .contains(&(Id::new(LIVING), "Native language".to_string())),
        "premise: the v17 template's first open reports \"Native language\" once, got: {:?}",
        first.unresolved_catalogued_parameters
    );

    let mut saved = first.entity.clone();
    saved.schema_version = SCHEMA_VERSION;
    saved.normalize();
    let json = serde_json::to_string_pretty(&saved).expect("entity serializes");

    let reopened = load(&json);
    assert!(
        reopened.unresolved_catalogued_parameters.is_empty(),
        "a save written at the current schema must reopen without the notice, got: {:?}",
        reopened.unresolved_catalogued_parameters
    );
    assert!(
        reopened.migrated_catalogued_parameters.is_empty(),
        "nothing was migrated on the reopen, got: {:?}",
        reopened.migrated_catalogued_parameters
    );
    assert!(
        reopened
            .entity
            .ability_scores
            .iter()
            .any(|s| s.ability == Id::new(LIVING)
                && s.parameter == Some(AbilityParameterValue::text("Native language"))),
        "the free text survives the round trip, got: {:?}",
        reopened.entity.ability_scores
    );
}

/// Green pin (acceptance). The book templates with typed "Latin" (schema 17)
/// still load, normalise it to the catalogue id, and raise no validation error.
#[test]
fn the_magus_templates_typed_latin_still_loads_and_validates() {
    let ruleset = full_ruleset();
    let loaded = load(include_str!("fixtures/book_templates/magus_bjornaer.json"));
    assert!(
        loaded
            .entity
            .ability_scores
            .iter()
            .any(|s| s.ability == Id::new(DEAD) && s.parameter == catalogued("language.latin")),
        "premise: the template's typed Latin is normalised, got: {:?}",
        loaded.entity.ability_scores
    );
    let errors: Vec<String> = validate(&loaded.entity, &ruleset)
        .errors()
        .map(|issue| issue.code.clone())
        .collect();
    assert!(errors.is_empty(), "the template validates: {errors:?}");
}

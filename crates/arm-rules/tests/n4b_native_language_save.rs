//! N4b (try-out 2026-10-04, finding N4): the native language is a list value in
//! the save, not free text.
//!
//! `LifeStagePlan::native_language` holds `{"id": …}` for a language picked from
//! (or spelling a name of) the native-language Ability's catalogue and
//! `{"text": …}` for any other language, the same JSON shape an Ability row's
//! `parameter` has — but without the `Linked` shape, which a plan cannot hold.
//! Schema 22 → 23.
//!
//! The migration dispatches on the SHAPE, never on the claimed version: a save is
//! the trust boundary, so a JSON string under `life_stages.native_language` is
//! wrapped whatever `schema_version` says (the `wrap_legacy_ability_parameters`
//! rule), and the text → catalogue-value fold runs on every load (the
//! `fold_catalogue_matching` rule). Every test here works through the save's own
//! bytes, so the file compiles before the type exists.
//!
//! Red-checkpoint protocol, phase 1: every test below fails on its own assertion,
//! except `a_native_language_in_a_shape_no_writer_produces_is_refused`, a guard
//! that holds today too (an object is not a string) and must keep holding.

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{Entity, Id};
use arm_rules::{DEFAULT_SAGA_YEAR, LoadedEntity, SCHEMA_VERSION, load_entity_migrating};
use serde_json::{Value, json};

/// The shipped core ruleset, life stages and catalogues included.
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
        childhoods: Some(include_str!("../../../rules/core/childhoods.json")),
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

/// Both locales' catalogue names, as the app reads them.
fn catalogue_names(ruleset: &Ruleset) -> BTreeMap<Id, Vec<String>> {
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

/// A life-stage companion's save claiming `version`, its plan naming `native`
/// verbatim — whatever JSON shape that is.
fn save_claiming(version: u32, native: Value) -> String {
    json!({
        "schema_version": version,
        "ruleset": { "id": "arm5-core", "version": "2024.1" },
        "entity_kind": "character",
        "type_id": "companion",
        "ability_funding": "life_stages",
        "saga_year": 1220,
        "age": 25,
        "life_stages": { "native_language": native }
    })
    .to_string()
}

fn load(json: &str) -> Result<LoadedEntity, serde_json::Error> {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    load_entity_migrating(json, DEFAULT_SAGA_YEAR, &ruleset, &names)
}

/// The plan's native language exactly as the entity serializes it.
fn native_of(entity: &Entity) -> Value {
    serde_json::to_value(entity).expect("an entity serializes")["life_stages"]["native_language"]
        .clone()
}

/// Saves as `ruleset_io.rs::save_entity_to_path` does: current version stamped,
/// normalized, pretty-printed.
fn save(mut entity: Entity) -> String {
    entity.schema_version = SCHEMA_VERSION;
    entity.normalize();
    serde_json::to_string_pretty(&entity).expect("an entity serializes")
}

/// The bump is a genuine shape move: a schema-22 reader cannot parse the object.
#[test]
fn the_schema_version_is_23() {
    assert_eq!(SCHEMA_VERSION, 23);
}

/// A schema-22 save typed its language. Text spelling a catalogue name, in any
/// case and either locale, loads as that catalogue value, and the shape move
/// stamps the current version.
#[test]
fn a_v22_string_naming_a_catalogue_value_loads_as_that_value() {
    for typed in ["arabic", "Arabic", " Arabisch "] {
        let loaded = load(&save_claiming(22, json!(typed))).expect("a v22 save loads");
        assert_eq!(
            native_of(&loaded.entity),
            json!({ "id": "language.arabic" }),
            "{typed:?}"
        );
        assert_eq!(loaded.entity.schema_version, 23, "{typed:?}");
    }
}

/// Text naming no catalogue value stays the player's own text.
#[test]
fn a_v22_string_naming_nothing_loads_as_text() {
    let loaded = load(&save_claiming(22, json!("Gaelic"))).expect("a v22 save loads");
    assert_eq!(native_of(&loaded.entity), json!({ "text": "Gaelic" }));
}

/// Shape, not version: a hand-edited save claiming the current schema while
/// still holding a bare string is wrapped all the same.
#[test]
fn a_string_in_a_save_claiming_23_is_wrapped_all_the_same() {
    let loaded = load(&save_claiming(23, json!("Arabic")))
        .expect("a bare string loads at any claimed version");
    assert_eq!(
        native_of(&loaded.entity),
        json!({ "id": "language.arabic" })
    );

    let loaded = load(&save_claiming(23, json!("Gaelic")))
        .expect("a bare string loads at any claimed version");
    assert_eq!(native_of(&loaded.entity), json!({ "text": "Gaelic" }));
}

/// The fold is value-driven, like an Ability row's: `{"text": "Arabic"}` in a
/// current save becomes the catalogue value on every load, and a catalogue
/// value or text naming nothing is left exactly as written.
#[test]
fn the_text_fold_runs_on_every_load() {
    let cases = [
        (
            json!({ "text": " ARABIC " }),
            json!({ "id": "language.arabic" }),
        ),
        (
            json!({ "id": "language.arabic" }),
            json!({ "id": "language.arabic" }),
        ),
        (json!({ "text": "Gaelic" }), json!({ "text": "Gaelic" })),
    ];
    for (stored, expected) in cases {
        let loaded = load(&save_claiming(23, stored.clone())).expect("a v23 save loads");
        assert_eq!(native_of(&loaded.entity), expected, "{stored}");
    }
}

/// `Linked` follows another selection's parameter — nothing a plan can hold — and
/// a value mixing two variants' keys is no shape any writer produces. Both fail
/// the load with an error rather than a panic or a quiet misread, at either
/// claimed version; so does a value that is not text at all.
#[test]
fn a_native_language_in_a_shape_no_writer_produces_is_refused() {
    let refused = [
        json!({ "item": "virtue.craft_guild_training", "param": "guild" }),
        json!({ "id": "language.arabic", "text": "Arabic" }),
        json!({ "id": "language.arabic", "item": "virtue.x", "param": "p" }),
        json!(42),
        json!(["Arabic"]),
    ];
    for version in [22, 23] {
        for value in &refused {
            assert!(
                load(&save_claiming(version, value.clone())).is_err(),
                "v{version} {value} must be refused"
            );
        }
    }
}

/// Migrated once, a save is canonical: saving it, reopening and saving again
/// writes the same bytes, and the value stays the catalogue value.
#[test]
fn a_migrated_save_round_trips_byte_stably() {
    for (typed, expected) in [
        ("arabic", json!({ "id": "language.arabic" })),
        ("Gaelic", json!({ "text": "Gaelic" })),
    ] {
        let first = load(&save_claiming(22, json!(typed))).expect("a v22 save loads");
        let saved = save(first.entity);
        let reopened = load(&saved).expect("the saved bytes load");
        assert_eq!(native_of(&reopened.entity), expected, "{typed:?}");
        assert!(
            saved.contains("\"schema_version\": 23"),
            "{typed:?}: {saved}"
        );
        assert_eq!(save(reopened.entity), saved, "{typed:?}");
    }
}

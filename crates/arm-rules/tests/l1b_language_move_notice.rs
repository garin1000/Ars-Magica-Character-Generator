//! L1b (try-out finding 6, decisions C5): the one-time notice for the 21 -> 22
//! language move.
//!
//! A load that moved a language instance between Dead Language and Living
//! Language reports each move on [`LoadedEntity::moved_ability_parameters`], the
//! way `migrated_catalogued_parameters` reports a recognized value: the Abilities
//! and the catalogue value travel as ids, never as a sentence, and the frontend
//! composes the localized notice. A collision also carries the score of the
//! instance that already sat under the target, so the notice can name both
//! scores. The notice is version-gated: a save at 22 or later never produces one.
//!
//! Red-checkpoint protocol, phase 1: `MovedAbilityParameter` and the field do not
//! exist yet, so this file fails to COMPILE. That is the accepted first red; the
//! assertions must still be seen failing once the bare type exists.
//!
//! [`LoadedEntity::moved_ability_parameters`]: arm_rules::migration::LoadedEntity

use std::collections::BTreeMap;

use arm_rules::DEFAULT_SAGA_YEAR;
use arm_rules::migration::{LoadedEntity, MovedAbilityParameter, load_entity_migrating};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::Id;

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

fn load(schema_version: u32, ability_scores: &str) -> LoadedEntity {
    let ruleset = full_ruleset();
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    let names: BTreeMap<Id, Vec<String>> =
        arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
            .expect("catalogue names load");
    let json = format!(
        r#"{{
          "schema_version": {schema_version},
          "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "ability_scores": [ {ability_scores} ]
        }}"#
    );
    load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .unwrap_or_else(|e| panic!("fixture save must load: {e}"))
}

fn moved(
    from: &str,
    to: &str,
    value: &str,
    score: u8,
    existing_score: Option<u8>,
) -> MovedAbilityParameter {
    MovedAbilityParameter {
        from: Id::new(from),
        to: Id::new(to),
        value: Id::new(value),
        score,
        existing_score,
    }
}

/// A plain move is reported with both Abilities, the catalogue value and the
/// moved instance's score; no collision, so no existing score.
#[test]
fn a_move_is_reported_naming_both_abilities_and_the_value() {
    let loaded = load(
        21,
        r#"{ "ability": "ability.dead_language", "score": 3, "parameter": { "id": "language.arabic" } }"#,
    );
    assert_eq!(
        loaded.moved_ability_parameters,
        vec![moved(
            "ability.dead_language",
            "ability.living_language",
            "language.arabic",
            3,
            None
        )]
    );
}

/// Typed text is reported by the catalogue value it was recognized as, and the
/// mirror direction (Living -> Dead) is reported the same way.
#[test]
fn typed_and_mirrored_moves_are_reported_by_their_catalogue_value() {
    let loaded = load(
        21,
        r#"{ "ability": "ability.dead_language", "score": 2, "parameter": { "text": "Griechisch" } },
           { "ability": "ability.living_language", "score": 4, "parameter": { "id": "language.latin" } }"#,
    );
    let mut reported = loaded.moved_ability_parameters.clone();
    reported.sort_by(|a, b| a.value.cmp(&b.value));
    assert_eq!(
        reported,
        vec![
            moved(
                "ability.dead_language",
                "ability.living_language",
                "language.greek",
                2,
                None
            ),
            moved(
                "ability.living_language",
                "ability.dead_language",
                "language.latin",
                4,
                None
            ),
        ]
    );
}

/// A collision names both scores: the moved instance's and the one that already
/// sat under the target. Which one was kept follows from the two (the higher;
/// on a tie, the one with more banked XP), so the notice can say so.
#[test]
fn a_collision_is_reported_with_both_scores() {
    let loaded = load(
        21,
        r#"{ "ability": "ability.dead_language", "score": 1, "parameter": { "id": "language.arabic" } },
           { "ability": "ability.living_language", "score": 5, "parameter": { "id": "language.arabic" } }"#,
    );
    assert_eq!(
        loaded.moved_ability_parameters,
        vec![moved(
            "ability.dead_language",
            "ability.living_language",
            "language.arabic",
            1,
            Some(5)
        )]
    );
}

/// The notice is one-time: a save written at 22 never produces it, even with the
/// same content (that save is not moved at all), and a pre-22 save with nothing
/// to move produces none either.
#[test]
fn no_notice_for_a_v22_save_or_a_save_with_nothing_to_move() {
    let misplaced = r#"{ "ability": "ability.dead_language", "score": 3, "parameter": { "id": "language.arabic" } }"#;
    assert_eq!(load(22, misplaced).moved_ability_parameters, Vec::new());

    let placed = r#"{ "ability": "ability.dead_language", "score": 3, "parameter": { "id": "language.latin" } }"#;
    assert_eq!(load(21, placed).moved_ability_parameters, Vec::new());
}

/// Version 0 is an old save like any other, so its move is reported.
#[test]
fn a_version_zero_save_reports_its_move() {
    let loaded = load(
        0,
        r#"{ "ability": "ability.dead_language", "score": 2, "parameter": { "id": "language.persian" } }"#,
    );
    assert_eq!(
        loaded.moved_ability_parameters,
        vec![moved(
            "ability.dead_language",
            "ability.living_language",
            "language.persian",
            2,
            None
        )]
    );
}

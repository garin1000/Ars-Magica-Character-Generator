//! L1a (try-out finding 6): Dead and Living Language get separate catalogues.
//!
//! An Ability names its catalogue through an explicit `catalogue` field on its
//! definition; when the field is absent the catalogue id falls back to
//! `catalogue.<parameter key>` (the CV2 rule). Every site that resolves an
//! Ability's catalogue reads it the same way: the picker options
//! (`effective/parameter_options.rs`), the load-time integrity checks
//! (`ruleset/integrity.rs`) and the catalogue-matching load fold
//! (`migration.rs`).
//!
//! Norbert's decision: Dead = Latin, Hebrew, Gothic (ArMDE:3565 "the dead
//! language"); Living = Arabic, Greek, Persian, Aramaic. ArMDE:7432: "Arabic,
//! Greek and Hebrew fill similar functions, although of these only Hebrew is a
//! dead language". The split IS the decision, so the shipped-data tests below
//! assert the exact sets.
//!
//! Red-checkpoint protocol, phase 1: nothing reads `catalogue` yet (`Ability`
//! ignores the unknown key) and the shipped data still has one shared
//! `catalogue.language`, so every test here except the two fallback pins fails
//! on its own assertion.

use std::collections::{BTreeMap, BTreeSet};

use arm_rules::migration::{SCHEMA_VERSION, load_entity_migrating};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityParameterValue, AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection,
};
use arm_rules::{DEFAULT_SAGA_YEAR, ability_parameter_options, checked_xp_allocation};

// --- Shipped data: each language Ability offers its own list -----------------

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

fn companion() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.xp_pool = 0;
    e
}

fn ids(values: &[&str]) -> BTreeSet<Id> {
    values.iter().map(|v| Id::new(*v)).collect()
}

/// The catalogue ids the picker offers `ability`, as a set.
fn offered(ruleset: &Ruleset, entity: &Entity, ability: &str) -> BTreeSet<Id> {
    ability_parameter_options(entity, ruleset)
        .into_iter()
        .find(|o| o.ability == Id::new(ability))
        .unwrap_or_else(|| panic!("{ability} must have an options entry (it is catalogued)"))
        .catalogued
        .into_iter()
        .collect()
}

#[test]
fn dead_language_offers_exactly_latin_hebrew_and_gothic() {
    let ruleset = full_ruleset();
    assert_eq!(
        offered(&ruleset, &companion(), "ability.dead_language"),
        ids(&["language.gothic", "language.hebrew", "language.latin"]),
        "Dead Language must offer only the dead languages (decision on try-out finding 6)"
    );
}

#[test]
fn living_language_offers_exactly_arabic_greek_persian_and_aramaic() {
    let ruleset = full_ruleset();
    assert_eq!(
        offered(&ruleset, &companion(), "ability.living_language"),
        ids(&[
            "language.arabic",
            "language.aramaic",
            "language.greek",
            "language.persian",
        ]),
        "Living Language must offer only the living languages (decision on try-out finding 6)"
    );
}

/// With the lists split, a Virtue's pool instance must name the Ability whose
/// catalogue holds the value. Educated (Islamic)'s 50 XP go to "Arabic,
/// Persian, Greek, Latin, …" (ArMDE:3721): Arabic is now a Living Language, so
/// the pool must fund Living Language: Arabic.
#[test]
fn educated_islamic_pool_funds_living_language_arabic() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.selections = vec![Selection::new(Id::new("virtue.educated_islamic"))];
    let mut a = AbilityScore::new(Id::new("ability.living_language"), 1);
    a.parameter = Some(AbilityParameterValue::Catalogued {
        id: Id::new("language.arabic"),
    });
    entity.ability_scores = vec![a];

    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "Educated (Islamic)'s pool must fund Living Language: Arabic"
    );
}

/// Educated (Hebrew)'s 50 XP go to "Hebrew, Aramaic, …" (ArMDE:3725): Aramaic
/// is now a Living Language.
#[test]
fn educated_hebrew_pool_funds_living_language_aramaic() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.selections = vec![Selection::new(Id::new("virtue.educated_hebrew"))];
    let mut a = AbilityScore::new(Id::new("ability.living_language"), 1);
    a.parameter = Some(AbilityParameterValue::Catalogued {
        id: Id::new("language.aramaic"),
    });
    entity.ability_scores = vec![a];

    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "Educated (Hebrew)'s pool must fund Living Language: Aramaic"
    );
}

/// Turb Trained may learn "whichever single dead language the magi speak"
/// (ArMDE:5181); Q-X6-2 made its `language` parameter range over "the
/// catalogue's dead languages". After the split that is Dead Language's own
/// list, so it must offer exactly Latin, Hebrew and Gothic.
#[test]
fn turb_trained_offers_exactly_the_dead_languages() {
    let ruleset = full_ruleset();
    let turb_trained = ruleset
        .item(&Id::new("virtue.turb_trained"))
        .expect("virtue.turb_trained is shipped");
    let language = turb_trained
        .parameters
        .iter()
        .find(|p| p.key == "language")
        .expect("Turb Trained declares a `language` parameter");
    let offered: BTreeSet<Id> = language.values.iter().cloned().collect();
    assert_eq!(
        offered,
        ids(&["language.gothic", "language.hebrew", "language.latin"]),
        "Turb Trained's language choice must be the dead languages only"
    );
}

// --- The `catalogue` field and its fallback, on a minimal ruleset ------------
//
// Synthetic ability ids (`ability.test_*`), so no later save migration keyed
// on the real language Abilities can touch these fixtures.

/// Every fixture with point items needs a `personality`-category item
/// (`Ruleset::validate_engine_required_categories`).
const DUMMY_PERSONALITY_ITEM: &str = r#"{ "id": "flaw.dummy_personality", "kind": "flaw",
  "classification": "narrative", "magnitude": "minor", "categories": ["personality"] }"#;

/// `catalogue.language` is the shared list both language Abilities would fall
/// back to by their `language` key; the two named catalogues each hold one of
/// its values. `catalogue.organization` serves the fallback Ability, which
/// declares no `catalogue` field.
const CATALOGUES_JSON: &str = r#"{
  "catalogues": [
    { "id": "catalogue.language", "values": [
      { "id": "language.arabic", "source": { "anchor": "educated-islamic", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3719, 3721] } },
      { "id": "language.latin", "source": { "anchor": "educated", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3711, 3713] } }
    ] },
    { "id": "catalogue.language_dead", "values": [
      { "id": "language.latin", "source": { "anchor": "educated", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3711, 3713] } }
    ] },
    { "id": "catalogue.language_living", "values": [
      { "id": "language.arabic", "source": { "anchor": "educated-islamic", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3719, 3721] } }
    ] },
    { "id": "catalogue.organization", "values": [
      { "id": "organization.order_of_hermes", "source": { "anchor": "hermetic-experience", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4063, 4065] } }
    ] }
  ]
}"#;

const ABILITIES_JSON: &str = r#"{ "abilities": [
  { "id": "ability.test_dead", "category": "academic", "parameter": "language", "catalogued": true, "catalogue": "catalogue.language_dead" },
  { "id": "ability.test_living", "category": "general", "parameter": "language", "catalogued": true, "catalogue": "catalogue.language_living" },
  { "id": "ability.test_org", "category": "general", "parameter": "organization", "catalogued": true }
] }"#;

fn test_ruleset(abilities: &str, point_items: &str) -> Result<Ruleset, arm_rules::RulesetError> {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items,
        type_profiles: "[]",
        abilities: Some(abilities),
        parameter_catalogues: Some(CATALOGUES_JSON),
        ..RulesetSources::default()
    })
}

fn ruleset() -> Ruleset {
    test_ruleset(ABILITIES_JSON, "[]").expect("the L1a fixture ruleset loads")
}

#[test]
fn an_ability_offers_the_catalogue_its_catalogue_field_names() {
    let ruleset = ruleset();
    let entity = companion();
    assert_eq!(
        offered(&ruleset, &entity, "ability.test_dead"),
        ids(&["language.latin"]),
        "ability.test_dead names catalogue.language_dead, not the shared key's catalogue.language"
    );
    assert_eq!(
        offered(&ruleset, &entity, "ability.test_living"),
        ids(&["language.arabic"]),
        "ability.test_living names catalogue.language_living, not the shared key's \
         catalogue.language"
    );
}

/// Fallback pin (green today, must stay green): an Ability with no `catalogue`
/// field still resolves `catalogue.<parameter key>`.
#[test]
fn an_ability_without_the_field_falls_back_to_its_parameter_key() {
    let ruleset = ruleset();
    assert_eq!(
        offered(&ruleset, &companion(), "ability.test_org"),
        ids(&["organization.order_of_hermes"]),
        "with no `catalogue` field, ability.test_org must fall back to catalogue.organization"
    );
}

#[test]
fn an_ability_naming_an_unknown_catalogue_fails_to_load() {
    let abilities = r#"{ "abilities": [
      { "id": "ability.test_dead", "category": "academic", "parameter": "language", "catalogued": true, "catalogue": "catalogue.nonexistent" }
    ] }"#;
    let err = test_ruleset(abilities, "[]").expect_err(
        "an Ability naming a catalogue that does not exist must fail the load, even though \
         its parameter key's fallback catalogue (catalogue.language) exists",
    );
    let message = err.to_string();
    assert!(
        message.contains("ability.test_dead") && message.contains("catalogue.nonexistent"),
        "error must name the ability and the missing catalogue, got: {message}"
    );
}

fn ruleset_with_dead_literal(literal: &str) -> Result<Ruleset, arm_rules::RulesetError> {
    let point_items = format!(
        r#"[
          {{ "id": "virtue.test", "kind": "virtue", "classification": "creation_effect",
             "magnitude": "minor", "categories": ["general"],
             "effects": [{{ "type": "ability_authorization", "abilities": [
               {{ "ability": "ability.test_dead", "instance": {{ "literal": "{literal}" }} }}
             ] }}] }},
          {DUMMY_PERSONALITY_ITEM}
        ]"#
    );
    test_ruleset(ABILITIES_JSON, &point_items)
}

/// A Literal instance resolves inside the catalogue the Ability's `catalogue`
/// field names: `language.arabic` is in the shared `catalogue.language` but
/// not in `catalogue.language_dead`, so it must fail the load.
#[test]
fn a_literal_outside_the_named_catalogue_fails_to_load() {
    let err = ruleset_with_dead_literal("language.arabic").expect_err(
        "a literal outside the Ability's named catalogue must fail the load, not resolve \
         through the parameter key's fallback catalogue",
    );
    let message = err.to_string();
    assert!(
        message.contains("language.arabic") && message.contains("catalogue.language_dead"),
        "error must name the literal and the catalogue it was checked against, got: {message}"
    );
}

/// Positive pin for the literal check (green today through the fallback; must
/// stay green through the named catalogue).
#[test]
fn a_literal_inside_the_named_catalogue_loads() {
    ruleset_with_dead_literal("language.latin")
        .expect("a literal inside the Ability's named catalogue must load");
}

/// The load fold matches typed text against the Ability's OWN catalogue:
/// "Arabic" typed on the dead-language Ability is not in
/// `catalogue.language_dead`, so it stays text and is reported; typed on the
/// living-language Ability it resolves to `language.arabic`.
#[test]
fn the_load_fold_matches_typed_text_against_the_named_catalogue() {
    let ruleset = ruleset();
    let names: BTreeMap<Id, Vec<String>> = BTreeMap::from([
        (
            Id::new("language.arabic"),
            vec!["Arabic".to_string(), "Arabisch".to_string()],
        ),
        (
            Id::new("language.latin"),
            vec!["Latin".to_string(), "Latein".to_string()],
        ),
        (
            Id::new("organization.order_of_hermes"),
            vec![
                "Order of Hermes".to_string(),
                "Orden des Hermes".to_string(),
            ],
        ),
    ]);
    let json = format!(
        r#"{{
          "schema_version": {SCHEMA_VERSION},
          "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
          "entity_kind": "character",
          "type_id": "companion",
          "ability_scores": [
            {{ "ability": "ability.test_dead", "score": 1, "parameter": {{ "text": "Arabic" }} }},
            {{ "ability": "ability.test_living", "score": 1, "parameter": {{ "text": "Arabic" }} }}
          ]
        }}"#
    );
    let loaded = load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .expect("the fixture save loads");

    assert_eq!(
        loaded.entity.ability_scores[0].parameter,
        Some(AbilityParameterValue::text("Arabic")),
        "\"Arabic\" is not in catalogue.language_dead, so on ability.test_dead it must stay text"
    );
    assert!(
        loaded
            .unresolved_catalogued_parameters
            .contains(&(Id::new("ability.test_dead"), "Arabic".to_string())),
        "the unmatched dead-language text must be reported, got: {:?}",
        loaded.unresolved_catalogued_parameters
    );
    assert_eq!(
        loaded.entity.ability_scores[1].parameter,
        Some(AbilityParameterValue::Catalogued {
            id: Id::new("language.arabic")
        }),
        "\"Arabic\" on ability.test_living must resolve in catalogue.language_living"
    );
}

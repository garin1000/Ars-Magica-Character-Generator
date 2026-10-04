//! N4b (try-out 2026-10-04, finding N4): every engine reader of the plan's native
//! language works on the list value.
//!
//! "In the first five years of life, characters gain 75 experience points in their
//! native language […] and 45 experience points to divide between […] Living
//! Language (other than the character's native language)" (ArMDE:2378).
//!
//! A `Catalogued` native language is the catalogue value itself: it matches a
//! picked row by id and a typed row spelling any of its names, it is what a Sample
//! Childhood writes, and a childhood slot naming it in any spelling repeats it.
//! `Text` keeps N4a's folded text match.
//!
//! Red-checkpoint protocol, phase 1: `arm_rules::life_stage::NativeLanguage` does
//! not exist yet, so this file fails to compile; phase 2 adds the bare type first
//! and shows each assertion failing before the readers are implemented.

use std::collections::BTreeMap;

use arm_rules::life_stage::{LifeStagePlan, NativeLanguage};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{AbilityParameterValue, AbilityScore, Entity, Id};
use arm_rules::{
    ChildhoodRejection, LifeStageBlock, RestrictedXpPool, XpPoolOrigin, apply_childhood_package,
    restricted_xp_pools, validate,
};

const LIVING: &str = "ability.living_language";
const ARABIC: &str = "language.arabic";
const MISSING_SCORE: &str = "life_stage_native_language_missing_score";

/// The shipped core ruleset, childhood packages included.
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

/// The shipped ruleset with both locales' names attached, as the app loads it.
fn session_ruleset() -> Ruleset {
    let ruleset = full_ruleset();
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    let names = arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load");
    ruleset.with_catalogue_names(names)
}

fn arabic() -> NativeLanguage {
    NativeLanguage::Catalogued {
        id: Id::new(ARABIC),
    }
}

fn living(parameter: AbilityParameterValue, score: u8) -> AbilityScore {
    let mut row = AbilityScore::new(Id::new(LIVING), score);
    row.parameter = Some(parameter);
    row
}

/// A 25-year-old companion funded from its life stages, speaking `native`.
fn companion(native: NativeLanguage, scores: Vec<AbilityScore>) -> Entity {
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.ability_funding = arm_rules::AbilityFunding::LifeStages;
    entity.age = Some(25);
    entity.life_stages = Some(LifeStagePlan {
        native_language: Some(native),
        ..Default::default()
    });
    entity.ability_scores = scores;
    entity
}

/// Childhood's 75-point native-language pool.
fn native_pool(entity: &Entity, ruleset: &Ruleset) -> RestrictedXpPool {
    restricted_xp_pools(entity, ruleset)
        .into_iter()
        .find(|pool| {
            pool.origin
                == XpPoolOrigin::LifeStage {
                    block: LifeStageBlock::ChildhoodNativeLanguage,
                }
        })
        .expect("a life-stage companion with a native language has the native pool")
}

/// The `language` argument of every "native language not bought" warning.
fn missing_score_languages(entity: &Entity, ruleset: &Ruleset) -> Vec<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .filter(|issue| issue.code == MISSING_SCORE)
        .map(|issue| issue.args.get("language").cloned().unwrap_or_default())
        .collect()
}

fn slots(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// The two variants write the Ability row's own JSON shape, and nothing else.
#[test]
fn the_value_serializes_in_the_ability_parameter_shape() {
    let picked = LifeStagePlan {
        native_language: Some(arabic()),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_string(&picked).unwrap(),
        r#"{"native_language":{"id":"language.arabic"}}"#
    );
    let typed = LifeStagePlan {
        native_language: Some(NativeLanguage::Text {
            text: "Gaelic".into(),
        }),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_string(&typed).unwrap(),
        r#"{"native_language":{"text":"Gaelic"}}"#
    );
    assert!(
        serde_json::from_str::<NativeLanguage>(r#"{"item":"virtue.x","param":"p"}"#).is_err(),
        "a plan never follows another selection's parameter"
    );
}

/// Arabic picked from the list is funded by the native block whether its row was
/// picked too or typed in either locale, and nothing warns it is unbought.
#[test]
fn a_catalogued_native_language_is_funded_by_the_native_pool() {
    let rs = session_ruleset();
    for row in [
        AbilityParameterValue::Catalogued {
            id: Id::new(ARABIC),
        },
        AbilityParameterValue::text("Arabisch"),
        AbilityParameterValue::text(" arabic "),
    ] {
        let entity = companion(arabic(), vec![living(row.clone(), 5)]);
        let pool = native_pool(&entity, &rs);
        assert_eq!((pool.used, pool.amount), (75, 75), "{row:?}");
        assert!(missing_score_languages(&entity, &rs).is_empty(), "{row:?}");
    }
}

/// Unbought, the warning names the language by its catalogue id, which the UI
/// localizes like every other id argument; a typed language is named as typed.
#[test]
fn the_missing_score_warning_names_the_language() {
    let rs = session_ruleset();
    let picked = companion(arabic(), Vec::new());
    assert_eq!(
        missing_score_languages(&picked, &rs),
        vec![ARABIC.to_string()]
    );

    let typed = companion(
        NativeLanguage::Text {
            text: "Gaelic".into(),
        },
        Vec::new(),
    );
    assert_eq!(
        missing_score_languages(&typed, &rs),
        vec!["Gaelic".to_string()]
    );
}

/// A package's native entry is written as the catalogue value itself.
#[test]
fn a_package_writes_the_catalogued_native_language_as_that_value() {
    let rs = session_ruleset();
    let applied = apply_childhood_package(
        &companion(arabic(), Vec::new()),
        &Id::new("childhood.athletic"),
        &BTreeMap::new(),
        &rs,
    )
    .expect("Athletic applies");

    let rows: Vec<(Option<AbilityParameterValue>, u8)> = applied
        .ability_scores
        .iter()
        .filter(|row| row.ability == Id::new(LIVING))
        .map(|row| (row.parameter.clone(), row.score))
        .collect();
    assert_eq!(
        rows,
        vec![(
            Some(AbilityParameterValue::Catalogued {
                id: Id::new(ARABIC)
            }),
            5
        )]
    );
}

/// Traveling's second language may not be the native one, in any spelling.
#[test]
fn a_slot_naming_the_catalogued_native_language_is_rejected() {
    let rs = session_ruleset();
    for answer in ["Arabic", " arabic ", "Arabisch"] {
        let rejections = apply_childhood_package(
            &companion(arabic(), Vec::new()),
            &Id::new("childhood.traveling"),
            &slots(&[
                ("area_a", "Rhine"),
                ("area_b", "Provence"),
                ("language", answer),
            ]),
            &rs,
        )
        .expect_err("the second language repeats the native one");
        assert!(
            rejections.iter().any(|rejection| matches!(
                rejection,
                ChildhoodRejection::SlotIsNativeLanguage { slot, .. } if slot == "language"
            )),
            "{answer:?}: {rejections:?}"
        );
    }
}

/// Text naming no catalogue value keeps N4a's folded text match.
#[test]
fn a_text_native_language_still_matches_case_folded() {
    let rs = session_ruleset();
    let entity = companion(
        NativeLanguage::Text {
            text: "Gaelic".into(),
        },
        vec![living(AbilityParameterValue::text("gaelic"), 5)],
    );
    let pool = native_pool(&entity, &rs);
    assert_eq!((pool.used, pool.amount), (75, 75));
}

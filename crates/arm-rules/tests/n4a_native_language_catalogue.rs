//! N4a (try-out 2026-10-04, wrong rules output): the native language matches by
//! catalogue value, not by spelling.
//!
//! "In the first five years of life, characters gain 75 experience points in their
//! native language […] and 45 experience points to divide between […] Living
//! Language (other than the character's native language)" (ArMDE:2378).
//!
//! The plan's `native_language` is free text ("Arabic"), while a Living Language
//! row is a `Catalogued` value whenever the player picked it from the list or the
//! load fold (`migration.rs::fold_catalogue_matching`, every load) recognized its
//! text. The 75-point pool, the "native language not bought" warning and the
//! childhood package's slot checks all compared the raw string, so a picked or
//! reloaded "Arabic" fell out of its own pool. They now resolve the plan's text
//! once against the Ability's catalogue names in every locale, and text naming no
//! catalogue value still matches case-folded.
//!
//! Red-checkpoint protocol, phase 1: every test below fails on its own assertion.

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{AbilityParameterValue, AbilityScore, Entity, Id};
use arm_rules::{
    ChildhoodEntry, ChildhoodPackage, ChildhoodRejection, DEFAULT_SAGA_YEAR, LifeStageBlock,
    RestrictedXpPool, SCHEMA_VERSION, XpPoolOrigin, apply_childhood_package, apply_package,
    load_entity_migrating, restricted_xp_pools, validate,
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

/// Both locales' catalogue names, as the app reads them.
fn catalogue_names(ruleset: &Ruleset) -> BTreeMap<Id, Vec<String>> {
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

/// The shipped ruleset with both locales' names attached, as the app loads it.
fn session_ruleset() -> Ruleset {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    ruleset.with_catalogue_names(names)
}

fn catalogued(id: &str) -> AbilityParameterValue {
    AbilityParameterValue::Catalogued { id: Id::new(id) }
}

fn living(parameter: AbilityParameterValue, score: u8) -> AbilityScore {
    let mut row = AbilityScore::new(Id::new(LIVING), score);
    row.parameter = Some(parameter);
    row
}

/// A 25-year-old companion funded from its life stages, speaking `native`.
fn companion(native: &str, scores: Vec<AbilityScore>) -> Entity {
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.ability_funding = arm_rules::AbilityFunding::LifeStages;
    entity.age = Some(25);
    entity.life_stages = Some(arm_rules::life_stage::LifeStagePlan {
        native_language: Some(native.into()),
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

fn missing_score_warnings(entity: &Entity, ruleset: &Ruleset) -> usize {
    validate(entity, ruleset)
        .issues
        .iter()
        .filter(|issue| issue.code == MISSING_SCORE)
        .count()
}

/// Saves as `ruleset_io.rs::save_entity_to_path` does, then reopens the bytes
/// through the load path with both locales' names.
fn save_and_reopen(mut entity: Entity, ruleset: &Ruleset) -> Entity {
    entity.schema_version = SCHEMA_VERSION;
    entity.normalize();
    let json = serde_json::to_string_pretty(&entity).expect("an entity serializes");
    let names = catalogue_names(ruleset);
    load_entity_migrating(&json, DEFAULT_SAGA_YEAR, ruleset, &names)
        .expect("the saved bytes load")
        .entity
}

fn living_rows(entity: &Entity) -> Vec<&AbilityScore> {
    entity
        .ability_scores
        .iter()
        .filter(|row| row.ability == Id::new(LIVING))
        .collect()
}

// --- Red 1: in the session ----------------------------------------------------------

/// Arabic picked from the list (`Catalogued`) is the plan's "Arabic": the native
/// block pays all 75 points of Arabic 5, and nothing warns it is unbought.
#[test]
fn a_catalogued_native_language_is_funded_by_the_native_pool() {
    let rs = session_ruleset();
    let entity = companion("Arabic", vec![living(catalogued(ARABIC), 5)]);

    let pool = native_pool(&entity, &rs);
    assert_eq!(
        (pool.used, pool.amount),
        (75, 75),
        "Arabic 5 is the native block"
    );
    assert_eq!(missing_score_warnings(&entity, &rs), 0);
}

/// The German name resolves to the same value: a plan reading "Arabisch" funds
/// the picked Arabic too.
#[test]
fn the_plan_language_resolves_in_every_locale() {
    let rs = session_ruleset();
    let entity = companion("  arabisch ", vec![living(catalogued(ARABIC), 5)]);

    let pool = native_pool(&entity, &rs);
    assert_eq!((pool.used, pool.amount), (75, 75));
    assert_eq!(missing_score_warnings(&entity, &rs), 0);
}

// --- Red 2: after save and reopen ---------------------------------------------------

/// The package used to write `Text("Arabic")`; the load fold turns that into
/// `Catalogued(language.arabic)` on reopening. The native block must still pay
/// for it, and nothing may warn.
#[test]
fn the_native_pool_survives_a_save_and_reopen() {
    let rs = session_ruleset();
    let typed = companion(
        "Arabic",
        vec![living(AbilityParameterValue::text("Arabic"), 5)],
    );

    let reopened = save_and_reopen(typed, &rs);
    assert_eq!(
        living_rows(&reopened)[0].parameter,
        Some(catalogued(ARABIC)),
        "premise: the load fold recognized the typed name"
    );
    let pool = native_pool(&reopened, &rs);
    assert_eq!((pool.used, pool.amount), (75, 75));
    assert_eq!(missing_score_warnings(&reopened, &rs), 0);
}

// --- Red 3: childhood package slots -------------------------------------------------

/// A test-only package with two Living Language slots, so two answers can name
/// the same language in different spellings.
fn two_language_package() -> ChildhoodPackage {
    let slot = |key: &str| ChildhoodEntry {
        ability: Id::new(LIVING),
        score: 1,
        slot: Some(key.to_string()),
        native: false,
    };
    ChildhoodPackage {
        id: Id::new("childhood.test_two_languages"),
        entries: vec![
            ChildhoodEntry {
                ability: Id::new(LIVING),
                score: 5,
                slot: None,
                native: true,
            },
            slot("language_a"),
            slot("language_b"),
        ],
        source: None,
    }
}

fn slots(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// "arabic" and "Arabic" are one language: the second slot would merge into the
/// first row, so it is rejected once, not merged silently.
#[test]
fn two_slots_spelling_one_language_differently_are_one_duplicate() {
    let rs = session_ruleset();
    let entity = companion("German", Vec::new());

    for (a, b) in [("arabic", "Arabic"), ("Arabic", "Arabisch")] {
        let rejections = apply_package(
            &entity,
            &two_language_package(),
            &slots(&[("language_a", a), ("language_b", b)]),
            &rs,
        )
        .expect_err("two spellings of one language collide");
        assert_eq!(rejections.len(), 1, "{a:?}/{b:?}: {rejections:?}");
        assert!(
            matches!(
                &rejections[0],
                ChildhoodRejection::DuplicateSlotValue { slot, other_slot, .. }
                    if slot == "language_b" && other_slot == "language_a"
            ),
            "{a:?}/{b:?}: {rejections:?}"
        );
    }
}

/// The same resolution holds for an uncatalogued Ability: "Rhine" and "rhine" are
/// one Area Lore (the picker's `derive.ts::childhoodSlotFault` mirrors this).
#[test]
fn two_area_lores_differing_only_in_case_are_one_duplicate() {
    let rs = session_ruleset();
    let entity = companion("German", Vec::new());

    let rejections = apply_childhood_package(
        &entity,
        &Id::new("childhood.traveling"),
        &slots(&[
            ("area_a", "Rhine"),
            ("area_b", "rhine"),
            ("language", "Greek"),
        ]),
        &rs,
    )
    .expect_err("one Area Lore answered twice");
    assert!(
        matches!(
            rejections.as_slice(),
            [ChildhoodRejection::DuplicateSlotValue { slot, .. }] if slot == "area_b"
        ),
        "{rejections:?}"
    );
}

/// Traveling's second language may not be the native one, however it is spelled.
#[test]
fn a_slot_spelling_the_native_language_differently_is_the_native_language() {
    let rs = session_ruleset();
    let entity = companion("Arabic", Vec::new());

    for answer in ["arabic", " ARABIC ", "Arabisch"] {
        let rejections = apply_childhood_package(
            &entity,
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

// --- Red 4: text naming no catalogue value, and what the package writes -------------

/// "Gaelic" is in no catalogue: it stays free text and still matches a row typed
/// in another case.
#[test]
fn an_uncatalogued_native_language_still_matches_case_folded() {
    let rs = session_ruleset();
    let entity = companion(
        "Gaelic",
        vec![living(AbilityParameterValue::text("gaelic"), 5)],
    );

    let pool = native_pool(&entity, &rs);
    assert_eq!((pool.used, pool.amount), (75, 75));
    assert_eq!(missing_score_warnings(&entity, &rs), 0);
}

/// The package writes the catalogue value where the text names one ("arabic" →
/// `Catalogued(language.arabic)`) and keeps text that names none ("Gaelic").
#[test]
fn the_package_writes_catalogue_values_where_the_text_names_one() {
    let rs = session_ruleset();
    let entity = companion("Gaelic", Vec::new());

    let applied = apply_childhood_package(
        &entity,
        &Id::new("childhood.traveling"),
        &slots(&[
            ("area_a", "Rhine"),
            ("area_b", "Provence"),
            ("language", "arabic"),
        ]),
        &rs,
    )
    .expect("Traveling applies");

    let mut written: Vec<(Option<AbilityParameterValue>, u8)> = living_rows(&applied)
        .into_iter()
        .map(|row| (row.parameter.clone(), row.score))
        .collect();
    written.sort_by_key(|(_, score)| *score);
    assert_eq!(
        written,
        vec![
            (Some(catalogued(ARABIC)), 1),
            (Some(AbilityParameterValue::text("Gaelic")), 5),
        ]
    );
}

/// A native language naming a catalogue value is written as that value, and it
/// raises the row the player already picked rather than adding a second one.
#[test]
fn the_native_entry_raises_the_picked_row_instead_of_adding_one() {
    let rs = session_ruleset();
    let entity = companion("arabic", vec![living(catalogued(ARABIC), 2)]);

    let applied = apply_childhood_package(
        &entity,
        &Id::new("childhood.athletic"),
        &BTreeMap::new(),
        &rs,
    )
    .expect("Athletic applies");

    let rows = living_rows(&applied);
    assert_eq!(rows.len(), 1, "one Arabic row: {rows:?}");
    assert_eq!(rows[0].parameter, Some(catalogued(ARABIC)));
    assert_eq!(rows[0].score, 5);
}

//! Slice A1 — the app's maximum age (after-deadline answer 7, robustness finding F2).
//!
//! `aging.rs::aging_schedule` builds one row per year from the first roll age up
//! to the character's age, on every refresh, so a huge age froze or OOM-crashed
//! the app. Norbert (2026-10-03) set an app MAXIMUM AGE of 500. It is an app
//! limit, not a rule: no rulebook passage sets a maximum age, so nothing here
//! cites one. The number is rules DATA (`rules/core/aging.json` `max_age`),
//! surfaced on the `Ruleset` the UI receives, never a constant in code.
//!
//! The engine enforces it in two places: `migration.rs::load_entity_migrating`
//! clamps a crafted save's `age` and `apparent_age` (value-driven and
//! version-free, like the aura clamp beside it, so the next save rewrites the
//! value and no schema version is stamped), and `aging_schedule` itself never
//! walks past the cap, whatever entity it is handed.
//!
//! The birth year is the age's other view, so it is held to the same bound: an
//! age derived from a birth year clamps at the maximum, and a save's birth year
//! earlier than `saga_year - max_age` loads clamped in step with the age. A
//! `max_age` below the first aging-roll age is nonsense data and fails the
//! ruleset load loudly.

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetError, RulesetSources};
use arm_rules::types::{Entity, EntityKind, Id, RulesetRef};
use arm_rules::{
    DEFAULT_SAGA_YEAR, SCHEMA_VERSION, age_in_saga_year, aging_schedule, load_entity_migrating,
};

/// The app maximum age the shipped `rules/core/aging.json` carries.
const SHIPPED_MAX_AGE: u32 = 500;

/// A ruleset carrying the shipped aging block — everything the age cap and the
/// schedule read — plus the shipped Abilities (the block's Crisis attendant names
/// one) and the parameter catalogues those Abilities are catalogued against, all
/// of which the integrity check resolves.
fn shipped_aging_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: "[]",
        type_profiles: "[]",
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("the shipped aging block loads")
}

/// A save carrying the given `age` and `apparent_age`, current in every other respect.
fn save_with_ages(age: u64, apparent_age: u64) -> String {
    format!(
        r#"{{
          "schema_version": {SCHEMA_VERSION},
          "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "age": {age},
          "apparent_age": {apparent_age}
        }}"#
    )
}

fn load(json: &str, ruleset: &Ruleset) -> Entity {
    load_entity_migrating(json, DEFAULT_SAGA_YEAR, ruleset, &BTreeMap::new())
        .expect("an out-of-range age is clamped, not refused")
        .entity
}

fn character_aged(age: u32) -> Entity {
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.age = Some(age);
    entity
}

/// The ruleset the UI receives carries the maximum, so the age input can be
/// capped from data rather than from a constant duplicated in TypeScript.
#[test]
fn the_shipped_ruleset_surfaces_a_max_age_of_500() {
    let surfaced = serde_json::to_value(shipped_aging_ruleset()).expect("the ruleset serializes");
    assert_eq!(
        surfaced["aging"]["max_age"],
        serde_json::json!(SHIPPED_MAX_AGE),
        "the aging block the UI receives must carry the app maximum age"
    );
}

/// A crafted save's age (and apparent age) is clamped to the maximum on load,
/// so the schedule built from it stays bounded. Value-driven, like the aura
/// clamp: no schema version is stamped.
#[test]
fn a_crafted_age_is_clamped_to_the_max_age_on_load() {
    let ruleset = shipped_aging_ruleset();
    for stored in [4_000_000_000_u64, u64::from(u32::MAX), 501] {
        let entity = load(&save_with_ages(stored, stored), &ruleset);
        assert_eq!(
            entity.age,
            Some(SHIPPED_MAX_AGE),
            "age {stored} must load clamped to {SHIPPED_MAX_AGE}"
        );
        assert_eq!(
            entity.apparent_age,
            Some(SHIPPED_MAX_AGE),
            "apparent age {stored} must load clamped to {SHIPPED_MAX_AGE}"
        );
        assert_eq!(
            entity.schema_version, SCHEMA_VERSION,
            "a value-driven clamp stamps no new version"
        );

        let schedule = aging_schedule(&entity, &ruleset);
        let first = ruleset.aging().expect("aging rules").first_roll_age();
        assert_eq!(schedule.len(), (SHIPPED_MAX_AGE - first + 1) as usize);
        assert_eq!(schedule.last().map(|year| year.age), Some(SHIPPED_MAX_AGE));
    }
}

/// The other half of the clamp: an age at or under the maximum is data, not
/// something to normalize — it loads and re-serializes untouched.
#[test]
fn a_legal_age_round_trips_unchanged() {
    let ruleset = shipped_aging_ruleset();
    let entity = load(&save_with_ages(u64::from(SHIPPED_MAX_AGE), 45), &ruleset);
    assert_eq!(
        entity.age,
        Some(SHIPPED_MAX_AGE),
        "the boundary is inclusive"
    );
    assert_eq!(entity.apparent_age, Some(45));

    let saved = serde_json::to_string_pretty(&entity).expect("the entity serializes");
    let reloaded = load(&saved, &ruleset);
    assert_eq!(reloaded, entity, "nothing was lost or invented");
    assert_eq!(
        serde_json::to_string_pretty(&reloaded).expect("the entity serializes"),
        saved
    );
}

/// Defence at the source: the schedule never walks past the maximum, even for an
/// entity that never went through the load clamp (one built in memory, or an age
/// derived from a far-past birth year).
#[test]
fn the_schedule_never_exceeds_the_max_age() {
    let ruleset = shipped_aging_ruleset();
    let first = ruleset.aging().expect("aging rules").first_roll_age();
    let bounded_len = (SHIPPED_MAX_AGE - first + 1) as usize;

    // A moderately large age first, so an uncapped schedule fails here quickly
    // instead of materializing billions of rows below.
    let schedule = aging_schedule(&character_aged(5_000), &ruleset);
    assert_eq!(schedule.len(), bounded_len);
    assert_eq!(schedule.last().map(|year| year.age), Some(SHIPPED_MAX_AGE));

    let schedule = aging_schedule(&character_aged(u32::MAX), &ruleset);
    assert_eq!(schedule.len(), bounded_len);
    assert_eq!(schedule.last().map(|year| year.age), Some(SHIPPED_MAX_AGE));
}

/// A ruleset whose aging block states no maximum leaves the age alone: the cap
/// is data, and without the data there is no cap to apply.
#[test]
fn an_aging_block_without_a_max_age_leaves_the_age_alone() {
    let without_cap = minimal_aging_ruleset(None).expect("an aging block without max_age loads");
    let entity = load(&save_with_ages(900, 900), &without_cap);
    assert_eq!(entity.age, Some(900));
    assert_eq!(entity.apparent_age, Some(900));
}

/// A minimal aging block (start age 35, so the first roll is owed at 36), with
/// the given `max_age` when one is stated.
fn minimal_aging_ruleset(max_age: Option<u32>) -> Result<Ruleset, RulesetError> {
    let max_age = max_age
        .map(|age| format!(r#""max_age": {age},"#))
        .unwrap_or_default();
    let aging = format!(
        r#"{{
          "start_age": 35,
          "age_divisor": 10,
          "apparent_age_increase_min": 3,
          {max_age}
          "outcomes": [
            {{ "min": 3, "effect": {{ "type": "next_decrepitude_level_and_crisis" }} }}
          ]
        }}"#
    );
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: "[]",
        type_profiles: "[]",
        aging: Some(&aging),
        ..RulesetSources::default()
    })
}

/// A `max_age` below the first aging-roll age would cap every character before
/// the aging subsystem could ever apply: nonsense data, refused loudly at load.
/// The first roll age itself is the lowest legal maximum.
#[test]
fn a_max_age_below_the_first_aging_roll_age_fails_the_ruleset_load() {
    match minimal_aging_ruleset(Some(35)) {
        Err(RulesetError::Integrity(error)) => {
            let message = format!("{error:?}");
            assert!(message.contains("max_age"), "{message}");
        }
        other => panic!("a max_age of 35 under a first roll at 36 must fail: {other:?}"),
    }
    minimal_aging_ruleset(Some(36)).expect("a max_age at the first roll age loads");
}

/// The birth year is the age's other view, so a save's birth year earlier than
/// `saga_year - max_age` loads clamped in step with the age: the two stay a
/// consistent pair, and the next save writes both clamped values.
#[test]
fn a_too_early_birth_year_is_clamped_in_step_with_the_age_on_load() {
    let ruleset = shipped_aging_ruleset();
    let json = format!(
        r#"{{
          "schema_version": {SCHEMA_VERSION},
          "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          "age": 4000000000,
          "birth_year": -2000000000
        }}"#
    );
    let entity = load(&json, &ruleset);
    assert_eq!(entity.age, Some(SHIPPED_MAX_AGE));
    assert_eq!(entity.birth_year, Some(1220 - 500));
    assert_eq!(entity.schema_version, SCHEMA_VERSION);

    // The earliest legal birth year itself is left alone.
    let json = json
        .replace("-2000000000", "720")
        .replace("4000000000", "500");
    let entity = load(&json, &ruleset);
    assert_eq!(entity.birth_year, Some(720));
    assert_eq!(entity.age, Some(SHIPPED_MAX_AGE));
}

/// An age derived from a birth year clamps at the maximum, so typing a far-past
/// birth year cannot reach the schedule with an age beyond it. Without a stated
/// maximum the derivation is unchanged.
#[test]
fn an_age_derived_from_a_birth_year_clamps_at_the_max_age() {
    let derived = age_in_saga_year(1220, 120, Some(SHIPPED_MAX_AGE));
    assert_eq!(derived.age, SHIPPED_MAX_AGE);
    assert!(derived.issues.is_empty(), "{:?}", derived.issues);

    assert_eq!(age_in_saga_year(1220, 720, Some(SHIPPED_MAX_AGE)).age, 500);
    assert_eq!(age_in_saga_year(1220, 120, None).age, 1100);
}

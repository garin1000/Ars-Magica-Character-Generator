//! Row 56, slice D: schema 24 → 25 folds the old one-total
//! `life_stages.post_gauntlet_lab_seasons` into `post_gauntlet_lab_years`.
//!
//! The old total was read as packed into full lab years (F1): `s / 4` years of
//! four seasons, plus one year holding the remainder `s % 4` when that is 1–3.
//! The fold writes exactly those year counts, so the charge is preserved — "the
//! character loses 10 points from the yearly 30 experience points, to a minimum
//! of 0 if three or four seasons are spent on lab work" (ArMDE:2550).
//!
//! A save is the trust boundary, and `LifeStagePlan` does not deny unknown
//! fields: a leftover or malformed old key would be dropped by serde in silence,
//! taking the charge with it. So a malformed value is a load error, never a drop.
//!
//! This file is the RED checkpoint: everything below fails for the right reason
//! until the fold lands and `SCHEMA_VERSION` is bumped.

use arm_rules::LabYears;
use arm_rules::migration::{SCHEMA_VERSION, load_entity_migrating};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::DEFAULT_SAGA_YEAR;

const OLD_KEY: &str = "post_gauntlet_lab_seasons";

fn shipped_ruleset() -> Ruleset {
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
        spell_mastery_abilities: Some(include_str!(
            "../../../rules/core/spell_mastery_abilities.json"
        )),
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

fn no_catalogue_names() -> std::collections::BTreeMap<Id, Vec<String>> {
    std::collections::BTreeMap::new()
}

/// A schema-24 magus save of `age`, gauntleted at 25, whose plan carries
/// `lab_fields` (raw JSON members, possibly empty) after its Gauntlet age.
fn v24_magus(age: u32, lab_fields: &str) -> String {
    let separator = if lab_fields.is_empty() { "" } else { ", " };
    format!(
        r#"{{
      "schema_version": 24,
      "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
      "entity_kind": "character",
      "type_id": "magus",
      "ability_funding": "life_stages",
      "saga_year": 1220,
      "age": {age},
      "life_stages": {{ "gauntlet_age": 25{separator}{lab_fields} }}
    }}"#
    )
}

fn load(json: &str) -> Result<Entity, serde_json::Error> {
    load_entity_migrating(
        json,
        DEFAULT_SAGA_YEAR,
        &shipped_ruleset(),
        &no_catalogue_names(),
    )
    .map(|loaded| loaded.entity)
}

fn lab_years_of(entity: &Entity) -> LabYears {
    entity
        .life_stages
        .as_ref()
        .expect("the plan survives the load")
        .post_gauntlet_lab_years
}

fn post_gauntlet_points(entity: &Entity) -> u32 {
    let rs = shipped_ruleset();
    rs.life_stages()
        .expect("shipped life-stage rules loaded")
        .budget(entity, &rs)
        .expect("a magus funded by its life stages has a budget")
        .post_gauntlet_points
}

fn years(one: u32, two: u32, three: u32, four: u32) -> LabYears {
    LabYears {
        one_season: one,
        two_seasons: two,
        three_seasons: three,
        four_seasons: four,
    }
}

#[test]
fn schema_version_is_25() {
    assert_eq!(
        SCHEMA_VERSION, 25,
        "row 56 bumps SCHEMA_VERSION past D87's 24 for the lab-years fold"
    );
}

/// `s / 4` full lab years, plus one year of `s % 4` seasons when that is 1–3;
/// and the post-Gauntlet points are those the packed total was worth.
#[test]
fn an_old_season_total_folds_into_year_counts_with_the_charge_preserved() {
    // (age, stored seasons, expected year counts, points before = points after)
    let cases = [
        (60, 1, years(1, 0, 0, 0), 1040),
        (60, 4, years(0, 0, 0, 1), 1020),
        (60, 13, years(1, 0, 0, 3), 950),
        (60, 14, years(0, 1, 0, 3), 940),
        (60, 15, years(0, 0, 1, 3), 930),
        // Beyond the span: the old total was capped at four seasons a year, which
        // left the one year worth 0; the folded counts charge at least as much, and
        // the points floor at the same 0.
        (26, 13, years(1, 0, 0, 3), 0),
    ];
    for (age, seasons, expected, points) in cases {
        let entity = load(&v24_magus(age, &format!(r#""{OLD_KEY}": {seasons}"#)))
            .unwrap_or_else(|e| panic!("{seasons} seasons must load: {e}"));
        assert_eq!(lab_years_of(&entity), expected, "{seasons} seasons");
        assert_eq!(
            post_gauntlet_points(&entity),
            points,
            "{seasons} seasons at age {age}"
        );
    }
}

/// A stored 0 carries no lab work: the key is dropped, nothing is invented, and
/// nothing is stamped.
#[test]
fn a_zero_season_total_is_dropped() {
    let entity = load(&v24_magus(60, &format!(r#""{OLD_KEY}": 0"#))).expect("0 seasons load");
    assert_eq!(lab_years_of(&entity), LabYears::default());
    let saved = serde_json::to_string(&entity).unwrap();
    assert!(!saved.contains(OLD_KEY), "{saved}");
    assert!(!saved.contains("post_gauntlet_lab_years"), "{saved}");
    assert_eq!(
        entity.schema_version, 24,
        "nothing moved, so nothing is stamped"
    );
}

/// The old field was a `u32`, so anything else failed the load before; it still
/// must, rather than being dropped with the charge.
#[test]
fn a_malformed_season_total_is_a_load_error_not_a_drop() {
    for malformed in [
        r#""13""#,
        "13.5",
        "-1",
        "4294967296",
        "{}",
        "[13]",
        "true",
        "null",
    ] {
        let result = load(&v24_magus(60, &format!(r#""{OLD_KEY}": {malformed}"#)));
        assert!(
            result.is_err(),
            "{OLD_KEY}: {malformed} must fail the load, not vanish: {result:?}"
        );
    }
}

/// A hand-edited save carrying both keys: the new shape is the current one, so
/// it wins and the old key is dropped rather than added on top.
#[test]
fn when_both_keys_are_present_the_new_one_wins() {
    let entity = load(&v24_magus(
        60,
        &format!(r#""{OLD_KEY}": 13, "post_gauntlet_lab_years": {{ "two_seasons": 1 }}"#),
    ))
    .expect("both keys load");
    assert_eq!(lab_years_of(&entity), years(0, 1, 0, 0));
    assert_eq!(post_gauntlet_points(&entity), 1030);
    let saved = serde_json::to_string(&entity).unwrap();
    assert!(!saved.contains(OLD_KEY), "{saved}");
}

/// The fold is a genuine value move, so it stamps the new version; a save with
/// no old key is left at the version it was written with.
#[test]
fn the_version_is_stamped_only_when_the_fold_fires() {
    let folded = load(&v24_magus(60, &format!(r#""{OLD_KEY}": 13"#))).expect("13 seasons load");
    assert_eq!(folded.schema_version, 25);

    let untouched = load(&v24_magus(60, "")).expect("a plan with no lab work loads");
    assert_eq!(untouched.schema_version, 24);
}

/// Round-trip fidelity: the old key never comes back out, and a second load of
/// the migrated save is byte-identical.
#[test]
fn the_old_key_never_round_trips_out() {
    let migrated = load(&v24_magus(60, &format!(r#""{OLD_KEY}": 13"#))).expect("13 seasons load");
    let saved = serde_json::to_string_pretty(&migrated).unwrap();
    assert!(!saved.contains(OLD_KEY), "{saved}");
    assert!(saved.contains(r#""four_seasons": 3"#), "{saved}");
    assert!(saved.contains(r#""one_season": 1"#), "{saved}");

    let reloaded = load(&saved).expect("the migrated save reloads");
    assert_eq!(reloaded, migrated);
    assert_eq!(serde_json::to_string_pretty(&reloaded).unwrap(), saved);
}

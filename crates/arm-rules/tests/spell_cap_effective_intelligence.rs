//! F6 (`tmp/ftl-rules-audit.md`): the spell-level cap must read the EFFECTIVE
//! Intelligence, the same score the Lab Total reads.
//!
//! ArMDE:2465: "The highest level spell you can learn is equal to Technique +
//! Form + Intelligence + Magic Theory +3 … This is the appropriate Lab Total,
//! assuming an aura modifier of +3". The Lab Total (`derived/lab.rs::lab_totals`)
//! reads Intelligence after aging drops and free Characteristic deltas
//! (`effective_characteristic_after_aging`); Great (Characteristic) raises the
//! score itself (ArMDE:3989), and the book's own Bonisagus template prints
//! "Int +5" for bought +3 with "Great Intelligence (x2)" (ArMDE:1656, :1668).
//! Every test runs against the real shipped ruleset.

use arm_rules::characteristics::Characteristic;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::{
    effective_characteristic_after_aging, lab_totals, spell_level_cap, spell_level_caps,
};
use std::collections::BTreeMap;

/// The shipped core ruleset — same set of files `book_templates.rs` loads;
/// duplicated because integration test binaries cannot share private helpers.
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

fn catalogue_names() -> BTreeMap<Id, Vec<String>> {
    let ruleset = full_ruleset();
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

/// A magus with Cr 5, Ig 5, no Magic Theory, no lab modifiers.
fn creo_ignem_magus(bought_int: i8) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.characteristics.insert(Characteristic::Int, bought_int);
    e.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 5),
        ArtScore::new(Id::new("art.ignem"), 5),
    ];
    e
}

fn great_intelligence() -> Selection {
    Selection::with_params(
        Id::new("virtue.great_characteristic"),
        BTreeMap::from([("characteristic".to_string(), Id::new("characteristic.int"))]),
    )
}

fn creo_ignem_cap(e: &Entity, rs: &Ruleset) -> i64 {
    spell_level_cap(
        e,
        rs,
        &Id::new("art.creo"),
        &Id::new("art.ignem"),
        &[],
        false,
        false,
        false,
    )
}

#[test]
fn great_intelligence_raises_the_spell_level_cap_as_it_raises_the_lab_total() {
    let rs = full_ruleset();
    let mut e = creo_ignem_magus(3);
    e.selections = vec![great_intelligence()];
    e.aura = 3;

    assert_eq!(
        effective_characteristic_after_aging(&e, &rs, Characteristic::Int),
        4,
        "precondition: Great (Intelligence) on a bought +3 is an effective +4 (ArMDE:3989)"
    );
    let lab_cell = lab_totals(&e, &rs)
        .into_iter()
        .find(|cell| cell.technique == Id::new("art.creo") && cell.form == Id::new("art.ignem"))
        .expect("the Lab Total grid carries Cr/Ig");
    // Cr 5 + Ig 5 + Int 4 + Magic Theory 0 + aura 3 = 17.
    assert_eq!(
        lab_cell.total, 17,
        "precondition: the Cr/Ig Lab Total at aura 3"
    );

    assert_eq!(
        creo_ignem_cap(&e, &rs),
        17,
        "ArMDE:2465: the cap IS the Lab Total at aura +3, so it must read the \
         effective Intelligence (+4), not the bought +3"
    );
    let grid_row = spell_level_caps(&e, &rs)
        .into_iter()
        .find(|row| {
            row.technique == Id::new("art.creo")
                && row.form == Id::new("art.ignem")
                && !row.range_beyond_touch
        })
        .expect("the cap grid carries Cr/Ig");
    assert_eq!(
        grid_row.cap, 17,
        "the picker's grid cap reads the same number"
    );
}

#[test]
fn an_aging_drop_in_intelligence_lowers_the_spell_level_cap() {
    let rs = full_ruleset();
    let mut e = creo_ignem_magus(0);
    // One aging point on a score of 0 exceeds it: one drop, to -1 (ArMDE:16579).
    e.aging_points.insert(Characteristic::Int, 1);

    assert_eq!(
        effective_characteristic_after_aging(&e, &rs, Characteristic::Int),
        -1,
        "precondition: one aging point drops a 0 Intelligence to -1"
    );
    // Cr 5 + Ig 5 + Int -1 + Magic Theory 0 + 3 = 12.
    assert_eq!(
        creo_ignem_cap(&e, &rs),
        12,
        "ArMDE:2465: an aged magus's Lab Total uses his aged Intelligence, so must the cap"
    );
}

#[test]
fn the_books_bonisagus_template_gets_both_great_intelligence_points_in_its_cap() {
    let rs = full_ruleset();
    let names = catalogue_names();
    let json = include_str!("fixtures/book_templates/magus_bonisagus.json");
    let bonisagus = arm_rules::load_entity_migrating(
        json,
        arm_rules::validation::DEFAULT_SAGA_YEAR,
        &rs,
        &names,
    )
    .expect("book template fixture parses")
    .entity;
    assert_eq!(
        effective_characteristic_after_aging(&bonisagus, &rs, Characteristic::Int),
        5,
        "precondition: the template prints Int +5 (ArMDE:1656)"
    );

    let mut without_great_int = bonisagus.clone();
    without_great_int
        .selections
        .retain(|s| s.item_ref != Id::new("virtue.great_characteristic"));

    let creo_auram = |e: &Entity| {
        spell_level_cap(
            e,
            &rs,
            &Id::new("art.creo"),
            &Id::new("art.auram"),
            &[],
            false,
            false,
            false,
        )
    };
    assert_eq!(
        creo_auram(&bonisagus) - creo_auram(&without_great_int),
        2,
        "Great Intelligence (x2) raises Int +3 to +5 (ArMDE:1656, :1668), and the \
         cap is the Lab Total (ArMDE:2465), so the two Virtues must be worth 2 levels"
    );
}

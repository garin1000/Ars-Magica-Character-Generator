//! bve S1 (`tmp/bve-sweep.md`): Uninspirational caps the character's EFFECTIVE
//! Presence and Communication, not only the bought score.
//!
//! ArMDE:6921: "His Presence and Communication may not be greater than 0."
//! The character's Presence is the score he plays with — bought plus every
//! free delta (Sidhe Faerie Blood's "+1 to your Presence", ArMDE:3815; Magical
//! Blood's Magic Human "+1 to one Characteristic", ArMDE:4367). Today
//! `validation/scores.rs::validate_characteristic_within_cap_and_floor`
//! compares only the bought score, and `validate_characteristics` visits only
//! keys present in `entity.characteristics`; the UI deletes a Characteristic's
//! key at 0 (`state.svelte.ts::setCharacteristic`), so a free delta on an
//! unbought Characteristic is never checked at all.
//!
//! Every test runs against the real shipped ruleset, and builds the entity
//! the way the app does: a Characteristic at 0 has no map entry.

use arm_rules::Characteristic;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, validate};
use arm_rules::{characteristic_cap, effective_characteristic_score};
use std::collections::BTreeMap;

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

fn companion(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn uninspirational() -> Selection {
    Selection::new(Id::new("flaw.uninspirational"))
}

fn sidhe_faerie_blood() -> Selection {
    Selection::with_params(
        Id::new("virtue.faerie_blood"),
        BTreeMap::from([("heritage".to_string(), Id::new("heritage.sidhe"))]),
    )
}

fn magic_human_blood(characteristic: &str) -> Selection {
    Selection::with_params(
        Id::new("virtue.magical_blood"),
        BTreeMap::from([
            ("bloodline".to_string(), Id::new("bloodline.magic_human")),
            ("characteristic".to_string(), Id::new(characteristic)),
        ]),
    )
}

/// The `characteristic_above_cap` issues raised against `characteristic`.
fn above_cap_issues(e: &Entity, rs: &Ruleset, characteristic: Characteristic) -> usize {
    validate(e, rs)
        .issues
        .into_iter()
        .filter(|issue| {
            issue.code == ValidationIssue::CODE_CHARACTERISTIC_ABOVE_CAP
                && issue.args.get("characteristic").map(String::as_str)
                    == Some(characteristic.to_string().as_str())
        })
        .count()
}

#[test]
fn sidhe_blood_on_an_unbought_presence_breaks_uninspirationals_cap() {
    let rs = load_ruleset();
    let e = companion(vec![sidhe_faerie_blood(), uninspirational()]);

    assert!(
        !e.characteristics.contains_key(&Characteristic::Pre),
        "precondition: Presence left at 0 has no map entry, as the app stores it"
    );
    // `<= 0`, not `== 0`: a fix may express the cap in bought terms (0 minus
    // the free +1 = -1), which this test must not forbid.
    assert!(
        characteristic_cap(&rs, &e, Characteristic::Pre) <= 0,
        "precondition: Uninspirational lowers the Presence cap to 0 or below (ArMDE:6921)"
    );
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Pre),
        1,
        "precondition: Sidhe Blood adds +1 Presence (ArMDE:3815)"
    );

    assert_eq!(
        above_cap_issues(&e, &rs, Characteristic::Pre),
        1,
        "ArMDE:6921: 'His Presence and Communication may not be greater than 0' — \
         an effective Presence of +1 must raise characteristic_above_cap"
    );
}

#[test]
fn magic_human_blood_on_an_unbought_communication_breaks_uninspirationals_cap() {
    let rs = load_ruleset();
    let e = companion(vec![
        magic_human_blood("characteristic.com"),
        uninspirational(),
    ]);

    assert!(
        !e.characteristics.contains_key(&Characteristic::Com),
        "precondition: Communication left at 0 has no map entry"
    );
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Com),
        1,
        "precondition: Magic Human raises Communication by 1 (ArMDE:4367)"
    );

    assert_eq!(
        above_cap_issues(&e, &rs, Characteristic::Com),
        1,
        "ArMDE:6921: an effective Communication of +1 must raise characteristic_above_cap"
    );
}

/// The mirror case: Monstrous Blood's Magic Human "must decrease one of his
/// Characteristics by 1" (ArMDE:6462) on Presence, bought +1 — the effective
/// Presence is 0, which Uninspirational allows, yet the bought +1 is flagged
/// today because the cap is compared with the bought score.
#[test]
fn monstrous_blood_lets_a_bought_presence_of_one_sit_at_an_effective_zero() {
    let rs = load_ruleset();
    let mut e = companion(vec![
        Selection::with_params(
            Id::new("flaw.monstrous_blood"),
            BTreeMap::from([
                ("bloodline".to_string(), Id::new("bloodline.magic_human")),
                ("characteristic".to_string(), Id::new("characteristic.pre")),
            ]),
        ),
        uninspirational(),
    ]);
    e.characteristics.insert(Characteristic::Pre, 1);

    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Pre),
        0,
        "precondition: bought +1 minus Monstrous Blood's 1 is an effective 0 (ArMDE:6462)"
    );
    assert_eq!(
        above_cap_issues(&e, &rs, Characteristic::Pre),
        0,
        "ArMDE:6921: an effective Presence of 0 is not greater than 0"
    );
}

/// Guard, green today: the legal way to combine the two — buy Presence -1 so
/// Sidhe Blood brings it to 0 — must stay clean after the fix.
#[test]
fn buying_presence_down_to_offset_sidhe_blood_satisfies_uninspirational() {
    let rs = load_ruleset();
    let mut e = companion(vec![sidhe_faerie_blood(), uninspirational()]);
    e.characteristics.insert(Characteristic::Pre, -1);

    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Pre),
        0,
        "precondition: bought -1 plus Sidhe's +1 is an effective 0"
    );
    assert_eq!(
        above_cap_issues(&e, &rs, Characteristic::Pre),
        0,
        "an effective Presence of 0 is not greater than 0 (ArMDE:6921)"
    );
}

/// Guard, green today: a bought Presence above the cap is already flagged, and
/// a fix must not flag it a second time for the same Characteristic.
#[test]
fn a_bought_presence_above_the_cap_is_flagged_exactly_once() {
    let rs = load_ruleset();
    let mut e = companion(vec![sidhe_faerie_blood(), uninspirational()]);
    e.characteristics.insert(Characteristic::Pre, 1);

    assert_eq!(
        above_cap_issues(&e, &rs, Characteristic::Pre),
        1,
        "one finding per Characteristic, whether the bought or the effective score trips it"
    );
}
